use std::sync::Mutex;
use kalosm::language::{Bert};
use tokio::sync::OnceCell;

pub static EMBEDDING_MODEL: OnceCell<Mutex<Bert>> = OnceCell::const_new();

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