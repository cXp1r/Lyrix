use crate::logger;
use crate::models::{LineInfo, TextInfo};
use once_cell::sync::Lazy;
use memchr::{memmem::Finder, memchr};

static FINDER_IN: Lazy<Finder<'static>> = Lazy::new(|| Finder::new(b"in=\""));
static FINDER_ND: Lazy<Finder<'static>> = Lazy::new(|| Finder::new(b"nd=\""));
static FINDER_P:  Lazy<Finder<'static>> = Lazy::new(|| Finder::new(b"<p"));
static FINDER_EP: Lazy<Finder<'static>> = Lazy::new(|| Finder::new(b"</p"));
static FINDER_EQ: Lazy<Finder<'static>> = Lazy::new(|| Finder::new(b"=\""));
static FINDER_DIV:Lazy<Finder<'static>> = Lazy::new(|| Finder::new(b"div"));

pub struct AppleMusicParser {}

impl AppleMusicParser {
    fn get_offset_time(&self, t1: u32, t2: u32) -> Result<u16, String> {
        let diff = t2
            .checked_sub(t1)
            .ok_or(format!("AppleMusic Parsers: overflow ({} {})", t1, t2))?;
        //u16够你offset用了
        u16::try_from(diff)
            .map_err(|_| format!("AppleMusic Parsers: offset overflow({})",diff))
    }
    // mm:ss.ccc
    pub fn parse_syllables_time(&self, tag: &str) -> Result<u32, String> {
        let bytes = tag.as_bytes();
        let mut time: u32;
        let cpos: usize;
        if let Some(m) = memchr(b':', bytes) {
            time = 60_000 * tag[..m].parse::<u32>()
                .map_err(|_| "AppleMusic Parser: failed to parse minutes")?;
            cpos = m + 1;
        } else {
            time = 0;
            cpos = 0;
        }
        let s = memchr(b'.', bytes)
            .ok_or("AppleMusic Parser: seconds not found")?;
        time += 1_000 * tag[cpos..s].parse::<u32>()
            .map_err(|_| "AppleMusic Parser: failed to parse seconds")?;
        time += tag[s + 1..].parse::<u32>()
            .map_err(|_| "AppleMusic Parser: failed to parse centis")?;
        Ok(time)
    }

    // hh:mm:ss.xx  喜欢哥哥的雷霆定位吗
    pub fn parse_time(&self, tag: &str) -> Result<u32, String> {
        let hours   = tag[0..2].parse::<u32>().map_err(|_| "AppleMusic Parser: failed to parse hours")?;
        let minutes = tag[3..5].parse::<u32>().map_err(|_| "AppleMusic Parser: failed to parse minutes")?;
        let seconds = tag[6..8].parse::<u32>().map_err(|_| "AppleMusic Parser: failed to parse seconds")?;
        let centis  = tag[9..11].parse::<u32>().map_err(|_| "AppleMusic Parser: failed to parse centis")?;
        Ok(hours * 3_600_000 + minutes * 60_000 + seconds * 1_000 + centis * 10)
    }

    pub fn parse_syllables_line(&self, line: &str) -> Result<LineInfo, String> {
        let bytes = line.as_bytes();
        let mut pos = 0usize;

        pos += FINDER_IN.find(&bytes[pos..]).ok_or("AppleMusic Parser: line start_time not found")? + 4;
        let w = memchr(b'"', &bytes[pos..]).ok_or("AppleMusic Parser: line start_time not found")?;
        let lst = self.parse_syllables_time(&line[pos..pos + w])?;
        pos += w + 1;

        pos += FINDER_ND.find(&bytes[pos..]).ok_or("AppleMusic Parser: line end_time not found")? + 4;
        let w = memchr(b'"', &bytes[pos..]).ok_or("AppleMusic Parser: line end_time not found")?;
        let ld = self.parse_syllables_time(&line[pos..pos + w])? - lst;
        pos += w + 1;

        let mut textinfo: Vec<TextInfo> = Vec::with_capacity(8);

        loop {
            let Some(off) = FINDER_IN.find(&bytes[pos..]) else { break };
            pos += off + 4;//避免一个雷霆定位定位到符号上面
            let w = memchr(b'"', &bytes[pos..]).ok_or("AppleMusic Parser: word start_time not found")?;
            let st = self.parse_syllables_time(&line[pos..pos + w])?;
            pos += w + 1;

            pos += FINDER_ND.find(&bytes[pos..]).ok_or("AppleMusic Parser: word end_time not found")? + 4;
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
                .map_err(|_| format!("AppleMusic Parsers: offset overflow({})",ld))?,
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
        let cpos = FINDER_DIV.find(lyrics.as_bytes())
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
                duration: (et - st) as u16,
                text: ulyrics[pos..pos + s].to_string(),
                syllables: vec![],
            });
        }
        Ok(lineinfo)
    }
    //本质测速
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
    //本质分发
    pub fn parse_without_st(&self, lyrics: String) -> Result<Vec<LineInfo>, String> {
        let has_span = lyrics.find("span").is_some();
        if has_span {
            self.parse_syllables(lyrics)
        } else {
            self.parse_w(lyrics)
        }
    }
}

