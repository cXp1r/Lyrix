///实际没啥用,占位给后面歌词元信息解析用
#[derive(Debug, Clone, Default)]
pub enum LyricsTypes {
    ///路边
    LRC,
    ///感谢大哥的解密算法
    QRC,
    ///路边
    YRC,
    ///感谢大哥的解密算法
    KRC,
    ///路边
    #[default]
    Unknown,
}
