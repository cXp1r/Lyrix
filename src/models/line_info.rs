///逐字歌词信息
#[derive(Debug, Clone, Default)]
pub struct TextInfo {
    ///offset当前行st
    pub start_time: u16,
    pub duration: u16,
    pub text: String,
}
///歌词行信息,text和syllables二选一
#[derive(Debug, Clone, Default)]
pub struct LineInfo {
    ///溢出了来这里改够你吃一壶了
    pub start_time: u32,
    pub duration: u16,
    pub text: String,
    pub syllables: Vec<TextInfo>,
}

impl LineInfo {
    pub fn is_empty(&self) -> bool {
        self.start_time == 0 || self.syllables.is_empty()
    }
}