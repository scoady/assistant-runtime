mod assistant_runtime;
mod chat;
mod core;
mod durable_truth;
mod planning;
mod proof_metrics;
mod program;
mod resources;
mod runtime;
mod store;
mod transcript_proof;

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    let base = env::current_dir().map_err(|e| format!("failed to read current dir: {e}"))?;

    if args.is_empty() {
        print_help();
        return Ok(());
    }

    if let Some(raw) = args.first() {
        if raw.starts_with('\\') {
            return run_os_passthrough(args, true);
        }
    }

    match args.remove(0).as_str() {
        "run" => {
            let runtime_type = args.first().cloned().ok_or("missing runtime type")?;
            let forwarded = args.into_iter().skip(1).collect::<Vec<_>>();
            dispatch_runtime_type(&runtime_type, forwarded, base)
        }
        "chat" => handle_chat(args, base),
        "runtime" => handle_runtime(args, base),
        "planning" => handle_planning(args),
        "program" => handle_program(args, base),
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        other => Err(format!("unknown command: {other}")),
    }
}

fn run_os_passthrough(args: Vec<String>, strip_backslash: bool) -> Result<(), String> {
    let raw = args.first().ok_or("missing passthrough command")?;
    let program = if strip_backslash {
        raw.trim_start_matches('\\')
    } else {
        raw.as_str()
    };
    if program.is_empty() {
        return Err("missing passthrough command after \\".into());
    }

    let executable = resolve_program(program)?;
    let status = Command::new(&executable)
        .args(args.iter().skip(1))
        .status()
        .map_err(|e| format!("failed to execute {}: {e}", executable.display()))?;

    match status.code() {
        Some(0) => Ok(()),
        Some(code) => Err(format!("os command exited with status {code}: {}", executable.display())),
        None => Err(format!("os command terminated by signal: {}", executable.display())),
    }
}

