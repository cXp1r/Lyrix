use crate::parsers::{IParsers, decrypt::krc::*};
use crate::models::LineInfo;
pub struct KugouParsers {}
impl KugouParsers {
    fn decrypt(&self, lyrics: &str) -> Result<String, String> {
        krc_decrypt(lyrics)
    }
    pub fn decrypt_and_parse(&self, lyrics: String) -> Result<Vec<LineInfo>, String>  {
        let lyrics = self.decrypt(&lyrics)?;
        self.parse(lyrics)
    }
}
impl IParsers for KugouParsers{

}
