pub mod netease;
pub mod qqmusic;
pub mod kugou;
pub mod soda_music;
pub mod spotify;
pub mod applemusic;
use crate::logger;
use async_trait::async_trait;
use crate::models::ITrackMetadata;



/// 搜索源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearcherType {
    Netease,
    QQMusic,
    Kugou,
    SodaMusic,
    Spotify,
}

/// 搜索结果 trait
pub trait ISearchResult: Send + Sync {
    fn title(&self) -> &str;
    fn artists(&self) -> &[String];
    fn album(&self) -> &str;
    fn album_artists(&self) -> Option<&[String]> { None }
    fn duration_ms(&self) -> Option<u32>;
    fn match_score(&self) -> i8;
    fn set_match_score(&mut self, mt: i8);
    fn as_any(&self) -> &dyn std::any::Any;
    fn trial(&self) -> Option<[u32; 2]>;
    fn is_trial(&self) -> bool;
    fn set_trial(&mut self, i: bool);
}

/// 搜索提供者 trait
#[async_trait]
pub trait ISearcher: Send + Sync {

    async fn search_for_results_by_string(&self, search_string: &str) -> Result<Vec<Box<dyn ISearchResult>>, Box<dyn std::error::Error + Send + Sync>>;

    fn make_search_string(&self, track: &dyn ITrackMetadata) -> Vec<String> {
        let title = track.title().unwrap_or_default().trim();
        let artist = track.artist().unwrap_or_default().trim();
        let album = track.album().unwrap_or_default().trim();

        let ct = self.clean_title(&self.remove_feat(title));
        let ca = self.clean_title(artist);
        let cal = self.clean_title(album);

        let join = |parts: &[&str]| {
            parts.iter().filter(|s| !s.is_empty()).copied().collect::<Vec<_>>().join(" ")
        };

        let mut strings: Vec<String> = Vec::with_capacity(8);
        let mut push = |s: String| {
            if !s.is_empty() && strings.last().map_or(true, |last| last != &s) {
                strings.push(s);
            }
        };

        push(join(&[title, artist]));
        push(join(&[&ct, &ca]));

        push(join(&[title, artist, album]));
        push(join(&[&ct, &ca, &cal]));
        
        push(title.to_string());
        push(ct.to_string());
        
        push(join(&[title, album]));
        push(join(&[&ct, &cal]));

        strings
    }
    /// 最低匹配分数线，低于此分数的结果将被丢弃（可 override）
    fn min_score(&self) -> i8 { 5 }
    /// 直接返回分数线，大于此分数线可以直接拿去请求歌词（可 override）
    fn wow_score(&self) -> i8 { 7 }
    //下面那个函数调用了这个
    async fn search_for_results(&self, track: &dyn ITrackMetadata, _full_search: bool) -> Result<Vec<Box<dyn ISearchResult>>, Box<dyn std::error::Error + Send + Sync>> {
        let strings = self.make_search_string(track);
        if strings.is_empty() {
            return Ok(vec![]);
        }

        let threshold = self.min_score();
        let wow = self.wow_score();
        let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
        logger::debug(
            "searcher",
            format_args!(
                "search started | title={} | artist={} | album={} | candidates={} | min_score={} | wow_score={}",
                track.title().unwrap_or_default(),
                track.artist().unwrap_or_default(),
                track.album().unwrap_or_default(),
                strings.join(" || "),
                threshold,
                wow
            ),
        );

        for s in &strings {
            if !seen.insert(s.as_str()) {
                continue;
            }

            logger::debug("searcher", format_args!("query started | query={}", s));
            let results = match self.search_for_results_by_string(s).await {
                Ok(r) => {
                    logger::debug(
                        "searcher",
                        format_args!("query completed | query={} | results={}", s, r.len()),
                    );
                    r
                }
                Err(e) => {
                    logger::warn(
                        "searcher",
                        format_args!("query failed | query={} | error={}", s, e),
                    );
                    continue;
                }
            };
            let mut group_best: Option<Box<dyn ISearchResult>> = None;

            for mut r in results {
                let (mt, is_trial) = self.compare_track(track, r.as_ref());
                r.set_match_score(mt);
                r.set_trial(is_trial);
                if mt > wow {
                    logger::debug(
                        "searcher",
                        format_args!(
                            "wow match selected | query={} | title={} | score={} | wow_score={}",
                            s,
                            r.title(),
                            mt,
                            wow
                        ),
                    );
                    return Ok(vec![r]);
                }
                if mt >= threshold && group_best.as_ref().map_or(true, |b| mt > b.match_score()) {
                    logger::debug(
                        "searcher",
                        format_args!(
                            "group best updated | query={} | title={} | score={} | min_score={}",
                            s,
                            r.title(),
                            mt,
                            threshold
                        ),
                    );
                    group_best = Some(r);
                }
            }

            if let Some(best) = group_best {
                logger::debug(
                    "searcher",
                    format_args!(
                        "group best selected | query={} | title={} | score={}",
                        s,
                        best.title(),
                        best.match_score()
                    ),
                );
                return Ok(vec![best]);
            }
        }

        logger::warn(
            "searcher",
            format_args!(
                "search failed | title={} | artist={} | album={} | reason=no acceptable result",
                track.title().unwrap_or_default(),
                track.artist().unwrap_or_default(),
                track.album().unwrap_or_default()
            ),
        );
        Err("Nothing here".into())
    }

    

