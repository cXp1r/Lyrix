use crate::providers::LyrixProvider;
use crate::error::{GeneralError, LyrixResult};
use crate::models::LineInfo;
use async_trait::async_trait;
use reqwest::Client;

pub(crate) struct QQMusicProvider {
    pub(crate) client: Client,
}

#[async_trait]
impl LyrixProvider for QQMusicProvider {
    type Searcher = crate::searchers::qqmusic::QQMusicSearcher;
    type Api = crate::fetchers::qqmusic::QQMusicApi;
    type SearchResult = crate::searchers::qqmusic::QQMusicSearchResult;

    async fn create_searcher(&self) -> LyrixResult<Self::Searcher> {
        Ok(crate::searchers::qqmusic::QQMusicSearcher::with_client(
            self.client.clone(),
        ))
    }
    async fn create_api(&self) -> LyrixResult<Self::Api> {
        Ok(crate::fetchers::qqmusic::QQMusicApi::with_client(
            self.client.clone(),
        ))
    }
    fn label() -> &'static str {
        "QQ音乐"
    }

    async fn fetch_and_parse(
        api: &Self::Api,
        best: &Self::SearchResult,
    ) -> LyrixResult<Vec<LineInfo>> {
        use crate::parsers::lrc::LrcParser;
        use crate::parsers::qqmusic::{QQMusicLrcParser, QQMusicParser};
        if let Ok(qrc) = api.get_lyrics_qrc(&best.id.to_string()).await {
            return Ok(QQMusicParser {}.decrypt_and_parse(qrc)?);
        }
        let lyric_result =
            api.get_lyric(&best.mid)
                .await?
                .ok_or_else(|| GeneralError::MissingField {
                    field: "QQ音乐: 获取歌词失败".to_string(),
                })?;
        if let Some(lrc) = lyric_result.lyric {
            if !lrc.is_empty() {
                return Ok(QQMusicLrcParser {}.parse(lrc)?);
            }
        }
        Err(GeneralError::MissingField {
            field: "QQ音乐: 未获取到歌词内容".to_string(),
        }
        .into())
    }
}
