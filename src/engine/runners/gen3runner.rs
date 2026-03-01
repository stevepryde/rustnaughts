use crate::engine::botfactory::BotFactory;
use crate::engine::gamebase::run_batch;
use crate::engine::gameconfig::GameConfig;
use crate::engine::gamefactory::create_game_factory;
use crate::engine::gameplayer::MutationIntensity;
use crate::engine::gameresult::GameScore;

use log::*;
use rand::Rng;

use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::{env, thread};

#[derive(Debug, Clone)]
struct Individual {
    recipe: serde_json::Value,
    score: GameScore,
    generation: u32,
    #[allow(dead_code)]
    id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MutationStrategy {
    SingleMutate,
    MultiMutate,
    Crossover,
    AdaptiveStep,
    FreshRandom,
    SimulatedAnnealing,
}

const ALL_STRATEGIES: [MutationStrategy; 6] = [
    MutationStrategy::SingleMutate,
    MutationStrategy::MultiMutate,
    MutationStrategy::Crossover,
    MutationStrategy::AdaptiveStep,
    MutationStrategy::FreshRandom,
    MutationStrategy::SimulatedAnnealing,
];

#[derive(Debug, Clone, Copy, Default)]
struct StrategyStats {
    successes: u32,
    attempts: u32,
}

struct PopulationPool {
    individuals: Vec<Individual>,
    capacity: usize,
    best_score: Option<GameScore>,
    best_recipe: Option<serde_json::Value>,
    temperature: f32,
    strategy_stats: [StrategyStats; 6],
    global_stagnation: u32,
    total_evaluated: u64,
    max_evaluations: u64,
    next_id: u64,
    scores_csv: Option<File>,
}

impl PopulationPool {
    fn new(capacity: usize, max_evaluations: u64, scores_csv: Option<File>) -> Self {
        let capacity = std::cmp::max(capacity, 5);
        Self {
            individuals: Vec::with_capacity(capacity),
            capacity,
            best_score: None,
            best_recipe: None,
            temperature: 1.0,
            strategy_stats: [StrategyStats::default(); 6],
            global_stagnation: 0,
            total_evaluated: 0,
            max_evaluations,
            next_id: 0,
            scores_csv,
        }
    }

    fn strategy_index(strategy: MutationStrategy) -> usize {
        match strategy {
            MutationStrategy::SingleMutate => 0,
            MutationStrategy::MultiMutate => 1,
            MutationStrategy::Crossover => 2,
            MutationStrategy::AdaptiveStep => 3,
            MutationStrategy::FreshRandom => 4,
            MutationStrategy::SimulatedAnnealing => 5,
        }
    }

    fn choose_strategy(&self) -> MutationStrategy {
        let mut rng = rand::thread_rng();
        let pop_size = self.individuals.len();

        let weights: Vec<f32> = ALL_STRATEGIES
            .iter()
            .map(|&s| {
                let idx = Self::strategy_index(s);
                let stats = &self.strategy_stats[idx];

                // Base weight from historical success rate.
                let base = if stats.attempts > 0 {
                    1.0 + (stats.successes as f32 / stats.attempts as f32) * 10.0
                } else {
                    3.0 // Untried strategies get a bonus to encourage exploration.
                };

                match s {
                    MutationStrategy::Crossover => {
                        if pop_size < 2 {
                            0.0
                        } else {
                            base
                        }
                    }
                    MutationStrategy::AdaptiveStep => base * (1.0 + self.temperature),
                    MutationStrategy::SimulatedAnnealing => base * (1.0 + self.temperature * 0.5),
                    MutationStrategy::FreshRandom => {
                        base * (1.0 + self.global_stagnation as f32 * 0.1)
                    }
                    _ => base,
                }
            })
            .collect();

        let total: f32 = weights.iter().sum();
        if total <= 0.0 {
            return MutationStrategy::SingleMutate;
        }

        let mut roll = rng.gen_range(0.0, total);
        for (i, &w) in weights.iter().enumerate() {
            roll -= w;
            if roll <= 0.0 {
                return ALL_STRATEGIES[i];
            }
        }

        MutationStrategy::SingleMutate
    }

