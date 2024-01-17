use crate::util::is_valid_api_key;
use crate::API_BASE_URL;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::ErrorResponse;
use log::error;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// ------- Whereis -------
enum SearchType {
    Name(String),
    Uuid(String),
}

pub async fn route_whereis_csv(
    Query(params): Query<HashMap<String, String>>,
) -> axum::response::Result<(StatusCode, String)> {
    let Some(api_key) = params.get("api_key") else {
        return Ok((
            StatusCode::UNAUTHORIZED,
            String::from("Please provide an API key using the api_key query parameter."),
        ));
    };
    if !is_valid_api_key(api_key)
        .await
        .map_err(|e| format!("Failed to send request to /user_info: {e}"))?
    {
        return Ok((StatusCode::UNAUTHORIZED, String::from("Invalid API key")));
    }
    let search_type = match (params.get("name"), params.get("uuid")) {
        (Some(name), None) => SearchType::Name(name.to_owned()),
        (None, Some(uuid)) => SearchType::Uuid(uuid.to_owned()),
        (Some(_), Some(_)) => {
            return Ok((
                StatusCode::BAD_REQUEST,
                String::from("You cannot provide a name and a UUID at the same time!"),
            ))
        }
        (None, None) => {
            return Ok((
                StatusCode::BAD_REQUEST,
                String::from("Please provide either name or uuid in the query!"),
            ))
        }
    };
    let body = match search_type {
        SearchType::Name(name) => format!("{{\"name\": \"{name}\"}}"),
        SearchType::Uuid(uuid) => format!("{{\"uuid\": \"{uuid}\"}}"),
    };
    let res = match reqwest::Client::new()
        .post(format!("{API_BASE_URL}/whereis"))
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .header(CONTENT_TYPE, "application/json")
        .body(body.to_string())
        .send()
        .await
    {
        Ok(res) => res,
        Err(e) => {
            error!("{e}");
            return Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error sending request to /whereis: {e}"),
            ));
        }
    };
    let json: Value = res
        .json()
        .await
        .map_err(|e| format!("Failed to deserialize json: {e}"))?;
    if let Some(e) = json.get("error") {
        return Err(e.as_str().unwrap().to_owned().into());
    }
    let mut csv = String::from("server,name,uuid,last_seen\n");
    for record in json["data"].as_array().unwrap() {
        csv.push_str(&format!(
            "{},{},{},{}\n",
            record["server"].as_str().unwrap(),
            record["name"].as_str().unwrap(),
            record["uuid"].as_str().unwrap(),
            record["last_seen"].as_u64().unwrap(),
        ));
    }
    Ok((StatusCode::OK, csv))
}

// ------- Servers -------
pub async fn route_servers_csv(
    Query(params): Query<HashMap<String, String>>,
) -> axum::response::Result<(StatusCode, String)> {
    let Some(api_key) = params.get("api_key") else {
        return Ok((
            StatusCode::UNAUTHORIZED,
            String::from("Please provide an API key using the api_key query parameter."),
        ));
    };
    if !is_valid_api_key(api_key)
        .await
        .map_err(|e| format!("Failed to send request to /user_info: {e}"))?
    {
        return Ok((StatusCode::UNAUTHORIZED, String::from("Invalid API key")));
    }
    let mut json_string = String::from("{");
    if let Some(online_players) = params.get("online_players") {
        if let Ok(num) = online_players.parse::<usize>() {
            json_string.push_str(&format!("\"online_players\":{num},"));
        }
        if online_players.contains('-') {
            if let Some((min, max)) = online_players.split_once('-') {
                if let (Ok(min), Ok(max)) = (min.parse::<usize>(), max.parse::<usize>()) {
                    json_string.push_str(&format!("\"online_players\": [{min}, {max}],"));
                }
            };
        }
    }
    if let Some(s) = json_string.strip_suffix(",") {
        json_string = s.to_string()
    }
    json_string.push('}');
    let res = match reqwest::Client::new()
        .post(format!("{API_BASE_URL}/servers"))
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .header(CONTENT_TYPE, "application/json")
        .body(json_string.to_string())
        .send()
        .await
    {
        Ok(res) => res,
        Err(e) => {
            error!("{e}");
            return Err(format!("Error sending request: {e}").into());
        }
    };
    let json: Value = res
        .json()
        .await
        .map_err(|e| format!("Failed to deserialize json: {e}"))?;
    if let Some(e) = json.get("error") {
        return Err(e.as_str().unwrap().to_owned().into());
    }
    let mut csv = String::from(
        "server,cracked,description,last_seen,max_players,online_players,protocol,version\n",
    );
    for record in json["data"].as_array().unwrap() {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            record["server"].as_str().unwrap(),
            record["cracked"]
                .as_bool()
                .map(|b| b.to_string())
                .unwrap_or_else(|| "null".to_string()),
            record["description"]
                .as_str()
                .unwrap()
                .replace(',', "\\,")
                .replace('\n', "\\n"),
            record["last_seen"].as_u64().unwrap(),
            record["max_players"].as_i64().unwrap(),
            record["online_players"].as_i64().unwrap(),
            record["protocol"].as_i64().unwrap(),
            record["version"].as_str().unwrap()
        ));
    }

    Ok((StatusCode::OK, csv))
}
