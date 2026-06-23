use crate::error::{LyrixResult, SearcherError};
use async_trait::async_trait;
use crate::fetchers::applemusic::ApplemusicApi;
use super::{log_score_gain, log_score_warn, log_score_total, ISearcher, ISearchResult};
use crate::models::ITrackMetadata;

pub struct ApplemusicSearcher {
    api: ApplemusicApi,
}

impl ApplemusicSearcher {
    pub fn new(token: String) -> Self {
        Self { api: ApplemusicApi::new(token) }
    }

    pub fn with_client(client: reqwest::Client, token: String) -> Self {
        Self { api: ApplemusicApi::with_client(client, token) }
    }
}

impl Default for ApplemusicSearcher {
    fn default() -> Self {
        Self::new(String::new())
    }
}

#[async_trait]
impl ISearcher for ApplemusicSearcher {
    async fn search_for_results_by_string(&self, search_string: &str) -> LyrixResult<Vec<Box<dyn ISearchResult>>> {
        let result = self.api.search(search_string).await?;
        let mut results: Vec<Box<dyn ISearchResult>> = Vec::new();
        let resp = result
            .ok_or_else(|| SearcherError::NoResults {
                label: self.label().to_string(),
                query: search_string.to_string(),
            })?;
        let res = resp.results
            .ok_or_else(|| SearcherError::NoResults {
                label: self.label().to_string(),
                query: search_string.to_string(),
            })?;
        let songs = res.songs
            .ok_or_else(|| SearcherError::NoResults {
                label: self.label().to_string(),
                query: search_string.to_string(),
            })?;
        let songsv = songs.data
            .ok_or_else(|| SearcherError::NoResults {
                label: self.label().to_string(),
                query: search_string.to_string(),
            })?;
        for song in songsv {
            let id = song.id.clone().unwrap_or_default();
            let info = song.attributes
                .ok_or_else(|| SearcherError::MissingField {
                    label: self.label().to_string(),
                    field: format!("attributes for id={}", id),
                })?;
            let title = info.name.clone().unwrap_or_default();
            let singer = info.artist_name.clone().unwrap_or_default();
            let artists: Vec<String> = singer
                .split('、')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            let album = info.album_name.clone().unwrap_or_default();
            let duration =
                info.duration_in_millis.map(|d| d as u32);
            let has_lyrics =
                info.has_lyrics.unwrap_or(false);
            results.push(Box::new(ApplemusicSearchResult {
                id,
                title,
                artists,
                album,
                duration_ms: duration,
                has_lyrics,
                match_score: 0,
            }));
        }
        Ok(results)

    }

    fn label(&self) -> &'static str { "applemusic" }

