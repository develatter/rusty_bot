//! # Rusty Bot
//! 
//! Main entry point for the Rusty Bot application.
//! This app uses Dioxus as a framework to create a reactive web user interface
//!     - Author: Alejandro López Martínez

use dioxus::prelude::*;
use components::{Conversation};

/// Module containing the UI components of the application
mod components;
/// Module containing the data model logic
mod model;
/// Module that handles server side components
mod server;
/// Module with server functions
mod server_functions;

/// Static resources used by the application
/// Favicon that will appear in the browser tab
const FAVICON: Asset = asset!("/assets/favicon.ico");
/// Tailwind CSS stylesheet for the interface design
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

/// Main function that launches the Dioxus application
/// with the App component as the root.
fn main() {
    dioxus::launch(App);
}

/// Root component of the application.
/// 
/// This component defines the basic structure of the HTML document,
/// including:
/// - Links to resources such as favicon and CSS styles
/// - Page body with dark background
/// - The Conversation component that handles the main interface
#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        body {
            class: "bg-[#0f1116]", // Dark background
            Conversation {}
        }
    }
}

