use comrak::{markdown_to_html_with_plugins, ExtensionOptions, ExtensionOptionsBuilder, Plugins, RenderOptions, RenderPlugins};
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
        let syntec_adapter = SyntectAdapterBuilder::new()
            .theme("base16-ocean.dark")
            .build();

        let plugins = Plugins::builder()
            .render(
                RenderPlugins::builder()
                    .codefence_syntax_highlighter(&syntec_adapter)
                    .build()
            ).build();


        let extension_options = ExtensionOptions::builder()
            .strikethrough(true)
            .tagfilter(true)
            .autolink(true)
            .table(true)
            .build();

        let render_options = RenderOptions::builder()
            .hardbreaks(true)
            .github_pre_lang(true)
            .build();

        let options = comrak::Options {
            extension: extension_options,
            render: render_options,
            ..Default::default()
        };
        markdown_to_html_with_plugins(msg_content, &options, &plugins)
    });

    let message_class = "max-w-[70rem] p-4 mb-2 self-start text-white break-words";

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
