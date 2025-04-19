use std::path::PathBuf;
use crate::components::Message;
use crate::model::chat::{ChatMessage, ChatRole};
use dioxus::html::input_data::keyboard_types::Key;
use dioxus::prelude::*;
use dioxus::CapturedError;
use dioxus::logger::tracing::instrument::WithSubscriber;
use wasm_bindgen::prelude::*;

#[component]
pub fn Hero() -> Element {
    let mut message = use_signal(|| String::new());
    let mut message_history = use_signal(|| Vec::<ChatMessage>::new());
    let mut is_model_answering = use_signal(|| false);
    let chat_div_id = "chat-container";

    let scroll_to_bottom = move || {
        let window = web_sys::window().expect("No hay window global");
        let document = window.document().expect("No hay document");
        if let Some(element) = document.get_element_by_id(chat_div_id) {
            let div = element.dyn_into::<web_sys::HtmlElement>().unwrap();
            div.set_scroll_top(div.scroll_height());
        }
    };

    // Usar un efecto después de que se actualice el componente
    use_effect(move || {
        // Ejecutar scroll_to_bottom cuando cambia message_history
        if !message_history().is_empty() {
            scroll_to_bottom();
        }
    });

    let mut send_message = move || {
        if !is_model_answering() && !message().is_empty() {
            is_model_answering.set(true);
            // Guarda el mensaje actual antes de limpiarlo
            let user_message = message().clone();

            // Actualiza el historial de mensajes

            let mut history = message_history.read().clone();
            history.push(ChatMessage {
                role: ChatRole::User,
                content: user_message.clone(),
            });

            // Añadir mensaje temporal con puntos suspensivos
            history.push(ChatMessage {
                role: ChatRole::Assistant,
                content: "...".to_string(),
            });

            message_history.set(history);
            message.set("".to_string());

            // Captura lo necesario para el spawn
            let mut message_history = message_history.clone();
            let mut is_model_answering = is_model_answering.clone();
            spawn(async move {
                let mut history = message_history.read().clone();
                // Reemplazar el último mensaje (puntos suspensivos) con la respuesta real
                if !history.is_empty() {
                    history.pop(); // Eliminar los puntos suspensivos
                }

                let mut content = String::new();

                match get_response(user_message).await {
                    Ok(response) => {
                        content.push_str(&response);
                    }
                    Err(e) => {
                        content.push_str(format!("Error: {}", e).as_str());
                    }
                }

                history.push(ChatMessage {
                    role: ChatRole::Assistant,
                    content,
                });

                message_history.set(history);
                is_model_answering.set(false);
            });
        }
    };

    rsx! {
        div {
            id: "hero",
            class: "w-full mx-auto h-screen flex flex-col items-center",
            div {
                id: chat_div_id,
                class: "max-w-2xl w-full flex-grow overflow-y-auto flex flex-col gap-4 p-4 items-center",
                for m in message_history.read().iter() {
                    Message {
                        msg: m.clone(),
                    }
                }
            }

            div {
                id: "input",
                class: "max-w-2xl w-full flex gap-4 p-4 justify-center",
                input {
                    class: "border-2 border-gray-300 rounded-lg p-2 w-full text-black",
                    placeholder: "Escribe tu mensaje...",
                    value: "{message}",
                    disabled: is_model_answering(),
                    oninput: move |event| {
                        if !is_model_answering() {
                            message.set(event.value());
                        }
                    },
                    onkeydown: move |event| {
                        if event.key() == Key::Enter && !event.modifiers().shift() {
                            event.prevent_default();
                            send_message();
                        }
                    }
                }
                button {
                    class: format!(
                        "bg-blue-500 text-white rounded-lg p-2 {}",
                        if is_model_answering() { "opacity-50 cursor-not-allowed" }
                        else { "" }
                    ),
                    disabled: is_model_answering(),
                    onclick: move |_| {
                        send_message();
                    },
                    "Enviar"
                }
            }
        }
    }
}

#[server]
pub async fn get_response(prompt: String) -> Result<String, ServerFnError> {
    use kalosm::language::{FileSource, StreamExt, TextStream, Llama, LlamaSource, GenerationParameters, ChatModelExt};

    let time = std::time::Instant::now();
    println!("Loading model");
    let model = Llama::builder().with_source(
        LlamaSource::new(
            FileSource::Local(PathBuf::from("models/Meta-Llama-3.1-8B-Instruct-Q8_0.gguf")),
        )
    ).build().await?;
    println!("Model loaded");

    // 1. Crear el chat_model
    let chat_model = model.chat();

    // 2. Aplicar el system prompt
    let mut chat_with_system = chat_model.with_system_prompt("Eres un asistente muy útil e inteligente que ayuda a estudiantes de instituto");

    // 3. Añadir el mensaje del usuario
    let mut chat = chat_with_system.add_message(prompt);

    let mut final_response = String::new();
    let mut token_count = 0;

    while let Some(token) = chat.next().await {
        token_count += 1;
        print!("{}", token);
        final_response.push_str(&token);

        if token_count >= 250 {
            println!("Límite de tokens alcanzado!");
            break;
        }
    }
    println!("\nTiempo total: {:?}", time.elapsed());
    Ok(final_response)
}

