use crate::providers::LyrixProvider;
use crate::error::{GeneralError, LyrixResult};
use crate::models::LineInfo;
use async_trait::async_trait;
use reqwest::Client;

pub(crate) struct NeteaseProvider {
    pub(crate) client: Client,
}

#[async_trait]
impl LyrixProvider for NeteaseProvider {
    type Searcher = crate::searchers::netease::NeteaseSearcher;
    type Api = crate::fetchers::netease::NeteaseApi;
    type SearchResult = crate::searchers::netease::NeteaseSearchResult;

    async fn create_searcher(&self) -> LyrixResult<Self::Searcher> {
        Ok(crate::searchers::netease::NeteaseSearcher::with_client(
            self.client.clone(),
        ))
    }
    async fn create_api(&self) -> LyrixResult<Self::Api> {
        Ok(crate::fetchers::netease::NeteaseApi::with_client(
            self.client.clone(),
        ))
    }
    fn label() -> &'static str {
        "网易云"
    }

    async fn fetch_and_parse(
        api: &Self::Api,
        best: &Self::SearchResult,
    ) -> LyrixResult<Vec<LineInfo>> {
        use crate::parsers::lrc::LrcParser;
        use crate::parsers::netease::{NeteaseLrcParser, NeteaseParser};
        use crate::parsers::IParsers;
        let lyric_result = api.get_lyric(&best.id).await?;
        if let Some(yrc) = lyric_result.yrc.and_then(|y| y.lyric) {
            if !yrc.is_empty() {
                return Ok(NeteaseParser {}.parse(yrc)?);
            }
        }
        let lrc = lyric_result.lrc.ok_or_else(|| GeneralError::MissingField {
            field: "网易云: LRC也没有哟".to_string(),
        })?;
        let parser = NeteaseLrcParser {
            version: lrc.version.unwrap_or(3) as u8,
        };
        Ok(
            parser.parse(lrc.lyric.ok_or_else(|| GeneralError::MissingField {
                field: "网易云: LRC也没有哟".to_string(),
            })?)?,
        )
    }
}
