use crate::{fetchers::spotify::TrackData, models::LyricsData};
use crate::error::LyrixResult;
pub(crate) trait LyricsReader {
    fn label() -> &'static str;
    async fn fetch_and_parse(
        track: &TrackData
    ) -> LyrixResult<LyricsData>;
}