//! Document Model Definitions
//!
//! This module defines data structures for representing document search results.
//! These structures are used to store and transport document data retrieved from
//! the database when providing context for conversations.

use serde::{Deserialize, Serialize};

/// Represents a simplified document search result
///
/// This structure contains the essential information of a document retrieved
/// during context search operations, including:
/// - The document title
/// - The document body text
/// - A relevance score indicating how well the document matches the search query
///
/// The score is used to rank and filter documents based on their relevance to
/// the current conversation context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleDocumentResult {
    /// The title of the document
    pub title: String,

    /// The main text content of the document
    pub body: String,

    /// A floating-point score representing the document's relevance
    /// Higher values indicate greater relevance to the search query
    pub score: f32,
}
