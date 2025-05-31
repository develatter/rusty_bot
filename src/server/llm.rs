use std::path::PathBuf;
use tokio::sync::OnceCell;
use std::sync::Mutex;
use kalosm::language::{Chat, ChatModelExt, IntoChatMessage, Llama, ToChatMessage};

// Singleton global para la sesión de chat
pub static CHAT_SESSION: OnceCell<Mutex<Chat<Llama>>> = OnceCell::const_new();
pub static MODEL: OnceCell<Mutex<Llama>> = OnceCell::const_new();

//Es más rápido con el prompt en inglés
const SYSTEM_PROMT: &str = "You are a virtual assistant answering user questions\
Be friendly and professional. \
Use emojis when appropriate.";

pub async fn init_chat_model() -> Result<(), String> {
    use kalosm::language::{LlamaSource, ChatModelExt, FileSource};

    if CHAT_SESSION.get().is_none() {
        println!("Initializing chat model...");

        let llama = Llama::builder()
            .with_source(
                LlamaSource::qwen_2_5_7b_instruct()
                //LlamaSource::new(FileSource::from(Local(PathBuf::from(MODEL_URL)))
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

pub fn try_get_stream(prompt: &str) -> Result<impl futures::Stream<Item=String>, &'static str> {
    use kalosm::language::{GenerationParameters};

    let chat_session = CHAT_SESSION
        .get()
        .ok_or("Model couldn't be initialized.")?;

    let mut guard = chat_session
        .try_lock()
        .map_err(|_| "Couldn't get model lock")?;

    Ok(guard(&prompt.into_chat_message()).with_sampler(GenerationParameters::default()
        .with_temperature(0.7)
        .with_top_p(0.9)
        .with_max_length(600)
    ))
}
pub fn reset_chat() -> Result<(), String> {
    let llama = MODEL
        .get()
        .ok_or("Model not initialized")?
        .lock()
        .map_err(|_| "Error locking model")?;
    let new_chat = llama.chat().with_system_prompt(SYSTEM_PROMT);
    let session_mutex = CHAT_SESSION
        .get()
        .ok_or("Session not initialized")?;
    *session_mutex
        .lock()
        .map_err(|_| "Error locking session")? = new_chat;
    Ok(())
}
