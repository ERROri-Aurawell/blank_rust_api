//Esse é um modelo blank de uma API em Rust usando Axum e Socket.IO, com conexão a um banco de dados MySQL.

// conexão com o módulo de banco de dados
mod db;
use db::create_pool;

// conexão com o módulo de rotas Socket.IO
mod routes;

// Importações necessárias :

// Mysql
use sqlx::MySqlPool;

// Serde para serialização/deserialização JSON
use serde::{Deserialize, Serialize};

// Axum framework
use axum::{
    Router,
    extract::Json as AxumJson, // Módulo que renomeia para evitar conflito com o Json do socketioxide
    extract::State,            // Módulo de variaveis "globais" (compartilhadas)
    response::{Html, Json},    // Tipos de resposta
    routing::{get, post},      // Métodos HTTP
};

//Socket.IO (Socketioxide)
use socketioxide::{
    SocketIo,
    extract::{Data, SocketRef},
};

//Cors
use tower_http::cors::{Any, CorsLayer};

// Modelo handler HTTP para a rota "/" (GET)
async fn root() -> Html<&'static str> {
    Html("<h1>Olá, mundo! Esta é a minha primeira API com Axum.</h1>\n <h1>Bem-vindo!</h1>")
}

// Modelo handler HTTP para salvar mensagem via JSON (POST)
async fn save_message(
    // State = "coneção" compartilhada do banco de dados
    State(pool): State<MySqlPool>,

    // Extrai o JSON do corpo da requisição e desserializa na struct Message
    AxumJson(payload): AxumJson<Message>,
) -> Json<&'static str> {
    println!(
        "Salvando mensagem com status: {}, conteúdo: {}",
        payload.status, payload.content
    );

    // Insere a mensagem no banco de dados
    sqlx::query!(
        "INSERT INTO mensagens (m_status, mensagem) VALUES (?, ?)",
        payload.status,
        payload.content
    )
    .execute(&pool)
    .await
    .expect("Falha ao inserir mensagem no banco de dados");

    // Retorna uma resposta de sucesso
    Json("Mensagem salva com sucesso!")
}

// Define a struct para montar e desmontar JSONs de mensagens
#[derive(Serialize, Deserialize)]
struct Message {
    status: String,
    content: String,
}


// Função principal + inicialização assíncrona (tokio)
#[tokio::main]
async fn main() {
    // Opcional, mas recomendado para ver os logs do socket.io
    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Cria o pool de conexões com o banco de dados MySQL
    let pool = create_pool()
        .await
        .expect("Failed to create database pool.");

    // Inicializa a camada do SocketIo
    let (layer, io) = SocketIo::new_layer();

    // Clona o `io` para uso dentro do closure
    let io_clone = io.clone();

    // Clona a Pool para uso dentro do closure
    let pool_clone = pool.clone();

    // Define o handler de conexão socket o namespace padrão "/"
    io.ns(
        "/",
        move |socket: SocketRef, data: Data<serde_json::Value>| {
            // Chama a lógica real do handler
            routes::on_connect(socket, data, pool_clone, io_clone);
        },
    );

    // Criar uma camada CORS permissiva para desenvolvimento
    let cors = CorsLayer::new()
        .allow_origin(Any) // Permite qualquer origem
        .allow_methods(Any) // Permite qualquer método (GET, POST, etc.)
        .allow_headers(Any); // Permite qualquer cabeçalho

    //1:Monta a aplicação Axum.
    //2:Adiciona as rotas HTTP "/" e "/save_message".
    //3:Adiciona a camada Socket.IO.
    //4:Adiciona a camada Pool do banco de dados
    let app = Router::new()
        .route("/", get(root))
        .route("/save_message", post(save_message))
        .layer(layer)
        .with_state(pool);

    //.route( rota, método( chamada ) ) adiciona rotas HTTP.
    //.layer( camada ) adiciona camadas (middleware) como Socket.IO, CORS
    //.with_state( variável ) adiciona variaveis "globais" (compartilhadas) para os handlers.
 
    // Inicia o servidor no endereço local, na porta 9000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:9000")
        .await
        .unwrap();
    println!(
        "Servidor rodando em:  http://{}",
        listener.local_addr().unwrap()
    );

    //Envolve a aplicação inteira (incluindo socket.io) com a camada CORS
    axum::serve(listener, app.layer(cors)).await.unwrap();
}
