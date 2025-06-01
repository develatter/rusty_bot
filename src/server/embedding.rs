//! Embedding Model Implementation
//!
//! This module provides functionality for text embedding generation using the BERT model.
//! It manages a singleton embedding model instance and offers methods to convert text
//! into numerical vector representations for semantic search and comparison.

use std::sync::Mutex;
use kalosm::language::{Bert};
use tokio::sync::OnceCell;

/// Global singleton for the BERT embedding model
/// Uses OnceCell and Mutex for thread-safe access and initialization
pub static EMBEDDING_MODEL: OnceCell<Mutex<Bert>> = OnceCell::const_new();

/// Initializes the BERT embedding model
///
/// This function:
/// 1. Checks if the model is already initialized
/// 2. If not, creates a new Bert model instance
/// 3. Stores the model in the global singleton
///
/// The embedding model is used to convert text into vector representations
/// that capture semantic meaning, which enables similarity-based searches.
///
/// Returns Ok(()) on success or an error message on failure
pub async fn init_embedding_model() -> Result<(), String> {
    use kalosm::language::{Bert, BertSource, FileSource};

    if EMBEDDING_MODEL.get().is_none() {
        println!("Initializing embedding model...");
        let bert = Bert::new().await.map_err(|e| e.to_string());
        println!("Embedding model loaded successfully");
        EMBEDDING_MODEL.set(Mutex::new(bert?))
            .map_err(|_| "Couldn't set embedding model".to_string())?;
    }
    Ok(())
}

/// Converts input text into vector embeddings
///
/// This function:
/// 1. Accesses the global embedding model
/// 2. Generates vector embeddings for the provided text
/// 3. Returns the vector representation
///
/// The generated embeddings capture the semantic meaning of the text
/// and can be used for similarity comparisons and semantic search.
///
/// # Parameters
/// * `text` - The text to convert into embeddings
///
/// # Returns
/// * `Result<Vec<f32>, String>` - The embedding vector or an error message
pub async fn embed_text(text: &str) -> Result<Vec<f32>, String> {
    use kalosm::language::EmbedderExt;
    let embedding_model = EMBEDDING_MODEL
        .get()
        .ok_or("Embedding model not initialized")?
        .lock()
        .map_err(|_| "Error locking embedding model")?;

    let embeddings = embedding_model.embed(text)
        .await
        .map_err(|e| e.to_string())?;
    println!("Embedding generated for text: {:?}", embeddings.vector().to_vec());
    Ok(embeddings.vector().to_vec())
}