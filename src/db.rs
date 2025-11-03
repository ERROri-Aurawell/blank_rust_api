// Modelo de criação de pool de conexões com o banco de dados MySQL usando SQLx.
use sqlx::MySqlPool;
use sqlx::mysql::MySqlPoolOptions;

// Para carregar variáveis de ambiente do arquivo .env (pasta raiz do projeto) ou do ambiente do sistema
use dotenvy::dotenv;
use std::env;

// Função assíncrona para criar e retornar um pool de conexões MySQL
pub async fn create_pool() -> Result<MySqlPool, sqlx::Error> {
    // Carrega as variáveis de ambiente do arquivo .env (se existir)
    dotenv().ok();

    // Lê a variável de ambiente DATABASE_URL
    let database_url = env::var("DATABASE_URL").map_err(|e| {
        eprintln!("Erro ao ler DATABASE_URL: {}", e);

        // Retorna um erro sqlx genérico se a variável não estiver definida
        sqlx::Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "DATABASE_URL not set",
        ))
    })?;

    // Cria o pool de conexões com as configurações desejadas
    let pool = MySqlPoolOptions::new()
        // Define o número máximo de conexões no pool
        .max_connections(10)
        // Conecta ao banco de dados usando a URL fornecida
        .connect(&database_url)
        .await?;
    Ok(pool)
}
