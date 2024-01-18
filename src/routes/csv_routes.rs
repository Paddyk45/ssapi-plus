use crate::util::{is_valid_api_key, string_bool_value};
use crate::API_BASE_URL;
use axum::extract::Query;
use axum::http::StatusCode;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{error, info};

// ------- Whereis -------
enum SearchType {
    Name(String),
    Uuid(String),
}

pub async fn route_whereis_csv(
    Query(params): Query<HashMap<String, String>>,
) -> axum::response::Result<(StatusCode, String)> {
    info!("Handling /whereis/csv");
    #[derive(Serialize, Deserialize)]
    struct Record {
        server: String,
        last_seen: String,
        name: String,
        uuid: String,
    }

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
        SearchType::Name(name) => json!({"name": name}),
        SearchType::Uuid(uuid) => json!({"uuid": uuid}),
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

    let mut wtr = csv::Writer::from_writer(vec![]);
    let records: Vec<Record> = serde_json::from_value(json["data"].clone()).unwrap();
    for record in records {
        wtr.serialize(record).unwrap();
    }
    let csv = String::from_utf8(wtr.into_inner().unwrap()).unwrap();

    Ok((StatusCode::OK, csv))
}

// ------- Servers -------
pub async fn route_servers_csv(
    Query(params): Query<HashMap<String, String>>,
) -> axum::response::Result<(StatusCode, String)> {
    info!("Handling /servers/csv");
    #[derive(Serialize, Deserialize)]
    struct Record {
        server: String,
        cracked: Option<bool>,
        description: String,
        last_seen: u32,
        max_players: i32,
        online_players: i32,
        protocol: i32,
        version: String,
    }

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

    let asn = params
        .get("asn")
        .map(|asn| asn.parse::<u16>().map_or(json!(null), |asn| json!(asn)));
    let country_code = params.get("country_code");
    let cracked = params.get("cracked").map(|val| string_bool_value(val));
    let description = params.get("description");
    let ignore_modded = params
        .get("ignore_modded")
        .map(|val| string_bool_value(val));
    let max_players = params
        .get("max_players")
        .map(|val| val.parse::<i32>().map_or(json!(null), |n| json!(n)));
    let online_players = params
        .get("online_players")
        .map(|val| val.parse::<i32>().map_or(json!(null), |n| json!(n)));
    let only_bungeespoofable = params
        .get("only_bungeespoofable")
        .map(|val| string_bool_value(val));
    let protocol = params
        .get("protocol")
        .map(|val| val.parse::<i32>().map_or(json!(null), |n| json!(n)));
    let version = params.get("version");

    let body = json!({
        "asn": asn,
        "country_code": country_code,
        "cracked": cracked,
        "description": description,
        "ignore_modded": ignore_modded,
        "max_players": max_players,
        "online_players": online_players,
        "only_bungeespoofable": only_bungeespoofable,
        "protocol": protocol,
        "version": version,
    });

    let res = match reqwest::Client::new()
        .post(format!("{API_BASE_URL}/servers"))
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .header(CONTENT_TYPE, "application/json")
        .body(body.to_string())
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

    let mut wtr = csv::Writer::from_writer(vec![]);
    let mut records: Vec<Record> = serde_json::from_value(json["data"].clone()).unwrap();
    records.iter_mut().for_each(|r| {
        r.description = r.description.replace('\n', "\\n");
        r.version = r.version.replace('\n', "\\n")
    });
    for record in records {
        wtr.serialize(record).unwrap();
    }
    let csv = String::from_utf8(wtr.into_inner().unwrap()).unwrap();

    Ok((StatusCode::OK, csv))
}
