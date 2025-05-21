use std::path::PathBuf;
use tokio::sync::OnceCell;
use std::sync::Mutex;
use kalosm::language::{Chat, ChatModelExt, Llama};

// Singleton global para la sesión de chat
pub static CHAT_SESSION: OnceCell<Mutex<Chat<Llama>>> = OnceCell::const_new();
pub static MODEL: OnceCell<Mutex<Llama>> = OnceCell::const_new();

//Es más rápido con el prompt en inglés
const SYSTEM_PROMT: &str = "You are a virtual assistant named Rusty. \
If you don’t know something, do not make it up. \
Be friendly and professional. \
Speak English. \
Use emojis when appropriate.";

const MODEL_URL: &str = "models/Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf";
const SECOND_MODEL_URL: &str = "models/Llama-3.2-3B-Instruct-Q8_0.gguf";
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

