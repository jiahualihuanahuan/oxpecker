# 🧹 Oxpecker

**Oxpecker** is a lightning-fast, native macOS command-line utility designed to safely reclaim system memory by identifying and terminating stalled, heavy, or unused background processes. Written in Rust, it operates with zero overhead and runs natively on both Intel and Apple Silicon chips. 

Modern macOS keeps memory full by design, caching applications so they open instantly. However, rogue background tasks, stalled development servers, and abandoned Electron helper apps can quietly consume gigabytes of RAM. Mac Cleaner gives developers and power users a surgical tool to clean up these specific resource-hogs without rebooting or using bloated third-party cleaner apps.

https://github.com/user-attachments/assets/a908b81e-ae59-469b-9aa3-f24b61ffb5c8

## ✨ Key Features

*   **Smart Detection:** Automatically filters out active applications, specifically targeting sleeping background processes utilizing less than 1% of your CPU but holding onto significant chunks of RAM.
*   **Dry-Run by Default:** By design, the tool will never terminate a process unless explicitly instructed. Running the base command audits your system and prints a formatted report of memory-heavy apps.
*   **System-Safe Architecture:** Built with strict safety rails, Mac Cleaner automatically detects your user ID and will **only** interact with processes you own, ensuring critical system (root) tasks are never touched.
*   **Customizable Thresholds:** Users can pass specific memory targets (e.g., `--mem-threshold-mb 200`) to ignore small background tasks and only target massive memory leaks.
*   **Instant Execution:** Powered by Rust and `sysinfo`, the tool scans the Mach kernel and Darwin processes in milliseconds.

## 🚀 Installation

### Option 1: Homebrew (Recommended)
You can install Mac Cleaner via Homebrew using our custom tap. *(Note: Update `yourusername` with your actual GitHub username)*
```bash
brew tap jiahualihuanahuan/tap
brew install oxpecker
```
### Option 2: Cargo (For Rust Developers)
If you already have Rust installed, you can compile and install directly from GitHub:
```bash
cargo install --git [https://github.com/jiahualihuanahuan/oxpecker](https://github.com/jiahualihuanahuan/oxpecker)
```
### 💻 Usage
By default, Mac Cleaner operates in Dry Run mode. It will only list the processes it finds and will not kill anything unless the --kill flag is provided.

View memory-heavy background processes (Default: > 50 MB)
```bash
oxpecker
```

Specify a custom memory threshold (e.g., processes using > 200 MB)

```bash
oxpecker --mem-threshold-mb 200
```

Execute and kill the identified processes


```bash
oxpecker --mem-threshold-mb 100 --kill
```

## 🛠️ Building from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/jiahualihuanahuan/oxpecker.git
   cd oxpecker
   ```
2. Build the optimized release binary:
   ```bash
   cargo build --release
   ```
3. Run the compiled binary:
   ```bash
   ./target/release/oxpecker --help
   ```

## ⚠️ Disclaimer
While Mac Cleaner includes safety rails to prevent terminating system processes, aggressively killing processes can result in unsaved data loss for those specific applications. Use the `--kill` flag thoughtfully.
