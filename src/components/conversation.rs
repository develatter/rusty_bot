//! Conversation Component
//!
//! This component implements a chat interface for interacting with an AI language model.
//! It handles the conversation flow, message history, response streaming, and UI state management.

use crate::components::Message;
use crate::model::chat::{ChatMessage, ChatRole};
use crate::server_functions::server_functions::{get_response, reset_chat, search_context, init_llm_model, init_embedding_model, init_db};
use dioxus::html::input_data::keyboard_types::Key;
use dioxus::prelude::*;
use futures::StreamExt;
use wasm_bindgen::prelude::*;

// Structure to keep application state organized
#[derive(Clone)]
struct ConversationState {
    input_message: String,
    message_history: Vec<ChatMessage>,
    is_model_answering: bool,
    is_model_loading: bool,
    is_database_loading: bool,
    cancel_token: bool,
    use_context: bool,
}

/// Main conversation component that provides the chat interface
#[component]
pub fn Conversation() -> Element {
    // Initialize conversation state
    let state = use_signal(|| ConversationState {
        input_message: String::new(),
        message_history: Vec::new(),
        is_model_answering: false,
        is_model_loading: true,
        is_database_loading: true,
        cancel_token: false,
        use_context: false,
    });

    // Initialize all systems when component loads
    use_effect(move || {
        initialize_systems(state.clone());
    });

    // Auto-scroll when message history changes
    use_effect(move || {
        if !state.read().message_history.is_empty() {
            scroll_to_bottom();
        }
    });
    
    // Render the user interface
    rsx! {
        div {
            id: "hero",
            class: "w-full max-w-[80rem] bg-[#0f1116] mx-auto h-screen flex flex-col items-center",

            // Loading screen during initialization
            if state.read().is_model_loading || state.read().is_database_loading {
                { render_loading_screen() }
            }

            // Message container with scrolling
            { render_message_container(&state) }

            // Input area and control buttons
            { render_input_area(&state) }
        }
    }
}

/// Initialize all systems: LLM, database and embeddings model
fn initialize_systems(state: Signal<ConversationState>) {
    // Initialize LLM model
    initialize_language_model(state.clone());
    
    // Initialize database
    initialize_database(state.clone());
    
    // Initialize embeddings model
    initialize_embedding_model();
}

/// Initialize the language model
fn initialize_language_model(mut state: Signal<ConversationState>) {
    spawn(async move {
        match init_llm_model().await {
            Ok(_) => {
                let mut current_state = state.read().clone();
                current_state.is_model_loading = false;
                state.set(current_state);
                println!("Model initialized successfully");
            }
            Err(e) => {
                let mut current_state = state.read().clone();
                current_state.is_model_loading = false;
                current_state.message_history.push(ChatMessage {
                    role: ChatRole::Assistant,
                    content: format!("Error initializing model: {}", e),
                });
                state.set(current_state);
                println!("Error initializing model: {}", e);
            }
        }
    });
}

/// Initialize the database
fn initialize_database(mut state: Signal<ConversationState>) {
    spawn(async move {
        println!("Initializing database...");
        match init_db().await {
            Ok(_) => {
                let mut current_state = state.read().clone();
                current_state.is_database_loading = false;
                state.set(current_state);
                println!("Database initialized successfully");
            }
            Err(e) => {
                let mut current_state = state.read().clone();
                current_state.is_database_loading = false;
                current_state.message_history.push(ChatMessage {
                    role: ChatRole::Assistant,
                    content: format!("Error initializing database: {}", e),
                });
                state.set(current_state);
                println!("Error initializing database: {}", e);
            }
        }
    });
}

/// Initialize the embeddings model
fn initialize_embedding_model() {
    spawn(async move {
        match init_embedding_model().await {
            Ok(_) => println!("Embeddings model initialized successfully"),
            Err(e) => println!("Error initializing embeddings model: {}", e),
        }
    });
}

