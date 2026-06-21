# Lyrix
![Rust](https://img.shields.io/badge/Rust-1.70+-red)
![Version](https://img.shields.io/badge/version-26.8.2-green)
## 优点
- 封装了统一函数可以直接接收smtc信息进行歌词解析
- memchr予以的超高性能，无需预热或优化即可实现<1ms解析

## 计划(按优先级排序)
- [ ] 审计代码
- [ ] 提升applemusic速度
- [ ] 新增spotify和酷狗音乐测试
- [ ] Spotify实装测试
- [ ] Spotify逐字同步部分
- [ ] 洛雪音乐
- [ ] AppleMusic 实装测试
## 已完成
- [x] 新增单元测试(由ai负责)
- [x] 优化错误返回格式
- [x] 检修汽水音乐命中率低下原因
- [x] 改用初始化时返回一个结构体,所有的操作使用impl避免重复初始化
- [x] 硬编码测试参数到json文件中
- [x] Spotify逐行测试
- [x] Spotify totp逆向(已完成, 待接入)
- [x] 汽水音乐试用区间测试
- [x] 试听音乐区间捕获
- [x] AppleMusic 逐字解析
- [x] AppleMusic 防碰撞

## 功能

- **Providers** — 网易云,QQ音乐,酷狗,汽水音乐,AppleMusic的 API 客户端.(酷狗,汽水,网易云,qq音乐api接口参考于[Lyricify-Lyrics-Helper](https://github.com/WXRIW/Lyricify-Lyrics-Helper))
- **Searchers** — 弱智评分机制 + 神人匹配字符串,返回最佳匹配
- **Parsers** — µs级别解析网易云,汽水,QQ音乐,酷狗音乐,AppleMusic歌词,可解析**逐字高亮歌词**
- **smtc_lyrics** — 一键从smtc信息到歌词, 另有试用区间歌词捕获
## 安装

cargo add lyrix

或

在 `Cargo.toml` 中添加：
```toml
[dependencies]
lyrix = { version = "26.8.2" }
tokio = { version = "1", features = ["full"] }
```


## 使用教程

### 1. 快速开始

```rust
use lyrix::smtc_lyrics::{Lyrix, Session};

#[tokio::main]
async fn main() {
    // 初始化 Lyrix（无需 session 时传 None）
    let lyrix = Lyrix::new(None);

    // 通过 SMTC 的 app_id 获取歌词
    match lyrix
        .get_lyrics_with_appid(
            "cloudmusic.exe",         // SMTC 上报的 app_id
            "晴天",                   // 歌曲标题（必填）
            Some("周杰伦"),           // 艺术家（可选）
            Some("叶惠美"),           // 专辑（可选）
            None,                     // 专辑艺术家（可选）
            269000,                   // 时长（毫秒，可选，填 0 亦可）
        )
        .await
    {
        Ok(data) => {
            for line in &data.lines {
                if line.text.is_empty() {
                    continue;
                }
                println!(
                    "[{}ms] {}",
                    line.start_time,
                    line.text
                );
            }
        }
        Err(e) => eprintln!("获取失败: {}", e),
    }
}
```

### 2. 枚举指定播放器

当你已知道播放器类型，可直接用枚举：

```rust
use lyrix::smtc_lyrics::MusicPlayer;

let result = lyrix
    .get_lyrics_with_player(
        &MusicPlayer::Netease,  // 直接指定网易云
        "晴天",
        Some("周杰伦"),
        None,
        None,
        269000,
    )
    .await;
```

### 3. 逐字高亮歌词（Syllable / Word-level）

`LineInfo` 中 `syllables` 字段承载逐字时间轴。非逐字歌词时为空，此时使用 `text` 整行展示：

```rust
for line in &data.lines {
    if !line.syllables.is_empty() {
        // 逐字模式：每个音节有独立 start_time + duration
        for syl in &line.syllables {
            println!(
                "  [{}+{}ms] {}",
                syl.start_time,
                syl.duration,
                syl.text
            );
        }
    } else {
        // 整行模式
        println!("[{}ms] {}", line.start_time, line.text);
    }
}
```

> **注意**：网易云 YRC、QQ 音乐 QRC、AppleMusic TTML 支持逐字；酷狗 KRC、标准 LRC 仅有行级。

### 4. 鉴权（AppleMusic / Spotify）

AppleMusic 和 Spotify 需要凭证。其他播放器（网易云、QQ、酷狗、汽水）无需 session：

```rust
use lyrix::smtc_lyrics::Session;

let session = Session {
    applemusic_token: Some("your-applemusic-token".into()),
    spotify_cookie: Some("your-spotify-cookie".into()),
};

// 方式一：构造时传入
let lyrix = Lyrix::new(Some(session));

// 方式二：后期设置（会覆盖旧 session）
lyrix.set_session(Some(session))?;
```

### 5. 试用歌曲裁剪

部分汽水音乐 / QQ 音乐返回的是试听片段（通常 60s），`track_metadata.is_trial` 标记。**裁剪后歌词时间轴偏移至试听开始部分**：

```rust
match result {
    Ok(data) => {
        let is_trial = data
            .track_metadata
            .as_ref()
            .map(|m| m.is_trial)
            .unwrap_or(false);

        let data = if is_trial {
            // 裁剪到试听区间，第一句 start_time 归零
            lyrix.get_trial_part(data).unwrap_or(data)
        } else {
            data
        };

        println!("共 {} 行歌词", data.lines.len());
    }
    Err(e) => eprintln!("{}", e),
}
```

### 6. 日志配置

Lyrix 自带轻量日志系统，输出到控制台和 `%APPDATA%/lyrix/Lyrix_<timestamp>.log`：

```rust
use lyrix::logger;

// 设置日志等级：debug / info / warn / error / none
logger::set_level("debug");   // 显示所有级别
logger::set_level("info");    // 默认：过滤 debug
logger::set_level("none");    // 关闭日志

// 按标签过滤（只打印匹配 tag 的日志）
logger::set_filter(vec!["netease".into(), "qqmusic".into()], false);

// 反向过滤（排除匹配 tag 的日志）
logger::set_filter(vec!["proxy".into()], true);

// 关闭控制台输出（仅写文件）
logger::set_console_output(false);

// 手动打日志
logger::info("my-app", "歌词获取成功");
logger::warn("my-app", "凭证即将过期");
logger::error("my-app", format!("请求失败: {}", err));
```

### 7. 代理设置

```rust
use lyrix::providers::proxy;
use lyrix::providers::netease::NeteaseApi;

let client = proxy::create_proxy_client(
    "127.0.0.1",  // 代理 IP
    7890,         // 代理端口
    None,         // 用户名（可选）
    None,         // 密码（可选）
)?;
let api = NeteaseApi::with_client(client);
```

### 8. LyricsData 结构

```rust
pub struct LyricsData {
    pub file: Option<LyricsFileInfo>,           // 文件来源信息（非 API 时使用）
    pub lines: Vec<LineInfo>,                   // 歌词行列表
    pub track_metadata: Option<TrackMetadata>,   // 曲目元数据（匹配评分、试听标记等）
}
```

`TrackMetadata` 关键字段：

| 字段 | 类型 | 说明 |
|------|------|------|
| `title` | `Option<String>` | 匹配到的曲目标题 |
| `artist` | `Option<String>` | 艺术家名 |
| `duration_ms` | `Option<u32>` | 时长（毫秒） |
| `score` | `i8` | 匹配评分（0-100） |
| `is_trial` | `bool` | 是否为试听歌曲 |
| `trial` | `Option<[u32; 2]>` | 试听区间 `[start_ms, duration_ms]` |

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
