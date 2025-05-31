use kalosm::EmbeddingIndexedTableSearchResult;
use kalosm::language::*;
use tokio::sync::{Mutex, OnceCell}; // Importa Mutex y OnceCell de tokio::sync
use surrealdb::{Connection, Surreal};
use surrealdb::engine::local::{Db, SurrealKv};
use crate::model::document::SimpleDocumentResult;


static DB_CONN: OnceCell<Mutex<Surreal<Db>>> = OnceCell::const_new();
static DOCUMENT_TABLE: OnceCell<Mutex<DocumentTable<Db>>> = OnceCell::const_new();

pub async fn connect_to_database() -> Result<(), String> {
    std::fs::remove_dir_all(std::path::PathBuf::from("./db"))
        .unwrap_or_else(|_| println!("No existing database found, creating a new one."));

    println!("Connecting to the database...");
    let db = Surreal::new::<SurrealKv>("./db/temp.db")
        .await
        .map_err(|e| e.to_string())?;

    println!("Database connected successfully");
    db.use_ns("test").use_db("test")
        .await
        .map_err(|e| {
            eprintln!("Error using namespace and database: {}", e);
            e.to_string()
        })?;

    println!("Using namespace and database");
    println!("Creating document table...");
    let dt = db.document_table_builder("documents")
        .with_chunker(SemanticChunker::default())
        .at("./db/embeddings.db")
        .build::<Document>()
        .await
        .map_err(|e| {
            eprintln!("Error creating document table: {}", e);
            e.to_string()
        })?;

    println!("Document table created successfully");
    println!("Setting up database connection...");
    DB_CONN.set(Mutex::new(db))
        .map_err(|_| {
            let err = "Failed to set database connection".to_string();
            eprintln!("{}", err);
            err
        })?;

    println!("Database connection set up successfully");
    println!("Setting up document table...");
    DOCUMENT_TABLE.set(Mutex::new(dt))
        .map_err(|_| {
            let err = "Failed to set document table".to_string();
            eprintln!("{}", err);
            err
        })?;
    println!("Document table set up successfully");
    println!("Adding documents to the table...");
    add_documents().await.map_err(|e| {
        eprintln!("Error adding documents: {}", e);
        e.to_string()
    })?;
    println!("Documents added successfully");
    Ok(())
}

async fn add_documents() -> Result<(), String> {
    let documents = DocumentFolder::try_from(std::path::PathBuf::from("./context")).unwrap();

    println!("Añadiendo documentos a la tabla...");

    let document_table_mutex_ref = DOCUMENT_TABLE
        .get()
        .ok_or("Document table not initialized")?;

    let mut table_guard = document_table_mutex_ref.lock().await;

    let context = documents.into_documents().await.unwrap()
        .into_iter()
        .map(|doc| {
            let title = doc.body().lines().next().unwrap_or("Unknown").to_string();
            let body = doc.body().to_string();
            Document::from_parts(title, body)
        }).collect::<Vec<Document>>();
    for d in context.into_iter() {
        let _ = table_guard.insert(d).await.map_err(|e| {
            eprintln!("Error adding document: {}", e);
            e.to_string()
        })?;
    }
    println!("Todos los documentos añadidos correctamente");
    Ok(())
}


pub async fn query(query: &str) -> Result<Vec<SimpleDocumentResult>, String> {
    let document_table_mutex_ref = DOCUMENT_TABLE
        .get()
        .ok_or("Document table not initialized")?;

    let document_table_guard = document_table_mutex_ref.lock().await;
    let query_embed = document_table_guard.embedding_model().embed(query).await.map_err(|e| {
        eprintln!("{}", e);
        e.to_string()
    })?;

    let results = document_table_guard.search(query_embed).with_results(1).await.map_err(|e| {
        e.to_string()
    })?;

    Ok(results.into_iter().map(|doc_result| {
        SimpleDocumentResult {
            title: doc_result.record.title().to_string(),
            body: doc_result.record.body().to_string(),
            score: doc_result.distance,
        }
    }).collect())
}