//! Database Implementation
//!
//! This module provides the core database functionality for the application,
//! including connection management, document storage, and semantic search capabilities.
//! It leverages SurrealDB for document storage and kalosm for embedding-based search.

use kalosm::EmbeddingIndexedTableSearchResult;
use kalosm::language::*;
use kalosm::language::Embedding;  // Corrigiendo la importación de Embedding
use tokio::sync::{Mutex, OnceCell}; 
use surrealdb::{Connection, Surreal};
use surrealdb::engine::local::{Db, SurrealKv};
use crate::model::document::SimpleDocumentResult;
use std::path::{Path, PathBuf};

/// Global singleton for the database connection
/// Uses OnceCell and Mutex for thread-safe access and initialization
static DB_CONN: OnceCell<Mutex<Option<Surreal<Db>>>> = OnceCell::const_new();

/// Global singleton for the document table
/// Stores documents with embedding-based search capabilities
static DOCUMENT_TABLE: OnceCell<Mutex<Option<DocumentTable<Db>>>> = OnceCell::const_new();

/// Constants for database configuration
const DB_PATH: &str = "./db";
const DB_FILE: &str = "./db/temp.db";
const EMBEDDINGS_FILE: &str = "./db/embeddings.db";
const NAMESPACE: &str = "test";
const DATABASE: &str = "test";
const TABLE_NAME: &str = "documents";
const CONTEXT_FOLDER: &str = "./context";

/// Establishes a connection to the database and initializes the document table
///
/// This function coordinates the entire database setup process.
///
/// Returns Ok(()) on success or an error message on failure
pub async fn connect_to_database() -> Result<(), String> {
    // Initialize global singletons
    initialize_globals().await;
    
    // Clean old database files
    cleanup_database_files()?;

    // Connect to database
    let db = create_database_connection().await?;
    
    // Configure namespace and database
    configure_database(&db).await?;

    // Create document table
    let dt = create_document_table(&db).await?;
    
    // Store connections in singletons
    store_connections(db, dt).await;
    
    // Add documents to the database
    add_documents().await?;
    
    println!("Database connection setup completed successfully");
    Ok(())
}

/// Initializes the global OnceCell singletons with empty values
async fn initialize_globals() {
    DB_CONN.get_or_init(|| async { Mutex::new(None) }).await;
    DOCUMENT_TABLE.get_or_init(|| async { Mutex::new(None) }).await;
}

/// Cleans up existing database files
fn cleanup_database_files() -> Result<(), String> {
    let db_path = PathBuf::from(DB_PATH);
    if db_path.exists() {
        std::fs::remove_dir_all(&db_path).map_err(|e| {
            eprintln!("Error removing existing database: {}", e);
            e.to_string()
        })?;
        println!("Removed existing database files");
    } else {
        println!("No existing database found, creating a new one");
    }
    Ok(())
}

/// Creates a new database connection
async fn create_database_connection() -> Result<Surreal<Db>, String> {
    println!("Connecting to the database...");
    let db = Surreal::new::<SurrealKv>(DB_FILE)
        .await
        .map_err(|e| e.to_string())?;
    println!("Database connected successfully");
    Ok(db)
}

/// Configures the namespace and database settings
async fn configure_database(db: &Surreal<Db>) -> Result<(), String> {
    db.use_ns(NAMESPACE).use_db(DATABASE)
        .await
        .map_err(|e| {
            eprintln!("Error using namespace and database: {}", e);
            e.to_string()
        })
}

/// Creates the document table with semantic chunking
async fn create_document_table(db: &Surreal<Db>) -> Result<DocumentTable<Db>, String> {
    println!("Creating document table...");
    let dt = db.document_table_builder(TABLE_NAME)
        .with_chunker(SemanticChunker::default())
        .at(EMBEDDINGS_FILE)
        .build::<Document>()
        .await
        .map_err(|e| {
            eprintln!("Error creating document table: {}", e);
            e.to_string()
        })?;
    println!("Document table created successfully");
    Ok(dt)
}

/// Stores database connections in the global singletons
async fn store_connections(db: Surreal<Db>, dt: DocumentTable<Db>) {
    {
        let mut db_guard = DB_CONN.get().unwrap().lock().await;
        *db_guard = Some(db);
    }
    
    {
        let mut dt_guard = DOCUMENT_TABLE.get().unwrap().lock().await;
        *dt_guard = Some(dt);
    }
}

