use simplelog::*;
use std::fs::File;
use std::path::Path;

pub fn setup_logger(log_filename: &Path) {
    let config = Config::default();
    let filename_str = log_filename.to_str().expect("No log filename!");
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Warn, config).expect("Unable to set up console logging"),
        WriteLogger::new(
            LevelFilter::Debug,
            config,
            File::create(filename_str).expect("Unable to set up file logging"),
        ),
    ])
    .expect("Unable to set up logging");
}
