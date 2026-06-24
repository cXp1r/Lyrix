# 错误处理

Lyrix 使用 `thiserror` 构建统一错误体系。所有公开 API 都返回 `LyrixResult<T>`，也就是 `Result<T, LyrixError>`。

## 错误层次

```text
LyrixError
├── ParserError
│   ├── ParseError
│   ├── DecryptError
│   └── TotpGenError
├── FetcherError
│   ├── HttpError
│   ├── JsonError
│   ├── AuthError
│   └── ProxyError
├── SearcherError
└── GeneralError
```

## 顶层匹配

```rust
use lyrix::error::{GeneralError, LyrixError, ParserError, FetcherError, SearcherError};

match lyrix.get_lyrics_with_appid(app_id, title, Some(artist), None, None, 0).await {
    Ok(data) => {
        // 正常处理
    }
    Err(e) => match e {
        LyrixError::Parser(e) => eprintln!("歌词解析失败: {e}"),
        LyrixError::Provider(e) => eprintln!("API 调用失败: {e}"),
        LyrixError::Searcher(e) => eprintln!("歌曲匹配失败: {e}"),
        LyrixError::General(e) => eprintln!("通用错误: {e}"),
    },
}
```

## 各层错误

### ParserError

Parser 层用于歌词文本解析、解密和 TOTP 生成。

```rust
use lyrix::error::ParserError;
use lyrix::error::parser::ParseError;

match err {
    LyrixError::Parser(e) => match e {
        ParserError::LyricsParse(inner) => match inner {
            ParseError::InvalidStructure { detail } => {
                eprintln!("歌词格式异常: {detail}");
            }
            ParseError::TimestampParse { field, raw } => {
                eprintln!("时间戳解析失败: field={field}, raw={raw}");
            }
            ParseError::EmptyContent => {
                eprintln!("歌词内容为空");
            }
            ParseError::UnknownSyncType => {
                eprintln!("无法识别的歌词同步类型");
            }
        },
        ParserError::Decrypt(e) => eprintln!("歌词解密失败: {e}"),
        ParserError::TotpGenerate(e) => eprintln!("Spotify TOTP 生成失败: {e}"),
    },
    _ => {}
}
```

### FetcherError

Fetcher 层用于 HTTP、JSON、鉴权和代理错误。

```rust
use lyrix::error::FetcherError;

match err {
    LyrixError::Fetcher(e) => match e {
        FetcherError::Http(e) => eprintln!("HTTP 错误: {e}"),
        FetcherError::Json(e) => eprintln!("JSON 解析失败 ({}): {}", e.api, e.source),
        FetcherError::Auth(e) => eprintln!("鉴权失败: {e}"),
        FetcherError::Proxy(e) => eprintln!("代理配置错误: {e}"),
    },
    _ => {}
}
```

### SearcherError

Searcher 层最常见的是“没搜到”或“匹配分太低”。

```rust
use lyrix::error::SearcherError;

match err {
    LyrixError::Searcher(e) => match e {
        SearcherError::NoResults { label, query } => {
            eprintln!("{label} 没有搜索结果（query={query}）");
        }
        SearcherError::LowScore { label, score, threshold, query } => {
            eprintln!("{label} 匹配分过低: {score}/{threshold}（query={query}）");
        }
        SearcherError::NoMatch { label, title } => {
            eprintln!("{label} 没有匹配到曲目: {title}");
        }
        SearcherError::MissingField { label, field } => {
            eprintln!("{label} 结果缺少字段: {field}");
        }
    },
    _ => {}
}
```

### GeneralError

General 层放跨模块、平台和 I/O 等通用错误。

```rust
use lyrix::error::GeneralError;

match err {
    LyrixError::General(e) => match e {
        GeneralError::UnsupportedPlayer { name } => {
            eprintln!("不支持的播放器: {name}");
        }
        GeneralError::MissingField { field } => {
            eprintln!("缺少必要字段: {field}");
        }
        GeneralError::Io(e) => {
            eprintln!("I/O 错误: {e}");
        }
        GeneralError::Internal { detail } => {
            eprintln!("内部错误: {detail}");
        }
        GeneralError::Platform { platform } => {
            eprintln!("平台错误: {platform}");
        }
    },
    _ => {}
}
```

## 实用模式

### 1. 多播放器降级

```rust
use lyrix::error::LyrixError;
use lyrix::smtc_lyrics::MusicPlayer;

let players = [MusicPlayer::Netease, MusicPlayer::QQMusic, MusicPlayer::Kugou];

for player in &players {
    match lyrix
        .get_lyrics_with_player(player, title, Some(artist), None, None, duration)
        .await
    {
        Ok(data) => return Some(data),
        Err(LyrixError::Searcher(_)) => continue,
        Err(e) => {
            eprintln!("{} 出错: {}", player.display_name(), e);
        }
    }
}
None
```

### 2. 重试 HTTP 失败

```rust
use lyrix::error::provider::HttpError;
use lyrix::error::{LyrixError, FetcherError, LyrixResult};

async fn get_lyrics_with_retry(
    lyrix: &Lyrix,
    app_id: &str,
    title: &str,
    artist: &str,
) -> LyrixResult<LyricsData> {
    for attempt in 0..3 {
        match lyrix
            .get_lyrics_with_appid(app_id, title, Some(artist), None, None, 0)
            .await
        {
            Ok(data) => return Ok(data),
            Err(LyrixError::Provider(FetcherError::Http(ref e))) => {
                match e {
                    HttpError::TooManyRequests { .. }
                    | HttpError::ServerError { .. }
                    | HttpError::BadGateway { .. }
                    | HttpError::ServiceUnavailable { .. } => {
                        if attempt < 2 {
                            tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt))).await;
                            continue;
                        }
                    }
                    _ => {}
                }
                return Err(LyrixError::Provider(FetcherError::Http(e.clone())));
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}
```

### 3. 终端友好提示

```rust
fn user_friendly_msg(e: &LyrixError) -> String {
    match e {
        LyrixError::Searcher(se) => match se {
            SearcherError::NoMatch { title, .. } => format!("未找到「{title}」的歌词"),
            SearcherError::NoResults { .. } => "搜索无结果，请检查歌曲信息是否正确".into(),
            SearcherError::LowScore { .. } => "匹配度过低，可能是冷门歌曲或信息不完整".into(),
            _ => format!("搜索失败: {se}"),
        },
        LyrixError::Provider(pe) => match pe {
            FetcherError::Auth(ae) => format!("鉴权失败: {ae}"),
            FetcherError::Http(he) => format!("网络请求失败: {he}"),
            _ => format!("请求失败: {pe}"),
        },
        LyrixError::Parser(pe) => format!("歌词解析失败: {pe}"),
        LyrixError::General(ge) => match ge {
            GeneralError::UnsupportedPlayer { name } => format!("暂不支持播放器: {name}"),
            GeneralError::MissingField { field } => format!("数据不完整: 缺少 {field}"),
            _ => format!("未知错误: {ge}"),
        },
    }
}
```