/// Handle message sending and response reception
async fn handle_message_send(mut state: Signal<ConversationState>) {
    let current_state = state.read().clone();
    
    // Debug output to help diagnose issues
    println!("handle_message_send called with: is_model_answering={}, is_model_loading={}, is_database_loading={}, message_empty={}", 
             current_state.is_model_answering, 
             current_state.is_model_loading,
             current_state.is_database_loading,
             current_state.input_message.trim().is_empty());
    
    // Case 1: If model is answering, cancel the generation
    if current_state.is_model_answering {
        println!("Canceling current response");
        let mut new_state = current_state.clone();
        new_state.cancel_token = true;
        new_state.is_model_answering = false;
        state.set(new_state);
        return;
    } 
    
    // Case 2: Check if we can proceed with sending a message
    if current_state.is_model_loading {
        println!("Cannot send: Model is still loading");
        return;
    }
    
    if current_state.is_database_loading {
        println!("Cannot send: Database is still loading");
        return;
    }
    
    if current_state.input_message.trim().is_empty() {
        println!("Cannot send: Message is empty");
        return;
    }

    println!("Sending message: {}", current_state.input_message);
    
    // Case 3: Prepare message and update state
    let mut new_state = current_state.clone();
    new_state.cancel_token = false;
    new_state.is_model_answering = true;
    
    let user_message = current_state.input_message.trim().to_string();
    
    // Add user message to history
    new_state.message_history.push(ChatMessage { 
        role: ChatRole::User, 
        content: user_message.clone() 
    });
    
    // Create empty assistant message that will be filled with streaming response
    new_state.message_history.push(ChatMessage { 
        role: ChatRole::Assistant, 
        content: String::new() 
    });
    
    // Clear input field
    new_state.input_message = String::new();
    
    // Update state with new messages
    state.set(new_state);

    // Process response asynchronously
    process_response(state.clone(), user_message);
}

/// Process model response asynchronously
fn process_response(mut state: Signal<ConversationState>, mut user_message: String) {
    println!("Starting response processing for message: {}", user_message);
    
    spawn(async move {
        let use_context_enabled = state.read().use_context;
        
        // Get relevant context when enabled
        if use_context_enabled {
            println!("Context search enabled, looking for relevant information");
            match search_context(user_message.clone()).await {
                Ok(context) => {
                    let context_string = format!("\n\n[Potentially useful context:\n{}]", context);
                    user_message.push_str(&context_string);
                    println!("Added context to message");
                },
                Err(e) => println!("Error searching for context: {:?}", e)
            }
        }

        // Get and process response stream
        println!("Requesting response from model");
        match get_response(user_message).await {
            Ok(response) => {
                let mut stream = response.into_inner();
                println!("Got response stream, processing chunks");
                
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(chunk) => {
                            // Check if response was canceled
                            if state.read().cancel_token {
                                println!("Response generation was canceled");
                                break;
                            }
                            
                            // Update response with new chunk
                            let mut current_state = state.read().clone();
                            if let Some(last_message) = current_state.message_history.last_mut() {
                                last_message.content.push_str(&chunk);
                                state.set(current_state);
                            }
                        },
                        Err(e) => println!("Error in stream chunk: {:?}", e)
                    }
                }
                println!("Finished processing response stream");
            },
            Err(e) => println!("Error getting response: {:?}", e)
        }

        // Finalize response state
        let mut current_state = state.read().clone();
        current_state.is_model_answering = false;
        state.set(current_state);
        println!("Response completed, reset answering state");
    });
}

/// Render the loading screen
fn render_loading_screen() -> Element {
    rsx! {
        div {
            class: "w-screen h-screen flex flex-col items-center justify-center",
            div {
                class: "loader",
                div { class: "loader-square" }
                div { class: "loader-square" }
                div { class: "loader-square" }
                div { class: "loader-square" }
                div { class: "loader-square" }
                div { class: "loader-square" }
                div { class: "loader-square" }
            }
            p {
                class: "mt-12 text-gray-600 font-semibold",
                "Initializing model..."
            }
        }
    }
}

