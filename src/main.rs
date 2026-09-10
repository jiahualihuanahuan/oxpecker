use clap::Parser;
use sysinfo::{Pid, Process, System};

/// A CLI tool to find and kill unused background processes on macOS
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Actually kill the processes (default is dry-run / list only)
    #[arg(short, long)]
    kill: bool,

    /// Minimum memory usage in Megabytes to be considered (default: 50)
    #[arg(short, long, default_value_t = 50.0)]
    mem_threshold_mb: f64,
}

fn main() {
    let args = Args::parse();
    let mut sys = System::new_all();
    
    // Refresh process lists and memory
    sys.refresh_all();

    // Safely get the current user ID without unsafe C code
    let current_uid = sysinfo::get_current_pid()
        .ok()
        .and_then(|pid| sys.process(pid))
        .and_then(|p| p.user_id().cloned());

    let mut target_processes: Vec<(&Pid, &Process)> = Vec::new();

    // 1. Identify Target Processes
    for (pid, process) in sys.processes() {
        let mem_usage_mb = process.memory() as f64 / 1_048_576.0;

        // Safety Filter: Only touch processes owned by the current user
        if process.user_id() != current_uid.as_ref() {
            continue;
        }

        // Filter: Must exceed memory threshold
        if mem_usage_mb < args.mem_threshold_mb {
            continue;
        }

        // Filter: Look for background/sleeping processes with 0% CPU usage
        if process.cpu_usage() < 1.0 {
            target_processes.push((pid, process));
        }
    }

    // Sort by memory usage (highest first)
    target_processes.sort_by(|a, b| b.1.memory().cmp(&a.1.memory()));

    if target_processes.is_empty() {
        println!("No background processes found exceeding {} MB.", args.mem_threshold_mb);
        return;
    }

    println!("Found {} sleeping background processes:", target_processes.len());
    println!("{:<8} | {:<25} | {:<10}", "PID", "NAME", "MEMORY (MB)");
    println!("--------------------------------------------------");

    // 2. Execute Action (List or Kill)
    let mut freed_memory = 0.0;

    for (pid, process) in target_processes {
        let mem_mb = process.memory() as f64 / 1_048_576.0;
        println!("{:<8} | {:<25} | {:.2} MB", pid, process.name(), mem_mb);

        if args.kill {
            if process.kill() {
                println!("  ↳ Successfully killed PID {}", pid);
                freed_memory += mem_mb;
            } else {
                eprintln!("  ↳ Failed to kill PID {}", pid);
            }
        }
    }

    if args.kill {
        println!("--------------------------------------------------");
        println!("Total memory freed: {:.2} MB", freed_memory);
    } else {
        println!("--------------------------------------------------");
        println!("DRY RUN: Run with --kill to terminate these processes.");
    }
}
