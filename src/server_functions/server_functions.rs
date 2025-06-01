//! # Server Functions Module
//!
//! This module contains Dioxus server functions It leverages Dioxus server functions to bridge client-server
//! communication.

use dioxus::prelude::{server, server_fn, ServerFnError};
use dioxus::prelude::server_fn::codec::{StreamingText, TextStream};

/// Initializes the language model for chat functionality.
///
/// This server function loads and prepares the chat model for use.
/// 
/// # Returns
/// 
/// * `Result<(), ServerFnError>` - Success or error with detailed message
#[server]
pub async fn init_llm_model() -> Result<(), ServerFnError> {
    use crate::server::llm::init_chat_model;
    init_chat_model().await.map_err(|e| {
        ServerFnError::new(&format!("Error initializing model: {}", e))
    })
}

/// Initializes the embedding model for text vectorization.
///
/// This server function loads and prepares the embedding model for use.
/// 
/// # Returns
/// 
/// * `Result<(), ServerFnError>` - Success or error with detailed message
#[server]
pub async fn init_embedding_model() -> Result<(), ServerFnError> {
    use crate::server::embedding::init_embedding_model;
    init_embedding_model().await.map_err(|e| {
        ServerFnError::new(&format!("Error initializing embedding model: {}", e))
    })
}

/// Generates embedding vectors for the provided text.
///
/// # Arguments
///
/// * `txt` - The text to embed
///
/// # Returns
///
/// * `Result<Vec<f32>, ServerFnError>` - Embedding vector or error message
#[server]
pub async fn get_embedding(txt: String) -> Result<Vec<f32>, ServerFnError> {
    let result = tokio::task::spawn_blocking(move || {
        futures::executor::block_on(crate::server::embedding::embed_text(&txt))
    })
        .await
        .map_err(|e| ServerFnError::new(&e.to_string()))?;

    result.map_err(|e| ServerFnError::new(&format!("Error embedding text: {}", e)))
}

/// Resets the current chat session.
///
/// Clears conversation history and resets the chat model's state.
///
/// # Returns
///
/// * `Result<(), ServerFnError>` - Success or error with detailed message
#[server]
pub async fn reset_chat() -> Result<(), ServerFnError> {
    use crate::server::llm::reset_chat;
    reset_chat().await.map_err(|e| ServerFnError::new(&format!("Error trying to reset chat: {}", e)))
}

/// Processes a user prompt and returns a streaming text response.
///
/// This function streams model responses token by token, allowing
/// for real-time display to users.
///
/// # Arguments
///
/// * `prompt` - The user's input text
///
/// # Returns
///
/// * `Result<TextStream, ServerFnError>` - Stream of response tokens or error
#[server(output = StreamingText)]
pub async fn get_response(prompt: String) -> Result<TextStream, ServerFnError> {
    use crate::server::llm;
    use futures;
    use kalosm::language::{ChatModelExt, StreamExt, TextStream};

    let (tx, rx) = futures::channel::mpsc::unbounded();

    // Check if the model is initialized
    if llm::CHAT_SESSION.get().is_none() {
        return Err(ServerFnError::new("Model not initialized"));
    }

    let time = std::time::Instant::now();
    println!("Processing prompt: {}", prompt);

    // Try to get a stream without restarting
    let mut stream = llm::try_get_stream(&prompt).expect("Error getting stream");

    tokio::spawn(async move {
        let _ = tx.unbounded_send(Ok("".to_string()));
        // Consume the stream and send tokens to the channel
        while let Some(token) = stream.next().await {
            if tx.unbounded_send(Ok(token)).is_err() {
                println!("Error sending token");
                break;
            }
        }
    });

    println!("\nTotal response time: {:?}", time.elapsed());
    Ok(server_fn::codec::TextStream::new(rx))
}

/// Searches the database for relevant context given a query.
///
/// Retrieves documents that match the query from the database.
///
/// # Arguments
///
/// * `q` - The search query
///
/// # Returns
///
/// * `Result<String, ServerFnError>` - Formatted context string or error
#[server]
pub async fn search_context(q: String) -> Result<String, ServerFnError> {
    println!("Searching context for query: {}", q);
    let context = crate::server::database_impl::query(&q).await.map_err(|e| {
        println!("Error querying database: {}", e);
        ServerFnError::new(&format!("Error querying database: {}", e))
    })?.into_iter()
        .map(|document| {
            format!(
                "Title: {}\nBody: {}\n",
                document.title,
                document.body
            )
        }).collect::<Vec<_>>().join("\n");
    Ok(context)
}

/// Initializes the database connection.
///
/// Must be called before any database operations can be performed.
///
/// # Returns
///
/// * `Result<(), ServerFnError>` - Success or error with detailed message
#[server]
pub async fn init_db() -> Result<(), ServerFnError> {
    crate::server::database_impl::connect_to_database()
        .await
        .map_err(|e| {
            eprintln!("Error: {:?}", e);
            ServerFnError::new(e)
        })?;
    Ok(())
}