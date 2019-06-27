use log::*;
use std::env;
use std::io;
use std::path::PathBuf;
use std::time::Instant;

pub mod bots;
pub mod engine;
pub mod games;

use engine::runners::batchrunner::batch_runner;
use engine::runners::geneticrunner::genetic_runner;
use engine::runners::singlerunner::single_runner;

fn get_exe_dir() -> io::Result<PathBuf> {
    let mut dir = env::current_exe()?;
    dir.pop();
    Ok(dir)
}

fn main() {
    let path = get_exe_dir().expect("Unable to get current path");
    let mut config = engine::gameconfig::GameConfig::new(path);
    config.init();

    info!("Using {} game runner", config.run_mode);
    let runner = match config.run_mode.as_str() {
        "SINGLE" => single_runner,
        "BATCH" => batch_runner,
        "GENETIC" => genetic_runner,
        _ => {
            println!("Invalid game runner: {}", config.run_mode);
            std::process::exit(1);
        }
    };

    let now = Instant::now();

    match runner(config) {
        Ok(_) => {}
        Err(x) => {
            println!("ERROR: {}", x);
        }
    }

    info!(
        "Completed in {:.3} seconds.",
        now.elapsed().as_millis() as f32 / 1000.0
    )
}
