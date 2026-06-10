use crate::logger;
use crate::models::{LineInfo, TextInfo};
use memchr::{memchr, memmem::Finder};
use std::sync::LazyLock;

static FINDER_IN: LazyLock<Finder<'static>> = LazyLock::new(|| Finder::new(b"in=\""));
static FINDER_ND: LazyLock<Finder<'static>> = LazyLock::new(|| Finder::new(b"nd=\""));
static FINDER_P:  LazyLock<Finder<'static>> = LazyLock::new(|| Finder::new(b"<p"));
static FINDER_EP: LazyLock<Finder<'static>> = LazyLock::new(|| Finder::new(b"</p"));
static FINDER_EQ: LazyLock<Finder<'static>> = LazyLock::new(|| Finder::new(b"=\""));
static FINDER_DIV:LazyLock<Finder<'static>> = LazyLock::new(|| Finder::new(b"div"));

pub struct AppleMusicParser {}

impl AppleMusicParser {
    fn get_offset_time(&self, t1: u32, t2: u32) -> Result<u16, String> {
        let diff = t2
            .checked_sub(t1)
            .ok_or_else(|| format!("AppleMusic Parsers: overflow ({} {})", t1, t2))?;
        u16::try_from(diff).map_err(|_| format!("AppleMusic Parsers: offset overflow({})", diff))
    }

    fn parse_u32_field(field: &str, name: &str) -> Result<u32, String> {
        if field.is_empty() {
            return Err(format!("AppleMusic Parser: missing {name}"));
        }
        field
            .parse::<u32>()
            .map_err(|e| format!("AppleMusic Parser: failed to parse {name}: {:?} raw={:?}", e, field))
    }

    pub fn parse_syllables_time(&self, tag: &str) -> Result<u32, String> {
        let tag = tag.trim();
        let (minutes, rest) = if let Some((m, rest)) = tag.split_once(':') {
            (Self::parse_u32_field(m, "minutes")? * 60_000, rest)
        } else {
            (0, tag)
        };
        let (seconds_str, centis_str) = rest
            .split_once('.')
            .ok_or_else(|| format!("AppleMusic Parser: seconds not found in {:?}", tag))?;
        let seconds = Self::parse_u32_field(seconds_str, "seconds")? * 1_000;
        let centis = Self::parse_u32_field(centis_str, "centis")?;
        Ok(minutes + seconds + centis)
    }

    pub fn parse_time(&self, tag: &str) -> Result<u32, String> {
        let tag = tag.trim();
        let (hours_str, rest) = tag
            .split_once(':')
            .ok_or_else(|| format!("AppleMusic Parser: hours not found in {:?}", tag))?;
        let (minutes_str, rest) = rest
            .split_once(':')
            .ok_or_else(|| format!("AppleMusic Parser: minutes not found in {:?}", tag))?;
        let (seconds_str, centis_str) = rest
            .split_once('.')
            .ok_or_else(|| format!("AppleMusic Parser: centis not found in {:?}", tag))?;

        let hours = Self::parse_u32_field(hours_str, "hours")? * 3_600_000;
        let minutes = Self::parse_u32_field(minutes_str, "minutes")? * 60_000;
        let seconds = Self::parse_u32_field(seconds_str, "seconds")? * 1_000;
        let centis = Self::parse_u32_field(centis_str, "centis")? * 10;
        Ok(hours + minutes + seconds + centis)
    }

    pub fn parse_syllables_line(&self, line: &str) -> Result<LineInfo, String> {
        let bytes = line.as_bytes();
        let mut pos = 0usize;

        pos += FINDER_IN
            .find(&bytes[pos..])
            .ok_or("AppleMusic Parser: line start_time not found")?
            + 4;
        let w = memchr(b'"', &bytes[pos..]).ok_or("AppleMusic Parser: line start_time not found")?;
        let lst = self.parse_syllables_time(&line[pos..pos + w])?;
        pos += w + 1;

        pos += FINDER_ND
            .find(&bytes[pos..])
            .ok_or("AppleMusic Parser: line end_time not found")?
            + 4;
        let w = memchr(b'"', &bytes[pos..]).ok_or("AppleMusic Parser: line end_time not found")?;
        let end_time = self.parse_syllables_time(&line[pos..pos + w])?;
        let ld = end_time
            .checked_sub(lst)
            .ok_or_else(|| format!("AppleMusic Parser: line end_time before start_time: {} < {}", end_time, lst))?;
        pos += w + 1;

        let mut textinfo: Vec<TextInfo> = Vec::with_capacity(8);

        loop {
            let Some(off) = FINDER_IN.find(&bytes[pos..]) else { break };
            pos += off + 4;
            let w = memchr(b'"', &bytes[pos..]).ok_or("AppleMusic Parser: word start_time not found")?;
            let st = self.parse_syllables_time(&line[pos..pos + w])?;
            pos += w + 1;

            pos += FINDER_ND
                .find(&bytes[pos..])
                .ok_or("AppleMusic Parser: word end_time not found")?
                + 4;
            let w = memchr(b'"', &bytes[pos..]).ok_or("AppleMusic Parser: word end_time not found")?;
            let et = self.parse_syllables_time(&line[pos..pos + w])?;
            pos += w + 1;

            let gt = memchr(b'>', &bytes[pos..]).ok_or("AppleMusic Parser: failed to parse lyrics")?;
            pos += gt + 1;
            let lt = memchr(b'<', &bytes[pos..]).ok_or("AppleMusic Parser: failed to parse lyrics")?;
            let text = line[pos..pos + lt].to_string();
            pos += lt + 1;

            textinfo.push(TextInfo {
                start_time: self.get_offset_time(lst, st)?,
                duration: self.get_offset_time(st, et)?,
                text,
            });
        }

        Ok(LineInfo {
            start_time: lst,
            duration: u16::try_from(ld)
                .map_err(|_| format!("AppleMusic Parsers: offset overflow({})", ld))?,
            text: String::new(),
            syllables: textinfo,
        })
    }

