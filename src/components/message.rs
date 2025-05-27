use comrak::{markdown_to_html_with_plugins, ExtensionOptions, Plugins, RenderOptions};
use comrak::plugins::syntect::SyntectAdapterBuilder;
use crate::model::chat::{ChatMessage, ChatRole};
use dioxus::prelude::*;

#[component]
pub fn Message(msg: ReadOnlySignal<ChatMessage>) -> Element {
    let assistant_placeholder = use_memo(move || {
        let message = msg.read();
        message.role == ChatRole::Assistant && message.content.is_empty()
    });
    let role = use_memo(move || msg().role.clone());
    let content = use_memo(move || {
        let msg = msg();
        let msg_content = &msg.content;

        let mut plugins = Plugins::default();

        let adapter = SyntectAdapterBuilder::new()
            .theme("base16-ocean.dark")
            .build();

        plugins.render.codefence_syntax_highlighter = Some(&adapter);
        let mut extension = ExtensionOptions::default();
        extension.strikethrough = true;
        extension.tagfilter = true;
        extension.table = true;
        extension.autolink = true;

        let mut render = RenderOptions::default();
        render.hardbreaks = true;
        render.github_pre_lang = true;

        let options = comrak::Options {
            extension,
            render,
            ..Default::default()
        };

        markdown_to_html_with_plugins(msg_content, &options, &plugins)
    });

    let message_class = "max-w-[36rem]  p-4 mb-5 self-start text-white overflow-x-auto white-space-pre-wrap break-words";

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