/// Loads documents from the context folder and adds them to the document table
///
/// Returns Ok(()) on success or an error message on failure
async fn add_documents() -> Result<(), String> {
    println!("Adding documents to the table...");
    
    // Load documents from folder
    let raw_documents = load_documents_from_folder(CONTEXT_FOLDER)?;
    
    // Get document table reference
    let table = get_document_table().await?;

    // Process documents
    let documents = process_documents(raw_documents).await?;
    
    // Insert documents into table
    insert_documents(&table, documents).await?;
    
    println!("All documents added successfully");
    Ok(())
}

/// Loads documents from the specified folder path
fn load_documents_from_folder(folder_path: &str) -> Result<DocumentFolder, String> {
    DocumentFolder::try_from(PathBuf::from(folder_path))
        .map_err(|e| format!("Error loading documents from folder: {}", e))
}

/// Processes raw documents into Document objects
async fn process_documents(document_folder: DocumentFolder) -> Result<Vec<Document>, String> {
    document_folder.into_documents().await
        .map_err(|e| format!("Error processing documents: {}", e))
        .map(|docs| {
            docs.into_iter()
                .map(|doc| {
                    let title = doc.body().lines().next().unwrap_or("Unknown").to_string();
                    let body = doc.body().to_string();
                    Document::from_parts(title, body)
                })
                .collect()
        })
}

/// Inserts multiple documents into the document table
async fn insert_documents(table: &DocumentTable<Db>, documents: Vec<Document>) -> Result<(), String> {
    for document in documents {
        insert_single_document(table, document).await?;
    }
    Ok(())
}

/// Inserts a single document into the document table
async fn insert_single_document(table: &DocumentTable<Db>, document: Document) -> Result<(), String> {
    table.insert(document).await
        .map_err(|e| {
            eprintln!("Error adding document: {}", e);
            e.to_string()
        })?;
    Ok(())
}

/// Gets a reference to the document table from the global singleton
async fn get_document_table() -> Result<impl std::ops::Deref<Target = DocumentTable<Db>> + 'static, String> {
    let document_table_mutex_ref = DOCUMENT_TABLE
        .get()
        .ok_or("Document table not initialized")?;

    let table_guard = document_table_mutex_ref.lock().await;
    if table_guard.is_none() {
        return Err("Document table is None".to_string());
    }
    
    Ok(tokio::sync::MutexGuard::map(table_guard, |t| {
        t.as_mut().unwrap()
    }))
}

/// Performs a semantic search query against the document database
///
/// # Parameters
/// * `query` - The search query text
///
/// # Returns
/// * `Result<Vec<SimpleDocumentResult>, String>` - A vector of matching document results or an error
pub async fn query(query: &str) -> Result<Vec<SimpleDocumentResult>, String> {
    // Get document table
    let table = get_document_table().await?;
    
    // Create embedding from query
    let query_embed = create_embedding_from_query(&table, query).await?;
    
    // Perform semantic search
    let results = perform_semantic_search(&table, query_embed).await?;
    
    // Convert results to SimpleDocumentResult
    Ok(convert_search_results(results))
}

/// Creates an embedding vector from the query text
async fn create_embedding_from_query(
    table: &DocumentTable<Db>, 
    query: &str
) -> Result<Embedding, String> {
    table.embedding_model().embed(query).await.map_err(|e| {
        eprintln!("Error creating embedding: {}", e);
        e.to_string()
    })
}

/// Performs semantic search using the embedding vector
async fn perform_semantic_search(
    table: &DocumentTable<Db>,
    query_embed: Embedding
) -> Result<Vec<EmbeddingIndexedTableSearchResult<Document>>, String> {
    table.search(query_embed)
        .with_results(1)
        .await
        .map_err(|e| e.to_string())
}

/// Converts search results to SimpleDocumentResult objects
fn convert_search_results(
    results: Vec<EmbeddingIndexedTableSearchResult<Document>>
) -> Vec<SimpleDocumentResult> {
    results.into_iter().map(|doc_result| {
        SimpleDocumentResult {
            title: doc_result.record.title().to_string(),
            body: doc_result.record.body().to_string(),
            score: doc_result.distance,
        }
    }).collect()
}