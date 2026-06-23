# Lyrix
![Rust](https://img.shields.io/badge/Rust-1.70+-red)
![Version](https://img.shields.io/badge/version-26.9.0-green)
## 优点
- 封装了统一函数可以直接接收smtc信息进行歌词解析
- memchr予以的超高性能，无需预热或优化即可实现<1ms解析

## TODO
- [待办与进度](docs/todo.md)

## 使用示例
- [使用示例](docs/examples.md)

## 功能

- **Providers** — 网易云,QQ音乐,酷狗,汽水音乐,AppleMusic的 API 客户端.(酷狗,汽水,网易云,qq音乐api接口参考于[Lyricify-Lyrics-Helper](https://github.com/WXRIW/Lyricify-Lyrics-Helper))
- **Searchers** — 弱智评分机制 + 神人匹配字符串,返回最佳匹配
- **Parsers** — µs级别解析网易云,汽水,QQ音乐,酷狗音乐,AppleMusic歌词,可解析**逐字高亮歌词**
- **smtc_lyrics** — 一键从smtc信息到歌词, 另有试用区间歌词捕获
## 安装

cargo add lyrix
在 `Cargo.toml` 中添加：
```toml
[dependencies]
lyrix = { version = "26.8.2" }
tokio = { version = "1", features = ["full"] }
```

## 支持的播放器

| 播放器 | 枚举值 | appid | 歌词源 |
|--------|--------|--------|--------|
| 酷狗音乐 | `MusicPlayer::Kugou` | `kugou` | 酷狗 API |
| 网易云音乐 | `MusicPlayer::Netease` | `cloudmusic.exe` | 网易云 API（优先 YRC 逐字，回退 LRC） |
| QQ音乐 | `MusicPlayer::QQMusic` | `qqmusic.exe` | QQ音乐 API（优先 QRC 逐字，回退 LRC） |
| 汽水音乐 | `MusicPlayer::SodaMusic` | `汽水音乐` | 汽水音乐 API |
| Spotify | `MusicPlayer::Spotify` | `Spotify.exe` | Spotify API（需 `spotify_cookie`） |
| AppleMusic | `MusicPlayer::AppleMusic` | `AppleInc.AppleMusicWin_nzyj5cx40ttqa!App` | AppleMusic API（需 `applemusic_token`） |
| 落雪音乐 | `MusicPlayer::LXMusic` | `cn.toside.music.desktop` | 开发中 |
| Any Listen | `MusicPlayer::AnyListen` | `cn.toside.anylisten.desktop` | 开发中 |

## 模块结构

```text
src/
├── lib.rs
├── logger.rs
├── smtc_lyrics.rs
├── error/
│   ├── mod.rs
│   ├── general/
│   │   └── mod.rs
│   ├── parser/
│   │   ├── mod.rs
│   │   ├── lyrics_parse.rs
│   │   ├── decrypt.rs
│   │   └── totp_gen.rs
│   ├── provider/
│   │   ├── mod.rs
│   │   ├── http.rs
│   │   ├── json.rs
│   │   ├── auth.rs
│   │   └── proxy.rs
│   └── searcher/
│       └── mod.rs
├── models/
│   ├── mod.rs
│   ├── file_info.rs
│   ├── line_info.rs
│   ├── lyrics_data.rs
│   ├── lyrics_types.rs
│   ├── sync_types.rs
│   └── track_metadata.rs
├── parsers/
│   ├── mod.rs
│   ├── applemusic.rs
│   ├── kugou.rs
│   ├── lrc.rs
│   ├── netease.rs
│   ├── qqmusic.rs
│   ├── soda_music.rs
│   ├── spotify.rs
│   ├── decrypt/
│   │   ├── mod.rs
│   │   ├── krc.rs
│   │   ├── netease.rs
│   │   └── qrc.rs
│   └── generate/
│       ├── mod.rs
│       └── spotify.rs
├── providers/
│   ├── mod.rs
│   ├── applemusic.rs
│   ├── base_api.rs
│   ├── kugou.rs
│   ├── netease.rs
│   ├── proxy.rs
│   ├── qqmusic.rs
│   ├── soda_music.rs
│   └── spotify.rs
└── searchers/
    ├── mod.rs
    ├── applemusic.rs
    ├── kugou.rs
    ├── netease.rs
    ├── qqmusic.rs
    ├── soda_music.rs
    └── spotify.rs
```

## 错误处理教程

Lyrix 使用 `thiserror` 构建了四层统一错误类型。所有公开 API 返回 `LyrixResult<T>`（即 `Result<T, LyrixError>`）。

### 错误层次结构

```
LyrixError          ← 库级统一错误
├── ParserError     ← 解析器层
│   ├── LyricsParseError   ← 歌词文本解析
│   ├── DecryptError       ← 解密（KRC/QRC/网易云EAPI）
│   └── TotpGenError       ← Spotify TOTP 生成
├── ProviderError   ← 提供器层（API 调用）
│   ├── HttpError          ← HTTP 请求
│   ├── JsonError          ← JSON 反序列化
│   ├── AuthError          ← 鉴权
│   └── ProxyError         ← 代理配置
├── SearcherError   ← 搜索器层（匹配歌曲）
└── GeneralError    ← 通用/杂项
```

### 基础错误匹配

使用 `match` 按层级细化处理：

```rust
use lyrix::error::{LyrixError, ParserError, ProviderError, SearcherError, GeneralError};

match lyrix.get_lyrics_with_appid(app_id, title, Some(artist), None, None, 0).await {
    Ok(data) => { /* 正常处理 */ }
    Err(e) => match e {
        LyrixError::Parser(e) => {
            eprintln!("歌词解析失败: {}", e);
            // 可继续细化 ↓
        }
        LyrixError::Provider(e) => {
            eprintln!("API 调用失败: {}", e);
        }
        LyrixError::Searcher(e) => {
            eprintln!("未匹配到歌曲: {}", e);
        }
        LyrixError::General(e) => {
            eprintln!("其他错误: {}", e);
        }
    },
}
```

### 细化匹配：处理具体错误变体

#### SearcherError —— 最常见的匹配失败

```rust
use lyrix::error::SearcherError;

Err(LyrixError::Searcher(e)) => match e {
    SearcherError::NoResults { label, query } => {
        // 搜索 API 返回空结果
        eprintln!("{label} 未搜到任何结果（query={query}）");
    }
    SearcherError::LowScore { label, score, threshold, query } => {
        // 有候选但匹配分太低
        eprintln!("{label} 匹配分 {score} 低于阈值 {threshold}，跳过");
    }
    SearcherError::NoMatch { label, title } => {
        // 搜到了但没命中当前曲目
        eprintln!("{label} 未匹配到曲目 \"{title}\"");
    }
    SearcherError::MissingField { label, field } => {
        eprintln!("{label} 搜索结果缺少字段: {field}");
    }
}
```

#### ProviderError —— API 调用层错误

```rust
use lyrix::error::ProviderError;

Err(LyrixError::Provider(e)) => match e {
    ProviderError::Http(e) => {
        // HTTP 状态码 / 连接问题
        eprintln!("HTTP 错误: {}", e);
    }
    ProviderError::Json(e) => {
        // API 返回了无法解析的 JSON
        eprintln!("JSON 解析失败 ({}): {}", e.api, e.source);
    }
    ProviderError::Auth(e) => {
        // 凭证缺失或过期
        eprintln!("鉴权失败: {}", e);
    }
    ProviderError::Proxy(e) => {
        eprintln!("代理配置错误: {}", e);
    }
}
```

#### HttpError —— 按 HTTP 状态码分支

```rust
use lyrix::error::provider::HttpError;

Err(ProviderError::Http(e)) => match e {
    HttpError::Unauthorized { url } | HttpError::Forbidden { url } => {
        // 401/403 → 凭证可能过期，提示用户刷新
        eprintln!("凭证无效，请刷新 token（{url}）");
    }
    HttpError::NotFound { url } => {
        eprintln!("接口不存在: {url}");
    }
    HttpError::TooManyRequests { url } => {
        // 429 → 可退避重试
        eprintln!("请求频率过高，稍后重试: {url}");
    }
    HttpError::ServerError { url }
    | HttpError::BadGateway { url }
    | HttpError::ServiceUnavailable { url } => {
        // 5xx → 可重试
        eprintln!("服务器错误，可稍后重试: {url}");
    }
    HttpError::Timeout { url } => {
        eprintln!("请求超时: {url}");
    }
    HttpError::ConnectionFailed { detail, url } => {
        eprintln!("连接失败（DNS/网络）: {detail} ({url})");
    }
    HttpError::TlsError { detail, url } => {
        eprintln!("TLS 握手失败: {detail} ({url})");
    }
    _ => eprintln!("其他 HTTP 错误: {}", e),
}
```

#### AuthError —— 鉴权专项

```rust
use lyrix::error::provider::AuthError;

Err(ProviderError::Auth(e)) => match e {
    AuthError::MissingCredential { provider, field } => {
        // 未调用 set_session / session 字段为 None
        eprintln!("{provider} 缺少 {field}，请先调用 set_session");
        // 引导用户填写凭证
    }
    AuthError::CredentialExpired { provider, field } => {
        // token/cookie 过期
        eprintln!("{provider} 的 {field} 已过期，请刷新");
    }
}
```

#### ParserError —— 解析层错误

```rust
use lyrix::error::ParserError;
use lyrix::error::parser::LyricsParseError;

Err(LyrixError::Parser(e)) => match e {
    ParserError::LyricsParse(inner) => match inner {
        LyricsParseError::InvalidStructure { detail } => {
            eprintln!("歌词格式异常: {}", detail);
        }
        LyricsParseError::TimestampParse { field, raw } => {
            eprintln!("时间戳解析失败: 字段={field}, 原始值={raw}");
        }
        LyricsParseError::EmptyContent => {
            eprintln!("歌词内容为空");
        }
        LyricsParseError::UnknownSyncType => {
            eprintln!("无法识别的歌词同步类型");
        }
        _ => eprintln!("歌词解析错误: {}", inner),
    },
    ParserError::Decrypt(e) => {
        eprintln!("歌词解密失败: {}", e);
    }
    ParserError::TotpGenerate(e) => {
        eprintln!("Spotify TOTP 生成失败: {}", e);
    }
}
```

#### GeneralError —— 通用错误

```rust
use lyrix::error::GeneralError;

Err(LyrixError::General(e)) => match e {
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
        // 防御性编程：理论上不应发生
        eprintln!("内部错误: {detail}");
    }
}
```

### 实用模式

#### 1. 降级策略：逐个播放器尝试

```rust
use lyrix::smtc_lyrics::MusicPlayer;

let players = [
    MusicPlayer::Netease,
    MusicPlayer::QQMusic,
    MusicPlayer::Kugou,
];

for player in &players {
    match lyrix
        .get_lyrics_with_player(player, title, Some(artist), None, None, duration)
        .await
    {
        Ok(data) => {
            // 成功，直接使用
            return Some(data);
        }
        Err(LyrixError::Searcher(_)) => {
            // 匹配失败，尝试下一个播放器
            continue;
        }
        Err(e) => {
            eprintln!("{} 出错: {}", player.display_name(), e);
            continue;
        }
    }
}
// 全部失败
None
```

#### 2. 重试 429 / 5xx 错误

```rust
use lyrix::error::provider::HttpError;

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
            Err(LyrixError::Provider(ProviderError::Http(ref e))) => {
                match e {
                    HttpError::TooManyRequests { .. }
                    | HttpError::ServerError { .. }
                    | HttpError::BadGateway { .. }
                    | HttpError::ServiceUnavailable { .. } => {
                        if attempt < 2 {
                            tokio::time::sleep(
                                std::time::Duration::from_secs(2u64.pow(attempt))
                            ).await;
                            continue;
                        }
                    }
                    _ => {}
                }
                return Err(LyrixError::Provider(ProviderError::Http(e.clone())));
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}
```

#### 3. 错误友好提示（面向终端用户）

```rust
fn user_friendly_msg(e: &LyrixError) -> String {
    match e {
        LyrixError::Searcher(se) => match se {
            SearcherError::NoMatch { title, .. } => {
                format!("未找到「{title}」的歌词")
            }
            SearcherError::NoResults { .. } => {
                "搜索无结果，请检查歌曲信息是否正确".into()
            }
            SearcherError::LowScore { .. } => {
                "匹配度过低，可能是冷门歌曲或信息不完整".into()
            }
            _ => format!("搜索失败: {se}"),
        },
        LyrixError::Provider(pe) => match pe {
            ProviderError::Auth(ae) => match ae {
                AuthError::MissingCredential { provider, .. } => {
                    format!("{provider} 需要登录凭证，请在设置中填写")
                }
                AuthError::CredentialExpired { provider, .. } => {
                    format!("{provider} 凭证已过期，请重新登录")
                }
            },
            ProviderError::Http(he) => match he {
                HttpError::Timeout { .. } => "请求超时，请检查网络".into(),
                HttpError::ConnectionFailed { .. } => "网络连接失败".into(),
                _ => format!("服务器错误: {he}"),
            },
            _ => format!("请求失败: {pe}"),
        },
        LyrixError::Parser(pe) => format!("歌词解析失败: {pe}"),
        LyrixError::General(ge) => match ge {
            GeneralError::UnsupportedPlayer { name } => {
                format!("暂不支持播放器: {name}")
            }
            GeneralError::MissingField { field } => {
                format!("数据不完整: 缺少 {field}")
            }
            _ => format!("未知错误: {ge}"),
        },
    }
}
```

## 许可证

Apache-2.0
