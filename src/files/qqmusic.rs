use crate::error::{GeneralError, LyrixResult};
use crate::models::ITrackMetadata;
use std::fs;
use std::path::{Path, PathBuf};

const QQMUSIC_WEBKIT_CACHE_INI_RELATIVE: &str = r"Tencent\QQMusic\WebkitCachePath.ini";
const QQMUSIC_LYRIC_NEW_DIR_NAME: &str = "QQMusicLyricNew";
const QQMUSIC_QRC_SUFFIX: &str = "_qm.qrc";
const QQMUSIC_QRC_PART_SEPARATOR: &str = " - ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QQMusicQrcFilenameInfo {
    pub artist: String,
    pub title: String,
    pub index: String,
    pub album: String,
}

/// Returns the QQ Music `QQMusicLyricNew` directory on Windows.
///
/// The lookup flow is:
/// 1. Read `%APPDATA%\Tencent\QQMusic\WebkitCachePath.ini`
/// 2. Extract the `Path=` entry
/// 3. Go to its parent directory
/// 4. Enter `QQMusicLyricNew`
pub fn qqmusic_lyric_new_dir() -> LyrixResult<PathBuf> {
    ensure_windows()?;

    let ini_path = qqmusic_webkit_cache_path_ini()?;
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

/// Returns the path of the `WebkitCachePath.ini` file.
pub fn qqmusic_webkit_cache_path_ini() -> LyrixResult<PathBuf> {
    ensure_windows()?;

    let app_data = std::env::var_os("APPDATA").ok_or_else(|| GeneralError::MissingField {
        field: "APPDATA".to_string(),
    })?;

    Ok(PathBuf::from(app_data).join(QQMUSIC_WEBKIT_CACHE_INI_RELATIVE))
}

/// Lists QQ Music qrc files under `QQMusicLyricNew`.
///
/// The directory is expected to contain only qrc files and no subfolders, so
/// this scans only the current directory and filters by QQ Music filename
/// pattern.
pub fn qqmusic_qrc_files() -> LyrixResult<Vec<PathBuf>> {
    let dir = qqmusic_lyric_new_dir()?;
    let mut files = Vec::new();

    collect_matching_qrc_files(&dir, &mut files)?;
    files.sort();
    Ok(files)
}

/// Reads a specific QQ Music qrc file.
pub fn read_qqmusic_qrc_file(path: impl AsRef<Path>) -> LyrixResult<String> {
    ensure_windows()?;
    Ok(fs::read_to_string(path).map_err(GeneralError::Io)?)
}

/// Reads the first `.qrc` file found in `QQMusicLyricNew`.
pub fn read_first_qqmusic_qrc() -> LyrixResult<String> {
    let qrc_path =
        qqmusic_qrc_files()?
            .into_iter()
            .next()
            .ok_or_else(|| GeneralError::MissingField {
                field: "QQMusic qrc file".to_string(),
            })?;

    read_qqmusic_qrc_file(qrc_path)
}

/// Finds a QQ Music qrc path by exact file name under `QQMusicLyricNew`.
pub fn find_qqmusic_qrc_path(file_name: impl AsRef<str>) -> LyrixResult<Option<PathBuf>> {
    let dir = qqmusic_lyric_new_dir()?;
    find_qqmusic_qrc_path_in_dir(dir, file_name)
}

/// Finds a QQ Music qrc path by exact file name inside a given directory.
///
/// This is useful for real-file tests and fixture-based tests.
pub fn find_qqmusic_qrc_path_in_dir(
    dir: impl AsRef<Path>,
    file_name: impl AsRef<str>,
) -> LyrixResult<Option<PathBuf>> {
    let target = file_name.as_ref().trim();
    if !is_qqmusic_qrc_filename(target) {
        return Ok(None);
    }

    for entry in fs::read_dir(dir.as_ref()).map_err(GeneralError::Io)? {
        let entry = entry.map_err(GeneralError::Io)?;
        let path = entry.path();
        if !is_qrc_file(&path) {
            continue;
        }

        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        if name == target {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

/// Reads a QQ Music qrc file by exact file name.
pub fn read_qqmusic_qrc_by_filename(file_name: impl AsRef<str>) -> LyrixResult<String> {
    let qrc_path = find_qqmusic_qrc_path(file_name)?.ok_or_else(|| GeneralError::MissingField {
        field: "QQMusic qrc file".to_string(),
    })?;

    read_qqmusic_qrc_file(qrc_path)
}

/// Finds a QQ Music qrc path by track metadata, ignoring the random index.
pub fn find_qqmusic_qrc_path_by_metadata(
    track: &dyn ITrackMetadata,
) -> LyrixResult<Option<PathBuf>> {
    let dir = qqmusic_lyric_new_dir()?;
    find_qqmusic_qrc_path_by_metadata_in_dir(dir, track)
}

/// Finds a QQ Music qrc path by track metadata inside a given directory.
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
    let Some(artist) = track.artist().map(str::trim) else {
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

        if info.artist == artist && info.title == title && info.album == album {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

/// Reads a QQ Music qrc file by track metadata.
pub fn read_qqmusic_qrc_by_metadata(track: &dyn ITrackMetadata) -> LyrixResult<String> {
    let qrc_path =
        find_qqmusic_qrc_path_by_metadata(track)?.ok_or_else(|| GeneralError::MissingField {
            field: "QQMusic qrc file".to_string(),
        })?;

    read_qqmusic_qrc_file(qrc_path)
}

/// Parses a QQ Music qrc file name such as:
/// `artist - title - 303 - album_qm.qrc`
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
        artist: artist.to_string(),
        title: title.to_string(),
        index: index.to_string(),
        album: album.to_string(),
    })
}

/// Returns true when the file name follows the QQ Music qrc naming pattern.
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

fn collect_matching_qrc_files(dir: &Path, out: &mut Vec<PathBuf>) -> LyrixResult<()> {
    for entry in fs::read_dir(dir).map_err(GeneralError::Io)? {
        let entry = entry.map_err(GeneralError::Io)?;
        let path = entry.path();

        if is_qrc_file(&path) && is_qqmusic_qrc_path(&path) {
            out.push(path);
        }
    }

    Ok(())
}

fn is_qrc_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("qrc"))
        .unwrap_or(false)
}

fn is_qqmusic_qrc_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .and_then(parse_qqmusic_qrc_filename)
        .is_some()
}
