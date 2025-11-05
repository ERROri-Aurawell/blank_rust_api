use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

pub type ClickQuantity = i32;

pub type SharedSender = Arc<mpsc::Sender<ClickQuantity>>;

pub type SharedCounter = Arc<Mutex<u64>>;

#[derive(Debug)]
#[derive(Clone)]
pub struct AppState2 {
    pub sender: SharedSender,
    pub counter: SharedCounter,
}