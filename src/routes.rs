// Modelo de rotas e handlers para Socket.IO usando socketioxide

// Importações necessárias:

// Serde para serialização/deserialização JSON
use serde::{Deserialize, Serialize};

// Serde JSON Value
use serde_json::Value;

// Socket.IO (Socketioxide)
use socketioxide::{
    SocketIo,
    extract::{Data, SocketRef},
};

// SQLx para conexão com MySQL
use sqlx::MySqlPool;

// Estrutura de dados para mensagens
#[derive(Serialize, Deserialize)]
struct Message {
    status: String,
    content: String,
}

// Modelo de evento brodcast para todos os clientes conectados
async fn broadcast_event(io: SocketIo) {
    io.emit("broadcast_event", "Um novo usuário conectou!")
        .await
        .ok();
}

// Modelo handler para evento de conexão Socket.IO
pub fn on_connect(socket: SocketRef, Data(data): Data<Value>, pool: MySqlPool, io: SocketIo) {
    println!("Socket.IO conectado: {:?} {:?}", socket.ns(), socket.id);
    println!("Dados de autenticação: {:?}", data);

    // Envia uma mensagem de boas-vindas ao cliente conectado
    // socket.emit( evento, dados )
    socket
        .emit("response", "Conexão estabelecida com sucesso!")
        .ok();

    // Exemplo de como usar o `io` para broadcast
    // tokio::spawn é necessário porque broadcast_event é async
    tokio::spawn(broadcast_event(io));

    // Para usar o pool dentro dos handlers de eventos, precisamos cloná-lo.
    // `MySqlPool` é um `Arc`, então clonar é barato.
    let pool_clone = pool.clone();

    // Define o handler para o evento "salvar_mensagem"
    socket.on(
        "salvar_mensagem",
        // Handler assíncrono para o evento "salvar_mensagem"

        // move | socket, data | async move {} é a sintaxe de closure assíncrona
        move |socket: SocketRef, Data::<Value>(data)| async move {
            println!("Recebido evento 'salvar_mensagem' com dados: {:?}", data);

            // Tenta deserializar o Value na nossa struct Message
            match serde_json::from_value::<Message>(data) {
                // Se deserialização for bem-sucedida
                Ok(payload) => {
                    println!("Payload deserializado com sucesso: {:?}", payload.content);

                    //AVISO
                    //RustAnalyzer tenta conectar previamente, ignorá-lo se houver esse erro:
                    //error returned from database: 1045 (28000): Access denied for user 'root'@'localhost' (using password: YES)
                    //Insira suas credenciais do arquivo .env em ../.vscode/settings.json

                    //O motivo técnico se encontra no arquivo Analyzer.md

                    // Insere a mensagem no banco de dados
                    let result = sqlx::query!(
                        "INSERT INTO mensagens (m_status, mensagem) VALUES (?, ?)",
                        payload.status,
                        payload.content
                    )
                    .execute(&pool_clone)
                    .await;

                    // Verifica o resultado da inserção
                    match result {
                        Ok(_) => {
                            socket.emit("response", "Mensagem salva com sucesso!").ok();
                        }
                        Err(e) => {
                            eprintln!("Falha ao inserir mensagem: {}", e);
                            socket.emit("response", "Erro ao salvar a mensagem.").ok();
                        }
                    }
                }
                // Se deserialização falhar
                Err(e) => {
                    eprintln!("Erro ao deserializar payload: {}", e);
                    socket
                        .emit("response", "Erro: formato de dados inválido.")
                        .ok();
                }
            }
        },
    );
    // Fim do handler "message"
}
