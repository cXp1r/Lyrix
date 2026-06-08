use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{OnceLock, RwLock};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

static LOG_FILE_PATH: OnceLock<PathBuf> = OnceLock::new();

// 0=DEBUG, 1=INFO, 2=WARN, 3=ERROR
static LOG_LEVEL: AtomicU8 = AtomicU8::new(1);
static LOG_FILTER_TAGS: OnceLock<RwLock<Vec<String>>> = OnceLock::new();
static LOG_FILTER_INVERT: AtomicU8 = AtomicU8::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

impl LogLevel {
    pub fn as_str_lower(&self) -> &'static str {
        match self {
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }

    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => LogLevel::Debug,
            1 => LogLevel::Info,
            2 => LogLevel::Warn,
            3 => LogLevel::Error,
            _ => LogLevel::Info,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_uppercase().as_str() {
            "DEBUG" => LogLevel::Debug,
            "INFO" => LogLevel::Info,
            "WARN" => LogLevel::Warn,
            "ERROR" => LogLevel::Error,
            _ => LogLevel::Info,
        }
    }
}

fn level_to_u8(level: &str) -> u8 {
    LogLevel::from_str(level) as u8
}

pub fn set_level(level: &str) {
    LOG_LEVEL.store(level_to_u8(level), Ordering::Relaxed);
}

pub fn get_level() -> String {
    LogLevel::from_u8(LOG_LEVEL.load(Ordering::Relaxed))
        .as_str_lower()
        .to_string()
}

pub fn set_filter(tags: Vec<String>, invert: bool) {
    let normalized: Vec<String> = tags
        .into_iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect();
    *LOG_FILTER_TAGS.get_or_init(|| RwLock::new(Vec::new())).write().unwrap() = normalized;
    LOG_FILTER_INVERT.store(if invert { 1 } else { 0 }, Ordering::Relaxed);
}

pub fn get_filter_tags() -> Vec<String> {
    LOG_FILTER_TAGS
        .get_or_init(|| RwLock::new(Vec::new()))
        .read()
        .unwrap()
        .clone()
}

pub fn get_filter_invert() -> bool {
    LOG_FILTER_INVERT.load(Ordering::Relaxed) != 0
}

pub fn log_file_path() -> Option<&'static PathBuf> {
    LOG_FILE_PATH.get()
}

enum LogMsg {
    Line(String),
}

fn get_sender() -> &'static Sender<LogMsg> {
    static SENDER: OnceLock<Sender<LogMsg>> = OnceLock::new();
    SENDER.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<LogMsg>();

        let log_dir: PathBuf = dirs::config_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
            .join("lyrix");
        let _ = std::fs::create_dir_all(&log_dir);

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let file_path = log_dir.join(format!("island_{}.log", ts));
        let _ = LOG_FILE_PATH.set(file_path.clone());

        thread::Builder::new()
            .name("logger".into())
            .spawn(move || {
                let mut file: Option<File> = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&file_path)
                    .ok();

                for msg in rx {
                    let LogMsg::Line(line) = msg;
                    println!("{}", line);
                    if let Some(ref mut f) = file {
                        let _ = writeln!(f, "{}", line);
                        let _ = f.flush();
                    }
                }
            })
            .ok();

        tx
    })
}

fn write_log<M>(tag: &str, level: LogLevel, message: M)
where
    M: fmt::Display,
{
    if (level as u8) < LOG_LEVEL.load(Ordering::Relaxed) {
        return;
    }
    let filter_tags = LOG_FILTER_TAGS.get_or_init(|| RwLock::new(Vec::new())).read().unwrap();
    if !filter_tags.is_empty() {
        let matched = filter_tags.iter().any(|filter_tag| filter_tag == tag);
        let invert = LOG_FILTER_INVERT.load(Ordering::Relaxed) != 0;
        if (!invert && matched) || (invert && !matched) {
            return;
        }
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let line = format!("[{}][{}][{}] {}", tag, level, now, message);
    let _ = get_sender().send(LogMsg::Line(line));
}

pub fn debug<M>(tag: &str, message: M)
where
    M: fmt::Display,
{
    write_log(tag, LogLevel::Debug, message);
}

pub fn info<M>(tag: &str, message: M)
where
    M: fmt::Display,
{
    write_log(tag, LogLevel::Info, message);
}

pub fn warn<M>(tag: &str, message: M)
where
    M: fmt::Display,
{
    write_log(tag, LogLevel::Warn, message);
}

pub fn error<M>(tag: &str, message: M)
where
    M: fmt::Display,
{
    write_log(tag, LogLevel::Error, message);
}
