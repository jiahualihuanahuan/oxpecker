use clap::{Parser, ValueEnum};
use colored::Colorize;
use dialoguer::{Confirm, MultiSelect};
use serde::Serialize;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use sysinfo::{Pid, Process, Signal, System};

#[derive(Clone, Copy, Debug, ValueEnum)]
enum SortBy {
    Mem,
    Cpu,
    Name,
    Pid,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Actually kill the processes (default is dry-run / list only)
    #[arg(short, long)]
    kill: bool,

    /// Skip confirmation prompt when using --kill
    #[arg(short, long)]
    yes: bool,

    /// Interactive multi-select before killing
    #[arg(short, long)]
    interactive: bool,

    /// Minimum memory usage in MB to consider (default: 50)
    #[arg(short, long, default_value_t = 50.0)]
    mem_threshold_mb: f64,

    /// Maximum CPU usage % to consider a process "idle" (default: 1.0)
    #[arg(long, default_value_t = 1.0)]
    max_cpu: f32,

    /// Process names to always skip (comma-separated, case-insensitive substring match)
    #[arg(long, value_delimiter = ',')]
    exclude: Vec<String>,

    /// Output format: text or json
    #[arg(long, default_value = "text")]
    format: String,

    /// Sort results by mem (default), cpu, name, or pid
    #[arg(long, value_enum, default_value_t = SortBy::Mem)]
    sort: SortBy,

    /// Watch mode: refresh every N seconds (0 = off)
    #[arg(long, default_value_t = 0)]
    watch: u64,

    /// Grace period in seconds after SIGTERM before SIGKILL (default: 3)
    #[arg(long, default_value_t = 3)]
    grace_period: u64,

    /// Show command line of processes
    #[arg(long)]
    show_cmd: bool,

    /// Disable color output
    #[arg(long)]
    no_color: bool,
}

#[derive(Serialize)]
struct ProcessInfo {
    pid: u32,
    name: String,
    mem_mb: f64,
    cpu: f32,
    cmd: String,
    parent: Option<u32>,
    start_time: u64,
}

fn default_excludes() -> Vec<String> {
    vec![
        "windowserver".to_string(),
        "loginwindow".to_string(),
        "finder".to_string(),
        "dock".to_string(),
        "systemuiserver".to_string(),
    ]
}

fn matches_exclude(name: &str, excludes: &[String]) -> bool {
    let name_lower = name.to_lowercase();
    excludes.iter().any(|e| name_lower.contains(&e.to_lowercase()))
}

