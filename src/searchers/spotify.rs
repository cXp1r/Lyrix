use crate::error::{LyrixResult, SearcherError};
use crate::fetchers::spotify::SpotifyApi;
use async_trait::async_trait;
use super::{ISearcher, ISearchResult};

pub struct SpotifySearcher {
    api: SpotifyApi,
}

impl SpotifySearcher {
    pub async fn new(cookie: String) -> LyrixResult<Self> {
        Ok(Self { api: SpotifyApi::new(cookie).await? })
    }

    pub async fn with_client(client: reqwest::Client, cookie: String) -> LyrixResult<Self> {
        Ok(Self { api: SpotifyApi::with_client(client, cookie).await? })
    }
}

#[async_trait]
impl ISearcher for SpotifySearcher {
    async fn search_for_results_by_string(&self, search_string: &str) -> LyrixResult<Vec<Box<dyn ISearchResult>>> {
        let result = self.api.search(search_string).await?;
        let mut results: Vec<Box<dyn ISearchResult>> = Vec::new();

        let resp = result.ok_or_else(|| SearcherError::NoResults {
            label: self.label().to_string(),
            query: search_string.to_string(),
        })?;
        let data = resp.data.ok_or_else(|| SearcherError::NoResults {
            label: self.label().to_string(),
            query: search_string.to_string(),
        })?;
        let search_v2 = data.search_v2.ok_or_else(|| SearcherError::NoResults {
            label: self.label().to_string(),
            query: search_string.to_string(),
        })?;
        let top = search_v2.top_results_v2.ok_or_else(|| SearcherError::NoResults {
            label: self.label().to_string(),
            query: search_string.to_string(),
        })?;
        let items = top.items_v2.ok_or_else(|| SearcherError::NoResults {
            label: self.label().to_string(),
            query: search_string.to_string(),
        })?;

        for song in items {
            let Some(i) = song.item else { continue };
            let Some(t) = i.data else { continue };
            let Some(id) = t.id else { continue };
            let Some(title) = t.name else { continue };
            let Some(album) = t.album_of_track else { continue };
            let Some(album_name) = album.name else { continue };
            let Some(artist) = t.artists else { continue };
            let Some(artist_items) = artist.items else { continue };
            let artists: Vec<String> = artist_items
                .iter()
                .filter_map(|s| s.profile.as_ref()?.name.clone())
                .collect();
            let duration_ms = t.duration.and_then(|d| d.total_milliseconds);

            results.push(Box::new(SpotifySearchResult {
                id,
                title,
                artists,
                album: album_name,
                duration_ms,
                match_score: 0,
                trial: None,
                is_trial: false,
            }));
        }

        Ok(results)
    }

    fn label(&self) -> &'static str { "Spotify" }
}

#[derive(Debug, Clone)]
pub struct SpotifySearchResult {
    pub id: String,
    pub title: String,
    pub artists: Vec<String>,
    pub album: String,
    pub duration_ms: Option<u32>,
    pub match_score: i8,
    pub trial: Option<[u32; 2]>,
    pub is_trial: bool,
}

impl ISearchResult for SpotifySearchResult {
    fn title(&self) -> &str { &self.title }
    fn artists(&self) -> &[String] { &self.artists }
    fn album(&self) -> &str { &self.album }
    fn duration_ms(&self) -> Option<u32> { self.duration_ms }
    fn match_score(&self) -> i8 { self.match_score }
    fn set_match_score(&mut self, score: i8) { self.match_score = score; }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn trial(&self) -> Option<[u32; 2]> { self.trial }
    fn set_trial(&mut self, i: bool) { self.is_trial = i; }
    fn is_trial(&self) -> bool { self.is_trial }
}
