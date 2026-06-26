use super::base_api::BaseApi;
use crate::error::fetcher::json::JsonError;
use crate::error::LyrixResult;
use memchr::memchr;
use serde::Deserialize;
use std::collections::HashMap;
pub struct QQMusicApi {
    api: BaseApi,
}

impl QQMusicApi {
    pub fn new() -> Self {
        Self {
            api: BaseApi::new(Some("https://c.y.qq.com/"), None),
        }
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self {
            api: BaseApi::with_client(client, Some("https://c.y.qq.com/"), None),
        }
    }

    /// 搜索歌曲
    pub async fn search(&self, keyword: &str) -> LyrixResult<Option<MusicFcgApiResult1>> {
        let data = serde_json::json!({
            "req_1": {
                "method": "DoSearchForQQMusicDesktop",
                "module": "music.search.SearchCgiService",
                "param": {
                    "num_per_page": "20",
                    "page_num": "1",
                    "query": keyword,
                    "search_type": 0
                }
            }
        });

        let resp = self
            .api
            .post_json_async("https://u.y.qq.com/cgi-bin/musicu.fcg", &data)
            .await?;
        let result: Option<MusicFcgApiResult1> =
            serde_json::from_str(&resp).map_err(|e| JsonError {
                api: "QQMusicSearch".to_string(),
                source: e,
            })?;
        Ok(result)
    }

    /// 获取歌词
    pub async fn get_lyric(&self, song_mid: &str) -> LyrixResult<Option<LyricResult1>> {
        let current_millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let callback = "MusicJsonCallback_lrc";
        let mut params = HashMap::new();
        params.insert("callback".to_string(), callback.to_string());
        params.insert("pcachetime".to_string(), current_millis.to_string());
        params.insert("songmid".to_string(), song_mid.to_string());
        params.insert("g_tk".to_string(), "5381".to_string());
        params.insert("jsonpCallback".to_string(), callback.to_string());
        params.insert("loginUin".to_string(), "0".to_string());
        params.insert("hostUin".to_string(), "0".to_string());
        params.insert("format".to_string(), "jsonp".to_string());
        params.insert("inCharset".to_string(), "utf8".to_string());
        params.insert("outCharset".to_string(), "utf8".to_string());
        params.insert("notice".to_string(), "0".to_string());
        params.insert("platform".to_string(), "yqq".to_string());
        params.insert("needNewCode".to_string(), "0".to_string());

        let resp = self
            .api
            .post_form_async(
                "https://c.y.qq.com/lyric/fcgi-bin/fcg_query_lyric_new.fcg",
                &params,
            )
            .await?;

        let json_str = resolve_resp_json(callback, &resp);
        if json_str.is_empty() {
            return Ok(None);
        }

        let mut result: LyricResult1 = serde_json::from_str(&json_str).map_err(|e| JsonError {
            api: "QQMusicLyric".to_string(),
            source: e,
        })?;
        result.decode();
        Ok(Some(result))
    }

    /// 获取QRC歌词,需要解密
    pub async fn get_lyrics_qrc(&self, id: &str) -> LyrixResult<String> {
        let mut params = HashMap::new();
        params.insert("version".to_string(), "15".to_string());
        params.insert("miniversion".to_string(), "82".to_string());
        params.insert("lrctype".to_string(), "4".to_string());
        params.insert("musicid".to_string(), id.to_string());

        let resp = self
            .api
            .post_form_async(
                "https://c.y.qq.com/qqmusic/fcgi-bin/lyric_download.fcg",
                &params,
            )
            .await?;

        let len = resp.len();
        let content = resp.as_bytes();
        let mut cpos = 0;

        while cpos < len {
            let Some(cc) = memchr(b'[', &content[cpos..]) else {
                break;
            };
            cpos += cc + 1; // 跳过 '['

            // 判断是不是 CDATA[
            if cpos + 6 <= len && &resp[cpos..cpos + 6] == "CDATA[" {
                cpos += 6; 
                //别几把改了, 这<![CDATA[xxx]]>里面是加密的结果第一个]就是结束的地方
                let Some(end) = memchr(b']', &content[cpos..]) else {
                    break;
                };
                return Ok(resp[cpos..cpos + end].to_string());
            }
        }
        Err(crate::error::SearcherError::NoResults {
            label: "QQMusic".to_string(),
            query: format!("qrc id={id}"),
        }
        .into())
    }
}

impl Default for QQMusicApi {
    fn default() -> Self {
        Self::new()
    }
}

fn resolve_resp_json(callback_sign: &str, val: &str) -> String {
    if !val.starts_with(callback_sign) {
        return String::new();
    }
    let json_str = val.replacen(&format!("{}(", callback_sign), "", 1);
    if json_str.ends_with(')') {
        json_str[..json_str.len() - 1].to_string()
    } else {
        json_str
    }
}

// ===== Response Models =====

#[derive(Debug, Deserialize, Default)]
pub struct MusicFcgApiResult1 {
    pub code: Option<i64>,
    pub req_1: Option<MusicFcgReq11>,
}

#[derive(Debug, Deserialize, Default)]
pub struct MusicFcgReq11 {
    pub code: Option<i64>,
    pub data: Option<MusicFcgReq1Data1>,
}

#[derive(Debug, Deserialize, Default)]
pub struct MusicFcgReq1Data1 {
    pub body: Option<MusicFcgReq1DataBody1>,
}

#[derive(Debug, Deserialize, Default)]
pub struct MusicFcgReq1DataBody1 {
    pub song: Option<SongList1>,
}

#[derive(Debug, Deserialize, Default)]
pub struct SongList1 {
    pub list: Option<Vec<Song1>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Song1 {
    pub album: Option<Album1>,
    pub id: Option<u32>,
    pub interval: Option<i32>,
    pub mid: Option<String>,
    pub name: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub singer: Option<Vec<Singer1>>,
    pub time_public: Option<String>,
    pub file: Option<Preview1>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Preview1 {
    pub b_30s: Option<u32>, //试听开始ms
    pub e_30s: Option<u32>,
}
#[derive(Debug, Deserialize, Default)]
pub struct Singer1 {
    pub id: Option<i64>,
    pub mid: Option<String>,
    pub name: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Album1 {
    pub id: Option<i32>,
    pub mid: Option<String>,
    pub name: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct LyricResult1 {
    pub code: Option<i64>,
    #[serde(rename = "lyric")]
    pub lyric: Option<String>,
    pub trans: Option<String>,
}

impl LyricResult1 {
    pub fn decode(&mut self) {
        use base64::engine::general_purpose::STANDARD;
        use base64::Engine;
        if let Some(ref lyric) = self.lyric {
            if let Ok(decoded) = STANDARD.decode(lyric) {
                self.lyric = String::from_utf8(decoded).ok();
            }
        }
        if let Some(ref trans) = self.trans {
            if let Ok(decoded) = STANDARD.decode(trans) {
                self.trans = String::from_utf8(decoded).ok();
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct QqLyricsResponse1 {
    pub lyrics: String,
    pub trans: String,
}
