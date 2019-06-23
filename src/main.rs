use std::env;
use std::io;
use std::path::PathBuf;

pub mod bots;
pub mod engine;
pub mod games;

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

    println!("Using {} game runner", config.run_mode);
    let runner = match config.run_mode.as_str() {
        // "GENETIC" => GeneticRunner::new(config),
        // "BATCH" => BatchRunner::new(config),
        "SINGLE" => single_runner,
        _ => {
            println!("Invalid game runner: {}", config.run_mode);
            std::process::exit(1);
        }
    };

    match runner(config) {
        Ok(_) => {}
        Err(x) => {
            println!("ERROR: {}", x);
        }
    }
}
