use crate::error::LyrixResult;
use crate::models::{ITrackMetadata, LyricsData};
use async_trait::async_trait;
#[allow(dead_code)]
#[async_trait]
pub(crate) trait LyrixReader {
    fn label() -> &'static str;

    async fn read_and_parse(track: &dyn ITrackMetadata) -> LyrixResult<LyricsData>;
}
