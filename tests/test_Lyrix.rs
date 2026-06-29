use dialoguer::{theme::ColorfulTheme, Input, Select};
use lyrix::{Lyrix, MusicPlayer, Session};

#[test]
fn test_logger() {
    lyrix::logger::set_level("debug");
    lyrix::logger::debug("test", "hello logger");
}

#[tokio::test]
async fn test_interactive() {
    lyrix::logger::set_level("debug");
    let track_db: Option<serde_json::Value> = std::fs::read_to_string("tests/track.json")
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok());

    let player = choose_player(
        &APP_IDS.iter().map(|(_, p)| *p).collect::<Vec<_>>(),
        "选择播放器",
    );
    let trial_key = choose_trial_key();
    let player_key = player_json_key(player);
    let (title, artist, album, album_artist, duration_ms) =
        choose_track(&track_db, player_key, Some(trial_key));

    let mut session = load_session();
    if player == MusicPlayer::AppleMusic && session.applemusic_token.is_none() {
        let token: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Apple Music token")
            .interact_text()
            .unwrap();
        session.applemusic_token = Some(token);
    }

    let lyrix = Lyrix::new(Some(session));
    let artist_opt = (!artist.is_empty()).then_some(artist.as_str());
    let album_opt = (!album.is_empty()).then_some(album.as_str());
    let album_artist_opt = (!album_artist.is_empty()).then_some(album_artist.as_str());

    let result = lyrix
        .get_lyrics_with_player(
            &player,
            &title,
            artist_opt,
            album_opt,
            album_artist_opt,
            duration_ms,
        )
        .await;

    print_result(result).await;
}

#[tokio::test]
async fn test_interactive_third_party() {
    lyrix::logger::set_level("debug");

    let player = choose_player(THIRD_PARTY_PLAYERS, "选择第三方播放器");
    let session = load_session();
    let lyrix = Lyrix::new(Some(session));

    let result = lyrix
        .get_lyrics_with_player(&player, "", None, None, None, 0)
        .await;

    print_result(result).await;
}

#[tokio::test]
async fn test_benchmark() {
    use std::time::Instant;

    lyrix::logger::set_level("error");
    let track_db: Option<serde_json::Value> = std::fs::read_to_string("tests/track.json")
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok());

    let player = choose_player(
        &APP_IDS.iter().map(|(_, p)| *p).collect::<Vec<_>>(),
        "选择播放器",
    );
    let trial_key = choose_trial_key();
    let player_key = player_json_key(player);
    let (title, artist, album, album_artist, duration_ms) =
        choose_track(&track_db, player_key, Some(trial_key));

    let mut session = load_session();
    if player == MusicPlayer::AppleMusic && session.applemusic_token.is_none() {
        let token: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Apple Music token")
            .interact_text()
            .unwrap();
        session.applemusic_token = Some(token);
    }

    let lyrix = Lyrix::new(Some(session));
    let artist_opt = (!artist.is_empty()).then_some(artist.as_str());
    let album_opt = (!album.is_empty()).then_some(album.as_str());
    let album_artist_opt = (!album_artist.is_empty()).then_some(album_artist.as_str());

    const N: usize = 10;
    let mut elapsed_ms: Vec<f64> = Vec::with_capacity(N);
    let mut ok_count = 0u32;
    let mut fail_count = 0u32;

    let total_start = Instant::now();
    for i in 0..N {
        let t0 = Instant::now();
        let result = lyrix
            .get_lyrics_with_player(
                &player,
                &title,
                artist_opt,
                album_opt,
                album_artist_opt,
                duration_ms,
            )
            .await;
        let dt = t0.elapsed().as_secs_f64() * 1000.0;
        elapsed_ms.push(dt);

        match &result {
            Ok(data) if data.lines.len() > 1 => {
                ok_count += 1;
                println!(
                    "[{}/{}] {:>10.3}ms  OK | {} lines | score={:?}",
                    i + 1,
                    N,
                    dt,
                    data.lines.len(),
                    data.track_metadata.as_ref().map(|m| m.score),
                );
            }
            Ok(data) => {
                fail_count += 1;
                println!(
                    "[{}/{}] {:>10.3}ms FAIL | {} lines (<=1) | score={:?}",
                    i + 1,
                    N,
                    dt,
                    data.lines.len(),
                    data.track_metadata.as_ref().map(|m| m.score),
                );
            }
            Err(e) => {
                fail_count += 1;
                println!("[{}/{}] {:>10.3}ms FAIL | Error: {}", i + 1, N, dt, e);
            }
        }
    }
    let total_ms = total_start.elapsed().as_secs_f64() * 1000.0;

    elapsed_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let sum: f64 = elapsed_ms.iter().sum();
    let avg = sum / N as f64;
    let min = elapsed_ms[0];
    let max = elapsed_ms[N - 1];
    let p50 = elapsed_ms[N / 2];
    let p90 = elapsed_ms[(N * 9) / 10];

    println!("\n========== 10次测试汇总 ==========");
    println!("成功: {} / 失败: {}", ok_count, fail_count);
    println!("总耗时:  {:.3}ms", total_ms);
    println!("平均:    {:.3}ms", avg);
    println!("最小:    {:.3}ms", min);
    println!("最大:    {:.3}ms", max);
    println!("P50:     {:.3}ms", p50);
    println!("P90:     {:.3}ms", p90);
    println!("==================================");
}

