use super::base_api::BaseApi;
use crate::error::fetcher::http::HttpError;
use crate::error::fetcher::json::JsonError;
use crate::error::LyrixResult;
use crate::logger;
use serde::Deserialize;
use std::collections::HashMap;
pub const COOKIE: &str = "os=pc;osver=Microsoft-Windows-10-Professional-build-19045-64bit;appver=3.1.32.205206;channel=netease;__remember_me=true";
pub struct NeteaseFetcher {
    api: BaseApi,
}

impl NeteaseFetcher {
    fn netease_headers() -> HashMap<String, String> {
        let mut h = HashMap::new();
        h.insert("cookie".to_string(), COOKIE.to_string());
        h
    }

    pub fn new() -> Self {
        Self {
            api: BaseApi::new(
                Some("https://music.163.com/"),
                Some(Self::netease_headers()),
            ),
        }
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self {
            api: BaseApi::with_client(
                client,
                Some("https://music.163.com/"),
                Some(Self::netease_headers()),
            ),
        }
    }

    /// 搜索歌曲
    pub async fn search(&self, keyword: &str, search_type: i32) -> LyrixResult<SearchResult1> {
        let mut params = HashMap::new();
        params.insert("s".to_string(), keyword.to_string());
        params.insert("type".to_string(), search_type.to_string());
        params.insert("limit".to_string(), "20".to_string());
        params.insert("offset".to_string(), "0".to_string());

        let resp = self
            .api
            .post_form_async("https://music.163.com/api/search/get/web", &params)
            .await?;

        let parsed: SearchResult1 = serde_json::from_str(&resp).map_err(|e| JsonError {
            api: "NeteaseSearch".to_string(),
            source: e,
        })?;

        Ok(parsed)
    }

    /// 获取歌词
    pub async fn get_lyric(&self, id: &str) -> LyrixResult<LyricResult1> {
        let mut params = HashMap::new();
        params.insert("id".to_string(), id.to_string());
        params.insert("lv".to_string(), "-1".to_string());
        params.insert("kv".to_string(), "-1".to_string());
        params.insert("tv".to_string(), "-1".to_string());
        params.insert("rv".to_string(), "-1".to_string());
        params.insert("yv".to_string(), "-1".to_string());
        params.insert("ytv".to_string(), "-1".to_string());
        params.insert("yrv".to_string(), "-1".to_string());

        let resp = self
            .api
            .post_form_async(
                "https://interface3.music.163.com/api/song/lyric/v1",
                &params,
            )
            .await?;

        let parsed: LyricResult1 = serde_json::from_str(&resp).map_err(|e| JsonError {
            api: "NeteaseLyric".to_string(),
            source: e,
        })?;

        Ok(parsed)
    }

    /// 获取歌曲详情
    pub async fn get_detail(&self, id: &str) -> LyrixResult<Option<DetailResult1>> {
        let url = "/api/song/enhance/player/url/v1";
        let body = format!(
            r#"{{"ids":"[\"{id}\"]","level":"exhigh","encodeType":"aac","csrf_token":""}}"#
        );
        let p = crate::parsers::decrypt::netease::eapi_encrypt(url, &body)?;

        let endpoint = "https://music.163.com/eapi/song/enhance/player/url/v1";
        let start = std::time::Instant::now();
        let result = async {
            let client = reqwest::Client::builder()
                .user_agent("Mozilla/5.0")
                .build()
                .map_err(|e| HttpError::ConnectionFailed {
                    detail: e.to_string(),
                    url: endpoint.to_string(),
                })?;

            let res = client
                .post(endpoint)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .header("Cookie", "WEVNSM=1.0.0; os=pc; osver=Microsoft-Windows-11-Professional-build-114514-64bit; channel=netease; mode=System Product Name;appver=3.1.32.205206")
                .form(&[("params", p.as_str())])
                .send()
                .await
                .map_err(|e| HttpError::ConnectionFailed {
                    detail: e.to_string(),
                    url: endpoint.to_string(),
                })?;
            res.text().await.map_err(|e| HttpError::ConnectionFailed {
                detail: e.to_string(),
                url: endpoint.to_string(),
            })
        }
        .await;
        match &result {
            Ok(resp) => logger::debug(
                "provider::netease",
                format_args!(
                    "request completed | method=POST_FORM | url={} | elapsed={:?} | bytes={}",
                    endpoint,
                    start.elapsed(),
                    resp.len()
                ),
            ),
            Err(err) => logger::warn(
                "provider::netease",
                format_args!(
                    "request failed | method=POST_FORM | url={} | elapsed={:?} | error={}",
                    endpoint,
                    start.elapsed(),
                    err
                ),
            ),
        }
        let resp = result?;
        let detail: Option<DetailResult1> = serde_json::from_str(&resp).map_err(|e| JsonError {
            api: "NeteaseDetail".to_string(),
            source: e,
        })?;
        Ok(detail)
    }
}

impl Default for NeteaseFetcher {
    fn default() -> Self {
        Self::new()
    }
}
// ===== Response Models =====

#[derive(Debug, Deserialize, Default)]
pub struct SearchResult1 {
    pub code: i64,
    pub result: Option<SearchResultData1>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultData1 {
    pub songs: Option<Vec<Song1>>,
    pub song_count: Option<i64>,
    pub albums: Option<Vec<Album1>>,
    pub album_count: Option<i64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct LyricResult1 {
    pub code: Option<i64>,
    pub nolyric: Option<bool>,
    pub uncollected: Option<bool>,
    pub lrc: Option<Lyrics1>,
    pub klyric: Option<Lyrics1>,
    pub tlyric: Option<Lyrics1>,
    pub romalrc: Option<Lyrics1>,
    pub yrc: Option<Lyrics1>,
    pub ytlrc: Option<Lyrics1>,
    pub yromalrc: Option<Lyrics1>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Lyrics1 {
    pub version: Option<i64>,
    pub lyric: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DetailResult1 {
    pub data: Option<Vec<Detail1>>,
    pub code: Option<i64>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Detail1 {
    pub free_trial_info: Option<Trial1>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Trial1 {
    pub start: Option<u8>,
    pub end: Option<u8>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Song1 {
    pub name: Option<String>,
    pub id: Option<serde_json::Value>,
    #[serde(alias = "ar")]
    pub artists: Option<Vec<Ar1>>,
    #[serde(alias = "al")]
    pub album: Option<Al1>,
    #[serde(alias = "dt")]
    pub duration: Option<i64>,
    pub publish_time: Option<i64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Ar1 {
    pub id: Option<i64>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Al1 {
    pub id: Option<i64>,
    pub name: Option<String>,
    #[serde(rename = "picUrl")]
    pub pic_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Album1 {
    pub name: Option<String>,
    pub id: Option<i64>,
    pub size: Option<i64>,
    pub artist: Option<Artist1>,
}

#[derive(Debug, Deserialize)]
pub struct Artist1 {
    pub name: Option<String>,
    pub id: Option<i64>,
}
