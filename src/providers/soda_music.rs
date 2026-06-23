use async_trait::async_trait;
use reqwest::Client;
use crate::error::{GeneralError, LyrixResult};
use crate::models::LineInfo;
use super::lyrics_provider::LyricsProvider;

pub(crate) struct SodaMusicProvider {
    pub(crate) client: Client,
}

#[async_trait]
impl LyricsProvider for SodaMusicProvider {
    type Searcher = crate::searchers::soda_music::SodaMusicSearcher;
    type Api = crate::fetchers::soda_music::SodaMusicApi;
    type SearchResult = crate::searchers::soda_music::SodaMusicSearchResult;

    async fn create_searcher(&self) -> LyrixResult<Self::Searcher> {
        Ok(crate::searchers::soda_music::SodaMusicSearcher::with_client(self.client.clone()))
    }
    async fn create_api(&self) -> LyrixResult<Self::Api> {
        Ok(crate::fetchers::soda_music::SodaMusicApi::with_client(self.client.clone()))
    }
    fn label() -> &'static str {
        "汽水音乐"
    }

    async fn fetch_and_parse(
        api: &Self::Api,
        best: &Self::SearchResult,
    ) -> LyrixResult<Vec<LineInfo>> {
        use crate::parsers::soda_music::SodaParser;
        use crate::parsers::IParsers;
        let detail = api
            .get_detail(&best.id)
            .await?
            .ok_or_else(|| GeneralError::MissingField {
                field: "汽水音乐: 获取歌曲详情失败".to_string(),
            })?;
        let lyric_info = detail.lyric.ok_or_else(|| GeneralError::MissingField {
            field: "汽水音乐: 歌曲没有歌词".to_string(),
        })?;
        let content = lyric_info.content.ok_or_else(|| GeneralError::MissingField {
            field: "汽水音乐: 无歌曲详细信息".to_string(),
        })?;
        if content.is_empty() {
            return Err(GeneralError::MissingField {
                field: "汽水音乐: 歌词内容为空".to_string(),
            }.into());
        }
        Ok(SodaParser {}.parse(content)?)
    }
}
