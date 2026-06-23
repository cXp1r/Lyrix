use crate::error::provider::http::HttpError;
use crate::error::LyrixResult;
use crate::logger;
use reqwest::{Client, header};
use std::collections::HashMap;
use std::time::Duration;

pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";
//失效了更新这个


#[derive(Clone)]
pub struct BaseApi {
    pub client: Client,
    pub http_refer: Option<String>,
    pub additional_headers: Option<HashMap<String, String>>,
}

impl BaseApi {
    pub fn new(http_refer: Option<&str>, additional_headers: Option<HashMap<String, String>>) -> Self {
        Self {
            client: Client::new(),
            http_refer: http_refer.map(|s| s.to_string()),
            additional_headers,
        }
    }

    pub fn with_client(client: Client, http_refer: Option<&str>, additional_headers: Option<HashMap<String, String>>) -> Self {
        Self {
            client,
            http_refer: http_refer.map(|s| s.to_string()),
            additional_headers,
        }
    }

    fn build_headers(&self) -> header::HeaderMap {
        let mut headers = header::HeaderMap::new();
        if let Ok(ua) = header::HeaderValue::from_str(USER_AGENT) {
            headers.insert(header::USER_AGENT, ua);
        }
        if let Some(ref refer) = self.http_refer {
            if let Ok(r) = header::HeaderValue::from_str(refer) {
                headers.insert(header::REFERER, r);
            }
        }
        if let Some(ref additional) = self.additional_headers {
            for (key, value) in additional {
                if let (Ok(k), Ok(v)) = (
                    header::HeaderName::from_bytes(key.as_bytes()),
                    header::HeaderValue::from_str(value),
                ) {
                    headers.insert(k, v);
                }
            }
        }
        headers
    }

    fn classify_reqwest_error(url: &str, e: &reqwest::Error) -> HttpError {
        if e.is_timeout() {
            HttpError::Timeout {
                url: url.to_string(),
            }
        } else if e.is_connect() {
            HttpError::ConnectionFailed {
                detail: e.to_string(),
                url: url.to_string(),
            }
        } else {
            HttpError::ConnectionFailed {
                detail: e.to_string(),
                url: url.to_string(),
            }
        }
    }

    fn status_to_error(status: reqwest::StatusCode, url: &str) -> HttpError {
        let url = url.to_string();
        match status.as_u16() {
            400 => HttpError::BadRequest { url },
            401 => HttpError::Unauthorized { url },
            403 => HttpError::Forbidden { url },
            404 => HttpError::NotFound { url },
            429 => HttpError::TooManyRequests { url },
            500 => HttpError::ServerError { url },
            502 => HttpError::BadGateway { url },
            503 => HttpError::ServiceUnavailable { url },
            301 | 302 => HttpError::Redirect {
                status: status.as_u16(),
                url,
            },
            s => HttpError::OtherStatus { status: s, url },
        }
    }

    pub async fn get_async(&self, url: &str) -> LyrixResult<String> {
        let start = std::time::Instant::now();
        let result = async {
            let resp = self
                .client
                .get(url)
                .headers(self.build_headers())
                .send()
                .await
                .map_err(|e| Self::classify_reqwest_error(url, &e))?;

            if !resp.status().is_success() {
                return Err(Self::status_to_error(resp.status(), url));
            }

            resp.text().await.map_err(|e| Self::classify_reqwest_error(url, &e))
        }
        .await;
        log_http_result("GET", url, start.elapsed(), &result);
        result.map_err(Into::into)
    }

    pub async fn post_form_async(
        &self,
        url: &str,
        params: &HashMap<String, String>,
    ) -> LyrixResult<String> {
        let start = std::time::Instant::now();
        let result = async {
            let resp = self
                .client
                .post(url)
                .headers(self.build_headers())
                .form(params)
                .send()
                .await
                .map_err(|e| Self::classify_reqwest_error(url, &e))?;

            if !resp.status().is_success() {
                return Err(Self::status_to_error(resp.status(), url));
            }

            resp.text().await.map_err(|e| Self::classify_reqwest_error(url, &e))
        }
        .await;
        log_http_result("POST_FORM", url, start.elapsed(), &result);
        result.map_err(Into::into)
    }

    pub async fn post_json_async<T: serde::Serialize + ?Sized>(
        &self,
        url: &str,
        body: &T,
    ) -> LyrixResult<String> {
        let start = std::time::Instant::now();
        let result = async {
            let resp = self
                .client
                .post(url)
                .headers(self.build_headers())
                .json(body)
                .send()
                .await
                .map_err(|e| Self::classify_reqwest_error(url, &e))?;

            if !resp.status().is_success() {
                return Err(Self::status_to_error(resp.status(), url));
            }

            resp.text().await.map_err(|e| Self::classify_reqwest_error(url, &e))
        }
        .await;
        log_http_result("POST_JSON", url, start.elapsed(), &result);
        result.map_err(Into::into)
    }

    pub async fn post_string_async(
        &self,
        url: &str,
        body: &str,
    ) -> LyrixResult<String> {
        let start = std::time::Instant::now();
        let result = async {
            let resp = self
                .client
                .post(url)
                .headers(self.build_headers())
                .header(header::CONTENT_TYPE, "application/json")
                .body(body.to_string())
                .send()
                .await
                .map_err(|e| Self::classify_reqwest_error(url, &e))?;

            if !resp.status().is_success() {
                return Err(Self::status_to_error(resp.status(), url));
            }

            resp.text().await.map_err(|e| Self::classify_reqwest_error(url, &e))
        }
        .await;
        log_http_result("POST_STRING", url, start.elapsed(), &result);
        result.map_err(Into::into)
    }
}

fn log_http_result(
    method: &str,
    url: &str,
    elapsed: Duration,
    result: &Result<String, HttpError>,
) {
    let url = sanitize_url(url);
    match result {
        Ok(body) => logger::debug(
            "provider::http",
            format_args!(
                "request completed | method={} | url={} | elapsed={:?} | bytes={}",
                method,
                url,
                elapsed,
                body.len()
            ),
        ),
        Err(err) => logger::warn(
            "provider::http",
            format_args!(
                "request failed | method={} | url={} | elapsed={:?} | error={}",
                method,
                url,
                elapsed,
                err
            ),
        ),
    }
}

fn sanitize_url(url: &str) -> String {
    let Ok(parsed) = reqwest::Url::parse(url) else {
        return url.split('?').next().unwrap_or(url).to_string();
    };
    let mut origin = format!(
        "{}://{}",
        parsed.scheme(),
        parsed.host_str().unwrap_or_default()
    );
    if let Some(port) = parsed.port() {
        origin.push_str(&format!(":{}", port));
    }
    format!("{}{}", origin, parsed.path())
}
