use crate::error::{GeneralError, LyrixResult};
use crate::providers::{LyrixProvider, RawLyricsContent, RawLyricsFormat};
use async_trait::async_trait;
use reqwest::Client;

pub(crate) struct AppleMusicProvider {
    pub(crate) token: String,
    pub(crate) client: Client,
}

#[async_trait]
impl LyrixProvider for AppleMusicProvider {
    type Searcher = crate::searchers::applemusic::ApplemusicSearcher;
    type Api = crate::fetchers::applemusic::AppleMusicFetcher;
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
        Ok(crate::fetchers::applemusic::AppleMusicFetcher::with_client(
            self.client.clone(),
            self.token.clone(),
        ))
    }

    fn label() -> &'static str {
        "applemusic"
    }

    async fn fetch(api: &Self::Api, best: &Self::SearchResult) -> LyrixResult<RawLyricsContent> {
        let detail = api
            .get_lyric(&best.id)
            .await?
            .ok_or_else(|| GeneralError::MissingField {
                field: "applemusic: 获取歌曲详情失败".to_string(),
            })?;
        let lyric_data = detail.data.ok_or_else(|| GeneralError::MissingField {
            field: "applemusic: 歌曲没有歌词".to_string(),
        })?;
        let item = lyric_data
            .first()
            .ok_or_else(|| GeneralError::MissingField {
                field: "applemusic: 歌曲没有歌词".to_string(),
            })?;
        let attributes = item
            .attributes
            .as_ref()
            .ok_or_else(|| GeneralError::MissingField {
                field: "applemusic: 无歌曲详细信息".to_string(),
            })?;
        let content = attributes
            .ttml_localizations
            .as_ref()
            .ok_or_else(|| GeneralError::MissingField {
                field: "applemusic: 歌词内容为空".to_string(),
            })?
            .to_string();
        if content.is_empty() {
            return Err(GeneralError::MissingField {
                field: "applemusic: 歌词内容为空".to_string(),
            }
            .into());
        }

        Ok(RawLyricsContent {
            content,
            format: RawLyricsFormat::AppleMusicTtml,
        })
    }
}