/// Render the message container
fn render_message_container(state: &Signal<ConversationState>) -> Element {
    rsx! {
        div {
            id: "chat-container",
            class: "w-full flex-grow overflow-y-auto flex flex-col gap-4 p-4 items-center",
            for m in state.read().message_history.iter() {
                Message {
                    msg: m.clone(),
                }
            }
        }
    }
}

/// Render the text input area and buttons
fn render_input_area(state: &Signal<ConversationState>) -> Element {
    let state_clone = state.clone();
    
    rsx! {
        div {
            id: "input",
            class: "w-full flex gap-4 p-4 justify-center items-end relative",

            // Textarea for message input
            { render_input_textarea(state) }
            
            // Toggle switch for context
            { render_context_toggle(state) }

            // Send/Cancel button with dynamic state
            { render_send_button(state) }

            // Button to reset conversation
            { render_reset_button(state_clone) }
        }
    }
}

/// Render the text input area
fn render_input_textarea(state: &Signal<ConversationState>) -> Element {
    let current_state = state.read();
    let is_disabled = current_state.is_model_answering || 
                      current_state.is_model_loading || 
                      current_state.is_database_loading;
    let placeholder = if current_state.is_model_loading || current_state.is_database_loading {
        "Waiting for model..."
    } else if current_state.is_model_answering {
        ""
    } else {
        "Type your message..."
    };

    let mut state_clone = state.clone();
    
    rsx! {
        textarea {
            id: "message-input",
            rows: "3",
            class: "border-2 border-gray-300 rounded-lg p-2 w-full text-black h-[72px] overflow-y-auto resize-none",
            placeholder: placeholder,
            value: "{current_state.input_message}",
            disabled: is_disabled,
            oninput: move |event| {
                let mut new_state = state_clone.read().clone();
                new_state.input_message = event.value();
                state_clone.set(new_state);
            },
            onkeydown: move |event| {
                println!("Key pressed: {:?}, shift: {}", event.key(), event.modifiers().shift());
                
                if event.key() == Key::Enter && !event.modifiers().shift() {
                    event.prevent_default();
                    let current = state_clone.read().clone(); // Clonar para pasar a spawn
                    
                    if !current.input_message.trim().is_empty() {
                        println!("Enter key pressed, calling handle_message_send");
                        // Envolver la llamada a la función async en spawn
                        spawn(handle_message_send(state_clone.clone()));
                    } else {
                        println!("Enter pressed but message is empty");
                    }
                }
            }
        }
    }
}

/// Render the context toggle switch
fn render_context_toggle(state: &Signal<ConversationState>) -> Element {
    let current_state = state.read();
    let is_disabled = current_state.is_model_loading || 
                      current_state.is_database_loading || 
                      current_state.is_model_answering;
    let opacity_class = if is_disabled { "" } else { " hover:opacity-100" };
    let mut state_clone = state.clone();
    
    rsx! {
        label {
            class: format!(
                "absolute left-5 bottom-5 inline-flex items-center cursor-pointer opacity-70{}", 
                opacity_class
            ),
            input {
                disabled: is_disabled,
                r#type: "checkbox",
                class: "sr-only peer",
                checked: "{current_state.use_context}",
                onchange: move |e| {
                    let mut new_state = state_clone.read().clone();
                    new_state.use_context = e.value().parse::<bool>().unwrap_or(false);
                    state_clone.set(new_state);
                },
            }
            div {
                class: "\
                    peer ring-2 ring-gray-900 \
                    bg-gray-300 \
                    rounded-full outline-none duration-300 after:duration-500 \
                    w-7 h-3  peer-checked:bg-blue-500 \
                    peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-gray-900 \
                    relative \
                    after:content-[''] after:rounded-full after:absolute after:outline-none \
                    after:h-4 after:w-4 after:bg-gray-50 after:-top-0.5 after:-left-0.5 \
                    after:flex after:justify-center after:items-center after:border after:border-gray-900 \
                    peer-checked:after:translate-x-4 \
                    transition-all",
            }
            span {
                class: "ml-1 text-[10px] text-gray-400 bg-transparent px-1 py-0 rounded select-none pointer-events-none",
                "Context"
            }
        }
    }
}

