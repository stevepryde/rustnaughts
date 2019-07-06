use argparse::{ArgumentParser, Store, StoreTrue};
use serde_json;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::exit;

use crate::engine::botdb::BotDB;
use crate::engine::log;

/// Exit with the specified error message.
fn exit_with_error(err: &str) {
    println!("{}", err);
    exit(1);
}


/// The config required to construct bots.
#[derive(Default, Clone)]
pub struct BotConfig {
    pub bot_names: [String; 2],
    pub game: String,
}

/// The config required to process one batch.
#[derive(Clone)]
pub struct BatchConfig {
    pub batch_size: u32,
    pub game: String,
    pub magic: bool,
    pub bot_config: BotConfig,
}

impl Default for BatchConfig {
    fn default() -> Self {
        BatchConfig {
            batch_size: 1,
            game: String::new(),
            magic: false,
            bot_config: BotConfig::default(),
        }
    }
}

/// The config required for genetic runner.
pub struct GeneticConfig {
    pub game: String,
    pub num_generations: u32,
    pub num_samples: u32,
    pub keep_samples: u32,
    pub wild_samples: u32,
    pub batch_config: BatchConfig,
}

impl Default for GeneticConfig {
    fn default() -> Self {
        GeneticConfig {
            game: String::new(),
            num_generations: 0,
            num_samples: 0,
            keep_samples: 0,
            wild_samples: 0,
            batch_config: BatchConfig::default(),
        }
    }
}

pub struct GameConfig {
    pub path: PathBuf,
    pub game: String,
    pub silent: bool,
    pub console_logging: bool,
    pub batch_mode: bool,
    pub genetic_mode: bool,
    pub no_batch_summary: bool,
    pub run_mode: String,
    pub botdb: bool,
    pub botrecipe: serde_json::Value,
    log_base_dir: PathBuf,
    data_base_dir: PathBuf,
    bot_names: [String; 2],
    batch_size: u32,
    magic: bool,
    num_generations: u32,
    num_samples: u32,
    keep_samples: u32,
    wild_samples: u32,
}

impl GameConfig {
    pub fn new(path: PathBuf) -> Self {
        let mut log_base_dir = path.clone();
        log_base_dir.push("log");

        let mut data_base_dir = path.clone();
        data_base_dir.push("data");

        GameConfig {
            path,
            game: String::new(),
            silent: false,
            console_logging: false,
            batch_mode: false,
            genetic_mode: false,
            no_batch_summary: false,
            run_mode: String::new(),
            log_base_dir,
            data_base_dir,
            bot_names: [String::new(), String::new()],
            batch_size: 1,
            magic: false,
            num_generations: 0,
            num_samples: 0,
            keep_samples: 0,
            wild_samples: 0,
            botrecipe: serde_json::Value::Null,
            botdb: false,
        }
    }

    /// Initialise config, parse CLI args, and set up logging.
    pub fn init(&mut self) {
        self.parse_args();
        self.sanitise_args();
        self.init_logging().expect("Error setting up logging");

        if self.genetic_mode {
            self.run_mode = String::from("GENETIC");
        } else if self.batch_mode {
            self.run_mode = String::from("BATCH");
        } else {
            self.run_mode = String::from("SINGLE");
        }
    }

