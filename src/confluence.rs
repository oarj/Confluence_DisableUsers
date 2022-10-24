use crate::config::Config;
use reqwest::Response;

// Perform the actual login request, this will create a valid JSESSIONID for future requests.
pub async fn login(
    client: &reqwest::Client,
    cfg: &Config,
) -> Result<Response, Box<dyn std::error::Error>> {
    let params = [
        ("os_username", cfg.username.to_string()),
        ("os_password", cfg.password.to_string()),
        ("login", "Log in".to_string()),
        ("os_destination", "/".to_string()),
    ];

    let url = format!("{}/dologin.action", cfg.wiki_url);
    let resp = client
        .post(url)
        .header("X-Atlassian-Token", "no-check")
        .form(&params)
        .send()
        .await?;

    Ok(resp)
}

// This will activate websudo permissions
pub async fn websudo(
    client: &reqwest::Client,
    cfg: &Config,
) -> Result<Response, Box<dyn std::error::Error>> {
    let params = [
        ("destination", "/".to_string()),
        ("authenticate", "Confirm".to_string()),
        ("password", cfg.password.to_string()),
    ];
    let url = format!("{}/doauthenticate.action", cfg.wiki_url);

    let resp = client
        .post(url)
        .header("X-Atlassian-Token", "no-check")
        .form(&params)
        .send()
        .await?;

    Ok(resp)
}

pub async fn disable_user(
    client: &reqwest::Client,
    cfg: &Config,
    user: &str,
) -> Result<Response, Box<dyn std::error::Error>> {
    let params = [("username", user), ("confirm", "Disable")];
    let url = format!("{}/admin/users/deactivateuser-confirm.action", cfg.wiki_url);

    let resp = client
        .post(url)
        .header("X-Atlassian-Token", "no-check")
        .form(&params)
        .send()
        .await?;

    Ok(resp)
}