    fn tournament_select(&self) -> &Individual {
        let mut rng = rand::thread_rng();
        let k = std::cmp::min(3, self.individuals.len());
        let mut best: Option<&Individual> = None;
        for _ in 0..k {
            let idx = rng.gen_range(0, self.individuals.len());
            let candidate = &self.individuals[idx];
            if best.is_none() || candidate.score > best.unwrap().score {
                best = Some(candidate);
            }
        }
        best.unwrap()
    }

    fn get_parent_recipe(&self) -> serde_json::Value {
        if self.individuals.is_empty() {
            return serde_json::Value::Null;
        }
        self.tournament_select().recipe.clone()
    }

    fn get_second_parent_recipe(&self) -> serde_json::Value {
        if self.individuals.len() < 2 {
            return serde_json::Value::Null;
        }
        self.tournament_select().recipe.clone()
    }

    fn propose_child(
        &mut self,
    ) -> Option<(serde_json::Value, MutationStrategy, Option<serde_json::Value>)> {
        if self.total_evaluated >= self.max_evaluations {
            return None;
        }

        let strategy = self.choose_strategy();
        let parent_recipe = self.get_parent_recipe();
        let crossover_recipe = if strategy == MutationStrategy::Crossover {
            Some(self.get_second_parent_recipe())
        } else {
            None
        };

        Some((parent_recipe, strategy, crossover_recipe))
    }

    fn submit(
        &mut self,
        recipe: serde_json::Value,
        raw_score: GameScore,
        strategy: MutationStrategy,
        parent_score: Option<GameScore>,
    ) {
        self.total_evaluated += 1;
        self.global_stagnation += 1;
        let strategy_idx = Self::strategy_index(strategy);
        self.strategy_stats[strategy_idx].attempts += 1;

        // EMA smoothing with alpha=0.3 for returning individuals.
        let score = raw_score;

        let accepted = if self.individuals.len() < self.capacity {
            // Population not full — always accept.
            true
        } else {
            let worst_score = self
                .individuals
                .iter()
                .map(|i| i.score)
                .fold(f32::INFINITY, f32::min);

            if score > worst_score {
                true
            } else if strategy == MutationStrategy::SimulatedAnnealing && self.temperature > 0.01 {
                // SA acceptance: accept worse solutions with some probability.
                let delta = score - worst_score;
                let prob = (delta / self.temperature).exp();
                rand::thread_rng().gen::<f32>() < prob
            } else {
                false
            }
        };

        if !accepted {
            return;
        }

        self.strategy_stats[strategy_idx].successes += 1;

        let id = self.next_id;
        self.next_id += 1;

        // Apply EMA if this recipe has a parent score we're improving on.
        let smoothed_score = match parent_score {
            Some(ps) if score > ps => score * 0.3 + ps * 0.7,
            _ => score,
        };

        let generation = if let Some(parent) = self.individuals.iter().max_by(|a, b| {
            a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal)
        }) {
            parent.generation + 1
        } else {
            0
        };

        let individual = Individual {
            recipe: recipe.clone(),
            score: smoothed_score,
            generation,
            id,
        };

        if self.individuals.len() < self.capacity {
            self.individuals.push(individual);
        } else {
            // Replace the worst individual.
            let worst_idx = self
                .individuals
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(i, _)| i)
                .unwrap();
            self.individuals[worst_idx] = individual;
        }

        // Check for new global best.
        let is_new_best = match self.best_score {
            Some(bs) => score > bs,
            None => true,
        };

        if is_new_best {
            info!(
                "GEN3: New best score: {:.3} (id: {}, gen: {}, strategy: {:?}, eval: {}/{})",
                score, id, generation, strategy, self.total_evaluated, self.max_evaluations
            );
            self.best_score = Some(score);
            self.best_recipe = Some(recipe);
            self.global_stagnation = 0;
            self.temperature *= 0.95; // Geometric decay on improvement.

            if let Some(ref mut csv) = self.scores_csv {
                if let Some(ref best) = self.best_recipe {
                    let _ = writeln!(csv, "{}", best);
                }
            }
        }

