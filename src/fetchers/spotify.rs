use super::base_api::BaseApi;
use crate::error::fetcher::http::HttpError;
use crate::error::fetcher::json::JsonError;
use crate::error::LyrixResult;
use crate::logger;
use crate::parsers::generate::spotify::build_totp;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct SpotifyApi {
    api: BaseApi,
}

impl SpotifyApi {
    pub async fn new(cookie: String) -> LyrixResult<Self> {
        init_spotify(&cookie, None).await
    }

    pub async fn with_client(client: reqwest::Client, cookie: String) -> LyrixResult<Self> {
        init_spotify(&cookie, Some(client)).await
    }

    /// 搜索歌曲
    pub async fn search(&self, keyword: &str) -> LyrixResult<Option<SearchResult1>> {
        let body = serde_json::json!({
            "variables": {
                "query": keyword,
                "limit": 30,
                "numberOfTopResults": 30,
                "offset": 0,
                "includeAuthors": false,
                "includeAlbumPreReleases": false,
                "includeEpisodeContentRatingsV2": false
            },
            "operationName": "searchSuggestions",
            "extensions": {
                "persistedQuery": {
                    "version": 1,
                    "sha256Hash": "556f5a15b2fdd3a7113ffd377ad9805e38a3a27b8bb1ca7d6d76bad54aa8ee12"
                }
            }
        });

        let resp = self
            .api
            .post_json_async("https://api-partner.spotify.com/pathfinder/v2/query", &body)
            .await?;

        let result: Option<SearchResult1> = serde_json::from_str(&resp).map_err(|e| JsonError {
            api: "SpotifySearch".to_string(),
            source: e,
        })?;
        Ok(result)
    }

    ///抓取歌词
    pub async fn get_lyrics(&self, id: &str) -> LyrixResult<String> {
        let url = format!(
            "https://spclient.wg.spotify.com/color-lyrics/v2/track/{}?format=json&market=from_token",
            urlencoding::encode(id)
        );
        let resp = self.api.get_async(&url).await?;
        Ok(resp)
    }
}

async fn init_spotify(
    cookie: &str,
    async_client: Option<reqwest::Client>,
) -> LyrixResult<SpotifyApi> {
    let start = std::time::Instant::now();
    logger::debug("provider::spotify", "initializing client tokens");

    let ts = build_totp(0)?;
    let totp = ts.generate_now();

    let token_url = format!(
        "https://open.spotify.com/api/token?reason=init&productType=web-player&totp={}&totpServer={}&totpVer={}",
        totp, totp, ts.version
    );

    let http = async_client.clone().unwrap_or_else(reqwest::Client::new);

    let token_resp = http
        .get(&token_url)
        .header("Referer", "https://open.spotify.com/")
        .header("User-Agent", super::base_api::USER_AGENT)
        .header("Cookie", cookie)
        .send()
        .await
        .map_err(|e| http_err(&token_url, &e))?
        .error_for_status()
        .map_err(|e| http_err(&token_url, &e))?
        .text()
        .await
        .map_err(|e| http_err(&token_url, &e))?;
    let token_result: TokenResult1 = serde_json::from_str(&token_resp).map_err(|e| JsonError {
        api: "SpotifyToken".to_string(),
        source: e,
    })?;

    let ct_body = ClientTokenRequest {
        client_data: ClientData {
            client_version: "1.2.91.72.g5337566e".to_string(),
            client_id: token_result.client_id.clone(),
            js_sdk_data: JsSdkData {
                device_brand: "unknown".to_string(),
                device_model: "unknown".to_string(),
                os: "windows".to_string(),
                os_version: "NT 10.0".to_string(),
                device_id: "325e4218-3239-4c14-9d62-39d4919b1570".to_string(),
                device_type: "computer".to_string(),
            },
        },
    };

    //不知道是不是没有options
    let options_url = "https://clienttoken.spotify.com/v1/clienttoken";
    http.request(reqwest::Method::OPTIONS, options_url)
        .header("Origin", "https://open.spotify.com")
        .header("Access-Control-Request-Method", "POST")
        .header("Access-Control-Request-Headers", "content-type")
        .send()
        .await
        .map_err(|e| http_err(options_url, &e))?
        .error_for_status()
        .map_err(|e| http_err(options_url, &e))?;

    let ct_resp = http
        .post("https://clienttoken.spotify.com/v1/clienttoken")
        .header("Accept", "application/json")
        .header("Origin", "https://open.spotify.com")
        .header("Referer", "https://open.spotify.com/")
        .header("User-Agent", super::base_api::USER_AGENT)
        .header("Content-Type", "application/json")
        .header("Cookie", cookie)
        .json(&ct_body)
        .send()
        .await
        .map_err(|e| http_err(options_url, &e))?
        .error_for_status()
        .map_err(|e| http_err(options_url, &e))?
        .text()
        .await
        .map_err(|e| http_err(options_url, &e))?;
    let client_token_result: ClientTokenResult1 =
        serde_json::from_str(&ct_resp).map_err(|e| JsonError {
            api: "SpotifyClientToken".to_string(),
            source: e,
        })?;
    //初始化baseapi的头
    let mut extra_headers = HashMap::new();
    extra_headers.insert(
        "Authorization".to_string(),
        format!("Bearer {}", token_result.access_token),
    );
    extra_headers.insert(
        "Client-Token".to_string(),
        client_token_result.granted_token.token.clone(),
    );
    extra_headers.insert("Origin".to_string(), "https://open.spotify.com".to_string());
    extra_headers.insert(
        "Referer".to_string(),
        "https://open.spotify.com/".to_string(),
    );
    extra_headers.insert(
        "User-Agent".to_string(),
        super::base_api::USER_AGENT.to_string(),
    );
    extra_headers.insert("App-platform".to_string(), "WebPlayer".to_string());

    let api = if let Some(c) = async_client {
        BaseApi::with_client(c, Some("https://open.spotify.com/"), Some(extra_headers))
    } else {
        BaseApi::new(Some("https://open.spotify.com/"), Some(extra_headers))
    };

    logger::debug(
        "provider::spotify",
        format_args!("client tokens initialized | elapsed={:?}", start.elapsed()),
    );

    Ok(SpotifyApi { api })
}

