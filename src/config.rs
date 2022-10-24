use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, required = true)]
    pub file: String,
}

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
