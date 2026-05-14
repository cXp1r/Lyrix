use super::{SyncTypes, LyricsTypes};

///歌词文件信息,占位给后面歌词元信息解析用
#[derive(Debug, Clone)]
pub struct LyricsFileInfo {
    pub lyrics_type: LyricsTypes,
    pub sync_type: SyncTypes,//重要 同步类型
}

impl Default for LyricsFileInfo {
    fn default() -> Self {
        Self {
            lyrics_type: LyricsTypes::LRC,
            sync_type: SyncTypes::Unsynced,
        }
    }
}
