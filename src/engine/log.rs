use fern;
use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;
use std::io;
use std::path::Path;

pub fn setup_logger(log_filename: &Path) {
    let colors = ColoredLevelConfig::new()
        .trace(Color::Blue)
        .debug(Color::Yellow)
        .info(Color::BrightGreen)
        .warn(Color::Magenta)
        .error(Color::BrightMagenta);

    fern::Dispatch::new()
        .chain(
            fern::Dispatch::new()
                .level(LevelFilter::Debug)
                .format(move |out, message, record| {
                    out.finish(format_args!(
                        "{color_line}{date} {level:7} {message}\x1B[0m",
                        color_line =
                            format_args!("\x1B[{}m", colors.get_color(&record.level()).to_fg_str()),
                        date = chrono::Local::now().format("%H:%M:%S"),
                        level = &record.level(),
                        message = message
                    ))
                })
                .chain(io::stdout()),
        )
        .chain(
            fern::Dispatch::new()
                .level(LevelFilter::Debug)
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "{} {:7} {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                        record.level(),
                        message
                    ))
                })
                .chain(fern::log_file(log_filename).expect("Error setting up log file")),
        )
        .apply()
        .expect("Error setting up logging");
}
