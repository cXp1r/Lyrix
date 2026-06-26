use crate::providers::LyrixProvider;
use crate::error::{GeneralError, LyrixResult};
use crate::models::LineInfo;
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

    async fn fetch_and_parse(
        api: &Self::Api,
        best: &Self::SearchResult,
    ) -> LyrixResult<Vec<LineInfo>> {
        use crate::parsers::spotify::SpotifyParser;
        let lryics = api.get_lyrics(&best.id).await?;
        if lryics.is_empty() {
            return Err(GeneralError::MissingField {
                field: "Spotify: 歌词内容为空".to_string(),
            }
            .into());
        }
        Ok(SpotifyParser {}.parse(lryics)?)
    }
}