    fn compare_track(&self, track: &dyn ITrackMetadata, result: &dyn ISearchResult) -> (i8, bool) {
        let mut score = 0i8;

        // 第一步没必要覆写,强制留着了
        let track_title = track.title().unwrap_or_default().to_lowercase();
        let result_title = result.title().to_lowercase();
        if !track_title.is_empty() && !result_title.is_empty() {
            if track_title == result_title {
                score += 4;
                log_score_gain(
                    "searcher::applemusic::score",
                    format_args!("track : {}\nresult: {}", track_title, result_title),
                    4,
                );
            } else if result_title.contains(&track_title) || track_title.contains(&result_title) {
                score += 2;
                log_score_gain(
                    "searcher::applemusic::score",
                    format_args!("track : {}\nresult: {}", track_title, result_title),
                    2,
                );
            } else {
                let clean_track = self.clean_title(&track_title);
                let clean_result = self.clean_title(&result_title);
                if clean_track == clean_result {
                    score += 3;
                    log_score_gain(
                        "searcher::applemusic::score",
                        format_args!("track : {}\nresult: {}", clean_track, clean_result),
                        3,
                    );
                } else if clean_result.contains(&clean_track) || clean_track.contains(&clean_result) {
                    score += 1;
                    log_score_gain(
                        "searcher::applemusic::score",
                        format_args!("track : {}\nresult: {}", clean_track, clean_result),
                        1,
                    );
                } else {
                    log_score_warn(
                        "searcher::applemusic::score",
                        result,
                        format_args!("track : {}\nresult: {}", clean_track, clean_result),
                    );
                }
            }
        } else {
            log_score_warn(
                "searcher::applemusic::score",
                result,
                format_args!("track : {}\nresult: {}", track_title, result_title),
            );
        }

        // Artist match
        let d: Vec<String> = track
            .artist()
            .unwrap_or_default()   //防止下面崩溃
            .split("—")
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        let artists: Vec<String> = d.get(0).unwrap_or(&String::new())
            .split("、")
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        for a in &artists {
            if result.artists().iter().any(|b| {
                let b = b.to_lowercase();
                a == &b || a.contains(&b) || b.contains(a)
            }) {
                score += 1;
                log_score_gain(
                    "searcher::applemusic::score",
                    format_args!("artist: {}\nresult: {}", a, result.artists().join(" / ")),
                    1,
                );
            } else {
                log_score_warn(
                    "searcher::applemusic::score",
                    result,
                    format_args!("artist: {}\nresult: {}", a, result.artists().join(" / ")),
                );
            }
        }
        if artists.is_empty() {
            log_score_warn(
                "searcher::applemusic::score",
                result,
                format_args!("artist: {}\nresult: {}", "(empty)", result.artists().join(" / ")),
            );
        }
        // Album match
        let track_album = d.get(1).unwrap_or(&String::new()).clone();
        let result_album = result.album().to_lowercase();
        if !track_album.is_empty() && !result_album.is_empty() && track_album == result_album {
            score += 1;
            log_score_gain(
                "searcher::applemusic::score",
                format_args!("album : {}\nresult: {}", track_album, result_album),
                1,
            );
        } else {
            log_score_warn(
                "searcher::applemusic::score",
                result,
                format_args!("album : {}\nresult: {}", track_album, result_album),
            );
        }
        // Album artist match
        let track_album_artist = self.clean_title(&track.album_artist().unwrap_or_default().to_lowercase());
        let result_album_artist = result.album_artists().unwrap_or_default().to_vec();

        if result_album_artist.iter().any(|s:&String| s.contains(&track_album_artist)) {
            score += 1;
            log_score_gain(
                "searcher::applemusic::score",
                format_args!(
                    "ab_art: {}\nresult: {}",
                    track_album_artist,
                    result_album_artist.join(" / ")
                ),
                1,
            );
        } else {
            log_score_warn(
                "searcher::applemusic::score",
                result,
                format_args!(
                    "ab_art: {}\nresult: {}",
                    track_album_artist,
                    result_album_artist.join(" / ")
                ),
            );
        }
        if let Some(duration_ms) = track.duration_ms() {
            if let Some(result_duration_ms) = result.duration_ms() {
                let diff = (duration_ms as i64 - result_duration_ms as i64).abs();
                if diff == 0 { // 完全匹配
                    score += 3;
                    log_score_gain(
                        "searcher::applemusic::score",
                        format_args!("dur   : {}ms\nresult: {}ms", duration_ms, result_duration_ms),
                        3,
                    );
                } else if diff <= 500 {
                    score += 2;
                    log_score_gain(
                        "searcher::applemusic::score",
                        format_args!(
                            "dur   : {}ms\nresult: {}ms",
                            duration_ms, result_duration_ms
                        ),
                        2,
                    );
                } else if diff <= 1000 {
                    score += 1;
                    log_score_gain(
                        "searcher::applemusic::score",
                        format_args!(
                            "dur   : {}ms\nresult: {}ms",
                            duration_ms, result_duration_ms
                        ),
                        1,
                    );
                } else {
                    log_score_warn(
                        "searcher::applemusic::score",
                        result,
                        format_args!("dur   : {}ms\nresult: {}ms", duration_ms, result_duration_ms),
                    );
                }

            }
            else {
                log_score_warn(
                    "searcher::applemusic::score",
                    result,
                    format_args!("dur   : {}ms\nresult: {}", duration_ms, "N/A"),
                );
            }
        } else {
            log_score_warn(
                "searcher::applemusic::score",
                result,
                format_args!("dur   : {}\nresult: {}ms", "N/A", result.duration_ms().unwrap_or(0)),
            );
        }
        log_score_total("searcher::applemusic::score", result, score, false);
        (score, false)//苹果不开会员听棍母呢
    }
}

pub struct ApplemusicSearchResult {
    pub id: String,
    pub title: String,
    pub artists: Vec<String>,
    pub album: String,
    pub duration_ms: Option<u32>,  // snake_case
    pub has_lyrics: bool,
    pub match_score: i8,
}

impl ISearchResult for ApplemusicSearchResult {
    fn title(&self) -> &str { &self.title }
    fn artists(&self) -> &[String] { &self.artists }
    fn album(&self) -> &str { &self.album }
    fn duration_ms(&self) -> Option<u32> { self.duration_ms }
    fn match_score(&self) -> i8 { self.match_score }
    fn set_match_score(&mut self, score: i8) { self.match_score = score; }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn trial(&self) -> Option<[u32; 2]> { None }
    fn set_trial(&mut self, _i: bool) {}
    fn is_trial(&self) -> bool { false }
}