    //smtc统一接口调用了这个
    async fn search_for_result(&self, track: &dyn ITrackMetadata) -> Result<Option<Box<dyn ISearchResult>>, Box<dyn std::error::Error + Send + Sync>> {
        let threshold = self.min_score();
        let search = self.search_for_results(track, false).await?;
        if let Some(best) = search.into_iter().next() {
            if best.match_score() >= threshold {
                return Ok(Some(best));
            }
            return Err(format!("Low score: {}/{}; id:{}", best.match_score(), threshold, best.title()).into());
        }
        let search = self.search_for_results(track, true).await?;
        if let Some(best) = search.into_iter().next() {
            return Ok((best.match_score() >= threshold).then_some(best));
        }
        Err("Nothing here".into())
    }
    fn get_split_char(&self) -> char {
        ' '
    }
    /// 比较曲目与搜索结果的匹配程度（默认通用实现，各 searcher 可 override）
    fn compare_track(&self, track: &dyn ITrackMetadata, result: &dyn ISearchResult) -> (i8, bool) {
        let mut score = 0i8;

        // 第一步没必要覆写,强制留着了
        let track_title = track.title().unwrap_or_default().to_lowercase();
        let result_title = result.title().to_lowercase();
        if !track_title.is_empty() && !result_title.is_empty() {
            if track_title == result_title {
                score += 4;
                log_score_gain(
                    "searcher::score",
                    format_args!("track : {}\nresult: {}", track_title, result_title),
                    4,
                );
            } else if result_title.contains(&track_title) || track_title.contains(&result_title) {
                score += 2;
                log_score_gain(
                    "searcher::score",
                    format_args!("track : {}\nresult: {}", track_title, result_title),
                    2,
                );
            } else {
                let clean_track = self.clean_title(&track_title);
                let clean_result = self.clean_title(&result_title);
                if clean_track == clean_result {
                    score += 3;
                    log_score_gain(
                        "searcher::score",
                        format_args!("track : {}\nresult: {}", clean_track, clean_result),
                        3,
                    );
                } else if clean_result.contains(&clean_track) || clean_track.contains(&clean_result) {
                    score += 1;
                    log_score_gain(
                        "searcher::score",
                        format_args!("track : {}\nresult: {}", clean_track, clean_result),
                        1,
                    );
                } else {
                    log_score_warn(
                        "searcher::score",
                        result,
                        format_args!("track : {}\nresult: {}", clean_track, clean_result),
                    );
                }
            }
        } else {
            log_score_warn(
                "searcher::score",
                result,
                format_args!("track : {}\nresult: {}", track_title, result_title),
            );
        }

        // Artist match
        let artists: Vec<String> = track
            .artist()
            .unwrap_or_default()  
            .split(self.get_split_char())
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
                    "searcher::score",
                    format_args!("artist: {}\nresult: {}", a, result.artists().join(" / ")),
                    1,
                );
            } else {
                log_score_warn(
                    "searcher::score",
                    result,
                    format_args!("artist: {}\nresult: {}", a, result.artists().join(" / ")),
                );
            }
        }
        if artists.is_empty() {
            log_score_warn(
                "searcher::score",
                result,
                format_args!("artist: {}\nresult: {}", "(empty)", result.artists().join(" / ")),
            );
        }
        // Album match
        let track_album = track.album().unwrap_or_default().to_lowercase();
        let result_album = result.album().to_lowercase();
        if !track_album.is_empty() && !result_album.is_empty(){
            if track_album == result_album {
                score += 2;
                log_score_gain(
                    "searcher::score",
                    format_args!("album : {}\nresult: {}", track_album, result_album),
                    2,
                );
            } else if result_album.contains(&track_album) || track_album.contains(&result_album){
                score += 1;
                log_score_gain(
                    "searcher::score",
                    format_args!("album : {}\nresult: {}", track_album, result_album),
                    1,
                );
            } else {
                log_score_warn(
                    "searcher::score",
                    result,
                    format_args!("album : {}\nresult: {}", track_album, result_album),
                );
            }
        } else {
            log_score_warn(
                "searcher::score",
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
                "searcher::score",
                format_args!(
                    "ab_art: {}\nresult: {}",
                    track_album_artist,
                    result_album_artist.join(" / ")
                ),
                1,
            );
        } else {
            log_score_warn(
                "searcher::score",
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
                        "searcher::score",
                        format_args!("dur   : {}ms\nresult: {}ms", duration_ms, result_duration_ms),
                        3,
                    );
                } else if diff <= 500 {
                    score += 2;
                    log_score_gain(
                        "searcher::score",
                        format_args!(
                            "dur   : {}ms\nresult: {}ms",
                            duration_ms, result_duration_ms
                        ),
                        2,
                    );
                } else if diff <= 1000 {
                    score += 1;
                    log_score_gain(
                        "searcher::score",
                        format_args!(
                            "dur   : {}ms\nresult: {}ms",
                            duration_ms, result_duration_ms
                        ),
                        1,
                    );
                } else {
                    log_score_warn(
                        "searcher::score",
                        result,
                        format_args!("dur   : {}ms\nresult: {}ms", duration_ms, result_duration_ms),
                    );
                }
            } else {
                log_score_warn(
                    "searcher::score",
                    result,
                    format_args!("dur   : {}ms\nresult: {}", duration_ms, "N/A"),
                );
            }
        } else {
            log_score_warn(
                "searcher::score",
                result,
                format_args!("dur   : {}\nresult: {}ms", "N/A", result.duration_ms().unwrap_or(0)),
            );
        }
        
        let is_trial = {
            if let Some(duration_ms) = track.duration_ms() {
                if let Some(result_duration_ms) = result.trial() {
                    let diff = (duration_ms as i64 - result_duration_ms[1] as i64).abs();
                    if diff <= 100 { // 完全匹配
                        score += 2;
                        log_score_gain(
                            "searcher::score",
                            format_args!(
                                "trial : {}ms\nresult: {}ms",
                                duration_ms, result_duration_ms[1]
                            ),
                            2,
                        );
                        true
                    } else if diff <= 1000 { // 近似匹配
                        score += 1;
                        log_score_gain(
                            "searcher::score",
                            format_args!(
                                "trial : {}ms\nresult: {}ms",
                                duration_ms, result_duration_ms[1]
                            ),
                            1,
                        );
                        true
                    } else {
                        log_score_warn(
                            "searcher::score",
                            result,
                            format_args!(
                                "trial : {}ms\nresult: {}ms",
                                duration_ms, result_duration_ms[1]
                            ),
                        );
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        };
        log_score_total("searcher::score", result, score, is_trial);
        (score, is_trial)
    }

    /// 清理标题中的常见符号（供 compare_track 使用，可 override）
    fn clean_title(&self, title: &str) -> String {
        let mut result = title.to_string();
        for pattern in &["(", "[", " - "] {
            if let Some(idx) = result.find(pattern) {
                result = result[..idx].trim().to_string();
            }
        }
        result = result
            .chars()
            .filter(|c| {
                !matches!(
                    c,
                    '《' | '》' | '「' | '」' | '『' | '』' |
                    '！' | '!' | '？' | '?' | '。' | '、' |
                    '·' | '•' | '…'
                )
            })
            .collect();
        result.trim().to_string()
    }

    fn remove_feat(&self, title: &str) -> String {
        let mut s = title.to_string();
        if let Some(idx) = s.find("(feat.") {
            s = s[..idx].trim().to_string();
        }
        if let Some(idx) = s.find(" - feat.") {
            s = s[..idx].trim().to_string();
        }
        s
    }
}

pub(crate) fn log_score_gain(
    tag: &str,
    current: impl std::fmt::Display,
    points: i8,
) {
    logger::info(
        tag,
        format_args!(
            "{}\n       +{}",
            current,
            points
        ),
    );
}

pub(crate) fn log_score_warn(
    tag: &str,
    _result: &dyn ISearchResult,
    current: impl std::fmt::Display,
) {
    logger::warn(
        tag,
        format_args!(
            "{}\n       +0",
            current,
        ),
    );
}

pub(crate) fn log_score_total(
    tag: &str,
    result: &dyn ISearchResult,
    total: i8,
    is_trial: bool,
) {
    logger::info(
        tag,
        format_args!(
            "score completed | result_title={} | result_artists={} | result_album={} | total={} | is_trial={}",
            result.title(),
            result.artists().join(" / "),
            result.album(),
            total,
            is_trial
        ),
    );
}
