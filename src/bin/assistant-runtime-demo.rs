use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        print_help();
        return Ok(());
    }

    match args.remove(0).as_str() {
        "init" => init_demo(args),
        "status" => status_demo(args),
        "run" => run_demo(args),
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        other => Err(format!("unknown demo command: {other}")),
    }
}

fn print_help() {
    println!("assistant-runtime-demo");
    println!("  init [--repo <dir>]");
    println!("  status [--repo <dir>]");
    println!("  run [--repo <dir>] [--ticks <n>] [--sleep-ms <n>]");
    println!();
    println!("Demo shape:");
    println!("  - installs the current assistant-runtime into a demo repo");
    println!("  - shows runtime, resource, task-queue, and worker status");
    println!("  - simulates occasional direct and planner work over time");
}

fn init_demo(args: Vec<String>) -> Result<(), String> {
    let repo = parse_repo_arg(&args)?.unwrap_or_else(default_demo_repo);
    ensure_demo_repo(&repo)?;
    install_runtime_into_repo(&repo)?;
    write_demo_readme(&repo)?;

    println!("initialized demo repo at {}", repo.display());
    println!("runtime binary: {}", runtime_binary()?.display());
    println!("try:");
    println!("  assistant-runtime-demo status --repo {}", repo.display());
    println!("  assistant-runtime-demo run --repo {} --ticks 8", repo.display());
    Ok(())
}

fn status_demo(args: Vec<String>) -> Result<(), String> {
    let repo = parse_repo_arg(&args)?.unwrap_or_else(default_demo_repo);
    ensure_demo_repo(&repo)?;
    maybe_install_if_missing(&repo)?;

    println!("demo repo: {}", repo.display());
    println!();

    println!("== runtime manifest ==");
    print_runtime_command(&repo, &["runtime", "manifest"])?;
    println!();

    println!("== managed skill/tool resources ==");
    print_runtime_command(&repo, &["runtime", "list-resources", "--class", "skill"])?;
    print_runtime_command(&repo, &["runtime", "list-resources", "--class", "tool"])?;
    println!();

    println!("== chat lane ==");
    print_runtime_command(&repo, &["chat", "status"])?;
    println!();

    println!("== task queue ==");
    print_runtime_command(&repo, &["chat", "queue"])?;
    println!();

    println!("== direct worker ==");
    print_json_file(repo.join(".runtime/workers/direct/tasks.json"))?;
    println!();

    println!("== planner worker ==");
    print_json_file(repo.join(".runtime/workers/planner/tasks.json"))?;
    Ok(())
}

fn run_demo(args: Vec<String>) -> Result<(), String> {
    let mut repo = None;
    let mut ticks: usize = 8;
    let mut sleep_ms: u64 = 200;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo" => repo = Some(PathBuf::from(iter.next().ok_or("missing value for --repo")?)),
            "--ticks" => {
                let value = iter.next().ok_or("missing value for --ticks")?;
                ticks = value.parse::<usize>().map_err(|_| format!("invalid ticks: {value}"))?;
            }
            "--sleep-ms" => {
                let value = iter.next().ok_or("missing value for --sleep-ms")?;
                sleep_ms = value.parse::<u64>().map_err(|_| format!("invalid sleep-ms: {value}"))?;
            }
            other => return Err(format!("unknown run flag: {other}")),
        }
    }

    let repo = repo.unwrap_or_else(default_demo_repo);
    ensure_demo_repo(&repo)?;
    maybe_install_if_missing(&repo)?;

    println!("demo repo: {}", repo.display());
    println!("ticks: {ticks}, sleep_ms: {sleep_ms}");
    println!();

    for tick in 1..=ticks {
        println!("-- tick {tick} --");
        let action = occasional_action_for_tick(tick);
        if let Some((intent, planner)) = action {
            let mut cmd = vec!["chat".to_string(), "accept".to_string(), intent.to_string()];
            if planner {
                cmd.push("--plan".to_string());
            }
            let rendered = run_runtime_command(&repo, &cmd.iter().map(String::as_str).collect::<Vec<_>>())?;
            println!("{rendered}");
        } else {
            println!("no new task this tick");
        }

        println!("task queue:");
        print_runtime_command(&repo, &["chat", "queue"])?;
        println!("chat lane:");
        print_runtime_command(&repo, &["chat", "status"])?;
        println!();
        thread::sleep(Duration::from_millis(sleep_ms));
    }

    println!("final worker state:");
    println!("direct:");
    print_json_file(repo.join(".runtime/workers/direct/tasks.json"))?;
    println!("planner:");
    print_json_file(repo.join(".runtime/workers/planner/tasks.json"))?;
    Ok(())
}

