mod config;
mod confluence;

use crate::config::Config;

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
        Ok(r) => match r {Some(r) => {
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

    // Create a new HTTP client, the client will also store cookies. This allows the client to be authenticated for future requests.
    let client = reqwest::Client::builder()
        .user_agent("Confluence-CLI")
        .cookie_store(true)
        .build()?;

    // Perform login and websudo auth.
    confluence::login(&client, &cfg).await?;
    confluence::websudo(&client, &cfg).await?;

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

    // get inactive users from SQL query
    let user_rows = db_client.query("
with users as
	(
	select cu.lower_user_name, count(cu.lower_user_name) user_count
	from cwd_user cu
	join cwd_directory cd on cd.id = cu.directory_id
	group by lower_user_name
	)
select distinct(cu.lower_user_name) as username
from logininfo li
join user_mapping um on li.username = um.user_key
join cwd_user cu on cu.user_name = um.username
join cwd_directory cd on cd.id = cu.directory_id
inner join users u on u.lower_user_name = cu.lower_user_name
where cu.active = 'T' and cu.lower_user_name != 'admin' and cd.lower_directory_name = 'confluence internal directory'
and (li.successdate < (current_date - integer '1825') or li.successdate is null) and cu.created_date < (current_date - integer '1825')
and u.user_count = 1
", &[]).await?;

    let mut disabled_users_count = 0;
    for row in user_rows.iter() {
        let user = row.get::<&str, &str>("username");
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

    println!("Disabled {}/{} users.", disabled_users_count, user_rows.len());

    Ok(())
}
