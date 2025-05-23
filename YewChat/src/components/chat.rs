use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);

        html! {
            <div class="flex w-screen gothic">
                <div class="flex-none w-56 h-screen bg-black border-r border-red-900">
                    <div class="text-xl p-3 border-b border-red-900">{"☠ Users"}</div>
                    {
                        self.users.iter().map(|u| {
                            html!{
                                <div class="flex m-3 bg-[#1f1f1f] rounded-lg p-2 border border-red-900 shadow-inner">
                                    <img class="w-12 h-12 rounded-full avatar-frame" src={u.avatar.clone()} alt="avatar"/>
                                    <div class="flex-grow p-3 text-sm">
                                        <div>{&u.name}</div>
                                        <div class="text-xs text-gray-400">{"Summoned..."}</div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
                <div class="grow h-screen flex flex-col bg-[#121212]">
                    <div class="w-full h-14 border-b-2 border-red-900">
                        <div class="text-xl p-3">{"💬 SpellCast Chat"}</div>
                    </div>
                    <div class="grow overflow-auto p-4 space-y-3">
                        {
                            self.messages.iter().map(|m| {
                                let user = self.users.iter().find(|u| u.name == m.from).unwrap();
                                html!{
                                    <div class="flex bg-[#1e1e1e] p-3 rounded-lg border border-red-800 shadow-sm w-fit max-w-[70%]">
                                        <img class="w-8 h-8 rounded-full avatar-frame mr-3" src={user.avatar.clone()} />
                                        <div>
                                            <div class="text-sm font-semibold text-red-500">{m.from.clone()}</div>
                                            {
                                                if m.message.ends_with(".gif") {
                                                    html! { <img class="mt-2 rounded" src={m.message.clone()} /> }
                                                } else {
                                                    html! { <div class="text-sm text-gray-300">{m.message.clone()}</div> }
                                                }
                                            }
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                    <div class="flex items-center px-3 py-4 border-t border-red-800 bg-black">
                        <input ref={self.chat_input.clone()} type="text" placeholder="Speak your mind..." class="w-full py-2 px-4 bg-[#1a1a1a] rounded-full text-white outline-none" />
                        <button onclick={submit} class="ml-3 bg-red-800 hover:bg-red-700 text-white p-3 rounded-full">
                            <svg viewBox="0 0 24 24" class="w-5 h-5 fill-white"><path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/></svg>
                        </button>
                    </div>
                </div>
            </div>
        }

    }
}