fn http_err(url: &str, e: &reqwest::Error) -> HttpError {
    if e.is_timeout() {
        HttpError::Timeout {
            url: url.to_string(),
        }
    } else if e.is_connect() {
        HttpError::ConnectionFailed {
            detail: e.to_string(),
            url: url.to_string(),
        }
    } else {
        HttpError::ConnectionFailed {
            detail: e.to_string(),
            url: url.to_string(),
        }
    }
}

// ===== Request Models =====

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ClientTokenRequest {
    client_data: ClientData,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ClientData {
    client_version: String,
    client_id: String,
    js_sdk_data: JsSdkData,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct JsSdkData {
    device_brand: String,
    device_model: String,
    os: String,
    os_version: String,
    device_id: String,
    device_type: String,
}

// ===== Response Models =====

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenResult1 {
    pub client_id: String,
    pub access_token: String,
    pub access_token_expiration_timestamp_ms: u64,
    pub is_anonymous: bool,
}

#[derive(Debug, Default, Deserialize)]
pub struct ClientTokenResult1 {
    pub granted_token: GrantedToken1,
}

#[derive(Debug, Default, Deserialize)]
pub struct GrantedToken1 {
    pub token: String,
    pub expires_after_seconds: u32,
    pub refresh_after_seconds: u32,
}

#[derive(Debug, Deserialize, Default)]
pub struct SearchResult1 {
    pub data: Option<SearchData1>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchData1 {
    pub search_v2: Option<SearchV21>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchV21 {
    pub top_results_v2: Option<Top1>,
}
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Top1 {
    pub items_v2: Option<Vec<SearchItem1>>,
}
#[derive(Debug, Deserialize, Default)]
pub struct SearchItem1 {
    pub item: Option<ItemWrapper1>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ItemWrapper1 {
    pub data: Option<TrackData1>,
}

#[derive(Debug, Deserialize, Default)]
pub struct TrackData1 {
    pub id: Option<String>,
    pub name: Option<String>,
    pub uri: Option<String>,
    pub artists: Option<Artists1>,
    pub duration: Option<Duration1>,

    #[serde(rename = "albumOfTrack")]
    pub album_of_track: Option<AlbumOfTrack1>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Artists1 {
    pub items: Option<Vec<ArtistItem1>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ArtistItem1 {
    pub profile: Option<ArtistProfile1>,
    pub uri: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ArtistProfile1 {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Duration1 {
    pub total_milliseconds: Option<u32>,
}

#[derive(Debug, Deserialize, Default)]
pub struct AlbumOfTrack1 {
    pub name: Option<String>,
    pub uri: Option<String>,
}
