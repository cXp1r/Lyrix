# Lyrix

![Rust](https://img.shields.io/badge/Rust-1.70+-red)
![Version](https://img.shields.io/badge/version-26.9.0-green)

## 优点

- 封装了统一函数可以直接接收smtc信息进行歌词解析
- memchr予以的超高性能，无需预热或优化即可实现<1ms解析

## 注意

- 短期内汽水音乐api校验貌似变严格了, 建议从github上clone库, 而不是cargo add lyrix了

## TODO

- [待办与进度](docs/todo.md)

## 使用示例

- [使用示例](docs/examples.md)

## 错误处理

- [错误处理](docs/error-handling.md)

## 功能

- **Lyrix** — 一键从 SMTC / app_id 获取歌词，自动分发到对应播放器，并支持试听区间裁剪
- **Fetchers** — 网易云、QQ 音乐、酷狗、汽水音乐、Apple Music、Spotify 的 API 客户端
- **Searchers** — 统一搜索与匹配流程，返回最佳匹配结果
- **Parsers** — 解析网易云、汽水、QQ 音乐、酷狗音乐、Apple Music 歌词，支持逐字高亮歌词
- **logger / proxy** — 内置轻量日志与代理客户端，方便调试和网络请求配置

## 安装

cargo add lyrix
在 `Cargo.toml` 中添加：
```toml
[dependencies]
lyrix = { version = "26.9.0" }
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
| MoeKoe Music | `MusicPlayer::MoeKoe` | `cn.MoeKoe.Music` | MoeKoe WebSocket（连接 `ws://127.0.0.1:6520`） |
| 落雪音乐 | `MusicPlayer::LXMusic` | `cn.toside.music.desktop` | 开发中 |
| Any Listen | `MusicPlayer::AnyListen` | `cn.toside.anylisten.desktop` | 开发中 |

## 模块结构

```text
src/
├── error
│   ├── fetcher
│   │   ├── auth.rs
│   │   ├── http.rs
│   │   ├── json.rs
│   │   ├── mod.rs
│   │   └── proxy.rs
│   ├── general
│   │   └── mod.rs
│   ├── mod.rs
│   ├── parser
│   │   ├── decrypt.rs
│   │   ├── lyrics_parse.rs
│   │   ├── mod.rs
│   │   └── totp_gen.rs
│   └── searcher
│       └── mod.rs
├── fetchers
│   ├── applemusic.rs
│   ├── base_api.rs
│   ├── kugou.rs
│   ├── mod.rs
│   ├── netease.rs
│   ├── proxy.rs
│   ├── qqmusic.rs
│   ├── soda_music.rs
│   └── spotify.rs
├── files
│   ├── mod.rs
│   └── qqmusic.rs
├── lib.rs
├── logger.rs
├── lyrix.rs
├── models
│   ├── file_info.rs
│   ├── line_info.rs
│   ├── lyrics_data.rs
│   ├── lyrics_types.rs
│   ├── mod.rs
│   ├── music_player.rs
│   ├── session.rs
│   ├── sync_types.rs
│   └── track_metadata.rs
├── parsers
│   ├── applemusic.rs
│   ├── decrypt
│   │   ├── krc.rs
│   │   ├── mod.rs
│   │   ├── netease.rs
│   │   └── qrc.rs
│   ├── generate
│   │   ├── mod.rs
│   │   └── spotify.rs
│   ├── kugou.rs
│   ├── lrc.rs
│   ├── mod.rs
│   ├── netease.rs
│   ├── qqmusic.rs
│   ├── soda_music.rs
│   └── spotify.rs
├── providers
│   ├── applemusic.rs
│   ├── kugou.rs
│   ├── lyrics_provider.rs
│   ├── mod.rs
│   ├── netease.rs
│   ├── qqmusic.rs
│   ├── soda_music.rs
│   └── spotify.rs
└── searchers
    ├── applemusic.rs
    ├── kugou.rs
    ├── mod.rs
    ├── netease.rs
    ├── qqmusic.rs
    ├── soda_music.rs
    └── spotify.rs
```

## 许可证

Apache-2.0