fn resolve_program(program: &str) -> Result<PathBuf, String> {
    if program.contains('/') {
        let path = PathBuf::from(program);
        if path.exists() {
            return Ok(path);
        }
        return Err(format!("os command not found: {}", path.display()));
    }

    let path_var = env::var_os("PATH").ok_or("PATH is not set")?;
    for candidate_root in env::split_paths(&path_var) {
        let candidate = candidate_root.join(program);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    Err(format!("os command not found in PATH: {program}"))
}

fn dispatch_runtime_type(runtime_type: &str, args: Vec<String>, base: PathBuf) -> Result<(), String> {
    match runtime_type {
        "assistant_runtime" => run_forwarded(args, base),
        "loop_runtime" => handle_program(alias_program_args(args), base),
        "conversation_runtime" => handle_chat(alias_chat_args(args), base),
        "governance_runtime" => Err("governance_runtime is intentionally not exposed in this build".into()),
        "host_runtime" => handle_runtime(args, base),
        "os_runtime" => run_os_passthrough(args, false),
        "registry_runtime" => Err("registry_runtime is intentionally not exposed in this build".into()),
        other => Err(format!("unknown runtime type: {other}")),
    }
}

fn run_forwarded(args: Vec<String>, base: PathBuf) -> Result<(), String> {
    if args.is_empty() {
        print_help();
        return Ok(());
    }
    match args[0].as_str() {
        "chat" => handle_chat(args[1..].to_vec(), base),
        "runtime" => handle_runtime(args[1..].to_vec(), base),
        "program" => handle_program(args[1..].to_vec(), base),
        other => Err(format!("unknown assistant runtime command: {other}")),
    }
}

fn alias_program_args(args: Vec<String>) -> Vec<String> {
    if let Some(first) = args.first() {
        if matches!(first.as_str(), "create" | "list" | "show" | "broad-plan" | "ladder" | "loop" | "tight-loop" | "while-loop") {
            return args;
        }
    }
    let mut forwarded = vec!["broad-plan".to_string()];
    forwarded.extend(args);
    forwarded
}

fn alias_chat_args(args: Vec<String>) -> Vec<String> {
    if let Some(first) = args.first() {
        if matches!(first.as_str(), "status" | "accept" | "queue") {
            return args;
        }
    }
    vec!["status".to_string()]
}

fn handle_runtime(args: Vec<String>, base: PathBuf) -> Result<(), String> {
    if args.is_empty() {
        return Err("missing runtime action".into());
    }
    match args[0].as_str() {
        "manifest" => {
            println!("{}", runtime::manifest(&base)?);
            Ok(())
        }
        "governed-runtime" => {
            println!("{}", runtime::governed_runtime_manifest(&base)?);
            Ok(())
        }
        "durable-truth" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&durable_truth::durable_truth_report())
                    .map_err(|e| format!("failed to serialize durable truth: {e}"))?
            );
            Ok(())
        }
        "implementation-plan" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&durable_truth::implementation_plan())
                    .map_err(|e| format!("failed to serialize implementation plan: {e}"))?
            );
            Ok(())
        }
        "proof-metrics" => {
            println!("{}", runtime::proof_metrics()?);
            Ok(())
        }
        "benchmark" => {
            println!("{}", runtime::benchmark()?);
            Ok(())
        }
        "transcript-proof" => {
            let mut file = None;
            let mut iter = args.into_iter().skip(1);
            while let Some(arg) = iter.next() {
                match arg.as_str() {
                    "--file" => {
                        file = Some(iter.next().ok_or("missing value for --file")?);
                    }
                    other => return Err(format!("unknown runtime transcript-proof flag: {other}")),
                }
            }
            let file = file.ok_or("missing --file for runtime transcript-proof")?;
            println!("{}", runtime::transcript_proof(&base.join(file))?);
            Ok(())
        }
        "migration-plan" => {
            println!("{}", runtime::migration_plan(&base)?);
            Ok(())
        }
        "list-types" => {
            println!("{}", runtime::list_types()?);
            Ok(())
        }
        "managed-resources" => {
            println!("{}", runtime::managed_resources()?);
            Ok(())
        }
        "security-policy" => {
            println!("{}", runtime::security_policy()?);
            Ok(())
        }
        "list-prompts" => {
            println!("{}", runtime::list_prompts()?);
            Ok(())
        }
        "show-prompt" => {
            let prompt_id = args.get(1).ok_or("missing prompt id")?;
            println!("{}", runtime::show_prompt(prompt_id)?);
            Ok(())
        }
        "list-resources" => {
            let mut class_filter = None;
            let mut iter = args.into_iter().skip(1);
            while let Some(arg) = iter.next() {
                match arg.as_str() {
                    "--class" => {
                        class_filter = Some(iter.next().ok_or("missing value for --class")?);
                    }
                    other => return Err(format!("unknown runtime list-resources flag: {other}")),
                }
            }
            println!("{}", runtime::list_resources(&base, class_filter.as_deref())?);
            Ok(())
        }
        "show-resource" => {
            let resource_id = args.get(1).ok_or("missing resource id")?;
            println!("{}", runtime::show_resource(&base, resource_id)?);
            Ok(())
        }
        "provenance" => {
            println!("{}", runtime::provenance(&base)?);
            Ok(())
        }
        "show-type" => {
            let runtime_type = args.get(1).ok_or("missing runtime type")?;
            println!("{}", runtime::show_type(runtime_type)?);
            Ok(())
        }
        "status" => {
            println!("{}", runtime::status(&base)?);
            Ok(())
        }
        "boot" => {
            let mut profile = core::RuntimeProfile::Auto;
            let mut port: u16 = 9944;
            let mut dry_run = false;
            let mut iter = args.into_iter().skip(1);
            while let Some(arg) = iter.next() {
                match arg.as_str() {
                    "--profile" => {
                        let value = iter.next().ok_or("missing value for --profile")?;
                        profile = core::RuntimeProfile::parse(&value)?;
                    }
                    "--port" => {
                        let value = iter.next().ok_or("missing value for --port")?;
                        port = value.parse::<u16>().map_err(|_| format!("invalid port: {value}"))?;
                    }
                    "--dry-run" => dry_run = true,
                    other => return Err(format!("unknown runtime boot flag: {other}")),
                }
            }
            runtime::boot(&base, profile, port, dry_run)
        }
        "post" => {
            let mut profile = core::RuntimeProfile::Auto;
            let mut port: u16 = 9944;
            let mut iter = args.into_iter().skip(1);
            while let Some(arg) = iter.next() {
                match arg.as_str() {
                    "--profile" => {
                        let value = iter.next().ok_or("missing value for --profile")?;
                        profile = core::RuntimeProfile::parse(&value)?;
                    }
                    "--port" => {
                        let value = iter.next().ok_or("missing value for --port")?;
                        port = value.parse::<u16>().map_err(|_| format!("invalid port: {value}"))?;
                    }
                    other => return Err(format!("unknown runtime post flag: {other}")),
                }
            }
            println!("{}", runtime::post(&base, profile, port)?);
            Ok(())
        }
        "package" => {
            let mut output = None;
            let mut iter = args.into_iter().skip(1);
            while let Some(arg) = iter.next() {
                match arg.as_str() {
                    "--output" => {
                        let value = iter.next().ok_or("missing value for --output")?;
                        output = Some(PathBuf::from(value));
                    }
                    other => return Err(format!("unknown runtime package flag: {other}")),
                }
            }
            println!("{}", runtime::package(&base, output.as_deref())?);
            Ok(())
        }
        other => Err(format!("unknown runtime action: {other}")),
    }
}

fn handle_planning(args: Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        return Err("missing planning action".into());
    }

    match args[0].as_str() {
        "status" => {
            let mut context = planning::PlanningContext::default();
            for arg in args.iter().skip(1) {
                planning::apply_planning_flag(&mut context, arg)?;
            }
            println!("{}", planning::evaluate_planning_status(context).format_default());
            Ok(())
        }
        other => Err(format!("unknown planning action: {other}")),
    }
}