        // Log strategy stats periodically.
        if self.total_evaluated.is_multiple_of(100) {
            self.log_stats();
        }
    }

    fn log_stats(&self) {
        let mut parts = Vec::new();
        for (i, s) in ALL_STRATEGIES.iter().enumerate() {
            let stats = &self.strategy_stats[i];
            if stats.attempts > 0 {
                let rate = stats.successes as f32 / stats.attempts as f32 * 100.0;
                parts.push(format!("{:?}:{:.0}%({}/{})", s, rate, stats.successes, stats.attempts));
            }
        }
        info!(
            "GEN3 [{}/{}] temp={:.3} stag={} pop={} best={:.3} | {}",
            self.total_evaluated,
            self.max_evaluations,
            self.temperature,
            self.global_stagnation,
            self.individuals.len(),
            self.best_score.unwrap_or(0.0),
            parts.join(" ")
        );
    }

    fn is_done(&self) -> bool {
        self.total_evaluated >= self.max_evaluations
    }
}

pub fn gen3_runner(config: GameConfig) -> Result<(), Box<dyn Error>> {
    let botrecipe = &config.botrecipe;
    if !botrecipe.is_null() {
        info!("GEN3: Loaded recipe from config");
    }

    let genetic_config = config.get_genetic_config();
    let batch_config = genetic_config.batch_config;
    let game_factory = create_game_factory(&genetic_config.game);
    let game = game_factory();
    let game_info = game.get_game_info();
    let mut bot_factory = BotFactory::new(game_info, config.get_bot_config());
    let (bot1, bot2) = bot_factory.create_bots();
    let bots = [bot1, bot2];

    let genetic_index = if bots[0].is_genetic() {
        if bots[1].is_genetic() {
            warn!(
                "GEN3: Both bots are genetic. Only first bot ({}) will use the genetic algorithm",
                bots[0].get_name()
            );
        }
        0
    } else {
        assert!(bots[1].is_genetic(), "Neither bot is a genetic bot!");
        1
    };

    bot_factory.set_genetic_index(genetic_index);

    let max_evaluations =
        (genetic_config.num_generations as u64) * (genetic_config.num_samples as u64);
    info!(
        "GEN3: Starting with {} max evaluations, keep={}, batch={}",
        max_evaluations, genetic_config.keep_samples, batch_config.batch_size
    );

    let scores_file = {
        let scores_path = match env::current_exe() {
            Ok(x) => {
                let mut p = x.parent().expect("Error getting parent dir").to_path_buf();
                p.push("scores.csv");
                p
            }
            _ => panic!("Error getting current exe path"),
        };
        Some(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(scores_path)
                .expect("Error opening scores.csv"),
        )
    };

    let pool: Arc<Mutex<PopulationPool>> = Arc::new(Mutex::new(PopulationPool::new(
        genetic_config.keep_samples as usize,
        max_evaluations,
        scores_file,
    )));

    // Seed the population with the initial bot.
    {
        let initial_recipe = bots[genetic_index].to_json();

        // Run an initial batch to get the seed score.
        let mut seed_factory = bot_factory.clone();
        seed_factory.set_genetic_recipe(initial_recipe.clone());
        let seed_result = run_batch(&batch_config, false, game_factory, &seed_factory);
        let seed_score = if genetic_index == 0 {
            seed_result.get_score1()
        } else {
            seed_result.get_score2()
        };

        let mut p = pool.lock().unwrap();
        p.individuals.push(Individual {
            recipe: initial_recipe,
            score: seed_score,
            generation: 0,
            id: 0,
        });
        p.next_id = 1;
        p.best_score = Some(seed_score);
        info!("GEN3: Seed score: {:.3}", seed_score);
    }

    let mut thread_pool = Vec::new();

    for thread_num in 0..6 {
        let mut factory = bot_factory.clone();
        let thread_batch_config = batch_config.clone();
        let thread_pool_ref = pool.clone();

        let handle = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            loop {
                // Brief lock: get work.
                let work = {
                    let mut p = thread_pool_ref.lock().unwrap();
                    if p.is_done() {
                        break;
                    }
                    p.propose_child()
                };

                let (parent_recipe, strategy, crossover_recipe) = match work {
                    Some(w) => w,
                    None => break,
                };

                // Create and mutate bot (unlocked).
                let (new_recipe, parent_score) = match strategy {
                    MutationStrategy::FreshRandom => {
                        let bot = factory.create_genetic_bot();
                        (bot.to_json(), None)
                    }
                    MutationStrategy::SingleMutate => {
                        let mut bot =
                            factory.create_genetic_bot_with_custom_recipe(&parent_recipe);
                        bot.mutate();
                        (bot.to_json(), None)
                    }
                    MutationStrategy::MultiMutate => {
                        let mut bot =
                            factory.create_genetic_bot_with_custom_recipe(&parent_recipe);
                        let count = rng.gen_range(2, 6);
                        bot.mutate_with_intensity(MutationIntensity {
                            count,
                            step_scale: 1.0,
                            correlated: false,
                        });
                        (bot.to_json(), None)
                    }
                    MutationStrategy::Crossover => {
                        let mut bot =
                            factory.create_genetic_bot_with_custom_recipe(&parent_recipe);
                        if let Some(ref cr) = crossover_recipe {
                            bot.crossover(cr);
                        }
                        // Apply a small mutation after crossover.
                        bot.mutate();
                        (bot.to_json(), None)
                    }
                    MutationStrategy::AdaptiveStep => {
                        let mut bot =
                            factory.create_genetic_bot_with_custom_recipe(&parent_recipe);
                        let temp = {
                            thread_pool_ref.lock().unwrap().temperature
                        };
                        let step = 0.1 + temp * 2.0;
                        bot.mutate_with_intensity(MutationIntensity {
                            count: 1,
                            step_scale: step,
                            correlated: rng.gen::<bool>(),
                        });
                        (bot.to_json(), None)
                    }
                    MutationStrategy::SimulatedAnnealing => {
                        let mut bot =
                            factory.create_genetic_bot_with_custom_recipe(&parent_recipe);
                        let temp = {
                            thread_pool_ref.lock().unwrap().temperature
                        };
                        let count = std::cmp::max(1, (temp * 3.0) as u32);
                        bot.mutate_with_intensity(MutationIntensity {
                            count,
                            step_scale: 1.0 + temp,
                            correlated: false,
                        });

                        // Get parent score for SA acceptance.
                        let ps = {
                            let p = thread_pool_ref.lock().unwrap();
                            if p.individuals.is_empty() {
                                None
                            } else {
                                Some(p.tournament_select().score)
                            }
                        };
                        (bot.to_json(), ps)
                    }
                };

                // Long unlocked batch run.
                factory.set_genetic_recipe(new_recipe.clone());
                let batch_result =
                    run_batch(&thread_batch_config, false, game_factory, &factory);
                let genetic_score = if genetic_index == 0 {
                    batch_result.get_score1()
                } else {
                    batch_result.get_score2()
                };

                // Brief lock: submit result.
                {
                    let mut p = thread_pool_ref.lock().unwrap();
                    p.submit(new_recipe, genetic_score, strategy, parent_score);
                }
            }

            info!("GEN3: Thread {} finished", thread_num);
        });

        thread_pool.push(handle);
    }

    for handle in thread_pool {
        if let Err(e) = handle.join() {
            error!("GEN3: Error joining thread: {:?}", e);
        }
    }

    // Final report.
    let p = pool.lock().unwrap();
    info!("GEN3: === Final Report ===");
    info!(
        "GEN3: Total evaluated: {} / {}",
        p.total_evaluated, p.max_evaluations
    );
    info!(
        "GEN3: Best score: {:.3}",
        p.best_score.unwrap_or(0.0)
    );
    info!("GEN3: Final temperature: {:.3}", p.temperature);
    info!(
        "GEN3: Population size: {} / {}",
        p.individuals.len(),
        p.capacity
    );
    p.log_stats();

    if let Some(ref recipe) = p.best_recipe {
        info!("GEN3: Best recipe: {}", recipe);

        let recipe_path = match env::current_exe() {
            Ok(x) => {
                let mut path = x.parent().expect("Error getting parent dir").to_path_buf();
                path.push("best.recipe");
                path
            }
            Err(_) => panic!("Error getting current exe path"),
        };
        std::fs::write(&recipe_path, recipe.to_string())
            .expect("Error writing best.recipe");
        info!("GEN3: Saved best recipe to {}", recipe_path.display());
    }

    Ok(())
}
