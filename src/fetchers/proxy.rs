use crate::error::provider::proxy::ProxyError;
use crate::error::LyrixResult;
use reqwest::{Client, Proxy};

/// 创建带代理的 HTTP Client
pub fn create_proxy_client(
    host: &str,
    port: u16,
    username: Option<&str>,
    password: Option<&str>,
) -> LyrixResult<Client> {
    let proxy_url = format!("http://{}:{}", host, port);
    let mut proxy = Proxy::all(&proxy_url).map_err(|e| ProxyError::InvalidUrl {
        url: proxy_url.clone(),
        reason: e.to_string(),
    })?;
    if let (Some(user), Some(pass)) = (username, password) {
        proxy = proxy.basic_auth(user, pass);
    }
    Client::builder().proxy(proxy).build().map_err(|e| {
        ProxyError::InvalidUrl {
            url: proxy_url,
            reason: e.to_string(),
        }
        .into()
    })
}

/// 创建不使用代理的 HTTP Client
pub fn create_no_proxy_client() -> LyrixResult<Client> {
    Client::builder().no_proxy().build().map_err(|e| {
        ProxyError::InvalidUrl {
            url: "no-proxy".to_string(),
            reason: e.to_string(),
        }
        .into()
    })
}

/// 创建默认 HTTP Client
pub fn create_default_client() -> LyrixResult<Client> {
    Client::builder().build().map_err(|e| {
        ProxyError::InvalidUrl {
            url: "default".to_string(),
            reason: e.to_string(),
        }
        .into()
    })
}
