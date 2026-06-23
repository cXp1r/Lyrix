# 使用示例

## 1. 快速开始

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
                println!("[{}ms] {}", line.start_time, line.text);
            }
        }
        Err(e) => eprintln!("获取失败: {}", e),
    }
}
```

## 2. 枚举指定播放器

当你已知道播放器类型，可直接用枚举：

```rust
use lyrix::smtc_lyrics::MusicPlayer;

let result = lyrix
    .get_lyrics_with_player(
        &MusicPlayer::Netease,
        "晴天",
        Some("周杰伦"),
        None,
        None,
        269000,
    )
    .await;
```

## 3. 逐字高亮歌词

`LineInfo` 中 `syllables` 字段承载逐字时间轴。非逐字歌词时为空，此时使用 `text` 整行展示：

```rust
for line in &data.lines {
    if !line.syllables.is_empty() {
        for syl in &line.syllables {
            println!(
                "  [{}+{}ms] {}",
                syl.start_time,
                syl.duration,
                syl.text
            );
        }
    } else {
        println!("[{}ms] {}", line.start_time, line.text);
    }
}
```

> **注意**：网易云 YRC、QQ 音乐 QRC、AppleMusic TTML 支持逐字；酷狗 KRC、标准 LRC 仅有行级。

## 4. 鉴权

AppleMusic 和 Spotify 需要凭证。其他播放器（网易云、QQ、酷狗、汽水）无需 session：

```rust
use lyrix::smtc_lyrics::Session;

let session = Session {
    applemusic_token: Some("your-applemusic-token".into()),
    spotify_cookie: Some("your-spotify-cookie".into()),
};

let lyrix = Lyrix::new(Some(session));
lyrix.set_session(Some(session))?;
```

## 5. 试用歌曲裁剪

部分汽水音乐 / QQ 音乐返回的是试听片段，`track_metadata.is_trial` 标记。裁剪后歌词时间轴偏移至试听开始部分：

```rust
match result {
    Ok(data) => {
        let is_trial = data
            .track_metadata
            .as_ref()
            .map(|m| m.is_trial)
            .unwrap_or(false);

        let data = if is_trial {
            lyrix.get_trial_part(data).unwrap_or(data)
        } else {
            data
        };

        println!("共 {} 行歌词", data.lines.len());
    }
    Err(e) => eprintln!("{}", e),
}
```

## 6. 日志配置

```rust
use lyrix::logger;

logger::set_level("debug");
logger::set_level("info");
logger::set_level("none");

logger::set_filter(vec!["netease".into(), "qqmusic".into()], false);
logger::set_filter(vec!["proxy".into()], true);

logger::set_console_output(false);

logger::info("my-app", "歌词获取成功");
logger::warn("my-app", "凭证即将过期");
logger::error("my-app", format!("请求失败: {}", err));
```

## 7. 代理设置

```rust
use lyrix::providers::proxy;
use lyrix::providers::netease::NeteaseApi;

let client = proxy::create_proxy_client(
    "127.0.0.1",
    7890,
    None,
    None,
)?;
let api = NeteaseApi::with_client(client);
```

## 8. LyricsData 结构

```rust
pub struct LyricsData {
    pub file: Option<LyricsFileInfo>,
    pub lines: Vec<LineInfo>,
    pub track_metadata: Option<TrackMetadata>,
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
