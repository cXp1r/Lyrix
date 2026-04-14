use async_trait::async_trait;
use crate::providers::qqmusic::QQMusicApi;
use crate::models::ITrackMetadata;
use super::{ISearcher, ISearchResult, SearcherType};

pub struct QQMusicSearcher {
    api: QQMusicApi,
}

impl QQMusicSearcher {
    pub fn new() -> Self {
        Self { api: QQMusicApi::new() }
    }
}

#[async_trait]
impl ISearcher for QQMusicSearcher {
    fn name(&self) -> &str { "QQMusic" }
    fn display_name(&self) -> &str { "QQ Music" }
    fn searcher_type(&self) -> SearcherType { SearcherType::QQMusic }

    async fn search_for_results_by_string(&self, search_string: &str) -> Result<Vec<Box<dyn ISearchResult>>, Box<dyn std::error::Error + Send + Sync>> {
        let result = self.api.search(search_string).await?;
        let mut results: Vec<Box<dyn ISearchResult>> = Vec::new();

        if let Some(resp) = result {
            if let Some(req1) = resp.req_1 {
                if let Some(data) = req1.data {
                    if let Some(body) = data.body {
                        if let Some(song_list) = body.song {
                            if let Some(songs) = song_list.list {
                                for song in songs {
                                    let title = song.name.or(song.title).unwrap_or_default();
                                    let artists: Vec<String> = song.singer
                                        .unwrap_or_default()
                                        .iter()
                                        .filter_map(|s| s.name.clone())
                                        .collect();
                                    let album = song.album.as_ref().and_then(|a| a.name.clone()).unwrap_or_default();
                                    let duration = song.interval.map(|i| i * 1000);
                                    let mid = song.mid.unwrap_or_default();

                                    results.push(Box::new(QQMusicSearchResult {
                                        mid,
                                        title,
                                        artists,
                                        album,
                                        duration_ms: duration,
                                        match_score: 0,
                                    }));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(results)
    }
    fn get_split_char(&self) -> char {
        '/'
    }
}

pub struct QQMusicSearchResult {
    pub mid: String,
    pub title: String,
    pub artists: Vec<String>,
    pub album: String,
    pub duration_ms: Option<i32>,
    pub match_score: i8,
}

impl ISearchResult for QQMusicSearchResult {
    fn title(&self) -> &str { &self.title }
    fn artists(&self) -> &[String] { &self.artists }
    fn album(&self) -> &str { &self.album }
    fn duration_ms(&self) -> Option<i32> { self.duration_ms }
    fn match_score(&self) -> i8 { self.match_score }
    fn set_match_score(&mut self, score: i8) { self.match_score = score; }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TrackMetadata;

    #[tokio::test]
    async fn test_qqmusic_search_for_duration_debug() {
        let searcher = QQMusicSearcher::new();

        let metadata = TrackMetadata {
            title: Some("私は、わたしの事が好き。".to_string()),
            artist: Some("HoneyWorks/夏吉ゆうこ".to_string()),
            album: Some("超かぐや姫！".to_string()),
            album_artist: Some("".to_string()),
            duration_ms: Some(251000),
            ..Default::default()
        };

        let Some(search_string) = searcher.make_search_string(&metadata).await else {
            return;
        };
        println!("search string = {}", search_string);
        let result = searcher
            .search_for_results_by_string(&search_string)
            .await;

        match result {
            Ok(mut list) => {
                for item in list.iter_mut() {
                    let mt = searcher.compare_track(&metadata, item.as_ref());
                    item.set_match_score(mt);
                }

                list.sort_by(|a, b| b.match_score().cmp(&a.match_score()));

                println!("result count = {}", list.len());
                for (i, item) in list.iter().enumerate() {
                    println!("--- item {} ---", i);
                    println!("title = {}", item.title());
                    println!("artists = {:?}", item.artists());
                    println!("album = {}", item.album());
                    println!("duration_ms = {:?}", item.duration_ms());
                    println!("match_score = {}", item.match_score());
                }
            }
            Err(e) => {
                panic!("search failed: {:?}", e);
            }
        }
    }
}