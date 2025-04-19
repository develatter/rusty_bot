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

    let message_class = "max-w-md p-4 mb-5 rounded-lg self-start text-white";

    rsx! {
        div {
            class: "{message_class}",
            class: if role() == ChatRole::Assistant {
                "self-start bg-gray-500"
            } else {
                "self-end bg-blue-500"
            },

            class: if assistant_placeholder() {
                "text-gray-400"
            },

            if assistant_placeholder() {
                "Thinking..."
            } else {
                div {
                    dangerous_inner_html: content
                }
            }
        }
    }
}
