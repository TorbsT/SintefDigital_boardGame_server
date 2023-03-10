use std::fmt::Display;
use chrono::Local;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum LogLevel {
    Debug = 1,
    Info = 2,
    Warning = 3,
    Error = 4,
    Ignore = 5,
}

#[derive(Debug, Clone, Copy)]
pub struct LogData<'a> {
    pub severity_level: LogLevel,
    pub log_data: &'a str,
    pub caller_identifier: &'a str
}

impl LogData<'_> {
    pub fn new<'a>(severity: LogLevel, data: &'a str, type_name: &'a str) -> LogData<'a> {
        LogData { severity_level: severity, log_data: data, caller_identifier: type_name }
    }
}

impl Display for LogData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{:?}] Identifier {} | {}", Local::now().format("%d-%m-%Y %H:%M:%S").to_string(), self.severity_level, self.caller_identifier, self.log_data)
    }
}

pub trait Logger {
    // Template for LogData ```LogData::new(LogLevel::Debug, "", type_name::<Self>()```
    fn log(&mut self, data: LogData);
}