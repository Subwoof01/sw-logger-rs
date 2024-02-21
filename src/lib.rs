use chrono::Local;
use core::fmt;
use lazy_static::lazy_static;
use std::{fs::OpenOptions, io::Write, sync::Mutex};

lazy_static! {
    static ref LOG_LEVEL: Mutex<LogLevel> = Mutex::new(LogLevel::Default);
    static ref LOG_PATH: Mutex<String> = Mutex::new(String::new());
}

/// `Verbose`    -> Logs all messages, regardless of `LogType`.  
/// `Debug`      -> Logs messages marked as `LogType::Error`, `LogType::Warning` and
///                 `LogType::Debug`.  
/// `Default`    -> Logs messages marked as `LogType::Error` and `LogType::Warning`.  
/// `ErrorsOnly` -> Only logs messages marked as `LogType::Error`.
#[derive(Debug, PartialEq, Clone)]
pub enum LogLevel {
    Verbose,
    Debug,
    Default,
    ErrorsOnly,
}

#[derive(Debug, PartialEq)]
pub enum LogType {
    Info,
    Debug,
    Warning,
    Error,
}

impl fmt::Display for LogType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Debug => write!(f, "DEBUG"),
            Self::Warning => write!(f, "WARNING"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// Sets the level of logging.
/// See `LogLevel` for a description of what each level means.
pub fn set_level(level: LogLevel) {
    *LOG_LEVEL.lock().unwrap() = level;
}

/// Sets the default path the logger uses to write to.
/// If left as an empty `String` (or set as one), the logger won't write to a file; only to the `stdout` and the `stderr`.
pub fn set_path(path: String) {
    *LOG_PATH.lock().unwrap() = path;
}

/// Log a message to stdout and, optionally, a file.
/// This function will format the message with a type-signifier and the timestamp of the moment of
/// logging.
///
/// Example:  
/// ```
/// [WARNING] 2024-02-21 12:05:51 -> This is a logged message!
/// ```
///
/// `message` -> the message to print.
/// `t`       -> the `LogType` to use. Defines both the type-signifier in the stdout/file and
/// whether this message should be logged at all, depending on the global `LogLevel` set.  
/// p         -> A custom path for the logger to write this message to. When `None` the logger will
/// write to the default path set with `set_path`. A custom path can be specified like so:
/// `Some("/the/path/here")`.
pub fn log(message: &str, t: LogType, p: Option<&str>) -> Result<String, std::io::Error> {
    let default_path = LOG_PATH.lock().unwrap().clone();
    let path = p.unwrap_or(&default_path);

    let timestamp = Local::now();
    let formatted_timestamp = timestamp.format("%Y-%m-%d %H:%M:%S");

    let formatted_message = format!(
        "[{log_type}] {time} -> {message}",
        log_type = t,
        time = formatted_timestamp
    );

    let level = LOG_LEVEL.lock().unwrap().clone();

    match level {
        LogLevel::ErrorsOnly => {
            if t != LogType::Error {
                return Ok("".to_string());
            }
        }
        LogLevel::Default => {
            if t == LogType::Debug || t == LogType::Info {
                return Ok("".to_string());
            }
        }
        LogLevel::Debug => {
            if t == LogType::Info {
                return Ok("".to_string());
            }
        }
        _ => {}
    }

    match t {
        LogType::Error => eprintln!("{}", formatted_message),
        _ => println!("{}", formatted_message),
    }

    if path != String::new() {
        let mut file = OpenOptions::new().append(true).create(true).open(&path)?;
        writeln!(file, "{}", &formatted_message)?;
    }

    Ok(formatted_message)
}

#[cfg(test)]
mod tests {
    use all_asserts::assert_false;

    use super::*;
    use std::fs;

    fn check_string_in_file(path: &str, string_to_find: &str) -> bool {
        let lines = fs::read_to_string(path).unwrap();

        for line in lines.lines() {
            println!("Line read: {}", line);
            if line == string_to_find {
                return true;
            }
        }
        false
    }

    #[test]
    fn calling_set_level_sets_level() {
        set_level(LogLevel::Debug);
        assert_eq!(*LOG_LEVEL.lock().unwrap(), LogLevel::Debug);
    }

    #[test]
    fn calling_set_path_sets_path() {
        set_path(String::from(
            "/home/jordan/projects/rust/logger/target/debug/test/test.log",
        ));
        assert_eq!(
            *LOG_PATH.lock().unwrap(),
            "/home/jordan/projects/rust/logger/target/debug/test/test.log"
        );
    }

    #[test]
    fn log_writes_to_default_file() {
        set_path(String::from(
            "/home/jordan/projects/rust/logger/target/debug/test/test.log",
        ));

        let path = LOG_PATH.lock().unwrap().clone();

        let logged_message = log("This is a test", LogType::Error, None).unwrap();
        assert!(
            check_string_in_file(&path, &logged_message),
            "Did not find test string in log file."
        );
    }

    #[test]
    fn log_writes_to_custom_file() {
        let path = "/home/jordan/projects/rust/logger/target/debug/test/custom.log";

        let logged_message = log("This is a test", LogType::Error, Some(path)).unwrap();
        assert!(
            check_string_in_file(&path, &logged_message),
            "Did not find test string in log file."
        );
    }

    #[test]
    fn log_level_debug_does_not_log_info() {
        set_path(String::from(
            "/home/jordan/projects/rust/logger/target/debug/test/test.log",
        ));
        let path = LOG_PATH.lock().unwrap().clone();
        assert_eq!(
            &path,
            "/home/jordan/projects/rust/logger/target/debug/test/test.log"
        );

        set_level(LogLevel::Debug);
        let log_level = LOG_LEVEL.lock().unwrap().clone();
        assert_eq!(log_level, LogLevel::Debug);

        let logged_message = log("This shouldn't get logged", LogType::Info, None).unwrap();
        assert_false!(
            check_string_in_file(&path, &logged_message),
            "Found test string in log file."
        );
    }

    #[test]
    fn log_level_error_only_prints_errors() {
        set_path(String::from(
            "/home/jordan/projects/rust/logger/target/debug/test/test.log",
        ));
        let path = LOG_PATH.lock().unwrap().clone();
        assert_eq!(
            &path,
            "/home/jordan/projects/rust/logger/target/debug/test/test.log"
        );

        set_level(LogLevel::ErrorsOnly);
        let log_level = LOG_LEVEL.lock().unwrap().clone();
        assert_eq!(log_level, LogLevel::ErrorsOnly);

        let path = LOG_PATH.lock().unwrap().clone();

        let mut logged_message = log("This shouldn't get logged (errors_only).", LogType::Info, None).unwrap();
        assert_false!(
            check_string_in_file(&path, &logged_message),
            "Found INFO test string in log file."
        );

        logged_message = log("This shouldn't get logged (errors_only).", LogType::Debug, None).unwrap();
        assert_false!(
            check_string_in_file(&path, &logged_message),
            "Found DEBUG test string in log file."
        );

        logged_message = log("This shouldn't get logged (errors_only).", LogType::Warning, None).unwrap();
        assert_false!(
            check_string_in_file(&path, &logged_message),
            "Found WARNING test string in log file."
        );

        logged_message = log("This should get logged (errors_only).", LogType::Error, None).unwrap();
        assert!(
            check_string_in_file(&path, &logged_message),
            "Did not find ERROR test string in log file."
        );
    }

    #[test]
    fn log_level_verbose_prints_everything() {
        set_path(String::from(
            "/home/jordan/projects/rust/logger/target/debug/test/test.log",
        ));
        let path = LOG_PATH.lock().unwrap().clone();
        assert_eq!(
            &path,
            "/home/jordan/projects/rust/logger/target/debug/test/test.log"
        );

        set_level(LogLevel::Verbose);
        let log_level = LOG_LEVEL.lock().unwrap().clone();
        assert_eq!(log_level, LogLevel::Verbose);

        let path = LOG_PATH.lock().unwrap().clone();

        let mut logged_message = log("This should get logged (verbose).", LogType::Info, None).unwrap();
        assert!(
            check_string_in_file(&path, &logged_message),
            "Did not find INFO test string in log file."
        );

        logged_message = log("This should get logged (verbose).", LogType::Debug, None).unwrap();
        assert!(
            check_string_in_file(&path, &logged_message),
            "Did not find DEBUG test string in log file."
        );

        logged_message = log("This should get logged (verbose).", LogType::Warning, None).unwrap();
        assert!(
            check_string_in_file(&path, &logged_message),
            "Did not find WARNING test string in log file."
        );

        logged_message = log("This should get logged (verbose).", LogType::Error, None).unwrap();
        assert!(
            check_string_in_file(&path, &logged_message),
            "Did not find ERROR test string in log file."
        );
    }

    #[test]
    fn log_level_default_does_not_log_info_debug() {
        set_path(String::from(
            "/home/jordan/projects/rust/logger/target/debug/test/test.log",
        ));
        let path = LOG_PATH.lock().unwrap().clone();
        assert_eq!(
            &path,
            "/home/jordan/projects/rust/logger/target/debug/test/test.log"
        );

        set_level(LogLevel::Default);
        let log_level = LOG_LEVEL.lock().unwrap().clone();
        assert_eq!(log_level, LogLevel::Default);

        let path = LOG_PATH.lock().unwrap().clone();

        let mut logged_message = log("This shouldn't get logged (default).", LogType::Info, None).unwrap();
        assert_false!(
            check_string_in_file(&path, &logged_message),
            "Found INFO test string in log file."
        );

        logged_message = log("This shouldn't get logged (default).", LogType::Debug, None).unwrap();
        println!("Logged message: \n{}", logged_message);
        assert_false!(
            check_string_in_file(&path, &logged_message),
            "Found DEBUG test string in log file."
        );

        logged_message = log("This should get logged (default).", LogType::Warning, None).unwrap();
        assert!(
            check_string_in_file(&path, &logged_message),
            "Did not find WARNING test string in log file."
        );

        logged_message = log("This should get logged (default).", LogType::Error, None).unwrap();
        assert!(
            check_string_in_file(&path, &logged_message),
            "Did not find ERROR test string in log file."
        );
    }
}
