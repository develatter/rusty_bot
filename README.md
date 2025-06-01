# Rusty Bot Beta

## Getting Started

This guide will help you get the Rusty Bot Beta application up and running on your local machine.

## Prerequisites

Before you begin, ensure you have the following installed:

1.  **Rust**: If you don't have Rust installed, you can install it by running the following command in your terminal:
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
    Follow the on-screen instructions to complete the installation. 

2.  **Dioxus CLI**: Once Rust and Cargo are set up, install the Dioxus CLI by running:
    ```bash
    cargo install dioxus-cli
    ```

## Running the Application

To run the application in development mode with web support, navigate to the project directory and execute the following command:

```bash
dx serve --platform web --release
```

This will build and serve your Dioxus application, typically making it accessible via `http://localhost:8080` in your web browser.