fn occasional_action_for_tick(tick: usize) -> Option<(&'static str, bool)> {
    match tick {
        1 => Some(("refresh the local runtime status snapshot", false)),
        3 => Some(("analyze a telemetry regression in the background", true)),
        5 => Some(("prepare the weekly observability review packet", true)),
        7 => Some(("summarize the latest host posture drift", false)),
        _ => None,
    }
}

fn parse_repo_arg(args: &[String]) -> Result<Option<PathBuf>, String> {
    let mut repo = None;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo" => repo = Some(PathBuf::from(iter.next().ok_or("missing value for --repo")?)),
            other => return Err(format!("unknown flag: {other}")),
        }
    }
    Ok(repo)
}

fn default_demo_repo() -> PathBuf {
    env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".demo-runtime-playground")
}

fn ensure_demo_repo(repo: &Path) -> Result<(), String> {
    fs::create_dir_all(repo).map_err(|e| format!("failed to create demo repo {}: {e}", repo.display()))
}

fn maybe_install_if_missing(repo: &Path) -> Result<(), String> {
    let installed = repo.join(".assistant-runtime/bin/assistant-runtime");
    if installed.exists() {
        return Ok(());
    }
    install_runtime_into_repo(repo)
}

fn install_runtime_into_repo(repo: &Path) -> Result<(), String> {
    let runtime = runtime_binary()?;
    let package_dir = repo.join(".demo-package");
    let installed_root = repo.join(".assistant-runtime");
    if package_dir.exists() {
        fs::remove_dir_all(&package_dir)
            .map_err(|e| format!("failed to remove old package dir {}: {e}", package_dir.display()))?;
    }
    if installed_root.exists() {
        fs::remove_dir_all(&installed_root)
            .map_err(|e| format!("failed to remove old installed runtime {}: {e}", installed_root.display()))?;
    }

    run_command(
        Command::new(&runtime)
            .current_dir(repo)
            .arg("runtime")
            .arg("package")
            .arg("--output")
            .arg(&package_dir),
        "failed to package assistant-runtime",
    )?;

    run_command(
        Command::new(package_dir.join("install.sh"))
            .current_dir(repo)
            .arg(repo),
        "failed to install assistant-runtime into demo repo",
    )?;

    Ok(())
}

fn write_demo_readme(repo: &Path) -> Result<(), String> {
    let content = r#"# Assistant Runtime Demo Repo

This is a disposable demo repo used by `assistant-runtime-demo`.

It shows:
- foreground orchestrator availability
- task-queue handoff ids
- direct and planner worker separation
- runtime-managed skill/tool resource inspection
"#;
    fs::write(repo.join("README.md"), content)
        .map_err(|e| format!("failed to write demo README: {e}"))
}

fn runtime_binary() -> Result<PathBuf, String> {
    let current = env::current_exe().map_err(|e| format!("failed to resolve current executable: {e}"))?;
    if let Some(dir) = current.parent() {
        let sibling = dir.join("assistant-runtime");
        if sibling.exists() {
            return Ok(sibling);
        }
    }
    Ok(PathBuf::from("assistant-runtime"))
}

fn run_runtime_command(repo: &Path, args: &[&str]) -> Result<String, String> {
    let runtime = repo.join(".assistant-runtime/bin/assistant-runtime");
    let output = Command::new(&runtime)
        .current_dir(repo)
        .args(args)
        .output()
        .map_err(|e| format!("failed to run {}: {e}", runtime.display()))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn print_runtime_command(repo: &Path, args: &[&str]) -> Result<(), String> {
    println!("{}", run_runtime_command(repo, args)?);
    Ok(())
}

fn print_json_file(path: PathBuf) -> Result<(), String> {
    if !path.exists() {
        println!("(missing: {})", path.display());
        return Ok(());
    }
    let body = fs::read_to_string(&path)
        .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    println!("{}", body.trim());
    Ok(())
}

fn run_command(command: &mut Command, context: &str) -> Result<(), String> {
    let output = command
        .output()
        .map_err(|e| format!("{context}: {e}"))?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!(
        "{context}: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    ))
}
