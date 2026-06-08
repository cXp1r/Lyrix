use crate::logger;
use crate::models::{LineInfo};
use serde::Deserialize;

/// Spotify color-lyrics API 返回的顶层结构
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LyricsResponse {
    lyrics: LyricData,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LyricData {
    sync_type: Option<String>,
    lines: Option<Vec<RawLine>>,
}

/// JSON 中 startTimeMs/endTimeMs 是字符串，留到解析时手工转
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawLine {
    start_time_ms: Option<String>,
    words: Option<String>,
    end_time_ms: Option<String>,
}

pub struct SpotifyParser;

impl SpotifyParser {
    pub fn parse(&self, lyrics: String) -> Result<Vec<LineInfo>, String> {
        let start = std::time::Instant::now();
        let result = self.parse_without_st(lyrics);
        let t = start.elapsed();
        match &result {
            Ok(lines) => logger::debug(
                "parser::spotify",
                format_args!("parse completed | elapsed={:?} | lines={}", t, lines.len()),
            ),
            Err(err) => logger::warn(
                "parser::spotify",
                format_args!("parse failed | elapsed={:?} | error={}", t, err),
            ),
        }
        result
    }
    pub fn parse_without_st(&self, lyrics: String) -> Result<Vec<LineInfo>, String> {
        let resp: LyricsResponse =
            serde_json::from_str(&lyrics).map_err(|e| e.to_string())?;

        let data = resp.lyrics;
        let sync_type = data.sync_type.as_deref().unwrap_or("");
        if sync_type != "LINE_SYNCED" {
            return Err(format!("SpotifyParser: unknown sync_type: {sync_type}"));
        }

        let lines = data.lines.ok_or("SpotifyParser: missing lines")?;
        let mut lineinfo = Vec::with_capacity(lines.len());

        for raw in lines {
            let words = raw.words.unwrap_or_default();
            // 跳过空行
            if words.is_empty() {
                continue;
            }

            let st: u32 = raw
                .start_time_ms
                .as_deref()
                .unwrap_or("0")
                .parse()
                .map_err(|_| "SpotifyParser: invalid startTimeMs")?;

            let et_raw: u32 = raw
                .end_time_ms
                .as_deref()
                .unwrap_or("0")
                .parse()
                .map_err(|_| "SpotifyParser: invalid endTimeMs")?;

            let duration = if et_raw > st { (et_raw - st) as u16 } else { 0 };

            lineinfo.push(LineInfo {
                start_time: st,
                duration,
                text: words,
                syllables: vec![],
            });
        }

        Ok(lineinfo)
    }
}
