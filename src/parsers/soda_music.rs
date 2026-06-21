use crate::error::parser::lyrics_parse::LyricsParseError;
use crate::error::LyrixResult;
use crate::parsers::{IParsers};
///汽水音乐逐字歌词解析器
pub struct SodaParser;
impl IParsers for SodaParser{
    //不要问为什么不用t1,问就是这里本来就是offset
    #[allow(unused_variables)]
    fn get_offset_time(&self, t1: u32, t2: u32) -> LyrixResult<u16> {
        u16::try_from(t2)
            .map_err(|_| LyricsParseError::OffsetOverflow { t1, t2 }.into())
    }
}
