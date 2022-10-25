use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct DBConfig {
    pub database: String,
    pub host: String,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub wiki_url: String,
    pub username: String,
    pub password: String,
    pub db: DBConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            wiki_url: "https://example.com".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            db: DBConfig {
                database: "".to_string(),
                host: "".to_string(),
                username: "".to_string(),
                password: "".to_string(),
            },
        }
    }
}
