use std::str::FromStr;

use chrono::Local;
use colored::*;
use log::{Level, LevelFilter};

/// Initializes the env_logger with a custom format
/// that also logs the thread names
pub fn init_logger() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            let color = get_level_style(record.level());
            let mut target = record.target().to_string();
            target.truncate(39);

            out.finish(format_args!(
                "{:<40}| {} {}: {}",
                target.dimmed().italic(),
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record
                    .level()
                    .to_string()
                    .to_lowercase()
                    .as_str()
                    .color(color),
                message
            ))
        })
        .level(
            std::env::var("RUST_LOG")
                .ok()
                .and_then(|level| log::LevelFilter::from_str(&level).ok())
                .unwrap_or(LevelFilter::Info),
        )
        .level_for("tokio", log::LevelFilter::Info)
        .level_for("tracing", log::LevelFilter::Warn)
        .level_for("rustls", log::LevelFilter::Warn)
        .level_for("h2", log::LevelFilter::Warn)
        .level_for("hyper", log::LevelFilter::Warn)
        .level_for("tokio_util", log::LevelFilter::Warn)
        .level_for("want", log::LevelFilter::Warn)
        .level_for("mio", log::LevelFilter::Warn)
        .chain(std::io::stdout())
        .apply()
        .expect("failed to init logger");
}

fn get_level_style(level: Level) -> colored::Color {
    match level {
        Level::Trace => colored::Color::Magenta,
        Level::Debug => colored::Color::Blue,
        Level::Info => colored::Color::Green,
        Level::Warn => colored::Color::Yellow,
        Level::Error => colored::Color::Red,
    }
}
