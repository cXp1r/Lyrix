use crate::error::{GeneralError, LyrixResult};
use crate::providers::{LyrixProvider, RawLyricsContent, RawLyricsFormat};
use async_trait::async_trait;
use reqwest::Client;

pub(crate) struct SpotifyProvider {
    pub(crate) cookie: String,
    pub(crate) client: Client,
}

#[async_trait]
impl LyrixProvider for SpotifyProvider {
    type Searcher = crate::searchers::spotify::SpotifySearcher;
    type Api = crate::fetchers::spotify::SpotifyFetcher;
    type SearchResult = crate::searchers::spotify::SpotifySearchResult;

    async fn create_searcher(&self) -> LyrixResult<Self::Searcher> {
        Ok(crate::searchers::spotify::SpotifySearcher::with_client(
            self.client.clone(),
            self.cookie.clone(),
        )
        .await?)
    }

    async fn create_api(&self) -> LyrixResult<Self::Api> {
        Ok(crate::fetchers::spotify::SpotifyFetcher::with_client(
            self.client.clone(),
            self.cookie.clone(),
        )
        .await?)
    }

    fn label() -> &'static str {
        "Spotify"
    }

    async fn fetch(api: &Self::Api, best: &Self::SearchResult) -> LyrixResult<RawLyricsContent> {
        let content = api.get_lyrics(&best.id).await?;
        if content.is_empty() {
            return Err(GeneralError::MissingField {
                field: "Spotify: 歌词内容为空".to_string(),
            }
            .into());
        }

        Ok(RawLyricsContent {
            content,
            format: RawLyricsFormat::SpotifyJson,
        })
    }
}
