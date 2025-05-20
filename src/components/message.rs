use crate::model::chat::{ChatMessage, ChatRole};
use dioxus::prelude::*;

#[component]
pub fn Message(msg: ReadOnlySignal<ChatMessage>) -> Element {
    let assistant_placeholder = use_memo(move || {
        let message = msg.read();
        message.role == ChatRole::Assistant && message.content.is_empty()
    });
    let role = use_memo(move || msg().role.clone());
    let content = use_memo(move || msg().content.clone());

    let message_class = "max-w-l  p-4 mb-5 self-start text-white";

    rsx! {
        div {
            class: "{message_class}",
            class: if role() == ChatRole::Assistant {
                "self-start bg-gray-500 rounded-tl-lg rounded-tr-lg rounded-br-lg"
            } else {
                "self-end bg-blue-500 rounded-tl-lg rounded-tr-lg rounded-bl-lg"
            },

            class: if assistant_placeholder() {
                "text-gray-400"
            },

            if assistant_placeholder() {
                div {
                    class: "flex flex-col items-center justify-center min-h-[20px] w-full",
                    div {
                        class: "flex flex-row gap-1 justify-center items-center",
                        div {
                            class: "w-2 h-2 rounded-full bg-gray-100 animate-bounce [animation-delay:.7s]"
                        }
                        div {
                            class: "w-2 h-2 rounded-full bg-gray-100 animate-bounce [animation-delay:.3s]"
                        }
                        div {
                            class: "w-2 h-2 rounded-full bg-gray-100 animate-bounce [animation-delay:.7s]"
                        }
                    }
                }
            } else {
                div {
                    dangerous_inner_html: content
                }
            }
        }
    }
}
