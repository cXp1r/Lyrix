use super::lyrics_provider::LyricsProvider;
use crate::error::{GeneralError, LyrixResult, SearcherError};
use crate::models::LineInfo;
use async_trait::async_trait;
use reqwest::Client;

pub(crate) struct KugouProvider {
    pub(crate) client: Client,
}

#[async_trait]
impl LyricsProvider for KugouProvider {
    type Searcher = crate::searchers::kugou::KugouSearcher;
    type Api = crate::fetchers::kugou::KugouApi;
    type SearchResult = crate::searchers::kugou::KugouSearchResult;

    async fn create_searcher(&self) -> LyrixResult<Self::Searcher> {
        Ok(crate::searchers::kugou::KugouSearcher::with_client(
            self.client.clone(),
        ))
    }
    async fn create_api(&self) -> LyrixResult<Self::Api> {
        Ok(crate::fetchers::kugou::KugouApi::with_client(
            self.client.clone(),
        ))
    }
    fn label() -> &'static str {
        "酷狗"
    }

    async fn fetch_and_parse(
        api: &Self::Api,
        best: &Self::SearchResult,
    ) -> LyrixResult<Vec<LineInfo>> {
        use crate::parsers::kugou::KugouParser;
        let keyword = format!("{} {}", best.title, best.artists.join(", "));
        let lyrics_resp = api
            .get_search_lyrics(Some(&keyword), Some(&best.hash))
            .await?
            .ok_or_else(|| SearcherError::NoResults {
                label: "酷狗".to_string(),
                query: keyword.clone(),
            })?;
        let candidates = lyrics_resp.candidates.unwrap_or_default();
        let candidate = candidates
            .first()
            .ok_or_else(|| SearcherError::MissingField {
                label: "酷狗".to_string(),
                field: "candidate".to_string(),
            })?;
        let id = candidate
            .id
            .as_deref()
            .ok_or_else(|| SearcherError::MissingField {
                label: "酷狗".to_string(),
                field: "id".to_string(),
            })?;
        let access_key =
            candidate
                .access_key
                .as_deref()
                .ok_or_else(|| SearcherError::MissingField {
                    label: "酷狗".to_string(),
                    field: "accesskey".to_string(),
                })?;
        let dl_resp = api.get_download_krc(id, access_key).await?.ok_or_else(|| {
            GeneralError::MissingField {
                field: "酷狗: 下载 KRC 失败".to_string(),
            }
        })?;
        let krc = dl_resp.content.ok_or_else(|| GeneralError::MissingField {
            field: "酷狗: KRC 内容为空".to_string(),
        })?;
        if krc.is_empty() {
            return Err(GeneralError::MissingField {
                field: "酷狗: KRC 内容为空".to_string(),
            }
            .into());
        }
        Ok(KugouParser {}.decrypt_and_parse(krc)?)
    }
}
