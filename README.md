A simple Rust logger.

### Usage
To log messages, first set up the default path and log level like so:

```Rust
fn main() {
    sw_logger_rs::set_path("/path/to/logfile.log");
    sw_logger_rs::set_level(sw_logger_rs::LogLevel::Debug);
}
```

If logging to a file is not necessary, simply set the path to an empty string, like so:

```Rust
sw_logger_rs::set_path("");
```

Then to log messages, call the `log()` function:

```Rust
use sw_logger_rs::*;

fn main() {
    sw_logger_rs::set_path("/path/to/logfile.log");
    sw_logger_rs::set_level(LogLevel::Debug);

    log("This is a logged message!", LogType::Warning, None);
}
```

For clarity, opt to explicitly state the package name when calling `set_path()` and `set_level()`.

To write the message to a different path than the default, change the `None` parameter to `Some("/custom/path/here")`.
