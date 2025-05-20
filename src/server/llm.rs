use std::path::PathBuf;
use tokio::sync::OnceCell;
use std::sync::Mutex;
use kalosm::language::{Chat, ChatModelExt, Llama};

// Singleton global para la sesión de chat
pub static CHAT_SESSION: OnceCell<Mutex<Chat<Llama>>> = OnceCell::const_new();
pub static MODEL: OnceCell<Mutex<Llama>> = OnceCell::const_new();

const SYSTEM_PROMT: &str = "Eres un asistente virtual llamado Rusty. Has sido programado por Álex López\
como parte de su proyecto de Final de Grado. Estás programado en Rust con Dioxus. Si no sabes algo, no te\
lo inventes. Habla en español de España. Usa emojis.";

const MODEL_URL: &str = "models/Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf";
pub async fn init_chat_model() -> Result<(), String> {
    use kalosm::language::{LlamaSource, ChatModelExt, FileSource};

    if CHAT_SESSION.get().is_none() {
        println!("Initializing chat model...");

        let llama = Llama::builder()
            .with_source(
                LlamaSource::new(
                    FileSource::Local(PathBuf::from(MODEL_URL))
                )
            )
            .build()
            .await
            .map_err(|e| e.to_string())?;

        println!("Model loaded successfully");
        let chat = llama.chat().with_system_prompt(SYSTEM_PROMT);
        MODEL.set(Mutex::new(llama))
            .map_err(|_| "Couldn't set model".to_string())?;
        CHAT_SESSION.set(Mutex::new(chat))
            .map_err(|_| "Couldn't set chat session".to_string())?;
    }
    Ok(())
}


//Reinicia el chat
pub async fn reset_chat_session() -> Result<(), String> {
    println!("Cleaning chat session...");
    // Obtener una referencia al modelo existente
    let model_guard = MODEL.get()
        .ok_or("Model is not initialized.")?
        .lock()
        .map_err(|_| "Couldn't get model lock".to_string())?;

    // Crear una nueva sesión de chat con el mismo modelo
    let new_chat = model_guard.chat().with_system_prompt(SYSTEM_PROMT);

    // Reemplazar la sesión de chat existente con la nueva
    *CHAT_SESSION.get()
        .ok_or("Session is not initialized.")?
        .lock()
        .map_err(|_| "Couldn't get session lock".to_string())? = new_chat;

    println!("Chat session was successfully reset");
    Ok(())
}