# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
cargo build                    # Debug build
cargo build --release          # Release build (use for benchmarking)
cargo test                     # Run all tests
cargo test <test_name>         # Run a single test (e.g., cargo test test_roundtrip)
cargo clippy                   # Lint (cognitive-complexity-threshold = 30)
```

## Running

```bash
./target/release/naughts <bot1> <bot2> --game <naughts|connect4> [options]

# Single game (human vs random)
./target/release/naughts naughts.human randombot --game naughts

# Batch run (1000 games)
./target/release/naughts genbot3 randombot --game naughts --batch 1000

# Genetic evolution (uses gen2_runner by default)
./target/release/naughts genbot3 randombot --game naughts --batch 1000 --genetic 100 --samples 50 --keep 5

# Load a saved recipe
./target/release/naughts genbot3 randombot --game naughts --batch 1000 --botrecipe my_bot.recipe

# Magic mode (exhaustive game tree with omnibot)
./target/release/naughts omnibot randombot --game naughts --magic --genetic 100
```

MongoDB (via `docker-compose.yml`) is optional, used with `--botdb` for recipe persistence.

## Architecture

This is a binary crate (`naughts`) with no `lib.rs` — all modules are declared inline in `main.rs`.

### Three-layer design: Engine / Games / Bots

**Engine** (`src/engine/`) orchestrates everything:
- `GameConfig` — CLI arg parsing, determines `RunMode` (Single/Batch/Genetic)
- `BotFactory` — creates bots by name, manages recipes, tracks which bot is genetic
- `GameFactory` — returns `fn() -> Box<dyn GameTrait>` function pointers (must be `fn`, not closures, to be `Copy`/thread-safe)
- Runners (`src/engine/runners/`) — `single_runner`, `batch_runner`, `genetic_runner`, `gen2_runner`

**Games** (`src/games/`) implement `GameTrait`:
- `get_inputs()` returns float vectors encoding the board from the current player's perspective
- `update()` applies a move; if the move is invalid, the game snaps to the closest legal move
- Naughts: 18 inputs (9 self + 9 opponent), 9 outputs
- Connect4: 98 inputs (49 self + 49 opponent), 7 outputs

**Bots** (`src/bots/`) implement `GamePlayer: GameObject + Send`:
- `process(inputs, available_moves) -> u32` — returns a single move index
- `process_magic()` — returns all legal moves (for exhaustive tree search)
- `mutate()` — in-place mutation for genetic bots
- `is_genetic()` / `is_magic()` — flags that determine runner behavior

### Core traits

- **`GameObject`** — `to_json()`/`from_json()` serialization, used for recipe persistence and threading
- **`GamePlayer`** — bot interface, extends `GameObject + Send`
- **`GameTrait`** — game interface, extends `GameObject`

### Key type aliases

```rust
pub type GameFactory = fn() -> Box<dyn GameTrait>;  // Must be fn pointer, not closure
pub type DynBot = Box<dyn GamePlayer>;
pub type GameScore = f32;
```

### Recipe system

A recipe is a `serde_json::Value` that fully encodes a bot's state. It enables persistence (MongoDB/files), mutation, and thread cloning.
- **GenBot3** recipes: compact string format `"NODE_INPUT,NODE_AND:0:1,..."` wrapped in `{"recipe": "..."}`
- **NBot1** recipes: full JSON `{"layers": [{"nodes": [{"input_weights": [...], "bias": 0.5}]}]}`
- CLI: `--botrecipe <file>` loads from file; `--botid <id>` loads from MongoDB

### Runners

- **single_runner** — one interactive game
- **batch_runner** — N games, reports win/loss/draw stats
- **gen2_runner** (active) — depth-first hill-climber using a shared `Arc<Mutex<BotStack>>` across 6 threads. Pushes better-scoring mutants deeper on the stack, drops bots after `child_limit` children without improvement. Score averaging dampens flukes.
- **genetic_runner** (legacy, commented out in main) — breadth-first generational GA with a 6-thread pool
- **magic batch** — exhaustive DFS over all game trees using OmniBot's `process_magic()`

### Bot types

- `randombot` — random legal move
- `genbot3` — boolean logic circuit (AND/OR/XOR/etc nodes), genetic
- `nbot1` — 3-layer neural network with sigmoid activation, genetic
- `omnibot` — magic bot for exhaustive search
- `naughts.human` / `connect4.human` — stdin-based interactive play

## Conventions

- Rust 2018 edition, 4-space indentation (`rustfmt.toml`)
- Clippy with high cognitive-complexity threshold (30) (`clippy.toml`)
- Logging via `fern` + `log` crate to stdout and `./log/naughts.log`
- Recipe files (`*.recipe`) are gitignored at project root
