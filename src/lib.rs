use actix_web::error::ResponseError;
use actix_web::http::{header::ContentType, StatusCode};
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

use std::fmt::Display;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize)]
pub enum BiliRomingError {
    BlockRequest(u32),
    FailedGetSecertKey,
    FailedMakeRequest,
    FailedParseResponse,
    WrongRequest,
    WrongResponse(i32, String),
}

#[derive(Debug, Deserialize)]
pub struct BiliKey {
    pub access_key: String,
    pub appkey: String,
}

#[derive(Debug, Deserialize)]
pub struct BiliKeyWeb {
    pub access_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub mid: u32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BiliResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<UserInfo>,
}

impl From<BiliKeyWeb> for BiliKey {
    fn from(key: BiliKeyWeb) -> BiliKey {
        BiliKey {
            access_key: key.access_key,
            appkey: "560c52ccd288fed045859ed18bffd973".to_string(),
        }
    }
}

impl Display for BiliRomingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ResponseError for BiliRomingError {
    fn error_response(&self) -> HttpResponse {
        let rep = match self {
            BiliRomingError::BlockRequest(uid) => BiliResponse {
                code: -1403,
                message: format!("来自 {} 的请求被禁止", uid),
                data: None,
            },
            BiliRomingError::FailedGetSecertKey => BiliResponse {
                code: -1403,
                message: "无法获取 Secert Key".to_string(),
                data: None,
            },
            BiliRomingError::FailedMakeRequest => BiliResponse {
                code: -1403,
                message: "无法发送请求".to_string(),
                data: None,
            },
            BiliRomingError::FailedParseResponse => BiliResponse {
                code: -1403,
                message: "无法解析响应".to_string(),
                data: None,
            },
            BiliRomingError::WrongRequest => BiliResponse {
                code: -1403,
                message: "非法请求".to_string(),
                data: None,
            },
            BiliRomingError::WrongResponse(code, message) => BiliResponse {
                code: *code,
                message: message.to_string(),
                data: None,
            },
        };
        log::warn!("Processing Failed: {:?}", rep);
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(serde_json::to_string(&rep).unwrap())
    }
    fn status_code(&self) -> StatusCode {
        actix_web::http::StatusCode::OK
    }
}

pub async fn get_url(
    client: &reqwest::Client,
    url: &str,
    ua: &str,
) -> Result<String, BiliRomingError> {
    client
        .get(url)
        .header("user-agent", ua)
        .send()
        .await
        .map_err(|_| BiliRomingError::FailedMakeRequest)?
        .text()
        .await
        .map_err(|_| BiliRomingError::FailedMakeRequest)
}

pub async fn get_uid(
    client: &reqwest::Client,
    cache: &Arc<Mutex<HashMap<String, UserInfo>>>,
    key: &BiliKey,
    ua: &str,
) -> Result<UserInfo, BiliRomingError> {
    use chrono::Utc;

    if let Some(info) = cache.lock().unwrap().get(&key.access_key) {
        log::info!("cache hinted: {:?}", info);
        return Ok(info.clone());
    }

    let ts = Utc::now().timestamp();
    let secert_key = get_secert_key(&key.appkey)?;

    let origin_query = format!(
        "access_key={}&appkey={}&ts={}",
        key.access_key, key.appkey, ts
    );
    let sign = md5::compute(format!("{}{}", origin_query, secert_key));
    let sign_query = format!("{}&sign={:x}", origin_query, sign);

    let rep_body = get_url(
        client,
        &format!(
            "https://app.bilibili.com/x/v2/account/myinfo?{}",
            sign_query
        ),
        ua,
    )
    .await?;

    let rep: BiliResponse =
        serde_json::from_str(&rep_body).map_err(|_| BiliRomingError::FailedParseResponse)?;

    if rep.code == 0 {
        let info = rep.data.ok_or(BiliRomingError::FailedParseResponse)?;
        cache
            .lock()
            .unwrap()
            .insert(key.access_key.clone(), info.clone());

        log::info!("cached: ({},{:?})", key.access_key, info);
        Ok(info)
    } else {
        Err(BiliRomingError::WrongResponse(rep.code, rep.message))
    }
}

