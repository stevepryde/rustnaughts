use log::*;
use std::env;
use std::io;
use std::path::PathBuf;
use std::time::Instant;

pub mod bots {
    pub mod randombot {
        pub mod rbot;
    }
    pub mod genbot3 {
        pub mod gbot;
        pub mod nodes;
    }
    pub mod omnibot {
        pub mod obot;
    }
    pub mod nbot1 {
        pub mod nbot;
        pub mod neurons;
    }
}
pub mod engine {
    pub mod botdb;
    pub mod botfactory;
    pub mod errors;
    pub mod gamebase;
    pub mod gameconfig;
    pub mod gamefactory;
    pub mod gameobject;
    pub mod gameplayer;
    pub mod gameresult;
    pub mod log;
    pub mod runners {
        pub mod batchrunner;
        pub mod gen2runner;
        pub mod geneticrunner;
        pub mod singlerunner;
        pub mod genetic {
            pub mod processor;
        }
    }
}
pub mod games {
    pub mod naughts {
        pub mod board;
        pub mod singlegame;
        pub mod bots {
            pub mod hbot;
        }
    }
    pub mod connect4 {
        pub mod singlegame;
        pub mod world;
        pub mod bots {
            pub mod hbot;
        }
    }
}

use crate::engine::gameconfig::RunMode;
use engine::runners::batchrunner::batch_runner;
use engine::runners::gen2runner::gen2_runner;
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

    info!("Using {:?} game runner", config.run_mode);
    let runner = match config.run_mode {
        RunMode::Single => single_runner,
        RunMode::Batch => batch_runner,
        RunMode::Genetic => genetic_runner,
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
