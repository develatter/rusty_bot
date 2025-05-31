use dioxus::prelude::{server, server_fn, ServerFnError};
use dioxus::prelude::server_fn::codec::{StreamingText, TextStream};

#[server]
pub async fn init_llm_model() -> Result<(), ServerFnError> {
    use crate::server::llm::init_chat_model;
    init_chat_model().await.map_err(|e| {
        ServerFnError::new(&format!("Error al inicializar el modelo: {}", e))
    })
}

#[server]
pub async fn init_embedding_model() -> Result<(), ServerFnError> {
    use crate::server::embedding::init_embedding_model;
    init_embedding_model().await.map_err(|e| {
        ServerFnError::new(&format!("Error al inicializar el modelo de embedding: {}", e))
    })
}


#[server]
pub async fn get_embedding(txt: String) -> Result<Vec<f32>, ServerFnError> {
    let result = tokio::task::spawn_blocking(move || {
        futures::executor::block_on(crate::server::embedding::embed_text(&txt))
    })
        .await
        .map_err(|e| ServerFnError::new(&e.to_string()))?;

    result.map_err(|e| ServerFnError::new(&format!("Error embedding text: {}", e)))
}


#[server]
pub async fn reset_chat() -> Result<(), ServerFnError> {
    use crate::server::llm::reset_chat;
    reset_chat().await.map_err(|e| ServerFnError::new(&format!("Error trying to reset chat: {}", e)))
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