use crate::API_BASE_URL;
use reqwest::header::AUTHORIZATION;
pub async fn is_valid_api_key(key: &str) -> Result<bool, reqwest::Error> {
    let res = reqwest::Client::new()
        .get(format!("{API_BASE_URL}/user_info"))
        .header(AUTHORIZATION, format!("Bearer {key}"))
        .send()
        .await?;

    Ok(res.status().eq(&200))
}
