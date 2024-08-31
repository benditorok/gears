pub enum LogLevel {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

pub struct LogConfig {
    pub level: LogLevel,
}

pub struct Config {
    pub log: LogConfig,
    pub threadpool_size: usize,
}