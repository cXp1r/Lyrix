use crate::error::{GeneralError, LyrixResult};
use crate::providers::LyrixProvider;
use async_trait::async_trait;
use reqwest::Client;

pub(crate) struct QQMusicProvider {
    pub(crate) client: Client,
}

#[async_trait]
impl LyrixProvider for QQMusicProvider {
    type Searcher = crate::searchers::qqmusic::QQMusicSearcher;
    type Api = crate::fetchers::qqmusic::QQMusicFetcher;
    type SearchResult = crate::searchers::qqmusic::QQMusicSearchResult;

    async fn create_searcher(&self) -> LyrixResult<Self::Searcher> {
        Ok(crate::searchers::qqmusic::QQMusicSearcher::with_client(
            self.client.clone(),
        ))
    }

    async fn create_api(&self) -> LyrixResult<Self::Api> {
        Ok(crate::fetchers::qqmusic::QQMusicFetcher::with_client(
            self.client.clone(),
        ))
    }

    fn label() -> &'static str {
        "QQ音乐"
    }

    async fn fetch(api: &Self::Api, best: &Self::SearchResult) -> LyrixResult<String> {
        let qrc = api.get_lyrics_qrc(&best.id.to_string()).await?;
        if qrc.is_empty() {
            return Err(GeneralError::MissingField {
                field: "QQMusic network qrc content".to_string(),
            }
            .into());
        }

        Ok(qrc)
    }
}