const APP_IDS: &[(&str, MusicPlayer)] = &[
    ("cloudmusic.exe", MusicPlayer::Netease),
    ("qqmusic.exe", MusicPlayer::QQMusic),
    ("kugou", MusicPlayer::Kugou),
    ("\u{6c7d}\u{6c34}\u{97f3}\u{4e50}", MusicPlayer::SodaMusic),
    (
        "AppleInc.AppleMusicWin_nzyj5cx40ttqa!App",
        MusicPlayer::AppleMusic,
    ),
    ("Spotify.exe", MusicPlayer::Spotify),
];

const THIRD_PARTY_PLAYERS: &[MusicPlayer] = &[MusicPlayer::MoeKoe];

fn player_json_key(player: MusicPlayer) -> Option<&'static str> {
    match player {
        MusicPlayer::Netease => Some("netease"),
        MusicPlayer::QQMusic => Some("qqmusic"),
        MusicPlayer::Kugou => Some("kugou"),
        MusicPlayer::SodaMusic => Some("soda_music"),
        MusicPlayer::AppleMusic => Some("applemusic"),
        MusicPlayer::Spotify => Some("spotify"),
        _ => None,
    }
}

fn choose_player(players: &[MusicPlayer], prompt: &str) -> MusicPlayer {
    let labels: Vec<&str> = players.iter().map(|p| p.display_name()).collect();
    let sel = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&labels)
        .default(0)
        .interact()
        .unwrap();
    players[sel]
}

fn choose_trial_key() -> &'static str {
    let trial_labels = &["非试听(ntrial)", "试听(trial)"];
    let trial_sel = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择模式")
        .items(trial_labels)
        .default(0)
        .interact()
        .unwrap();
    if trial_sel == 0 {
        "ntrial"
    } else {
        "trial"
    }
}

fn choose_track(
    track_db: &Option<serde_json::Value>,
    player_key: Option<&str>,
    trial_key: Option<&str>,
) -> (String, String, String, String, u32) {
    let mut track_keys: Vec<String> = Vec::new();
    let mut track_labels: Vec<String> = Vec::new();

    if let (Some(db), Some(player_key), Some(trial_key)) =
        (track_db.as_ref(), player_key, trial_key)
    {
        if let Some(tracks) = db.get(player_key).and_then(|p| p.get(trial_key)) {
            if let Some(obj) = tracks.as_object() {
                for (key, val) in obj {
                    let title = val.get("title").and_then(|v| v.as_str()).unwrap_or(key);
                    track_labels.push(format!("{} ({})", key, title));
                    track_keys.push(key.clone());
                }
            }
        }
    }

    track_labels.push("手动输入".to_string());

    let track_sel = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择曲目")
        .items(&track_labels)
        .default(0)
        .interact()
        .unwrap();

    if track_sel < track_keys.len() {
        let track_key = &track_keys[track_sel];
        if let (Some(db), Some(player_key), Some(trial_key)) =
            (track_db.as_ref(), player_key, trial_key)
        {
            if let Some(track_data) = db
                .get(player_key)
                .and_then(|p| p.get(trial_key))
                .and_then(|t| t.get(track_key))
            {
                return (
                    track_data
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    track_data
                        .get("artist")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    track_data
                        .get("album")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    track_data
                        .get("album_artist")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    track_data
                        .get("duration_ms")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32,
                );
            }
        }
        unreachable!()
    } else {
        let title: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("title")
            .interact_text()
            .unwrap();
        let artist: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("artist (可空)")
            .allow_empty(true)
            .interact_text()
            .unwrap();
        let album: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("album (可空)")
            .allow_empty(true)
            .interact_text()
            .unwrap();
        let duration_ms: u32 = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("duration_ms")
            .default(0u32)
            .interact_text()
            .unwrap();
        (title, artist, album, String::new(), duration_ms)
    }
}

fn load_session() -> Session {
    let mut applemusic_token = None;
    let mut spotify_cookie = None;

    if let Ok(content) = std::fs::read_to_string("../auth.json") {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            applemusic_token = json
                .get("applemusic_token")
                .and_then(|v| v.as_str().map(String::from));
            spotify_cookie = json
                .get("spotify_cookie")
                .and_then(|v| v.as_str().map(String::from));
        }
    }

    Session {
        applemusic_token,
        spotify_cookie,
    }
}

async fn print_result(result: lyrix::error::LyrixResult<lyrix::models::LyricsData>) {
    match &result {
        Ok(data) => {
            println!("track_metadata: {:?}", &data.track_metadata);
            println!("\n=== {} lines ===", data.lines.len());
            for line in data.lines.iter().take(1) {
                if line.syllables.is_empty() {
                    println!("[{}ms] {}", line.start_time, line.text);
                } else {
                    for word in &line.syllables {
                        println!("[{}ms] {}", word.start_time, word.text);
                    }
                }
            }
        }
        Err(e) => println!("\nError: {}", e),
    }
}
