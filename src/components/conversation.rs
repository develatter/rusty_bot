use std::ops::Add;
use crate::components::Message;
use crate::model::chat::{ChatMessage, ChatRole};
use dioxus::html::input_data::keyboard_types::Key;
use dioxus::logger::tracing::instrument::WithSubscriber;
use dioxus::prelude::server_fn::codec::{StreamingText, TextStream};
use dioxus::prelude::*;
use dioxus::CapturedError;
use futures::{StreamExt, TryFutureExt};
use std::path::PathBuf;
use dioxus::prelude::server_fn::ServerFn;
use wasm_bindgen::prelude::*;
use crate::server;


#[component]
pub fn Conversation() -> Element {
    let mut message = use_signal(|| String::new());
    let mut message_history = use_signal(|| Vec::<ChatMessage>::new());
    let mut is_model_answering = use_signal(|| false);
    let mut is_model_loading = use_signal(|| true);

    // Inicializar el modelo al cargar el componente
    use_effect(move || {
        spawn(async move {
            match init_model().await {
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
    });

    // Scroll al fondo cuando cambie el historial de mensajes
    use_effect(move || {
        if !message_history().is_empty() {
            scroll_to_bottom();
        }
    });

    let mut send_message = move || {
        let allowed_to_send = !is_model_answering() && !is_model_loading() && !message().is_empty();
        if allowed_to_send {
            is_model_answering.set(true);
            // Guarda el mensaje actual antes de limpiarlo
            let user_message = message().clone();

            // Actualiza el historial de mensajes
            let mut history = message_history().clone();
            history.push(ChatMessage {
                role: ChatRole::User,
                content: user_message.clone(),
            });

            // Añadir mensaje vacío para el asistente desde el principio
            history.push(ChatMessage {
                role: ChatRole::Assistant,
                content: "".to_string(),
            });

            message_history.set(history.clone());
            message.set("".to_string());

            // Captura lo necesario para el spawn
            let mut message_history = message_history.clone();
            let mut is_model_answering = is_model_answering.clone();

            spawn(async move {
                let mut history = message_history().clone();

                match get_response(user_message).await {
                    Ok(response) => {
                        let mut stream = response.into_inner();
                        while let Some(Ok(chunk)) = stream.next().await {
                            // Actualizar el último mensaje incrementalmente
                            let last_index = history.len() - 1;
                            history[last_index].content.push_str(&chunk);

                            // Actualizar la UI después de cada token
                            message_history.set(history.clone());
                        }
                    }
                    Err(e) => {
                        // En caso de error, actualizar el mensaje con el error
                        let last_index = history.len() - 1;
                        history[last_index].content = format!("Error: {}", e);
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
            class: "w-full bg-[#0f1116] mx-auto h-screen flex flex-col items-center",

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
                class: "w-full flex gap-4 p-4 justify-center",

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
                            send_message();
                        }
                    }
                }
                button {
                    class: format!(
                        "bg-blue-500 text-white rounded-lg h-[72px] flex items-center justify-center px-4 {}",
                        if is_model_answering() || is_model_loading() || message().is_empty() {
                            "opacity-50 cursor-not-allowed"
                        } else { "" }
                    ),
                    disabled: is_model_answering() || is_model_loading() || message().is_empty(),
                    onclick: move |_| {
                        send_message();
                    },
                    "Enviar"
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

#[server]
async fn init_model() -> Result<(), ServerFnError> {
    use crate::server::llm::init_chat_model;
    // Inicializar el modelo
    init_chat_model().await.map_err(|e| {
        ServerFnError::new(&format!("Error al inicializar el modelo: {}", e))
    })
}

#[server(output = StreamingText)]
pub async fn get_response(prompt: String) -> Result<TextStream, ServerFnError> {
    use crate::server::llm;
    use futures;
    use kalosm::language::{ChatModelExt, StreamExt, TextStream};

    let (tx, rx) = futures::channel::mpsc::unbounded();

    // Verificar si el modelo está inicializado
    if llm::CHAT_SESSION.get().is_none() {
        return Err(ServerFnError::new("Model not ininitalized"));
    }

    let time = std::time::Instant::now();
    println!("Procesando prompt: {}", prompt);

    // Intentar obtener un stream sin reiniciar
    let mut stream = llm::try_get_stream(&prompt).expect("Error getting stream");

    tokio::spawn(async move {
        let _ = tx.unbounded_send(Ok("".to_string()));
        // Consumir el stream y enviar los tokens al canal
        while let Some(token) = stream.next().await {
            if tx.unbounded_send(Ok(token)).is_err() {
                println!("Error al enviar el token");
                break;
            }
        }
    });

    println!("\nTotal response time: {:?}", time.elapsed());
    Ok(server_fn::codec::TextStream::new(rx))
}