// Source: https://github.com/pchpub/BiliRoaming-Rust-Server/blob/v0.2.12/src/mods/get_user_info.rs#L176
fn get_secert_key(appkey: &str) -> Result<String, BiliRomingError> {
    match appkey {
        "9d5889cf67e615cd" => Ok("8fd9bb32efea8cef801fd895bef2713d".to_string()), // Ai4cCreatorAndroid
        "1d8b6e7d45233436" => Ok("560c52ccd288fed045859ed18bffd973".to_string()), // Android
        "07da50c9a0bf829f" => Ok("25bdede4e1581c836cab73a48790ca6e".to_string()), // AndroidB
        "8d23902c1688a798" => Ok("710f0212e62bd499b8d3ac6e1db9302a".to_string()), // AndroidBiliThings
        "dfca71928277209b" => Ok("b5475a8825547a4fc26c7d518eaaa02e".to_string()), // AndroidHD
        "bb3101000e232e27" => Ok("36efcfed79309338ced0380abd824ac1".to_string()), // AndroidI
        "4c6e1021617d40d9" => Ok("e559a59044eb2701b7a8628c86aa12ae".to_string()), // AndroidMallTicket
        "c034e8b74130a886" => Ok("e4e8966b1e71847dc4a3830f2d078523".to_string()), // AndroidOttSdk
        "4409e2ce8ffd12b8" => Ok("59b43e04ad6965f34319062b478f83dd".to_string()), // AndroidTV
        "37207f2beaebf8d7" => Ok("e988e794d4d4b6dd43bc0e89d6e90c43".to_string()), // BiliLink
        "9a75abf7de2d8947" => Ok("35ca1c82be6c2c242ecc04d88c735f31".to_string()), // BiliScan
        "7d089525d3611b1c" => Ok("acd495b248ec528c2eed1e862d393126".to_string()), // BstarA
        "178cf125136ca8ea" => Ok("34381a26236dd1171185c0beb042e1c6".to_string()), // AndroidB
        "27eb53fc9058f8c3" => Ok("c2ed53a74eeefe3cf99fbd01d8c9c375".to_string()), // ios
        "57263273bc6b67f6" => Ok("a0488e488d1567960d3a765e8d129f90".to_string()), // Android
        "7d336ec01856996b" => Ok("a1ce6983bc89e20a36c37f40c4f1a0dd".to_string()), // AndroidB
        "85eb6835b0a1034e" => Ok("2ad42749773c441109bdc0191257a664".to_string()), // unknown
        "84956560bc028eb7" => Ok("94aba54af9065f71de72f5508f1cd42e".to_string()), // unknown
        "8e16697a1b4f8121" => Ok("f5dd03b752426f2e623d7badb28d190a".to_string()), // AndroidI
        "aae92bc66f3edfab" => Ok("af125a0d5279fd576c1b4418a3e8276d".to_string()), // PC	投稿工具
        "ae57252b0c09105d" => Ok("c75875c596a69eb55bd119e74b07cfe3".to_string()), // AndroidI
        "bca7e84c2d947ac6" => Ok("60698ba2f68e01ce44738920a0ffe768".to_string()), // login
        "4ebafd7c4951b366" => Ok("8cb98205e9b2ad3669aad0fce12a4c13".to_string()), // iPhone
        "iVGUTjsxvpLeuDCf" => Ok("aHRmhWMLkdeMuILqORnYZocwMBpMEOdt".to_string()), //Android	取流专用
        "YvirImLGlLANCLvM" => Ok("JNlZNgfNGKZEpaDTkCdPQVXntXhuiJEM".to_string()), //ios	取流专用
        //_ => Ok("560c52ccd288fed045859ed18bffd973".to_string()),
        _ => Err(BiliRomingError::FailedGetSecertKey),
    }
}
