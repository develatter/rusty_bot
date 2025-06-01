use crate::components::Message;
use crate::model::chat::{ChatMessage, ChatRole};
use dioxus::html::input_data::keyboard_types::Key;
use dioxus::prelude::*;
use futures::{StreamExt};
use wasm_bindgen::prelude::*;
use crate::server_functions::server_functions::{get_response, reset_chat, search_context, init_llm_model, init_embedding_model, init_db};

#[component]
pub fn Conversation() -> Element {
    let mut message = use_signal(|| String::new());
    let mut message_history = use_signal(|| Vec::<ChatMessage>::new());
    let mut is_model_answering = use_signal(|| false);
    let mut is_model_loading = use_signal(|| true);
    let mut cancel_token = use_signal(|| false);
    let mut use_context = use_signal(|| false);

    // Inicializar el modelo al cargar el componente
    use_effect(move || {
        spawn(async move {
            match init_llm_model().await {
                Ok(_) => {
                    is_model_loading.set(false);
                    println!("Model succesfully initialized");
                }
                Err(e) => {
                    is_model_loading.set(false);
                    let mut history = message_history.read().clone();
                    history.push(ChatMessage {
                        role: ChatRole::Assistant,
                        content: format!("Error initializing model: {}", e),
                    });
                    message_history.set(history);
                    println!("Error initializing model: {}", e);
                }
            }
        });

        spawn(async move {
            println!("Initializing database...");
            // Initialize the database connection and document table
            if let Err(e) = init_db().await {
                println!("Error initializing database: {}", e);
            }
        });

        spawn(async move {
            match init_embedding_model().await {
                Ok(_) => println!("Embedding model succesfully initialized"),
                Err(e) => println!("Error initializing embedding model: {}", e),
            }
        });
    });


    // Scroll al fondo cuando cambie el historial de mensajes
    use_effect(move || {
        if !message_history().is_empty() {
            scroll_to_bottom();
        }
    });

    let mut button_action = move || {
        if is_model_answering() {
            // Cancelar
            cancel_token.set(true);
            is_model_answering.set(false);
        } else if !is_model_loading() && !message().is_empty() {
            cancel_token.set(false);
            is_model_answering.set(true);

            let mut user_message = message().clone();
            let mut history = message_history().clone();

            history.push(ChatMessage { role: ChatRole::User, content: user_message.clone() });
            history.push(ChatMessage { role: ChatRole::Assistant, content: String::new() });

            message_history.set(history.clone());
            message.set(String::new());

            let mut history = history.clone();
            let mut is_model_answering = is_model_answering.clone();
            let cancel_token = cancel_token.clone();

            let use_context_val = use_context();
            spawn(async move {
                if use_context_val {
                    if let Ok(context) = search_context(user_message.clone()).await {
                        let context_string = format!("\n\n[Possible useful context:\n{}]", context);
                        user_message.push_str(&context_string);
                    }
                }

                if let Ok(mut stream) = get_response(user_message).await.map(|r| r.into_inner()) {
                    while let Some(Ok(chunk)) = stream.next().await {
                        if cancel_token() {
                            break;
                        }
                        let last = history.len() - 1;
                        history[last].content.push_str(&chunk);
                        message_history.set(history.clone());
                    }
                }
                is_model_answering.set(false);
            });
        }
    };

    rsx! {
        div {
            id: "hero",
            class: "w-full max-w-[80rem] bg-[#0f1116] mx-auto h-screen flex flex-col items-center",

            if is_model_loading() {
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
                        "Inicializando modelo..."
                    }
                }
            }

            div {
                id: "chat-container",
                class: "w-full flex-grow overflow-y-auto flex flex-col gap-4 p-4 items-center",
                for m in message_history.read().iter() {
                    Message {
                        msg: m.clone(),
                    }
                }
            }

            div {
                id: "input",
                class: "w-full flex gap-4 p-4 justify-center items-end relative",

                textarea {
                    rows: "3",
                    class: "border-2 border-gray-300 rounded-lg p-2 w-full text-black h-[72px] overflow-y-auto resize-none",
                    placeholder: {
                        if is_model_loading() { "Esperando al modelo..." }
                        else if is_model_answering() { "" }
                        else { "Escribe tu mensaje..." }
                    },
                    value: "{message}",
                    disabled: is_model_answering() || is_model_loading(),
                    oninput: move |event| {
                        if !is_model_answering() && !is_model_loading() {
                            message.set(event.value());
                        }
                    },
                    onkeydown: move |event| {
                        if event.key() == Key::Enter
                        && !event.modifiers().shift()
                        && !is_model_loading()
                        && !is_model_answering()
                        && !message().is_empty() {
                            event.prevent_default();
                            button_action();
                        }
                    }
                }
                label {
                    class: format!(
                        "absolute left-5 bottom-5 inline-flex items-center cursor-pointer opacity-70{}",
                        if is_model_loading() || is_model_answering() { "" } else { " hover:opacity-100" }
                    ),
                    input {
                        disabled: is_model_loading() || is_model_answering(),
                        r#type: "checkbox",
                        class: "sr-only peer",
                        checked: "{use_context}",
                        onchange: move |e| use_context.set(e.value().parse::<bool>().unwrap_or(false)),
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
                        "Contexto"
                    }
                }


                button {
                    class: format!(
                        "bg-blue-500 text-white rounded-lg h-[72px] flex items-center justify-center px-4 {}",
                        if is_model_loading() || ( !is_model_answering() && message().is_empty() ) {
                            "opacity-50 cursor-not-allowed"
                        } else { "" }
                    ),
                    disabled: is_model_loading() || ( !is_model_answering() && message().is_empty() ),
                    onclick: move |_| {
                        button_action();
                    },
                    if is_model_answering() { "Cancelar" } else { "Enviar" }
                }


                button {
                    class: format! (
                        "fixed top-4 left-4 bg-blue-500 hover:bg-blue-600 text-white rounded-full \
                        w-12 h-12 flex items-center justify-center shadow-lg {}",
                        if is_model_loading() || is_model_answering() {
                            "opacity-50 cursor-not-allowed"
                        } else { "" }
                    ),
                    disabled: is_model_loading() || is_model_answering(),
                    onclick:  move |_| {
                        spawn(async move {
                            reset_chat().await.unwrap();
                            message_history.set(Vec::new());
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
    }
}


pub fn scroll_to_bottom() -> () {
    let window = web_sys::window().expect("There is no global window");
    let document = window.document().expect("Thre is no document");
    if let Some(element) = document.get_element_by_id("chat-container") {
        let div = element.dyn_into::<web_sys::HtmlElement>().unwrap();
        div.set_scroll_top(div.scroll_height());
    }
}
