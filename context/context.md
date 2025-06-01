[Contexto del proyecto]
Rusty bot es un proyecto de código abierto.
El propósito de Rusty Bot es proporcionar una interfaz de comunicación entre un modelo de lenguaje entrenado y un
usuario.
Rusty bot está programado íntegramente en Rust, utilizando Dioxus como framework tanto para frontend como para backend.
El backend corre en un servidor usando como base la librería Axum, y el frontend compila a WebAssembly y usa TailwindCSS
para los estilos.
Para inferir respuestas desde el modelo de lenguaje, se ha usado Kalosm, una interfaz para modelos preentrenados.
El contexto se añade al bot a través de búsqueda semántica, implementada con las capacidades como base de datos
vectorial de SurrealDB.
Todo el stack del proyecto, salvo TailwindCSS, está escrito en Rust.
El proyecto forma parte del Trabajo de Fin de Grado de Alejandro López Martínez, estudiante de 2º de Desarrollo de
Aplicaciones Multiplataforma de Almería. 