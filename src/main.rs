use clap::Parser;
use std::io::{self, Write};
use std::thread;
use sysinfo::{Pid, Process, System};

/// Find and optionally kill high-memory, low-CPU processes owned by the current user (macOS).
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Actually kill the processes (default is dry-run / list only)
    #[arg(short, long)]
    kill: bool,

    /// Skip confirmation prompt when using --kill
    #[arg(short, long)]
    yes: bool,

    /// Minimum memory usage in MB to consider (default: 50)
    #[arg(short, long, default_value_t = 50.0)]
    mem_threshold_mb: f64,

    /// Maximum CPU usage % to consider a process "idle" (default: 1.0)
    #[arg(long, default_value_t = 1.0)]
    max_cpu: f32,

    /// Process names to always skip (comma-separated)
    #[arg(long, value_delimiter = ',')]
    exclude: Vec<String>,
}

fn confirm_kill(count: usize, total_mb: f64) -> bool {
    print!(
        "\nAbout to terminate {} process(es) (~{:.1} MB). Continue? [y/N] ",
        count, total_mb
    );
    let _ = io::stdout().flush();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

fn main() {
    let args = Args::parse();
    let mut sys = System::new_all();

    // CPU usage needs two samples
    sys.refresh_all();
    thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_processes();

    let current_pid = sysinfo::get_current_pid().ok();
    let current_uid = current_pid
        .and_then(|pid| sys.process(pid))
        .and_then(|p| p.user_id().cloned());

    let mut targets: Vec<(&Pid, &Process)> = Vec::new();

    for (pid, process) in sys.processes() {
        // Never touch ourselves
        if Some(*pid) == current_pid {
            continue;
        }

        // Only our own processes
        if process.user_id() != current_uid.as_ref() {
            continue;
        }

        let name = process.name().to_string();
        if args.exclude.iter().any(|e| name.contains(e)) {
            continue;
        }

        let mem_mb = process.memory() as f64 / 1_048_576.0;
        if mem_mb < args.mem_threshold_mb {
            continue;
        }

        if process.cpu_usage() > args.max_cpu {
            continue;
        }

        targets.push((pid, process));
    }

    targets.sort_by(|a, b| b.1.memory().cmp(&a.1.memory()));

    if targets.is_empty() {
        println!(
            "No matching background processes (mem ≥ {:.0} MB, CPU ≤ {:.1}%).",
            args.mem_threshold_mb, args.max_cpu
        );
        return;
    }

    let total_mb: f64 = targets
        .iter()
        .map(|(_, p)| p.memory() as f64 / 1_048_576.0)
        .sum();

    println!("Found {} candidate processes (~{:.1} MB total):", targets.len(), total_mb);
    println!("{:<8} | {:<28} | {:>10} | {:>8}", "PID", "NAME", "MEM (MB)", "CPU %");
    println!("{}", "-".repeat(62));

    for (pid, process) in &targets {
        let mem_mb = process.memory() as f64 / 1_048_576.0;
        println!(
            "{:<8} | {:<28} | {:>10.1} | {:>8.1}",
            pid,
            process.name(),
            mem_mb,
            process.cpu_usage()
        );
    }

    println!("{}", "-".repeat(62));

    if !args.kill {
        println!("Dry run. Re-run with --kill to terminate these processes.");
        return;
    }

    if !args.yes && !confirm_kill(targets.len(), total_mb) {
        println!("Aborted.");
        return;
    }

    let mut freed = 0.0;
    for (pid, process) in &targets {
        let mem_mb = process.memory() as f64 / 1_048_576.0;
        if process.kill() {
            println!("Killed PID {} ({}) — {:.1} MB", pid, process.name(), mem_mb);
            freed += mem_mb;
        } else {
            eprintln!("Failed to kill PID {} ({})", pid, process.name());
        }
    }

    println!("Approximate memory freed: {:.1} MB", freed);
}
