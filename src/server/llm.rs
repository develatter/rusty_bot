//! Language Model Implementation
//!
//! This module provides functionality for interacting with the large language model (LLM).
//! It manages a singleton instance of the Llama chat model and provides methods for
//! generating responses, streaming text output, and resetting conversation state.

use std::path::PathBuf;
use tokio::sync::OnceCell;
use std::sync::Mutex;
use kalosm::language::{Chat, ChatModelExt, IntoChatMessage, Llama, ToChatMessage};

/// Global singleton for the chat session
/// Uses OnceCell and Mutex for thread-safe access and initialization
pub static CHAT_SESSION: OnceCell<Mutex<Chat<Llama>>> = OnceCell::const_new();

/// Global singleton for the language model
/// Stores the base LLM that can generate new chat sessions when needed
pub static MODEL: OnceCell<Mutex<Llama>> = OnceCell::const_new();

/// Initializes the language model and creates a chat session
///
/// This function:
/// 1. Checks if the model is already initialized
/// 2. If not, creates a new Llama model instance with the Qwen 2.5 7B model
/// 3. Creates a chat session from the model
/// 4. Stores both in their respective global singletons
///
/// Returns Ok(()) on success or an error message on failure
pub async fn init_chat_model() -> Result<(), String> {
    use kalosm::language::{LlamaSource, ChatModelExt, FileSource};

    if CHAT_SESSION.get().is_none() {
        println!("Initializing chat model...");

        let llama = Llama::builder()
            .with_source(
                LlamaSource::qwen_2_5_7b_instruct()
            )
            .build()
            .await
            .map_err(|e| e.to_string())?;


        println!("Model loaded successfully");
        let chat = llama.chat();
        MODEL.set(Mutex::new(llama))
            .map_err(|_| "Couldn't set model".to_string())?;
        CHAT_SESSION.set(Mutex::new(chat))
            .map_err(|_| "Couldn't set chat session".to_string())?;
    }
    Ok(())
}

/// Creates a stream for generating text responses from the language model
///
/// This function:
/// 1. Retrieves the chat session from the global singleton
/// 2. Sends the user's prompt to the model
/// 3. Configures generation parameters (temperature, top_p, etc.)
/// 4. Returns a stream that will yield text chunks as they are generated
///
/// # Parameters
/// * `prompt` - The user's input message
///
/// # Returns
/// * `Result<impl Stream<Item=String>, &'static str>` - A text generation stream or an error
pub fn try_get_stream(prompt: &str) -> Result<impl futures::Stream<Item=String>, &'static str> {
    use kalosm::language::{GenerationParameters};

    let chat_session = CHAT_SESSION
        .get()
        .ok_or("Model couldn't be initialized.")?;

    let mut guard = chat_session
        .try_lock()
        .map_err(|_| "Couldn't get model lock")?;

    Ok(guard(&prompt.into_chat_message()).with_sampler(GenerationParameters::default()
        .with_temperature(0.7)     // Controls randomness (higher = more random)
        .with_top_p(0.9)           // Nucleus sampling parameter (higher = more diverse)
        .with_max_length(600)      // Maximum response length in tokens
    ))
}

/// Resets the chat session to start a new conversation
///
/// This function:
/// 1. Retrieves the base language model
/// 2. Creates a fresh chat session
/// 3. Replaces the existing chat session in the global singleton
///
/// This effectively clears the conversation history and starts with a clean state.
///
/// # Returns
/// * `Result<(), String>` - Success or an error message
pub async fn reset_chat() -> Result<(), String> {
    let llama = MODEL
        .get()
        .ok_or("Model not initialized")?
        .lock()
        .map_err(|_| "Error locking model")?;
    let new_chat = llama.chat();
    let session_mutex = CHAT_SESSION
        .get()
        .ok_or("Session not initialized")?;
    *session_mutex
        .lock()
        .map_err(|_| "Error locking session")? = new_chat;
    Ok(())
}