/// Render the send/cancel button
fn render_send_button(state: &Signal<ConversationState>) -> Element {
    let current_state_for_render = state.read();
    let is_answering_for_render = current_state_for_render.is_model_answering;
    let is_loading_for_render = current_state_for_render.is_model_loading || current_state_for_render.is_database_loading;
    let is_empty_for_render = current_state_for_render.input_message.trim().is_empty();
    
    let is_disabled_for_render = is_loading_for_render || (!is_answering_for_render && is_empty_for_render);
    let button_class = if is_disabled_for_render && !is_answering_for_render {
        "opacity-50 cursor-not-allowed"
    } else { 
        "" 
    };
    let button_text = if is_answering_for_render { "Cancel" } else { "Send" };
    
    let state_for_handler = state.clone(); // Clonar Signal para el manejador
    
    rsx! {
        button {
            id: "send-button",
            class: format!(
                "bg-blue-500 text-white rounded-lg h-[72px] flex items-center justify-center px-4 {}",
                button_class
            ),
            disabled: is_disabled_for_render, // Añadir el atributo disabled
            onclick: move |_| {
                // Leer el estado actual *dentro* del manejador onclick
                let current_state_at_click = state_for_handler.read().clone(); // Clonar para pasar a spawn
                let is_answering_at_click = current_state_at_click.is_model_answering;
                let is_loading_at_click = current_state_at_click.is_model_loading || current_state_at_click.is_database_loading;
                let is_empty_at_click = current_state_at_click.input_message.trim().is_empty();

                println!("Send button clicked. (Live state) is_answering={}, is_loading={}, is_empty={}", 
                         is_answering_at_click, is_loading_at_click, is_empty_at_click);
                
                // Usar el estado actual para la condición
                if is_answering_at_click || (!is_loading_at_click && !is_empty_at_click) {
                    println!("Conditions met (live state), calling handle_message_send");
                    // Envolver la llamada a la función async en spawn
                    spawn(handle_message_send(state_for_handler.clone()));
                } else {
                    println!("Button clicked but conditions not met for sending (live state)");
                }
            },
            "{button_text}"
        }
    }
}

/// Render the reset button
fn render_reset_button(mut state: Signal<ConversationState>) -> Element {
    let current_state = state.read();
    let is_disabled = current_state.is_model_loading || current_state.is_model_answering;
    let button_class = if is_disabled {
        "opacity-50 cursor-not-allowed"
    } else { 
        "" 
    };
    
    rsx! {
        button {
            class: format!(
                "fixed top-4 left-4 bg-blue-500 hover:bg-blue-600 text-white rounded-full \
                w-12 h-12 flex items-center justify-center shadow-lg {}", 
                button_class
            ),
            disabled: is_disabled,
            onclick: move |_| {
                spawn(async move {
                    reset_chat().await.unwrap();
                    let mut new_state = state.read().clone();
                    new_state.message_history = Vec::new();
                    state.set(new_state);
                });
            },
            svg {
                xmlns: "http://www.w3.org/2000/svg",
                fill: "none",
                view_box: "0 0 24 24",
                stroke_width: "1.5",
                stroke: "currentColor",
                class: "size-6",

                path {
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    d: "M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7l3.181 3.182m0-4.991v4.99"
                }
            }
        }
    }
}

/// Helper function to scroll the chat container to the bottom
/// Used to keep the most recent messages visible
pub fn scroll_to_bottom() -> () {
    let window = web_sys::window().expect("There is no global window");
    let document = window.document().expect("There is no document");
    if let Some(element) = document.get_element_by_id("chat-container") {
        let div = element.dyn_into::<web_sys::HtmlElement>().unwrap();
        div.set_scroll_top(div.scroll_height());
    }
}
