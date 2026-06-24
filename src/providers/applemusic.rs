use super::providers::LyrixProvider;
use crate::error::{GeneralError, LyrixResult};
use crate::models::LineInfo;
use async_trait::async_trait;
use reqwest::Client;

pub(crate) struct AppleMusicProvider {
    pub(crate) token: String,
    pub(crate) client: Client,
}

#[async_trait]
impl LyrixProvider for AppleMusicProvider {
    type Searcher = crate::searchers::applemusic::ApplemusicSearcher;
    type Api = crate::fetchers::applemusic::ApplemusicApi;
    type SearchResult = crate::searchers::applemusic::ApplemusicSearchResult;

    async fn create_searcher(&self) -> LyrixResult<Self::Searcher> {
        Ok(
            crate::searchers::applemusic::ApplemusicSearcher::with_client(
                self.client.clone(),
                self.token.clone(),
            ),
        )
    }
    async fn create_api(&self) -> LyrixResult<Self::Api> {
        Ok(crate::fetchers::applemusic::ApplemusicApi::with_client(
            self.client.clone(),
            self.token.clone(),
        ))
    }
    fn label() -> &'static str {
        "applemusic"
    }

    async fn fetch_and_parse(
        api: &Self::Api,
        best: &Self::SearchResult,
    ) -> LyrixResult<Vec<LineInfo>> {
        use crate::parsers::applemusic::AppleMusicParser;
        let detail = api
            .get_lyric(&best.id)
            .await?
            .ok_or_else(|| GeneralError::MissingField {
                field: "applemusic: 获取歌曲详情失败".to_string(),
            })?;
        //虽然是vec 但是实际上只有一项(目前来看是的)
        let lyric_data = detail.data.ok_or_else(|| GeneralError::MissingField {
            field: "applemusic: 歌曲没有歌词".to_string(),
        })?;
        let u = lyric_data
            .get(0)
            .ok_or_else(|| GeneralError::MissingField {
                field: "applemusic: 歌曲没有歌词".to_string(),
            })?;
        let att = u
            .attributes
            .as_ref()
            .ok_or_else(|| GeneralError::MissingField {
                field: "applemusic: 无歌曲详细信息".to_string(),
            })?;
        let lyrics = att
            .ttml_localizations
            .as_ref()
            .ok_or_else(|| GeneralError::MissingField {
                field: "applemusic: 歌词内容为空".to_string(),
            })?;
        if lyrics.is_empty() {
            return Err(GeneralError::MissingField {
                field: "applemusic: 歌词内容为空".to_string(),
            }
            .into());
        }
        Ok(AppleMusicParser {}.parse(lyrics.to_string())?)
    }
}
