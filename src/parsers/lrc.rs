use crate::error::parser::parse::ParseError;
use crate::error::LyrixResult;
use crate::logger;
use crate::models::LineInfo;
use memchr::memchr;

pub trait LrcParser {
    fn parse_lrc_time(&self, tag: &str) -> LyrixResult<u32> {
        let tag = tag.trim();
        let (minutes_str, rest) =
            tag.split_once(':')
                .ok_or_else(|| ParseError::InvalidLrcFormat {
                    detail: format!("时间标签缺少 ':' : {:?}", tag),
                })?;
        let (seconds_str, centis_str) =
            rest.split_once('.')
                .ok_or_else(|| ParseError::InvalidLrcFormat {
                    detail: format!("时间标签缺少 '.' : {:?}", tag),
                })?;

        if minutes_str.is_empty() || seconds_str.is_empty() || centis_str.is_empty() {
            return Err(ParseError::InvalidLrcFormat {
                detail: format!("时间标签不完整: {:?}", tag),
            }
            .into());
        }

        let minutes = minutes_str
            .parse::<u32>()
            .map_err(|_| ParseError::TimestampParse {
                field: "minutes".to_string(),
                raw: minutes_str.to_string(),
            })?;
        let seconds = seconds_str
            .parse::<u32>()
            .map_err(|_| ParseError::TimestampParse {
                field: "seconds".to_string(),
                raw: seconds_str.to_string(),
            })?;
        let centis = centis_str
            .parse::<u32>()
            .map_err(|_| ParseError::TimestampParse {
                field: "centis".to_string(),
                raw: centis_str.to_string(),
            })?;

        Ok(minutes * 60_000 + seconds * 1_000 + centis * 10)
    }

    fn parse(&self, lyrics: String) -> LyrixResult<Vec<LineInfo>> {
        let start = std::time::Instant::now();
        let result = self.parse_without_st(lyrics);
        let t = start.elapsed();
        match &result {
            Ok(lines) => logger::debug(
                "parser::lrc",
                format_args!("parse completed | elapsed={:?} | lines={}", t, lines.len()),
            ),
            Err(err) => logger::warn(
                "parser::lrc",
                format_args!("parse failed | elapsed={:?} | error={}", t, err),
            ),
        }
        result
    }

    fn parse_without_st(&self, lyrics: String) -> LyrixResult<Vec<LineInfo>> {
        let mut lineinfo: Vec<LineInfo> = Vec::new();
        let len = lyrics.len();
        let cbytes = lyrics.as_bytes();
        let mut c = 0;

        while c < len {
            let Some(lb) = memchr(b'[', &cbytes[c..]) else {
                break;
            };
            c += lb + 1;

            if c >= len || !cbytes[c].is_ascii_digit() {
                if let Some(rb) = memchr(b']', &cbytes[c..]) {
                    c += rb + 1;
                } else {
                    break;
                }
                continue;
            }

            let Some(rb) = memchr(b']', &cbytes[c..]) else {
                break;
            };
            let tag = &lyrics[c..c + rb];
            let s = self.parse_lrc_time(tag)?;
            c += rb + 1;

            let content_end = memchr(b'[', &cbytes[c..]).map(|x| c + x).unwrap_or(len);
            let text = lyrics[c..content_end]
                .trim_matches(|ch| ch == '\r' || ch == '\n')
                .to_string();
            c = content_end;

            lineinfo.push(LineInfo {
                start_time: s,
                duration: 0,
                text,
                syllables: vec![],
            });
        }

        Ok(lineinfo)
    }
}

#[cfg(test)]
mod tests {
    use super::LrcParser;

    struct Dummy;
    impl LrcParser for Dummy {}

    #[test]
    fn parse_lrc_time_rejects_bad_input() {
        let parser = Dummy;
        assert!(parser.parse_lrc_time("not-a-time").is_err());
        assert!(parser.parse_lrc_time("00:01").is_err());
        assert!(parser.parse_lrc_time("00:01.").is_err());
        assert!(parser.parse_lrc_time("00:xx.10").is_err());
    }

    #[test]
    fn parse_lrc_time_accepts_valid_input() {
        let parser = Dummy;
        assert_eq!(parser.parse_lrc_time("01:02.03").unwrap(), 62_030);
    }
}
