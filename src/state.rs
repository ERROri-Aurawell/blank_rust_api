// `tokio::sync` fornece primitivas de concorrência para ambientes assíncronos.
// `Mutex` é usado para garantir acesso exclusivo a um dado compartilhado.
// `mpsc` (multi-producer, single-consumer) é um canal para enviar mensagens entre tarefas.
use tokio::sync::{Mutex, mpsc};

// `std::sync::Arc` (Atomic Reference Counting) é um ponteiro inteligente que permite
// compartilhar a posse de um valor entre múltiplos threads/tarefas de forma segura.
use std::sync::Arc;

// `ClickEvent` define o tipo de mensagem que será enviada pelo canal. `()` é um tipo
// de tamanho zero, útil quando apenas a ocorrência do evento importa, não o dado.
pub type ClickEvent = ();

// `SharedSender` é um tipo alias para um remetente de canal (`mpsc::Sender`)
// que é compartilhado (`Arc`) e protegido por um `Mutex`.
pub type SharedSender = Arc<Mutex<mpsc::Sender<ClickEvent>>>;

// `SharedCounter` é um tipo alias para um contador `u64` que pode ser compartilhado
// (`Arc`) e modificado de forma segura (`Mutex`).
pub type SharedCounter = Arc<Mutex<u64>>;

// `AppState` agrupa todo o estado compartilhado da aplicação.
// Derivar `Clone` é importante para que o estado possa ser facilmente
// distribuído para os handlers do Axum e Socket.IO. Clonar um `AppState`
// apenas clona os `Arc`s, não os dados internos, o que é uma operação barata.
#[derive(Clone)]
pub struct AppState {
    pub sender: SharedSender,
    pub counter: SharedCounter,
}