    pub fn parse_syllables(&self, lyrics: String) -> Result<Vec<LineInfo>, String> {
        let bytes = lyrics.as_bytes();
        let mut its = FINDER_P.find_iter(bytes);
        let mut ite = FINDER_EP.find_iter(bytes);
        let mut lineinfo: Vec<LineInfo> = Vec::with_capacity(128);
        loop {
            let Some(l) = its.next() else { break };
            let Some(e) = ite.next() else { break };
            if l >= e {
                return Err("AppleMusic Parser: Unexpected error".into());
            }
            lineinfo.push(self.parse_syllables_line(&lyrics[l..e])?);
        }
        Ok(lineinfo)
    }

    pub fn parse_w(&self, lyrics: String) -> Result<Vec<LineInfo>, String> {
        let cpos = FINDER_DIV
            .find(lyrics.as_bytes())
            .ok_or("AppleMusic Parser: lyrics body not found")?;
        let ulyrics = &lyrics[cpos..];
        let bytes = ulyrics.as_bytes();
        let mut it = FINDER_EQ.find_iter(bytes);
        let mut lineinfo: Vec<LineInfo> = Vec::with_capacity(128);
        let mut pos;

        loop {
            let Some(u) = it.next() else { break };
            pos = u + 2;
            let Some(c) = memchr(b'"', &bytes[pos..]) else {
                return Err("AppleMusic Parser: start_time not found".into());
            };
            let st = self.parse_time(&ulyrics[pos..pos + c])?;

            let Some(u) = it.next() else { break };
            pos = u + 2;
            let Some(c) = memchr(b'"', &bytes[pos..]) else {
                return Err("AppleMusic Parser: end_time not found".into());
            };
            let et = self.parse_time(&ulyrics[pos..pos + c])?;
            pos += c + 1;

            let Some(s) = memchr(b'>', &bytes[pos..]) else {
                return Err("AppleMusic Parser: failed to parse lyrics".into());
            };
            pos += s + 1;
            let Some(s) = memchr(b'<', &bytes[pos..]) else {
                return Err("AppleMusic Parser: failed to parse lyrics".into());
            };
            lineinfo.push(LineInfo {
                start_time: st,
                duration: self.get_offset_time(st, et)?,
                text: ulyrics[pos..pos + s].to_string(),
                syllables: vec![],
            });
        }
        Ok(lineinfo)
    }

    pub fn parse(&self, lyrics: String) -> Result<Vec<LineInfo>, String> {
        let start = std::time::Instant::now();
        let r = self.parse_without_st(lyrics);
        let elapsed = start.elapsed();
        match &r {
            Ok(lines) => logger::debug(
                "parser::applemusic",
                format_args!("parse completed | elapsed={:?} | lines={}", elapsed, lines.len()),
            ),
            Err(err) => logger::warn(
                "parser::applemusic",
                format_args!("parse failed | elapsed={:?} | error={}", elapsed, err),
            ),
        }
        r
    }

    pub fn parse_without_st(&self, lyrics: String) -> Result<Vec<LineInfo>, String> {
        let has_span = lyrics.find("span").is_some();
        if has_span {
            self.parse_syllables(lyrics)
        } else {
            self.parse_w(lyrics)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppleMusicParser;

    #[test]
    fn parse_time_rejects_bad_input() {
        let parser = AppleMusicParser {};
        assert!(parser.parse_time("bad").is_err());
        assert!(parser.parse_time("01:02").is_err());
        assert!(parser.parse_time("01:02:03").is_err());
        assert!(parser.parse_time("01:xx:03.10").is_err());
    }

    #[test]
    fn parse_time_accepts_valid_input() {
        let parser = AppleMusicParser {};
        assert_eq!(parser.parse_time("01:02:03.04").unwrap(), 3_723_040);
        assert_eq!(parser.parse_syllables_time("01:02.03").unwrap(), 62_003);
    }
}
