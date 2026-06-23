use crate::error::parser::lyrics_parse::LyricsParseError;
use crate::error::LyrixResult;
use crate::models::*;
use crate::parsers::{decrypt::krc::*, IParsers};
///酷狗歌词解析器
pub struct KugouParser;
impl KugouParser {
    fn decrypt(&self, lyrics: &str) -> LyrixResult<String> {
        krc_decrypt(lyrics)
    }
    pub fn decrypt_and_parse(&self, lyrics: String) -> LyrixResult<Vec<LineInfo>> {
        let lyrics = self.decrypt(&lyrics)?;
        self.parse(lyrics)
    }
}
impl IParsers for KugouParser {
    //不要问为什么不用t1,问就是这里本来就是offset
    #[allow(unused_variables)]
    fn get_offset_time(&self, t1: u32, t2: u32) -> LyrixResult<u16> {
        u16::try_from(t2).map_err(|_| LyricsParseError::OffsetOverflow { t1, t2 }.into())
    }
}
