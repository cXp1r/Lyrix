# Lyricify Lyrics Provider
## 澹版槑

閫昏緫(C#婧愪唬鐮?婧愪簬[Lyricify-Lyrics-Helper](https://github.com/WXRIW/Lyricify-Lyrics-Helper)



## 鍔熻兘

- **Providers** 鈥?缃戞槗浜戙€丵Q闊充箰銆侀叿鐙椼€佹苯姘撮煶涔愮殑 API 瀹㈡埛绔?
- **Searchers** 鈥?寮辨櫤璇勫垎鏈哄埗 + 绁炰汉鍖归厤瀛楃涓诧紝杩斿洖鏈€浣冲尮閰?
- **SMTC 姝岃瘝绠＄嚎** 鈥?浼犲叆姝屾洸淇℃伅锛岃嚜鍔ㄦ娴嬭繍琛屼腑鐨勬挱鏀惧櫒杩涚▼锛岀敤鑷婧愯幏鍙栨瓕璇?

## 瀹夎

鍦?`Cargo.toml` 涓坊鍔狅細

```toml
[dependencies]
lyricify-lyrics-provider = { path = "../lyricify-lyrics-provider-rs" }
tokio = { version = "1", features = ["full"] }
```


## 蹇€熶笂鎵?

### SMTC 涓€绔欏紡鑾峰彇姝岃瘝

```rust
use lyricify_lyrics_provider::smtc_lyrics;

#[tokio::main]
async fn main() {
    match smtc_lyrics::get_lyrics(
        "鏅村ぉ",              // 姝屾洸鍚嶏紙蹇呭～锛?
        Some("鍛ㄦ澃浼?),      // 姝屾墜鍚嶏紙鍙€夛級
        Some("鍙舵儬缇?),      // 涓撹緫鍚嶏紙鍙€夛級
        None,                // 鏃堕暱姣锛堝彲閫夛級
    ).await {
        Ok((player, lyrics)) => {
            println!("閫氳繃 {} 鑾峰彇鍒?{} 琛屾瓕璇?,
                player.display_name(), lyrics.lines.len());
        }
        Err(e) => eprintln!("鑾峰彇澶辫触: {}", e),
    }
}
```

**鍐呴儴娴佺▼**锛氭娴嬭繘绋?鈫?鎸夐瀛楁瘝鎺掑簭 (K鈫扤鈫扱鈫扴) 鈫?鍙栫涓€涓?鈫?鐢ㄨ嚜瀹舵簮鎼滅储+鑾峰彇姝岃瘝 鈫?杩斿洖 `LyricsData`

### 鎸囧畾鎾斁鍣ㄦ簮

```rust
use lyricify_lyrics_provider::smtc_lyrics::{self, MusicPlayer};

let lyrics = smtc_lyrics::get_lyrics_with_player(
    &MusicPlayer::Netease,
    "鏅村ぉ", Some("鍛ㄦ澃浼?), None, None,
).await?;
```

### 杩涚▼妫€娴?

```rust
use lyricify_lyrics_provider::smtc_lyrics;

let players = smtc_lyrics::get_running_players();
if let Some(first) = smtc_lyrics::get_first_running_player() {
    println!("灏嗕娇鐢? {}", first.display_name());
}
```

### 鐩存帴璋冪敤骞冲彴 API

```rust
use lyricify_lyrics_provider::providers::netease::NeteaseApi;

let api = NeteaseApi::new();
let result = api.search("鏅村ぉ 鍛ㄦ澃浼?, 1).await?;
let lyric = api.get_lyric("186016").await?;
```

### 鏅鸿兘鎼滅储

```rust
use lyricify_lyrics_provider::searchers::{ISearcher, netease::NeteaseSearcher};

let searcher = NeteaseSearcher::new();
let best = searcher.search_for_result(&track_metadata).await?;
```

### 璁块棶瑙ｆ瀽/妯″瀷/宸ュ叿妯″潡

```rust
use lyricify_lyrics_provider::parsers;
use lyricify_lyrics_provider::models;
use lyricify_lyrics_provider::helpers;
```

## 鏀寔鐨勬挱鏀惧櫒

| 鎾斁鍣?| 鏋氫妇鍊?| 杩涚▼鍚?| 姝岃瘝婧?|
|--------|--------|--------|--------|
| 閰风嫍闊充箰 | `MusicPlayer::Kugou` | `KGMusic.exe` | 閰风嫍 API |
| 缃戞槗浜戦煶涔?| `MusicPlayer::Netease` | `cloudmusic.exe` | 缃戞槗浜?API锛堜紭鍏?YRC 閫愬瓧锛屽洖閫€ LRC锛?|
| QQ闊充箰 | `MusicPlayer::QQMusic` | `QQMusic.exe` | QQ闊充箰 API |
| 姹芥按闊充箰 | `MusicPlayer::SodaMusic` | `SodaMusic.exe` | 姹芥按闊充箰 API |

## 模块结构

```text
src/
├── lib.rs
├── smtc_lyrics.rs
├── helpers/
│   ├── mod.rs
│   └── string_helper.rs
├── models/
│   ├── mod.rs
│   ├── additional_file_info.rs
│   ├── file_info.rs
│   ├── line_info.rs
│   ├── lyrics_data.rs
│   ├── lyrics_types.rs
│   ├── sync_types.rs
│   └── track_metadata.rs
├── parsers/
│   ├── mod.rs
│   ├── attributes_helper.rs
│   ├── kugou.rs
│   ├── lrc.rs
│   ├── netease.rs
│   ├── qqmusic.rs
│   ├── soda_music.rs
│   ├── decrypt/
│   │   ├── mod.rs
│   │   ├── krc.rs
│   │   └── qrc.rs
│   └── models/
│       ├── mod.rs
│       └── yrc_models.rs
├── providers/
│   ├── mod.rs
│   ├── base_api.rs
│   ├── proxy.rs
│   ├── netease.rs
│   ├── qqmusic.rs
│   ├── kugou.rs
│   └── soda_music.rs
└── searchers/
    ├── mod.rs
    ├── netease.rs
    ├── qqmusic.rs
    ├── kugou.rs
    └── soda_music.rs
```

## 浠ｇ悊璁剧疆

```rust
use lyricify_lyrics_provider::providers::proxy;
use lyricify_lyrics_provider::providers::netease::NeteaseApi;

let client = proxy::create_proxy_client("127.0.0.1", 7890, None, None)?;
let api = NeteaseApi::with_client(client);
```

## 璁稿彲璇?

Apache-2.0
