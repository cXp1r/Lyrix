use crate::models::{TrackMetadata, LyricsData};
use crate::error::LyrixResult;
use async_trait::async_trait;
#[async_trait]
pub(crate) trait LyrixReader {
    fn label() -> &'static str;

    async fn read_and_parse<'a>(
        track: &'a TrackMetadata
    ) -> LyrixResult<LyricsData>;
}