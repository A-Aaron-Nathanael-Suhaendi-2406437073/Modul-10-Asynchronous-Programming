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
            <div class="flex w-screen h-screen bg-white">
                <div class="flex-none w-64 bg-gray-900 text-white shadow-lg z-10">
                    <div class="text-xl p-5 font-bold border-b border-gray-700 flex items-center gap-2">
                        <span>{"🟢"}</span> {"Online Users"}
                    </div>
                    <div class="overflow-y-auto h-full p-3 space-y-2">
                    {
                        self.users.clone().iter().map(|u| {
                            html!{
                                <div class="flex items-center bg-gray-800 rounded-lg p-2 hover:bg-gray-700 transition cursor-pointer">
                                    <div class="relative">
                                        <img class="w-10 h-10 rounded-full border-2 border-cyan-400" src={u.avatar.clone()} alt="avatar"/>
                                        <div class="absolute bottom-0 right-0 w-3 h-3 bg-green-500 border-2 border-gray-800 rounded-full"></div>
                                    </div>
                                    <div class="flex-grow ml-3">
                                        <div class="text-sm font-semibold text-gray-100">{u.name.clone()}</div>
                                        <div class="text-xs text-gray-400">{"Online"}</div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                    </div>
                </div>

                // Main Chat Area
                <div class="grow flex flex-col bg-slate-50">
                    // Chat Header
                    <div class="w-full h-16 bg-white border-b border-gray-200 shadow-sm flex items-center px-6 z-0">
                        <div class="text-lg font-bold text-gray-800">{"# general-chat"}</div>
                    </div>

                    // Chat Messages Container
                    <div class="w-full grow overflow-auto p-6 space-y-4">
                        {
                            self.messages.iter().map(|m| {
                                let user = self.users.iter().find(|u| u.name == m.from).unwrap();
                                html!{
                                    <div class="flex w-full mt-2 space-x-3 max-w-2xl">
                                        <img class="w-10 h-10 rounded-full flex-shrink-0" src={user.avatar.clone()} alt="avatar"/>
                                        <div>
                                            <div class="text-xs text-gray-500 mb-1 ml-1">{m.from.clone()}</div>
                                            <div class="bg-white p-3 rounded-r-2xl rounded-bl-2xl shadow-sm border border-gray-100">
                                                <p class="text-sm text-gray-800">
                                                    if m.message.ends_with(".gif") {
                                                        <img class="mt-2 rounded-lg" src={m.message.clone()}/>
                                                    } else {
                                                        {m.message.clone()}
                                                    }
                                                </p>
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>

                    // Input Area
                    <div class="w-full bg-white border-t border-gray-200 p-4">
                        <div class="flex items-center bg-gray-100 rounded-full px-4 py-2 focus-within:ring-2 focus-within:ring-cyan-300 transition">
                            <input ref={self.chat_input.clone()} type="text" placeholder="Type your message here..." class="block w-full bg-transparent outline-none text-sm text-gray-700 placeholder-gray-400" name="message" required=true />
                            <button onclick={submit} class="ml-2 flex-shrink-0 bg-cyan-500 hover:bg-cyan-600 text-white p-2 w-10 h-10 rounded-full flex justify-center items-center transition shadow-md">
                                <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="fill-current w-5 h-5">
                                    <path d="M0 0h24v24H0z" fill="none"></path><path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                                </svg>
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
