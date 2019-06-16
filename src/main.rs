use std::env;
use std::io;
use std::path::PathBuf;

pub mod engine;
pub mod games;

fn get_exe_dir() -> io::Result<PathBuf> {
    let mut dir = env::current_exe()?;
    dir.pop();
    Ok(dir)
}

fn main() {
    let path = get_exe_dir().expect("Unable to get current path");
    let mut config = engine::gameconfig::GameConfig::new(path);
    config.init();

    println!("Path is {}", config.path.display());
}
