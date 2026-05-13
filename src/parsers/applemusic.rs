use crate::models::{LineInfo, TextInfo};
use memchr::{memchr, memmem};

pub struct AppleMusicParser {

}

impl AppleMusicParser {
    #[allow(unused_variables)]
    pub fn parse_syllables_time(&self, tag: &str) -> Result<u32, String> {
        //mm:ss.xxx
        let mut cpos = 0;
        let len = tag.len();
        let bytes = tag.as_bytes();
        
        let mut time: u32 = if let Some(m) = memchr(b':',bytes) {
            cpos = m + 1;
            60_000 * tag[0..m].parse::<u32>().map_err(|_e| "Applemusic Parser: failed to parse hours")?
        } else {
            0
        };
        let Some(s) = memchr(b'.',bytes) else {
            return Err("Applemusic Parser: seconds not found".into());
        };
        time += 1000 * tag[cpos..s].parse::<u32>().map_err(|_e| "Applemusic Parser: failed to parse seconds")?;
        time += tag[s + 1..].parse::<u32>().map_err(|_e| "Applemusic Parser: failed to parse centis")?;
    
        Ok(time)
    }
    pub fn parse_time(&self, tag: &str) -> Result<u32, String> {
        //时:分:秒.毫秒
        let hours = tag[0..2].parse::<u32>()
            .map_err(|_e| "Applemusic Parser: failed to parse hours")?;
        let minutes = tag[3..5].parse::<u32>()
            .map_err(|_e| "Applemusic Parser: failed to parse minutes")?;
        let seconds = tag[6..8].parse::<u32>()
            .map_err(|_e| "Applemusic Parser: failed to parse seconds")?;
        let centis = tag[9..11].parse::<u32>()
            .map_err(|_e| "Applemusic Parser: failed to parse centis")?;

        Ok(hours * 3_600_000 +minutes * 60_000 + seconds * 1_000 + centis * 10)
    }
    pub fn parse_syllables_line(&self, line: &str) -> Result<LineInfo, String> {
        let mut textinfo: Vec<TextInfo> = Vec::new();
        let mut cpos ;
        let bytes = line.as_bytes();
        let mut it = memmem::find_iter(bytes, "=\"");
        let lst = loop {
            let c: usize = match it.next() {
                Some(l) => {
                    if &line[l - 3 .. l] != "gin"{
                        continue;
                    }
                    l
                },
                None => {
                    return Err("Applemusic Parser: line start_time not found".into());
                }
            } + 2;
            let Some(w) = memchr(b'\"', &bytes[c..]) else {
                return Err("Applemusic Parser: line start_time not found".into());
            };
            break self.parse_syllables_time(&line[c .. c + w])?
        };
        let ld = loop {
            let c: usize = match it.next() {
                Some(l) => {
                    if &line[l - 3 .. l] != "end"{
                        continue;
                    }
                    l
                },
                None => {
                    return Err("Applemusic Parser: line start_time not found".into());
                }
            } + 2;
            let Some(w) = memchr(b'\"', &bytes[c + 2..]) else {
                return Err("Applemusic Parser: line start_time not found".into());
            };
            break self.parse_syllables_time(&line[c .. c + w])?
        } - lst;
        'outer: loop {
            match it.next() {
                Some(l) => {
                    if &line[l - 3 .. l] != "gin"{
                        continue;
                    }//抓取begin,如果不是 走下一个
                    cpos = l + 2;
                    let Some(c) = memchr(b'\"', &bytes[cpos..]) else {
                        return Err("Applemusic Parser: word start_time not found".into());
                    };
                    let st = self.parse_syllables_time(&line[cpos..cpos + c])?;

                    let e = loop {
                        match it.next() {
                            Some(e) => {
                                if &line[e - 3 .. e] != "end"{
                                    continue;
                                }
                                break e
                            },
                            None => {
                                break 'outer;
                            }
                        };
                    };
                    cpos = e + 2;
                    let Some(c2) = memchr(b'\"', &bytes[cpos..]) else {
                        return Err("Applemusic Parser: word end_time not found".into());
                    };
                    let du = self.parse_syllables_time(&line[cpos..cpos + c2])? - st;
                    cpos += c2;
                    let Some(t1) = memchr(b'>',&bytes[cpos..]) else {
                        return Err("Applemusic Parser: failed to parse lyrics".into());
                    };
                    cpos += t1 + 1;
                    
                    let Some(t2) = memchr(b'<',&bytes[cpos..]) else {
                        return Err("Applemusic Parser: failed to parse lyrics".into());
                    };
                    let text =  line[cpos..cpos + t2].to_string();
                    
                    

                    textinfo.push(
                        TextInfo { start_time: (st - lst) as u16, duration: du as u16, text: text }
                    );
                },
                None => {
                    break;
                }
            }
        }
        Ok(
            LineInfo { start_time: lst, duration: ld as u16, text: String::new(), syllables: textinfo }
        )

    }

    pub fn parse_syllables(&self, lyrics: String) -> Result<Vec<LineInfo>, String> {
        let mut lineinfo: Vec<LineInfo> = Vec::new();
        let bytes = lyrics.as_bytes();
        let mut its = memmem::find_iter(bytes, "<p");
        let mut ite = memmem::find_iter(bytes, "</p");
        loop {
            match its.next() {
                Some(l) => {
                    let e = match ite.next() {
                        Some(e) => {
                            e
                        },
                        None => {
                            break;
                        }
                    };
                    if l >= e {
                        return Err("Applemusic Parser: Unexpected error".into());
                    }
                    lineinfo.push(self.parse_syllables_line(&lyrics[l..e])?);
                },
                None => {
                    break;
                }
            }
        }
        Ok(lineinfo)
    }
    pub fn parse_w(&self, lyrics: String) -> Result<Vec<LineInfo>, String> {
        let mut lineinfo: Vec<LineInfo> = Vec::new();
        
        let Some(mut cpos) = memmem::find(lyrics.as_bytes(), b"div") else {
            return Err("Applemusic Parser: lyrics body not found".into());
        };
        let ulyrics = &lyrics[cpos..];
        let len = ulyrics.len();
        let bytes = ulyrics.as_bytes();
        let mut it = memmem::find_iter(bytes, "=\"");
        while cpos < len {
            
            //<p begin=\"00:03:26.910\" end=\"00:03:30.830\">すれ違っても ずっと君でいて</p><p begin=\"00:03:30.830\" end=\"00:03:47.830\">きっと会いに行くから</p>
            let st = match it.next() {
                Some(u) => {//定位到时长开始
                    cpos = u + 2;
                    //println!("{}",cpos);
                    let Some(c) = memchr(b'\"', &bytes[cpos..]) else {
                        return Err("Applemusic Parser: start_time not found".into());
                    };
                    //println!("{}",&ulyrics[cpos..cpos + c]);
                    self.parse_time(&ulyrics[cpos..cpos + c])?
                },
                None => {
                    break;
                }
            };
            let et = match it.next() {
                Some(u) => {//定位到时长开始
                    cpos = u + 2;
                    let Some(c) = memchr(b'\"', &bytes[cpos..]) else {
                        return Err("Applemusic Parser: start_time not found".into());
                    };
                    self.parse_time(&ulyrics[cpos..cpos + c])?
                },
                None => {
                    break;
                }
            };
            cpos += 2;
            let Some(s) = memchr(b'>',&bytes[cpos..]) else {
                return Err("Applemusic Parser: failed to parse lyrics".into());
            };
            cpos += s + 1;
            let Some(s) = memchr(b'<',&bytes[cpos..]) else {
                return Err("Applemusic Parser: failed to parse lyrics".into());
            };

            lineinfo.push(LineInfo {
                    start_time: st,
                    duration: (et - st) as u16,
                    text: ulyrics[cpos..cpos + s].to_string(),
                    syllables: vec![],
            });
        }
        Ok(lineinfo)
    }
    pub fn parse(&self, lyrics: String) -> Result<Vec<LineInfo>, String> {
        let start = std::time::Instant::now();
        let r = match lyrics.find("span") {
            Some(_e) => {
                self.parse_syllables(lyrics)
            },
            None => {
                self.parse_w(lyrics)
            }
        };        
        println!("parse took: {:?}", start.elapsed());
        r
    }
}


