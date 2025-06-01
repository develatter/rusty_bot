//! Message Component
//!
//! This component renders individual chat messages with Markdown support.
//! It handles both user and assistant messages with appropriate styling,
//! and includes features like syntax highlighting and loading animations.

use comrak::{markdown_to_html_with_plugins, ExtensionOptions, Plugins, RenderOptions, RenderPlugins};
use comrak::plugins::syntect::SyntectAdapterBuilder;
use crate::model::chat::{ChatMessage, ChatRole};
use dioxus::prelude::*;

/// Message component for rendering individual chat messages
/// Supports rendering markdown content with syntax highlighting
#[component]
pub fn Message(msg: ReadOnlySignal<ChatMessage>) -> Element {
    // Detect if this is an empty assistant message (used for loading animation)
    let assistant_placeholder = use_memo(move || {
        let message = msg.read();
        message.role == ChatRole::Assistant && message.content.is_empty()
    });
    
    // Extract the message role for conditionally styling the component
    let role = use_memo(move || msg().role.clone());
    
    // Process markdown content to HTML with syntax highlighting
    let content = use_memo(move || {
        let msg = msg();
        let msg_content = &msg.content;
        
        // Configure syntax highlighter with dark theme
        let syntec_adapter = SyntectAdapterBuilder::new()
            .theme("base16-ocean.dark")
            .build();

        // Set up Comrak plugins for rendering with syntax highlighting
        let plugins = Plugins::builder()
            .render(
                RenderPlugins::builder()
                    .codefence_syntax_highlighter(&syntec_adapter)
                    .build()
            ).build();

        // Configure markdown extension options
        let extension_options = ExtensionOptions::builder()
            .strikethrough(true)  // Support ~~strikethrough~~ syntax
            .tagfilter(true)      // Filter potentially dangerous HTML tags
            .autolink(true)       // Auto-detect URLs and make them clickable
            .table(true)          // Support markdown tables
            .build();

        // Configure HTML rendering options
        let render_options = RenderOptions::builder()
            .hardbreaks(true)     // Treat newlines as <br> tags
            .github_pre_lang(true) // GitHub-style language tags for code blocks
            .build();

        // Combine all options for markdown processing
        let options = comrak::Options {
            extension: extension_options,
            render: render_options,
            ..Default::default()
        };
        
        // Convert markdown to HTML with all configured options
        markdown_to_html_with_plugins(msg_content, &options, &plugins)
    });

    // Base styling for all messages
    let message_class = "max-w-[70rem] p-4 mb-2 break-words";
    
    // Role-specific styling
    let user_message_class = "self-end bg-blue-500 rounded-tl-lg rounded-tr-lg rounded-bl-lg text-white";
    let assistant_message_class = "self-start max-w-full text-gray-200";

    // Render the message component with appropriate styling
    rsx! {
        div {
            class: "{message_class}",
            // Apply different styling based on message role (user vs assistant)
            class: if role() == ChatRole::Assistant {
                "{assistant_message_class}"
            } else {
                "{user_message_class}"
            },

            // Apply placeholder styling for empty assistant messages
            class: if assistant_placeholder() {
                "text-gray-400"
            },

            // Show loading animation for empty assistant messages (waiting for response)
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
                // Render the processed HTML content for non-empty messages
                div {
                    dangerous_inner_html: content
                }
            }
        }
    }
}
