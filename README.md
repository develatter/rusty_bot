# 🤖 Rusty Bot

<div align="center">

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![WebAssembly](https://img.shields.io/badge/WebAssembly-654FF0?style=for-the-badge&logo=webassembly&logoColor=white)
![License](https://img.shields.io/badge/License-GPL%20v3-blue.svg?style=for-the-badge)

*A modern AI chatbot built entirely in Rust with RAG capabilities*

[Features](#-features) •
[Installation](#-installation) •
[Usage](#-usage) •
[Development](#-development)

</div>

---

## 🎯 About

Rusty Bot is an open-source AI chatbot application that demonstrates the power of Rust in building modern web applications. This project serves as both a functional chatbot and an educational resource, showcasing how to integrate large language models with semantic search capabilities using a full-stack Rust approach.

**Academic Context**: This project is part of the Final Degree Project by Alejandro López Martínez, a student in the 2nd year of Multiplatform Application Development in Almería, Spain.

## ✨ Features

### 🧠 **AI-Powered Conversations**
- Integration with language models via [Kalosm](https://github.com/floneum/floneum)
- Real-time streaming responses for better user experience
- Conversation history management with reset functionality

### 🔍 **Semantic Search & RAG**
- Document embedding using BERT models
- Context-aware responses through Retrieval-Augmented Generation (RAG)
- Vector database powered by SurrealDB for semantic search

### 🎨 **Modern Web Interface**
- Responsive design built with Dioxus framework
- Real-time streaming text display
- Markdown support with syntax highlighting
- Dark theme optimized for extended use

### 🛠 **Full-Stack Rust Architecture**
- Use of Dioxus as full-stack framework.
- Type-safe communication between client and server
- Zero JavaScript dependencies (except for styling with TailwindCSS)


### Core Components

- **Frontend**: Dioxus-based reactive UI compiled to WebAssembly
- **Backend**: Axum server with Dioxus server functions
- **LLM Engine**: Kalosm integration with Qwen 2.5 7B model
- **Vector Database**: SurrealDB with embedding-based semantic search
- **Context System**: Markdown-based document ingestion with RAG

## 🚀 Installation

### Prerequisites

Ensure you have the following installed on your system:

#### 1. Rust Toolchain
```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the on-screen instructions and restart your terminal
# Verify installation
rustc --version
cargo --version
```

#### 2. Dioxus CLI
```bash
# Install Dioxus CLI for development
cargo install dioxus-cli

# Verify installation
dx --version
```

#### 3. System Requirements
- **RAM**: At least 8GB (16GB recommended for LLM operations)
- **Storage**: ~10GB free space for model downloads
- **OS**: Linux, macOS, or Windows with WSL2

### 🔧 Setup

1. **Clone the Repository**
   ```bash
   git clone https://github.com/your-username/rusty_bot.git
   cd rusty_bot
   ```

2. **Create Context Directory (if it doesn't exist)**
   ```bash
   mkdir -p context
   ```

3. **Add Your Documents** (Optional)
   
   Place your Markdown files in the `context/` directory. These will be used for RAG-based responses.

## 🎮 Usage

```bash
dx serve --platform web --release
```

### 💬 Using the Chatbot

1. **Initialize**: The application will automatically download and initialize the language model on first run (this may take several minutes)

2. **Chat**: Type your message in the text area and press Enter or click Send

3. **Context Toggle**: Enable the "Context" toggle to use RAG-based responses with your documents

4. **Reset**: Click the reset button (↻) in the top-left to start a new conversation

## 🛠 Development

### Project Structure

```
rusty_bot/
├── src/
│   ├── components/          # UI components
│   │   ├── conversation.rs  # Main chat interface
│   │   └── message.rs       # Individual message rendering
│   ├── model/               # Data models
│   │   ├── chat.rs          # Chat message structures
│   │   └── document.rs      # Document result structures
│   ├── server/              # Server-side modules
│   │   ├── llm.rs           # Language model integration
│   │   ├── embedding.rs     # Text embedding functionality
│   │   └── database_impl.rs # Database operations
│   ├── server_functions/    # Dioxus server functions
│   └── main.rs              # Application entry point
├── context/                 # Knowledge base documents
├── assets/                  # Static assets
└── Cargo.toml              # Project dependencies
```

### Adding New Features

1. **Server Functions**: Add new server functions in `src/server_functions/`
2. **UI Components**: Create new components in `src/components/`
3. **Models**: Define data structures in `src/model/`
4. **Context**: Add knowledge base documents to `context/`


## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## 📄 License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Kalosm](https://github.com/floneum/floneum) - For providing excellent Rust LLM integration
- [Dioxus](https://dioxuslabs.com/) - For the amazing full-stack Rust framework
- [SurrealDB](https://surrealdb.com/) - For the powerful multi-model database
- The Rust community for creating such amazing tools and libraries

---

<div align="center">
Made with ❤️ and 🦀 by Alejandro López Martínez (develatter)
</div>


