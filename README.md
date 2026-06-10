# Lyrix
![Rust](https://img.shields.io/badge/Rust-1.70+-red)
![Version](https://img.shields.io/badge/version-26.7.2-green)
## 优点
- 封装了统一函数可以直接接收smtc信息进行歌词解析
- memchr予以的超高性能，无需预热或优化即可实现<1ms解析

## 计划(按优先级排序)

- [ ] Spotify实装测试
- [ ] Spotify逐字同步部分
- [ ] 洛雪音乐
- [ ] AppleMusic 实装测试
## 已完成
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
lyrix = { version = "26.7.2" }
tokio = { version = "1", features = ["full"] }
```


### 使用

```rust
use lyrix::logger;
use lyrix::smtc_lyrics::{self, MusicPlayer, Session};

// 鉴权结构（AppleMusic / Spotify 需要 token 才能实行下一步操作）
let session = Session {
    applemusic_token: None,
    spotify_cookie: None,
};

// 启用 debug 级别日志（默认 Info 级别，debug 消息会被过滤）
logger::set_level("debug");

// app_id 从 SMTC 获取，也可自行指定Player,见下方表格
let result = smtc_lyrics::get_lyrics_with_appid(
    app_id,
    title,
    artist_opt,
    album_opt,
    album_artist_opt,
    duration_ms_u32,
)
.await;

match result {
    Ok(data) => {
        // 试用区间裁剪：试听歌曲只保留可播放部分
        let data = match data.track_metadata.as_ref().map(|m| m.is_trial) {
            Some(true) => smtc_lyrics::get_trial_part(data).unwrap_or(data),
            _ => data,
        };
        // 使用 data.lines 获取歌词行列表
        println!("共 {} 行歌词", data.lines.len());
    }
    Err(e) => {
        eprintln!("获取歌词失败 [{}]: {}", player.display_name(), e);
    }
}
```

## 支持的播放器

| 播放器 | 枚举值 | appid | 歌词源 |
|--------|--------|--------|--------|
| 酷狗音乐 | `MusicPlayer::Kugou` | `kugou` | 酷狗 API |
| 网易云音乐 | `MusicPlayer::Netease` | `cloudmusic.exe` | 网易云 API（优先 YRC 逐字，回退 LRC） |
| QQ音乐 | `MusicPlayer::QQMusic` | `qqmusic.exe` | QQ音乐 API |
| 汽水音乐 | `MusicPlayer::SodaMusic` | `汽水音乐` | 汽水音乐 API |
| AppleMusic | `MusicPlayer::AppleMusic` | `AppleInc.AppleMusicWin_nzyj5cx40ttqa!App` | AppleMusic API |

## 模块结构

```text
src/
├── lib.rs
├── logger.rs
├── smtc_lyrics.rs
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

## 代理设置

```rust
use lyrix::providers::proxy;
use lyrix::providers::netease::NeteaseApi;

let client = proxy::create_proxy_client("127.0.0.1", 7890, None, None)?;
let api = NeteaseApi::with_client(client);
```

## 许可证

Apache-2.0
