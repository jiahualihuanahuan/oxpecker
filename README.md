# Oxpecker

A fast native macOS CLI that finds high-memory, low-CPU background processes you own and can optionally terminate them. Written in Rust.

macOS keeps a lot of memory cached on purpose. Oxpecker helps you surgically clean up stalled helpers, abandoned Electron processes, and other memory hogs without rebooting or installing a full cleaner app.

## Features

- Dry-run by default (never kills unless you pass `--kill`)
- Confirmation prompt before any kill (skip with `--yes`)
- Only touches processes owned by your user
- Accurate idle detection (double-samples CPU)
- Configurable memory and CPU thresholds
- Optional name exclusion list
- Excludes itself automatically

## Install

### Homebrew (recommended)

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

The formula builds from source. Homebrew will install Rust automatically if needed.

### Cargo

```bash
cargo install --git https://github.com/jiahualihuanahuan/oxpecker
```

### From source

```bash
git clone https://github.com/jiahualihuanahuan/oxpecker.git
cd oxpecker
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

Kill without prompt (scripting):

```bash
oxpecker --kill --yes
```

Skip specific names:

```bash
oxpecker --exclude "Google Chrome,Code"
```

Adjust the idle CPU threshold:

```bash
oxpecker --max-cpu 0.5
```

## Disclaimer

Killing processes can lose unsaved data in those apps. Use `--kill` carefully. The tool only acts on processes owned by your user.

## License

MIT — see [LICENSE](LICENSE)
