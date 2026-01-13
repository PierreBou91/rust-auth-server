pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub max_db_connections: u32,
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/postgres".into()),
            server_host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            server_port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .expect("PORT must be a valid u16"),
            max_db_connections: std::env::var("MAX_DB_CONNECTIONS")
                .unwrap_or_else(|_| "20".into())
                .parse()
                .expect("MAX_DB_CONNECTIONS must be a valid u32"),
        }
    }
}
