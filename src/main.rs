mod config;
mod confluence;

use crate::config::{Args, Config};

use clap::Parser;
use tokio::fs;
use tokio_postgres::{Client, NoTls};

use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};

// Return true if user is active
async fn check_user_active(user: &str, db_client: &Client) -> bool {
    let row = db_client
        .query_opt("SELECT x.* FROM cwd_user x WHERE user_name = $1", &[&user])
        .await;

    match row {
        Ok(r) => match r {
            Some(r) => {
                let status: &str = r.get("active");
                match status {
                    // User is not disabled.
                    "T" => return true,
                    // User is disabled.
                    _ => return false,
                }
            }
            None => println!("User '{}' not found", user),
        },
        Err(e) => {
            // User most likely exists locally and in external auth platform.
            eprintln!("Error when querying for '{}': '{}'", user, e);
            return false;
        }
    }
    false
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load config file
    let cfg: Config = Figment::from(Serialized::defaults(Config::default()))
        .merge(Toml::file("settings.toml"))
        .extract()?;

    let args = Args::parse();

    // Create a new HTTP client, the client will also store cookies. This allows the client to be authenticated for future requests.
    let client = reqwest::Client::builder()
        .user_agent("Confluence-CLI")
        .cookie_store(true)
        .build()?;

    // Perform login and websudo auth.
    confluence::login(&client, &cfg).await?;
    confluence::websudo(&client, &cfg).await?;

    // load users from file
    let users = fs::read_to_string(args.file).await?;
    let total_users = users.trim().lines().count();
    println!("Loaded {} users.", total_users);

    // format connection flags for postgres
    let conn_config = format!(
        "host={} dbname={} user={} password={}",
        cfg.db.host, cfg.db.database, cfg.db.username, cfg.db.password
    );

    // create database client and connection.
    let (db_client, connection) = tokio_postgres::connect(conn_config.as_ref(), NoTls).await?;

    // allow connection to be used concurrently.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let mut disabled_users_count = 0;
    for user in users.lines() {
        // If user is active, try to disable it.
        if check_user_active(user, &db_client).await {
            confluence::disable_user(&client, &cfg, user).await?;
            // Check if user actually got disabled, this is done because confluence doesn't
            // return a helpful status code.
            if !check_user_active(user, &db_client).await {
                disabled_users_count += 1;
            } else {
                println!("Couldn't disable user '{}'", user)
            }
        } else {
            println!(
                "User '{}' is already disabled or couldn't be disabled.",
                user
            )
        }
    }

    println!("Disabled {}/{} users.", disabled_users_count, total_users);

    Ok(())
}
