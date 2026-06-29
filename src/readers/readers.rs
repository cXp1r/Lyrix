use crate::error::LyrixResult;
use crate::models::ITrackMetadata;
use async_trait::async_trait;
#[allow(dead_code)]
#[async_trait]
pub(crate) trait LyrixReader {
    fn label() -> &'static str;

    async fn read_raw(track: &dyn ITrackMetadata) -> LyrixResult<String>;
}