fn collect_targets<'a>(
    sys: &'a System,
    current_pid: Option<Pid>,
    current_uid: Option<&sysinfo::Uid>,
    args: &Args,
) -> Vec<(&'a Pid, &'a Process)> {
    let mut excludes = args.exclude.clone();
    excludes.extend(default_excludes());

    let mut targets: Vec<(&Pid, &Process)> = Vec::new();

    for (pid, process) in sys.processes() {
        if Some(*pid) == current_pid {
            continue;
        }
        if process.user_id() != current_uid {
            continue;
        }

        let name = process.name().to_string_lossy();
        if matches_exclude(&name, &excludes) {
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

    match args.sort {
        SortBy::Mem => targets.sort_by(|a, b| b.1.memory().cmp(&a.1.memory())),
        SortBy::Cpu => targets.sort_by(|a, b| b.1.cpu_usage().partial_cmp(&a.1.cpu_usage()).unwrap_or(std::cmp::Ordering::Equal)),
        SortBy::Name => targets.sort_by(|a, b| a.1.name().cmp(b.1.name())),
        SortBy::Pid => targets.sort_by_key(|(pid, _)| *pid),
    }

    targets
}

fn to_process_info(pid: &Pid, process: &Process) -> ProcessInfo {
    ProcessInfo {
        pid: pid.as_u32(),
        name: process.name().to_string_lossy().to_string(),
        mem_mb: process.memory() as f64 / 1_048_576.0,
        cpu: process.cpu_usage(),
        cmd: process.cmd().iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>().join(" "),
        parent: process.parent().map(|p| p.as_u32()),
        start_time: process.start_time(),
    }
}

fn print_system_memory(sys: &System) {
    let total = sys.total_memory() as f64 / 1_048_576.0;
    let used = sys.used_memory() as f64 / 1_048_576.0;
    let free = total - used;
    println!(
        "System memory: {:.1} MB used / {:.1} MB total ({:.1}% used)",
        used, total, (used / total) * 100.0
    );
}

fn print_table(targets: &[(&Pid, &Process)], args: &Args) {
    if args.no_color {
        colored::control::set_override(false);
    }

    println!(
        "{:<8} | {:<28} | {:>10} | {:>8}{}",
        "PID".bold(),
        "NAME".bold(),
        "MEM (MB)".bold(),
        "CPU %".bold(),
        if args.show_cmd { " | CMD" } else { "" }
    );
    println!("{}", "-".repeat(if args.show_cmd { 90 } else { 62 }));

    for (pid, process) in targets {
        let mem_mb = process.memory() as f64 / 1_048_576.0;
        let name = process.name().to_string_lossy();
        let name_display = if name.len() > 28 { format!("{}...", &name[..25]) } else { name.to_string() };
        let cpu = process.cpu_usage();

        let mem_str = format!("{:>10.1}", mem_mb);
        let colored_mem = if mem_mb > 500.0 {
            mem_str.red()
        } else if mem_mb > 200.0 {
            mem_str.yellow()
        } else {
            mem_str.normal()
        };

        print!(
            "{:<8} | {:<28} | {} | {:>8.1}",
            pid, name_display, colored_mem, cpu
        );
        if args.show_cmd {
            let cmd = process.cmd().iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>().join(" ");
            print!(" | {}", cmd);
        }
        println!();
    }
    println!("{}", "-".repeat(if args.show_cmd { 90 } else { 62 }));
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

fn kill_processes(targets: &[(&Pid, &Process)], grace: u64) -> (usize, f64) {
    let mut killed = 0;
    let mut freed = 0.0;

    for (pid, process) in targets {
        let mem_mb = process.memory() as f64 / 1_048_576.0;
        let name = process.name().to_string_lossy();

        // Try SIGTERM first
        if process.kill_with(Signal::Term).unwrap_or(false) {
            println!("Sent SIGTERM to PID {} ({})", pid, name);
            thread::sleep(Duration::from_secs(grace));

            // Check if still alive (simple re-check via kill with 0 or just proceed to SIGKILL)
            // For simplicity, always follow up with SIGKILL if grace period elapsed
            if process.kill() {
                println!("Killed PID {} ({}) — {:.1} MB", pid, name, mem_mb);
                killed += 1;
                freed += mem_mb;
            } else {
                eprintln!("Failed to SIGKILL PID {} ({}) after grace period", pid, name);
            }
        } else {
            // Fallback to SIGKILL immediately
            if process.kill() {
                println!("Killed PID {} ({}) — {:.1} MB", pid, name, mem_mb);
                killed += 1;
                freed += mem_mb;
            } else {
                eprintln!("Failed to kill PID {} ({})", pid, name);
            }
        }
    }

    (killed, freed)
}

fn run_once(args: &Args) -> bool {
    let mut sys = System::new_all();
    sys.refresh_all();
    thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let current_pid = sysinfo::get_current_pid().ok();
    let current_uid = current_pid
        .and_then(|pid| sys.process(pid))
        .and_then(|p| p.user_id());

    let targets = collect_targets(&sys, current_pid, current_uid, args);

    if targets.is_empty() {
        if args.format == "json" {
            println!("{{\"processes\": [], \"total_mb\": 0.0}}");
        } else {
            println!(
                "No matching background processes (mem ≥ {:.0} MB, CPU ≤ {:.1}%).",
                args.mem_threshold_mb, args.max_cpu
            );
            print_system_memory(&sys);
        }
        return false;
    }

    let total_mb: f64 = targets.iter().map(|(_, p)| p.memory() as f64 / 1_048_576.0).sum();

    if args.format == "json" {
        let infos: Vec<ProcessInfo> = targets.iter().map(|(pid, p)| to_process_info(pid, p)).collect();
        let output = serde_json::json!({
            "processes": infos,
            "total_mb": total_mb,
            "count": infos.len()
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        return true;
    }

    println!(
        "Found {} candidate processes (~{:.1} MB total):",
        targets.len(), total_mb
    );
    print_system_memory(&sys);
    print_table(&targets, args);

    if !args.kill {
        println!("Dry run. Re-run with --kill to terminate these processes.");
        return true;
    }

    let mut selected = targets.clone();
    if args.interactive && !args.yes {
        let items: Vec<String> = targets
            .iter()
            .map(|(pid, p)| {
                format!(
                    "{} | {} | {:.1} MB | {:.1}%",
                    pid,
                    p.name().to_string_lossy(),
                    p.memory() as f64 / 1_048_576.0,
                    p.cpu_usage()
                )
            })
            .collect();
        let selections = MultiSelect::new()
            .with_prompt("Select processes to terminate")
            .items(&items)
            .interact()
            .unwrap_or_default();
        selected = selections.iter().map(|&i| targets[i]).collect();
        if selected.is_empty() {
            println!("No processes selected.");
            return true;
        }
    }

    let selected_total: f64 = selected.iter().map(|(_, p)| p.memory() as f64 / 1_048_576.0).sum();

    if !args.yes && !args.interactive {
        if !confirm_kill(selected.len(), selected_total) {
            println!("Aborted.");
            return true;
        }
    } else if !args.yes && args.interactive {
        if !Confirm::new()
            .with_prompt(format!("Terminate {} selected process(es) (~{:.1} MB)?", selected.len(), selected_total))
            .interact()
            .unwrap_or(false)
        {
            println!("Aborted.");
            return true;
        }
    }

    let (killed, freed) = kill_processes(&selected, args.grace_period);
    println!(
        "Terminated {} process(es). Approximate memory freed: {:.1} MB",
        killed, freed
    );

    true
}

fn main() {
    let args = Args::parse();

    if args.watch > 0 {
        loop {
            // Clear screen for watch mode
            print!("\x1B[2J\x1B[1;1H");
            let _ = io::stdout().flush();
            println!("Oxpecker watch mode (refresh every {}s) — Ctrl-C to exit", args.watch);
            println!();
            run_once(&args);
            thread::sleep(Duration::from_secs(args.watch));
        }
    } else {
        run_once(&args);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_exclude_case_insensitive() {
        let excludes = vec!["Chrome".to_string(), "node".to_string()];
        assert!(matches_exclude("Google Chrome Helper", &excludes));
        assert!(matches_exclude("NODE", &excludes));
        assert!(!matches_exclude("Safari", &excludes));
    }

    #[test]
    fn test_default_excludes_not_empty() {
        assert!(!default_excludes().is_empty());
    }
}
