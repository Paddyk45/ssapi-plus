use crate::API_BASE_URL;
use reqwest::header::AUTHORIZATION;
use serde_json::json;

pub async fn is_valid_api_key(key: &str) -> Result<bool, reqwest::Error> {
    let res = reqwest::Client::new()
        .get(format!("{API_BASE_URL}/user_info"))
        .header(AUTHORIZATION, format!("Bearer {key}"))
        .send()
        .await?;

    Ok(res.status().eq(&200))
}

pub fn string_bool_value(val: impl AsRef<str>) -> serde_json::Value {
    match val.as_ref() {
        "true" | "yes" => json!(true),
        "false" | "no" => json!(false),
        _ => json!(null),
    }
}
