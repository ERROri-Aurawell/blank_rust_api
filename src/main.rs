//Esse é um modelo blank de uma API em Rust usando Axum e Socket.IO, com conexão a um banco de dados MySQL.

// conexão com o módulo de banco de dados
mod db;
use db::create_pool;

// conexão com o módulo de rotas Socket.IO
mod routes;

// conexão com o módulo de estado (fila)
mod state;

mod state2;

// Importações necessárias :

// `AppState` é a struct que contém nosso estado compartilhado.
// `ClickEvent` é o tipo de mensagem que passaremos pelo canal.
// `Arc` permite o compartilhamento de estado entre tarefas.
// `Mutex` e `mpsc` são as primitivas de sincronização e comunicação assíncrona do Tokio.
use state::{AppState, ClickEvent};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

use state2::{AppState2, ClickQuantity};

// Mysql
use sqlx::MySqlPool;

// Serde para serialização/deserialização JSON
use serde::{Deserialize, Serialize};

// Axum framework
use axum::{
    Extension,
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
    // -> impl IntoResponse
    // use axum::http::StatusCode;
    // use axum::response::IntoResponse;
    // (StatusCode::INTERNAL_SERVER_ERROR, Json("Falha ao inserir mensagem no banco de dados."))

    println!(
        "Salvando mensagem com status: {}, conteúdo: {}",
        payload.status, payload.content
    );

    //AVISO
    //RustAnalyzer tenta conectar previamente, ignorá-lo se houver esse erro:
    //error returned from database: 1045 (28000): Access denied for user 'root'@'localhost' (using password: YES)
    //Insira suas credenciais do arquivo .env em .vscode/settings.json

    // Insere a mensagem no banco de dados
    let result = sqlx::query!(
        "INSERT INTO mensagens (m_status, mensagem) VALUES (?, ?)",
        payload.status,
        payload.content
    )
    .execute(&pool)
    .await;

    match result {
        Ok(_) => Json("Mensagem salva com sucesso!"),
        Err(e) => {
            eprintln!("Falha ao inserir mensagem no banco de dados: {}", e);
            Json("Falha ao inserir mensagem no banco de dados.")
        }
    }
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

    // Configuração do estado compartilhado da aplicação.
    // 1. `mpsc::channel` cria um canal de comunicação assíncrono com um buffer de 100 mensagens.
    //    `tx` (transmitter) é o remetente, `rx` (receiver) é o receptor.
    let (tx, mut rx) = mpsc::channel::<ClickEvent>(100);

    // 2. O contador é envolvido em um `Mutex` para garantir modificações seguras e em um `Arc`
    //    para permitir o compartilhamento entre múltiplas tarefas.
    let counter = Arc::new(Mutex::new(0));

    // 3. O `AppState` é criado, agrupando o remetente do canal (`tx`) e o contador.
    let state = AppState {
        // O remetente também é protegido por Arc e Mutex para ser compartilhado com segurança.
        sender: Arc::new(Mutex::new(tx)),
        counter: counter.clone(),
    };


    // Configuração do segundo estado compartilhado (state_2).
    // Esta estrutura é semelhante à primeira, mas gerencia um tipo de evento diferente (`ClickQuantity`),
    // que carrega um valor `i32`. Isso demonstra como a aplicação pode lidar com múltiplos
    // fluxos de eventos de forma independente.
    let (tx_2, mut rx_2) = mpsc::channel::<ClickQuantity>(100);
    let state_2 = AppState2 {
        sender: Arc::new(tx_2),
        // Este contador não está sendo usado atualmente, mas foi incluído na struct.
        counter: Arc::new(Mutex::new(0)), 
    };

    // Tenta criar o pool de conexões com o banco de dados.
    // Em caso de falha, entra em um loop de tentativas a cada 5 segundos.
    let pool = loop {
        match create_pool().await {
            Ok(pool) => {
                println!("Conexão com o banco de dados estabelecida com sucesso.");
                break pool;
            }
            Err(e) => {
                eprintln!(
                    "Falha ao conectar ao banco de dados: {}. Tentando novamente em 5 segundos... Verifique suas variaveis de ambiente",
                    e
                );
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
        }
    };

    // Inicializa a camada do SocketIo
    let (layer, io) = SocketIo::new_layer();

    // Clona o `io` para uso dentro do closure
    let io_clone = io.clone();
    let io_clone_2 = io.clone();
    let io_clone_3 = io.clone();

    // Clona a Pool para uso dentro do closure
    let pool_clone = pool.clone();

    // Clona os estados para serem movidos para o handler de conexão do Socket.IO.
    // `Arc` permite clonagens "baratas" que apenas incrementam a contagem de referências,
    // permitindo que o mesmo estado seja acessado de múltiplos locais concorrentemente.
    let state_clone = state.clone();
    let state_2_clone = state_2.clone();

    // Define o handler de conexão socket o namespace padrão "/"
    io.ns(
        "/",
        move |socket: SocketRef, data: Data<serde_json::Value>| {
            // Chama a lógica real do handler
            routes::on_connect(
                socket,
                data,
                pool_clone,
                io_clone,
                state_clone,
                state_2_clone,
            );
        },
    );

    // Esta tarefa (task) atua como o "consumidor" do nosso canal mpsc.
    // Ela roda em segundo plano, independente do servidor web.
    let io_for_task = state.clone();
    tokio::spawn(async move {
        // `rx.recv().await` espera de forma assíncrona por uma mensagem no canal.
        // O loop continua enquanto o canal estiver aberto e mensagens chegarem.
        while (rx.recv().await).is_some() {
            // Bloqueia o contador para incrementá-lo com segurança.
            let mut count = io_for_task.counter.lock().await;
            *count += 1;

            // Após incrementar, emite um evento "update" para todos os clientes
            // conectados via Socket.IO, enviando o novo valor do contador.
            println!("Novo valor: {}", count);
            io_clone_2
                .of("/")
                .unwrap()
                .emit("update", &serde_json::json!({ "value": *count }))
                .await
                .ok();
        }
    });

    // Tarefa (task) consumidora para o segundo canal (rx_2).
    // Ela aguarda por valores `ClickQuantity` (i32) enviados pelo evento "dynamic" do socket.
    tokio::spawn(async move {
        while let Some(valor) = rx_2.recv().await {
            println!("Valor recebido do evento 'dynamic': {valor}");
            // Aqui, o valor recebido é retransmitido para todos os clientes
            // através do evento "update_2". Uma lógica de negócio mais complexa
            // poderia ser implementada, como atualizar um contador ou estado no servidor.
            io_clone_3
            .of("/")
            .unwrap()
            .emit("update_2", &serde_json::json!({ "value": valor }))
            .await
            .ok();
        }
    });

    // Criar uma camada CORS permissiva para desenvolvimento
    let cors = CorsLayer::new()
        .allow_origin(Any) // Permite qualquer origem
        .allow_methods(Any) // Permite qualquer método (GET, POST, etc.)
        .allow_headers(Any); // Permite qualquer cabeçalho

    //1: Monta a aplicação Axum.
    //2: Adiciona as rotas HTTP "/" e "/save_message".
    //3: Adiciona a camada Socket.IO (`layer`).
    //4: Adiciona o pool de conexões do banco de dados como estado gerenciado por Axum (`with_state`).
    //5:
    //6: 
    let app = Router::new()
        .route("/", get(root))
        .route("/save_message", post(save_message))
        .layer(layer)
        .with_state(pool)
        .layer(Extension(state))
        .layer(Extension(state_2));

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
