pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub max_db_connections: u32,
    pub argon2_memory: u32, // TODO: maybe there is a better way to handle this
    pub argon2_iteration: u32,
    pub argon2_parallelism: u32,
}

// TODO : Create config error
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
            argon2_memory: std::env::var("ARGON2_MEMORY")
                .unwrap_or_else(|_| "19456".into())
                .parse()
                .expect("ARGON2_MEMORY must be a u32 between ARGON2_PARALLELISM * 8 and (2^32)-1"),
            argon2_iteration: std::env::var("ARGON2_ITERATION")
                .unwrap_or_else(|_| "2".into())
                .parse()
                .expect("ARGON2_ITERATION must be a u32 between 1 and (2^32)-1"),
            argon2_parallelism: std::env::var("ARGON2_PARALLELISM")
                .unwrap_or_else(|_| "1".into())
                .parse()
                .expect("ARGON2_PARALLELISM must be a u32 between 1 and (2^24)-1"),
        }
    }
}

// let mut builder = ParamsBuilder::new();
// let argon_param = builder
//     .m_cost(19456)
//     .t_cost(2)
//     .p_cost(1)
//     .output_len(32)
//     .build()
//     .unwrap();
