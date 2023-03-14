use chrono::Local;

use crate::logger::{LogData, LogLevel, Logger};
use crate::{LOG_FOLDER_NAME, MAX_FILE_SIZE};
use std::any::type_name;
use std::env;
use std::fs::{metadata, OpenOptions};
use std::io::Write;
use std::path::Path;

pub struct ThresholdLogger {
    print_threshold: LogLevel,
    store_threshold: LogLevel,
    file_index: u128,
}

impl ThresholdLogger {
    #[must_use]
    pub const fn new(print_threshold: LogLevel, store_threshold: LogLevel) -> Self {
        Self {
            print_threshold,
            store_threshold,
            file_index: 0,
        }
    }

    fn handle_log_print(&mut self, data: LogData) {
        if self.print_threshold == LogLevel::Ignore || data.severity_level < self.print_threshold {
            return;
        }

        println!("{data}");
    }

    fn handle_storing_of_log(&mut self, data: LogData) {
        if self.store_threshold == LogLevel::Ignore || data.severity_level < self.store_threshold {
            return;
        }

        let file_path: String = match self.create_file_path() {
            Ok(path) => path,
            Err(e) => {
                let error_string = format!("Failed to make get filepath because: {e}");
                let write_log =
                    LogData::new(LogLevel::Error, error_string.as_str(), type_name::<Self>());
                self.handle_log_print(write_log);
                return;
            }
        };

        match OpenOptions::new().append(true).create(true).open(file_path) {
            Ok(file) => self.write_to_file(file, data),
            Err(e) => {
                let error_string = format!("Failed to open file because: {e}");
                let write_log =
                    LogData::new(LogLevel::Error, error_string.as_str(), type_name::<Self>());
                self.handle_log_print(write_log);
            }
        }
    }

    fn write_to_file(&mut self, mut file: std::fs::File, data: LogData) {
        match writeln!(file, "{data}") {
            Ok(_) => (),
            Err(e) => {
                let error_string = format!("Failed to write {data} to file. Error: {e}");
                let write_log =
                    LogData::new(LogLevel::Error, error_string.as_str(), type_name::<Self>());
                self.handle_log_print(write_log);
            }
        }
    }

    fn create_file_path(&mut self) -> Result<String, String> {
        let mut file_name: String = self.create_file_name();
        let mut file_path: String = match Self::create_file_path_for_file_name(&file_name) {
            Ok(path) => path,
            Err(e) => return Err(format!("Failed to create file path because: {e}")),
        };

        while metadata(&file_path).map(|m| m.len()).unwrap_or(0) >= MAX_FILE_SIZE {
            self.file_index += 1;
            file_name = self.create_file_name();
            file_path = match Self::create_file_path_for_file_name(&file_name) {
                Ok(path) => path,
                Err(e) => return Err(format!("Failed to create file path because: {e}")),
            };
        }

        Ok(file_path)
    }

    fn create_file_name(&self) -> String {
        format!(
            "threshold_logger_{}_{}.txt",
            Local::now().format("%d-%m-%Y"),
            self.file_index
        )
    }

    fn create_file_path_for_file_name(file_name: &str) -> Result<String, String> {
        match env::current_exe() {
            Ok(path) => match path.parent() {
                Some(exe_folder) => {
                    let file_path =
                        Path::new(exe_folder).join(LOG_FOLDER_NAME.to_string() + "/" + file_name);
                    return Ok(file_path.to_string_lossy().to_string());
                }
                None => Err("Failed to get path of the folder the executable is in.".to_string()),
            },
            Err(_) => Err("Failed to get the path to the executable.".to_string()),
        }
    }
}

impl Logger for ThresholdLogger {
    fn log(&mut self, data: LogData) {
        self.handle_log_print(data);
        self.handle_storing_of_log(data);
    }
}
