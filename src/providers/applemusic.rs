use super::base_api::BaseApi;
use serde::Deserialize;
use std::collections::HashMap;

pub struct ApplemusicApi {
    api: BaseApi,
    _auth: String,
    _token: String,
}

impl ApplemusicApi {
    fn applemusic_headers(auth: &str, token: &str) -> HashMap<String, String> {
        let mut h = HashMap::new();
        h.insert("Authorization".to_string(), auth.to_string());
        h.insert("Media-User-Token".to_string(), token.to_string());
        h.insert("Origin".to_string(), "https://music.apple.com".to_string());
        h
    }

    pub fn new(auth: String, token: String) -> Self {
        let headers = Self::applemusic_headers(&auth, &token);
        Self {
            api: BaseApi::new(None, Some(headers)),
            _auth: auth,
            _token: token,
        }
    }

    pub fn with_client(client: reqwest::Client, auth: String, token: String) -> Self {
        let headers = Self::applemusic_headers(&auth, &token);
        Self {
            api: BaseApi::with_client(client, None, Some(headers)),
            _auth: auth,
            _token: token,
        }
    }

    pub async fn search(&self, keyword: &str) -> Result<SearchResult, reqwest::Error> {
        let encoded = urlencoding::encode(keyword);
        let url = format!(
            "https://amp-api-edge.music.apple.com/v1/catalog/cn/search?term={}&types=songs&limit=20",
            encoded
        );
        let resp = self.api.get_async(&url).await?;
        let parsed: SearchResult = serde_json::from_str(&resp).unwrap_or_default();
        Ok(parsed)
    }

    pub async fn get_lyric(&self, id: &str) -> Result<LyricResult, reqwest::Error> {
        let url = format!(
            "https://amp-api.music.apple.com/v1/catalog/cn/songs/{}/syllable-lyrics?{}={}&{}={}&extend=ttmlLocalizations",
            id,
            urlencoding::encode("l[lyrics]"),
            urlencoding::encode("zh-hans-cn"),
            urlencoding::encode("l[script]"),
            urlencoding::encode("zh-Hans"),
        );
        let resp = self.api.get_async(&url).await?;
        let parsed: LyricResult = serde_json::from_str(&resp).unwrap_or_default();
        Ok(parsed)
    }
}

impl Default for ApplemusicApi {
    fn default() -> Self {
        Self::new(String::new(), String::new())
    }
}

// ===== Response Models =====

#[derive(Debug, Deserialize, Default)]
pub struct SearchResult {
    pub results: Option<Results>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Results {
    pub songs: Option<Songs>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Songs {
    pub data: Vec<Song>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub song_type: Option<String>,
    pub attributes: Option<SongAttributes>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SongAttributes {
    pub name: Option<String>,
    pub artist_name: Option<String>,
    pub album_name: Option<String>,
    pub duration_in_millis: Option<u32>,  // snake_case
    pub url: Option<String>,
    pub has_lyrics: Option<bool>,         // snake_case
}

#[derive(Debug, Deserialize, Default)]
pub struct LyricResult {
    #[serde(rename = "type")]
    pub lyrics_type: Option<String>,
    pub attributes: Option<LyricAttributes>,
}

#[derive(Debug, Deserialize, Default)]
pub struct LyricAttributes {
    #[serde(rename = "ttmlLocalizations")]
    pub ttml_localizations: Option<String>,
}