use crate::parsers::{IParsers, PREFIX_RE};
use regex::{Regex};
pub struct SodaParsers {}
impl IParsers for SodaParsers{
    fn get_syllables_re(&self) -> &Regex {
        &PREFIX_RE
    }
}