    /// Parse CLI args.
    fn parse_args(&mut self) {
        let mut game = String::new();
        let mut bot1 = String::new();
        let mut bot2 = String::new();
        let mut botid = String::new();

        {
            let mut ap = ArgumentParser::new();
            ap.set_description("Naughts");
            ap.refer(&mut bot1)
                .add_argument("bot1", Store, "First bot, e.g. 'human'")
                .required();

            ap.refer(&mut bot2)
                .add_argument("bot2", Store, "Second bot")
                .required();

            ap.refer(&mut game)
                .add_option(&["--game"], Store, "The game to run")
                .required();
            ap.refer(&mut self.batch_size).add_option(
                &["--batch"],
                Store,
                "Batch mode. Specify the number of games to run per batch",
            );
            ap.refer(&mut self.magic).add_option(
                &["--magic"],
                StoreTrue,
                "Magic batch mode. Run all possible games against this bot.",
            );
            ap.refer(&mut self.num_generations).add_option(
                &["--genetic"],
                Store,
                "Genetic mode. Specify number of generations to run (Requires --batch or --magic)",
            );
            ap.refer(&mut self.num_samples).add_option(
                &["--samples"],
                Store,
                "Number of samples per generation. (Requires --genetic)",
            );
            ap.refer(&mut self.keep_samples).add_option(
                &["--keep"],
                Store,
                "Number of winning samples to 'keep' (Requires --genetic)",
            );
            ap.refer(&mut self.wild_samples).add_option(
                &["--wild"],
                Store,
                "Number of 'wild' (fresh, randomly generated) samples to include \
                 in each generation",
            );
            ap.refer(&mut botid)
                .add_option(&["--botid"], Store, "The botID to load from botdb");
            ap.refer(&mut self.botdb).add_option(
                &["--botdb"],
                StoreTrue,
                "Use BotDB to store recipes",
            );
            ap.parse_args_or_exit();
        }

        self.bot_names[0] = bot1;
        self.bot_names[1] = bot2;
        self.game = game;

        if !botid.is_empty() {
            match BotDB::new().load_bot(botid.as_str()) {
                Ok(x) => self.botrecipe = x,
                Err(x) => exit_with_error(format!("Failed to load bot: {}", x).as_str()),
            };
        }
    }

    /// Sanitise CLI args into sane defaults and catch errors.
    fn sanitise_args(&mut self) {
        // Tidy up default args.
        if self.batch_size > 1 {
            self.batch_mode = true;
        }

        if self.num_generations > 0 {
            self.genetic_mode = true;
        }

        if !self.batch_mode && !self.magic {
            if self.genetic_mode {
                exit_with_error("Option --genetic requires --batch");
            }
            if self.num_samples > 0 {
                exit_with_error("Option --samples requires --batch");
            }
            if self.keep_samples > 0 {
                exit_with_error("Option --keep requires --batch");
            }
            if self.wild_samples > 0 {
                exit_with_error("Option --wild requires --batch");
            }
        }

        if !self.genetic_mode {
            if self.num_samples > 0 {
                exit_with_error("Option --samples requires --genetic");
            }
            if self.keep_samples > 0 {
                exit_with_error("Option --keep requires --genetic");
            }
            if self.wild_samples > 0 {
                exit_with_error("Option --wild requires --genetic");
            }
        }

        if self.magic {
            if self.batch_mode {
                exit_with_error("Cannot specify --batch with --magic");
            }

            // Magic flag implicitly enables batch mode.
            self.batch_size = 0;
            self.batch_mode = true;
        }

        if self.magic || self.batch_mode {
            self.silent = true;

            if self.genetic_mode {
                self.no_batch_summary = true;
            }
        }
    }

    /// Initialise logging.
    pub fn init_logging(&self) -> io::Result<()> {
        if !self.log_base_dir.exists() {
            fs::create_dir(&self.log_base_dir)?;
        }

        if !self.data_base_dir.exists() {
            fs::create_dir(&self.data_base_dir)?;
        }

        let mut log_filename = self.log_base_dir.clone();
        log_filename.push("naughts.log");
        log::setup_logger(log_filename.as_path());

        Ok(())
    }

    /// Get the base data path. Bots should clone this and add the appropriate subdir.
    pub fn get_data_path(&self) -> &Path {
        &self.data_base_dir.as_path()
    }

    /// Get the bot config.
    pub fn get_bot_config(&self) -> BotConfig {
        BotConfig {
            bot_names: self.bot_names.clone(),
            game: self.game.clone(),
        }
    }

    /// Get the batch config.
    pub fn get_batch_config(&self) -> BatchConfig {
        BatchConfig {
            batch_size: self.batch_size,
            game: self.game.clone(),
            magic: self.magic,
            bot_config: self.get_bot_config(),
        }
    }

    /// Get the genetic config.
    pub fn get_genetic_config(&self) -> GeneticConfig {
        GeneticConfig {
            game: self.game.clone(),
            num_generations: self.num_generations,
            num_samples: self.num_samples,
            keep_samples: self.keep_samples,
            wild_samples: self.wild_samples,
            batch_config: self.get_batch_config(),
        }
    }
}
