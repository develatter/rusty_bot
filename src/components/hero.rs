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
pub fn Hero() -> Element {
    let mut message = use_signal(|| String::new());
    let mut message_history = use_signal(|| Vec::<ChatMessage>::new());
    let mut is_model_answering = use_signal(|| false);
    let mut is_model_loading = use_signal(|| true);
    let mut message_counter = use_signal(|| 0);

    // Inicializar el modelo al cargar el componente
    use_effect(move || {
        spawn(async move {
            match init_model().await {
                Ok(_) => {
                    is_model_loading.set(false);
                    println!("Modelo inicializado correctamente");
                }
                Err(e) => {
                    is_model_loading.set(false);
                    let mut history = message_history.read().clone();
                    history.push(ChatMessage {
                        role: ChatRole::Assistant,
                        content: format!("Error al inicializar el modelo: {}", e),
                    });
                    message_history.set(history);
                    println!("Error al inicializar el modelo: {}", e);
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
        if message_counter() >= 5 {
            // Reiniciar la sesión si se han enviado 5 mensajes
            spawn(async move {
                match reset_session().await {
                    Ok(_) => {
                        message_counter.set(0);
                        println!("Sesión reiniciada correctamente");
                    }
                    Err(e) => {
                        let mut history = message_history.read().clone();
                        history.push(ChatMessage {
                            role: ChatRole::Assistant,
                            content: format!("Error al reiniciar la sesión: {}", e),
                        });
                        message_history.set(history);
                        println!("Error al reiniciar la sesión: {}", e);
                    }
                }
            });
        }

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
                        message_counter.add(1);
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
                class: "max-w-3xl w-full flex-grow overflow-y-auto flex flex-col gap-4 p-4 items-center",
                for m in message_history.read().iter() {
                    Message {
                        msg: m.clone(),
                    }
                }
            }

            div {
                id: "input",
                class: "max-w-3xl w-full flex gap-4 p-4 justify-center",
                input {
                    class: "border-2 border-gray-300 rounded-lg p-2 w-full text-black",
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
                        "bg-blue-500 text-white rounded-lg p-2 {}",
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

#[server]
pub async fn reset_session() -> Result<(), ServerFnError> {
    use crate::server::llm::reset_chat_session;
    // Reiniciar la sesión de chat
    reset_chat_session().await.map_err(|e| {
        ServerFnError::new(&format!("Error al reiniciar sesión: {}", e))
    })
}

#[server(output = StreamingText)]
pub async fn get_response(prompt: String) -> Result<TextStream, ServerFnError> {
    use crate::server::llm;
    use futures;
    use kalosm::language::{GenerationParameters, ChatModelExt, StreamExt, TextStream};

    let (tx, rx) = futures::channel::mpsc::unbounded();

    // Verificar si el modelo está inicializado
    if llm::CHAT_SESSION.get().is_none() {
        return Err(ServerFnError::new("Modelo no inicializado"));
    }

    let time = std::time::Instant::now();
    println!("Procesando prompt: {}", prompt);

    // Primero intentamos obtener un stream sin reiniciar
    let mut stream = match try_get_stream(&prompt) {
        Ok(s) => s,
        Err(_) => {
            // Si falla, intentamos reiniciar la sesión
            println!("No se pudo obtener un stream inicial, intentando reiniciar la sesión...");

            // Reiniciar la sesión de chat
            llm::reset_chat_session().await
                .map_err(|e| ServerFnError::new(&format!("Error al reiniciar sesión: {}", e)))?;

            // Intentar de nuevo después del reinicio
            try_get_stream(&prompt)
                .map_err(|_| ServerFnError::new("No se pudo obtener un stream después de reiniciar"))?
        }
    };

    tokio::spawn(async move {
        let mut token_count = 0;

        // Enviar un token vacío inicial para asegurar que el stream no esté vacío
        let _ = tx.unbounded_send(Ok("".to_string()));

        // Consumir el stream y enviar los tokens al canal
        while let Some(token) = stream.next().await {
            token_count += 1;
            if token_count % 10 == 0 {
                println!("Tokens enviados: {}", token_count);
            }

            // Reenviar el token
            if tx.unbounded_send(Ok(token)).is_err() {
                println!("Error al enviar el token");
                break;
            }
        }
        println!("Stream completado, tokens totales: {}", token_count);
    });

    println!("\nTiempo total: {:?}", time.elapsed());
    Ok(server_fn::codec::TextStream::new(rx))
}


#[cfg(feature = "server")]
fn try_get_stream(prompt: &str) -> Result<impl futures::Stream<Item=String>, &'static str> {
    use crate::server::llm;
    use kalosm::language::{GenerationParameters, ChatModelExt};

    let chat_session = llm::CHAT_SESSION
        .get()
        .ok_or("Model couldn't be initialized.")?;

    let mut guard = chat_session
        .try_lock()
        .map_err(|_| "Couldn't get model lock")?;

    Ok(guard(prompt).with_sampler(GenerationParameters::default()
        .with_temperature(0.7)
        .with_top_p(0.9)
        .with_max_length(500)
    ))
}