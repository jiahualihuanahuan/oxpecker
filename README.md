# Oxpecker

**v1.0.0** — A fast native macOS CLI that finds high-memory, low-CPU background processes you own and can optionally terminate them. Written in Rust.

macOS keeps a lot of memory cached on purpose. Oxpecker helps you surgically clean up stalled helpers, abandoned Electron processes, and other memory hogs without rebooting or installing a full cleaner app.

## Features

- Dry-run by default (never kills unless you pass `--kill`)
- Confirmation prompt before any kill (skip with `--yes`)
- Interactive multi-select mode (`--interactive`)
- Only touches processes owned by your user
- Accurate idle detection (double-samples CPU)
- Configurable memory and CPU thresholds
- Case-insensitive name exclusion list (plus sensible defaults)
- Safer killing: SIGTERM first, then SIGKILL after a grace period
- JSON output for scripting (`--format json`)
- Sort by memory, CPU, name, or PID
- Watch mode (`--watch`)
- Optional command-line display (`--show-cmd`)
- Colorized output (respects `NO_COLOR` / `--no-color`)
- System memory context
- Excludes itself automatically
- MIT licensed

## What "memory" means

Memory figures are **RSS** (Resident Set Size) — the amount of physical RAM the process currently occupies. This is a practical proxy for “memory hog” on macOS, though shared memory and compressed pages can make the numbers imperfect.

## What it will not touch

- Processes owned by other users
- Itself
- Processes above the CPU threshold (so active work is left alone)
- A small default exclude list (`WindowServer`, `loginwindow`, `Finder`, `Dock`, `SystemUIServer`)

## Install

### Homebrew (recommended)

Installs a prebuilt macOS binary (no compile step).

Recent Homebrew requires an explicit trust step for third-party taps:

```bash
brew tap jiahualihuanahuan/tap
brew trust jiahualihuanahuan/tap
brew install oxpecker
```

One-liner (after the tap is trusted):

```bash
brew install jiahualihuanahuan/tap/oxpecker
```

### Cargo

```bash
cargo install --git https://github.com/jiahualihuanahuan/oxpecker --tag v1.0.0
```

Or, once published to crates.io:

```bash
cargo install oxpecker
```

### From source

```bash
git clone https://github.com/jiahualihuanahuan/oxpecker.git
cd oxpecker
git checkout v1.0.0
cargo build --release
./target/release/oxpecker --help
```

## Usage

List processes using more than 50 MB and ≤ 1% CPU (default):

```bash
oxpecker
```

Example output:

```
Found 3 candidate processes (~842.3 MB total):
System memory: 12450.2 MB used / 16384.0 MB total (76.0% used)
PID      | NAME                         |   MEM (MB) |   CPU %
--------------------------------------------------------------
12345    | Google Chrome Helper         |      412.1 |    0.0
23456    | node                         |      280.4 |    0.2
34567    | Code Helper                  |      149.8 |    0.0
--------------------------------------------------------------
Dry run. Re-run with --kill to terminate these processes.
```

Higher memory threshold:

```bash
oxpecker --mem-threshold-mb 200
```

Kill with confirmation:

```bash
oxpecker --mem-threshold-mb 100 --kill
```

Interactive selection:

```bash
oxpecker --kill --interactive
```

Kill without prompt (scripting):

```bash
oxpecker --kill --yes
```

JSON output:

```bash
oxpecker --format json
```

Skip specific names (case-insensitive):

```bash
oxpecker --exclude "Google Chrome,Code"
```

Adjust the idle CPU threshold:

```bash
oxpecker --max-cpu 0.5
```

Show full command lines:

```bash
oxpecker --show-cmd
```

Sort by CPU instead of memory:

```bash
oxpecker --sort cpu
```

Watch mode (refresh every 5 seconds):

```bash
oxpecker --watch 5
```

Custom grace period after SIGTERM:

```bash
oxpecker --kill --grace-period 5
```

## Disclaimer

Killing processes can lose unsaved data in those apps. Use `--kill` carefully. The tool only acts on processes owned by your user and prefers SIGTERM before SIGKILL.

## Development

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

CI runs formatting, Clippy, and tests on every push and pull request.

## License

MIT — see [LICENSE](LICENSE)