fn handle_chat(args: Vec<String>, base: PathBuf) -> Result<(), String> {
    let action = args.first().map(String::as_str).unwrap_or("status");

    match action {
        "status" => {
            println!("{}", chat::status(&base)?);
            Ok(())
        }
        "queue" => {
            println!("{}", chat::queue(&base)?);
            Ok(())
        }
        "accept" => {
            let intent = args.get(1).ok_or("missing chat intent")?;
            let mut context = planning::PlanningContext::default();
            for arg in args.iter().skip(2) {
                planning::apply_planning_flag(&mut context, arg)?;
            }
            println!("{}", chat::accept(&base, intent, context)?);
            Ok(())
        }
        other => Err(format!("unknown chat action: {other}")),
    }
}

fn handle_program(args: Vec<String>, base: PathBuf) -> Result<(), String> {
    if args.is_empty() {
        return Err("missing program action".into());
    }
    match args[0].as_str() {
        "create" => {
            let goal = args.get(1).ok_or("missing program goal")?;
            println!("{}", program::create_program(&base, goal)?);
            Ok(())
        }
        "list" => {
            println!("{}", program::list_programs(&base)?);
            Ok(())
        }
        "show" => {
            let program_id = args.get(1).ok_or("missing program id")?;
            println!("{}", program::show_program(&base, program_id)?);
            Ok(())
        }
        "broad-plan" => {
            let mut program_id: Option<String> = None;
            let mut execute = false;
            let mut approve = false;
            for arg in args.iter().skip(1) {
                match arg.as_str() {
                    "--execute" => execute = true,
                    "--approve" => approve = true,
                    value if program_id.is_none() => program_id = Some(value.to_string()),
                    other => return Err(format!("unknown program broad-plan arg: {other}")),
                }
            }
            let output = if approve {
                program::broad_plan_approve_and_execute(&base, program_id.as_deref())?
            } else if execute {
                program::broad_plan_execute(&base, program_id.as_deref())?
            } else {
                program::broad_plan(&base, program_id.as_deref())?
            };
            println!("{output}");
            Ok(())
        }
        "ladder" => {
            let program_id = args.get(1).ok_or("missing program id")?;
            println!("{}", program::show_truth_ladder(&base, program_id)?);
            Ok(())
        }
        "loop" => {
            let program_id = args.get(1).ok_or("missing program id")?;
            println!("{}", program::loop_once(&base, program_id)?);
            Ok(())
        }
        "tight-loop" => {
            let program_id = args.get(1).ok_or("missing program id")?;
            println!("{}", program::tight_loop(&base, program_id)?);
            Ok(())
        }
        "while-loop" => {
            let mut program_id: Option<String> = None;
            let mut max_epochs: Option<usize> = None;
            for arg in args.iter().skip(1) {
                if let Ok(value) = arg.parse::<usize>() {
                    max_epochs = Some(value);
                } else if program_id.is_none() {
                    program_id = Some(arg.clone());
                } else {
                    return Err(format!("unknown program while-loop arg: {arg}"));
                }
            }
            println!("{}", program::while_loop(&base, program_id.as_deref(), max_epochs)?);
            Ok(())
        }
        other => Err(format!("unknown program action: {other}")),
    }
}

fn print_help() {
    println!("assistant-runtime");
    println!("  \\<os-command> [args...]");
    println!("  run <runtime-type> ...");
    println!("  run os_runtime <os-command> [args...]");
    println!("  chat status");
    println!("  chat accept <intent> [--plan] [--multi-step] [--ordering-matters] [--has-stop-conditions] [--defines-success-up-front] [--mutates-real-state] [--has-dependencies] [--needs-durable-record]");
    println!("  chat queue");
    println!("  runtime manifest");
    println!("  runtime governed-runtime");
    println!("  runtime durable-truth");
    println!("  runtime implementation-plan");
    println!("  runtime proof-metrics");
    println!("  runtime benchmark");
    println!("  runtime transcript-proof --file <path>");
    println!("  runtime migration-plan");
    println!("  runtime list-types");
    println!("  runtime managed-resources");
    println!("  runtime security-policy");
    println!("  runtime list-prompts");
    println!("  runtime show-prompt <prompt-id>");
    println!("  runtime list-resources [--class skill|tool|mcp|prompt|capability|utility|agent]");
    println!("  runtime show-resource <resource-id>");
    println!("  runtime provenance");
    println!("  runtime show-type <runtime-type>");
    println!("  runtime status");
    println!("  runtime boot [--profile auto|desktop|server] [--port <port>] [--dry-run]");
    println!("  runtime post [--profile auto|desktop|server] [--port <port>]");
    println!("  runtime package [--output <dir>]");
    println!("  planning status [--plan] [--multi-step] [--ordering-matters] [--has-stop-conditions] [--defines-success-up-front] [--mutates-real-state] [--has-dependencies] [--needs-durable-record]");
    println!("  program create <operator-intent>");
    println!("  program list");
    println!("  program show <id>");
    println!("  program broad-plan [--execute] [--approve] [<program-id>]");
    println!("  program ladder <id>");
    println!("  program loop <id>");
    println!("  program tight-loop <id>");
    println!("  program while-loop [<epochs>] [<program-id>]");
}
