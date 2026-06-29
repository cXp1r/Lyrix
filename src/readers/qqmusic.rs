use crate::error::{GeneralError, LyrixResult};
use crate::models::ITrackMetadata;
use crate::parsers::decrypt::qrc::qrc_decrypt_file;
use crate::readers::readers::LyrixReader;
use async_trait::async_trait;
use std::fs;
use std::path::{Path, PathBuf};

const QQMUSIC_WEBKIT_CACHE_INI_RELATIVE: &str = r"Tencent\QQMusic\WebkitCachePath.ini";
const QQMUSIC_LYRIC_NEW_DIR_NAME: &str = "QQMusicLyricNew";
const QQMUSIC_QRC_SUFFIX: &str = "_qm.qrc";
const QQMUSIC_QRC_PART_SEPARATOR: &str = " - ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QQMusicQrcFilenameInfo {
    pub artist: Vec<String>,
    pub title: String,
    pub index: String,
    pub album: String,
}

pub fn qqmusic_lyric_new_dir() -> LyrixResult<PathBuf> {
    ensure_windows()?;

    let app_data = std::env::var_os("APPDATA").ok_or_else(|| GeneralError::MissingField {
        field: "APPDATA".to_string(),
    })?;
    let ini_path = PathBuf::from(app_data).join(QQMUSIC_WEBKIT_CACHE_INI_RELATIVE);
    let ini = fs::read_to_string(&ini_path).map_err(GeneralError::Io)?;
    let webkit_cache_path = parse_webkit_cache_path(&ini)?;
    let cache_parent = webkit_cache_path
        .parent()
        .ok_or_else(|| GeneralError::Internal {
            detail: format!(
                "webkit cache path has no parent: {}",
                webkit_cache_path.display()
            ),
        })?;

    Ok(cache_parent.join(QQMUSIC_LYRIC_NEW_DIR_NAME))
}

pub fn find_qqmusic_qrc_path_by_metadata_in_dir(
    dir: impl AsRef<Path>,
    track: &dyn ITrackMetadata,
) -> LyrixResult<Option<PathBuf>> {
    let Some(title) = track.title().map(str::trim) else {
        return Err(GeneralError::MissingField {
            field: "track_metadata.title".to_string(),
        }
        .into());
    };
    let Some(_artist) = track.artist().map(str::trim) else {
        return Err(GeneralError::MissingField {
            field: "track_metadata.artist".to_string(),
        }
        .into());
    };
    let Some(album) = track.album().map(str::trim) else {
        return Err(GeneralError::MissingField {
            field: "track_metadata.album".to_string(),
        }
        .into());
    };

    for entry in fs::read_dir(dir.as_ref()).map_err(GeneralError::Io)? {
        let entry = entry.map_err(GeneralError::Io)?;
        let path = entry.path();
        if !is_qrc_file(&path) {
            continue;
        }

        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        let Some(info) = parse_qqmusic_qrc_filename(name) else {
            continue;
        };
        // artist比较暂不处理
        if info.title == title && info.album == album {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

pub fn parse_qqmusic_qrc_filename(file_name: impl AsRef<str>) -> Option<QQMusicQrcFilenameInfo> {
    let file_name = file_name.as_ref().trim();
    let base = file_name.strip_suffix(QQMUSIC_QRC_SUFFIX)?;
    let mut parts = base.rsplitn(4, QQMUSIC_QRC_PART_SEPARATOR);
    let album = parts.next()?.trim();
    let index = parts.next()?.trim();
    let title = parts.next()?.trim();
    let artist = parts.next()?.trim();

    if artist.is_empty() || title.is_empty() || index.is_empty() || album.is_empty() {
        return None;
    }

    if !index.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    Some(QQMusicQrcFilenameInfo {
        artist: artist.split("_").map(String::from).collect(),
        title: title.to_string(),
        index: index.to_string(),
        album: album.to_string(),
    })
}

pub fn is_qqmusic_qrc_filename(file_name: impl AsRef<str>) -> bool {
    parse_qqmusic_qrc_filename(file_name).is_some()
}

fn ensure_windows() -> LyrixResult<()> {
    if cfg!(target_os = "windows") {
        Ok(())
    } else {
        Err(GeneralError::Platform {
            platform: std::env::consts::OS.to_string(),
        }
        .into())
    }
}

fn parse_webkit_cache_path(content: &str) -> LyrixResult<PathBuf> {
    for raw_line in content.lines() {
        let line = raw_line.trim().trim_start_matches('\u{feff}');
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        if key.trim().eq_ignore_ascii_case("Path") {
            let value = value.trim().trim_matches('"');
            if value.is_empty() {
                return Err(GeneralError::MissingField {
                    field: "WebkitCachePath.ini Path".to_string(),
                }
                .into());
            }

            return Ok(PathBuf::from(value));
        }
    }

    Err(GeneralError::MissingField {
        field: "WebkitCachePath.ini Path".to_string(),
    }
    .into())
}

fn is_qrc_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("qrc"))
        .unwrap_or(false)
}

pub struct QQMusicReaders;

#[async_trait]
impl LyrixReader for QQMusicReaders {
    fn label() -> &'static str {
        "QQMusic"
    }

    async fn read_raw(track: &dyn ITrackMetadata) -> LyrixResult<String> {
        let qrc_path = find_qqmusic_qrc_path_by_metadata_in_dir(qqmusic_lyric_new_dir()?, track)?
            .ok_or_else(|| GeneralError::MissingField {
            field: "QQMusic qrc file".to_string(),
        })?;

        qrc_decrypt_file(qrc_path)
    }
}
