use actix_web::web::Query;
use actix_web::{get, App, HttpRequest, HttpServer, Responder};

use serde::Deserialize;

use biliroaming_rust_simple::{get_uid, get_url, BiliKey, BiliKeyWeb, BiliRomingError, UserInfo};

use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, Mutex};

#[get("/pgc/player/api/playurl")]
async fn cnplayurl(
    req: HttpRequest,
    key: Query<BiliKey>,
) -> Result<impl Responder, BiliRomingError> {
    let (client, white_list, cache_map) = req
        .app_data::<(
            reqwest::Client,
            Arc<[u32]>,
            Arc<Mutex<HashMap<String, UserInfo>>>,
        )>()
        .unwrap();

    let key = key.into_inner();
    let user_agent = req
        .headers()
        .get("user-agent")
        .ok_or(BiliRomingError::FailedMakeRequest)?
        .to_str()
        .map_err(|_| BiliRomingError::FailedMakeRequest)?;

    let user_info = get_uid(client, cache_map, &key, user_agent).await?;
    if white_list.binary_search(&user_info.mid).is_err() {
        return Err(BiliRomingError::BlockRequest(user_info.mid));
    }

    let body = get_url(
        client,
        &format!(
            "https://api.bilibili.com/pgc/player/api/playurl?{}",
            req.query_string(),
        ),
        user_agent,
    )
    .await?;

    log::debug!("Get response: {}", body);
    Ok(body)
}

#[get("/pgc/player/web/playurl")]
async fn cnplayurl_web(
    req: HttpRequest,
    key: Query<BiliKeyWeb>,
) -> Result<impl Responder, BiliRomingError> {
    let (client, white_list, cache_map) = req
        .app_data::<(
            reqwest::Client,
            Arc<[u32]>,
            Arc<Mutex<HashMap<String, UserInfo>>>,
        )>()
        .unwrap();

    let key: BiliKey = key.into_inner().into();
    let user_agent = req
        .headers()
        .get("user-agent")
        .ok_or(BiliRomingError::FailedMakeRequest)?
        .to_str()
        .map_err(|_| BiliRomingError::FailedMakeRequest)?;

    let user_info = get_uid(client, cache_map, &key, user_agent).await?;
    if white_list.binary_search(&user_info.mid).is_err() {
        return Err(BiliRomingError::BlockRequest(user_info.mid));
    }

    let body = get_url(
        client,
        &format!(
            "https://api.bilibili.com/pgc/player/web/playurl?{}",
            req.query_string(),
        ),
        user_agent,
    )
    .await?;

    log::debug!("Get response: {}", body);
    Ok(body)
}

#[derive(Debug, Deserialize)]
struct Config {
    address: String,
    port: u16,
    users: Vec<u32>,
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let mut config_file = std::fs::File::open("config.json")?;
    let mut raw_contents = String::new();
    config_file.read_to_string(&mut raw_contents)?;
    let mut config: Config = serde_json::from_str(&raw_contents).map_err(|err| {
        log::error!("Failed parse json {}", err);
        std::io::ErrorKind::Other
    })?;
    config.users.sort();

    let client = reqwest::Client::new();
    let white_list: Arc<[u32]> = config.users.into();
    let cache_map: Arc<Mutex<HashMap<String, UserInfo>>> = Arc::new(Mutex::new(HashMap::new()));
    HttpServer::new(move || {
        App::new()
            .app_data((client.clone(), white_list.clone(), cache_map.clone()))
            .service(cnplayurl)
    })
    .bind((config.address, config.port))?
    .run()
    .await
}
