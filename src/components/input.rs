use dioxus::prelude::*;
use crate::server;

#[component]
pub fn Input() -> Element {
    let mut msg = use_signal(|| "Hola, mundo!".to_string());
    rsx! {
        div {
            class: "input-container",
            input {
                class: "input",
                placeholder: "Type your message here...",
                oninput: move |event| {
                    msg.set(event.value());
                },
            }
            button {
                class: "send-button",
                onclick:  move |event| {
                    event.prevent_default();
                    //server::llama::get_response(msg.get()).await.unwrap();
                },
                "Send",
            }
        }
    }
}