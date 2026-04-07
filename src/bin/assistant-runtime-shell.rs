use serde_json::Value;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

struct Message {
    role: &'static str,
    text: String,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let runtime_bin = runtime_binary()?;
    let workspace = env::current_dir().map_err(|e| format!("failed to read current dir: {e}"))?;
    let mut history = vec![
        Message {
            role: "system",
            text: "interactive runtime shell ready".into(),
        },
        Message {
            role: "system",
            text: "type a normal message to route through chat accept and local codex".into(),
        },
        Message {
            role: "system",
            text: "commands: /status, /queue, /help, /quit".into(),
        },
        Message {
            role: "system",
            text: "type exactly skynet to trigger the shutdown path".into(),
        },
    ];

    loop {
        render(&history)?;
        print!("\nuser> ");
        io::stdout()
            .flush()
            .map_err(|e| format!("failed to flush prompt: {e}"))?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("failed to read input: {e}"))?;
        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        history.push(Message {
            role: "user",
            text: input.to_string(),
        });

        match input {
            "/quit" | "/exit" => {
                history.push(Message {
                    role: "system",
                    text: "shell exiting".into(),
                });
                render(&history)?;
                break;
            }
            "/help" => {
                history.push(Message {
                    role: "assistant",
                    text: "use /status to inspect the foreground lane, /queue to inspect the task queue, or type a normal request to route it through the runtime".into(),
                });
            }
            "/status" => {
                match run_runtime(&runtime_bin, &["chat", "status"]) {
                    Ok(output) => history.push(Message {
                        role: "assistant",
                        text: summarize_response(&output),
                    }),
                    Err(err) => history.push(Message {
                        role: "system",
                        text: format!("runtime error: {err}"),
                    }),
                }
            }
            "/queue" => {
                match run_runtime(&runtime_bin, &["chat", "queue"]) {
                    Ok(output) => history.push(Message {
                        role: "assistant",
                        text: summarize_response(&output),
                    }),
                    Err(err) => history.push(Message {
                        role: "system",
                        text: format!("runtime error: {err}"),
                    }),
                }
            }
            "skynet" => {
                match run_runtime(&runtime_bin, &["chat", "accept", "skynet"]) {
                    Ok(output) => {
                        history.push(Message {
                            role: "assistant",
                            text: summarize_response(&output),
                        });
                        history.push(Message {
                            role: "system",
                            text: "shutdown initiated; shell exiting".into(),
                        });
                        render(&history)?;
                        break;
                    }
                    Err(err) => history.push(Message {
                        role: "system",
                        text: format!("runtime error: {err}"),
                    }),
                }
            }
            _ => {
                match run_runtime(&runtime_bin, &["chat", "accept", input]) {
                    Ok(output) => {
                        history.push(Message {
                            role: "system",
                            text: summarize_response(&output),
                        });
                        match run_codex(&workspace, input) {
                            Ok(reply) => history.push(Message {
                                role: "assistant",
                                text: reply,
                            }),
                            Err(err) => history.push(Message {
                                role: "system",
                                text: format!("codex error: {err}"),
                            }),
                        }
                    }
                    Err(err) => history.push(Message {
                        role: "system",
                        text: format!("runtime error: {err}"),
                    }),
                }
            }
        }
    }

    Ok(())
}

fn runtime_binary() -> Result<PathBuf, String> {
    let current = env::current_exe().map_err(|e| format!("failed to resolve current executable: {e}"))?;
    let sibling = current
        .parent()
        .ok_or("failed to resolve binary directory")?
        .join("assistant-runtime");
    if sibling.is_file() {
        return Ok(sibling);
    }
    Err(format!(
        "missing runtime binary: {}. build once with cargo build",
        sibling.display()
    ))
}

fn run_runtime(runtime_bin: &PathBuf, args: &[&str]) -> Result<String, String> {
    let output = Command::new(runtime_bin)
        .args(args)
        .output()
        .map_err(|e| format!("failed to run {:?}: {e}", args))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn summarize_response(raw: &str) -> String {
    let Ok(json) = serde_json::from_str::<Value>(raw) else {
        return raw.to_string();
    };

    if json["schema"].as_str() == Some("assistant.runtime.chat.status.v1") {
        let status = json["chat_lane"]["status"].as_str().unwrap_or("unknown");
        return format!("foreground lane: {status}");
    }

    if json["schema"].as_str() == Some("assistant.runtime.queue_lane.tasks.v1") {
        let tasks = json["tasks"].as_array().map(|items| items.len()).unwrap_or(0);
        return format!("task queue depth: {tasks}");
    }

    if json["schema"].as_str() == Some("assistant.runtime.chat.skynet.v1") {
        let killed = json["killed_pids"].as_array().map(|items| items.len()).unwrap_or(0);
        let shutdown = json["shutdown_state"].as_str().unwrap_or("unknown");
        return format!("skynet accepted; killed {killed} process(es); state: {shutdown}");
    }

    if json["schema"].as_str() == Some("assistant.runtime.chat.accept.v1") {
        let planning = json["planning_status"]["planning_decision"]
            .as_str()
            .unwrap_or("unknown");
        let path = json["planning_status"]["execution_path"]
            .as_str()
            .unwrap_or("unknown");
        let task_id = json["queue_task"]["id"].as_str().unwrap_or("unknown");
        let handler = json["worker_handoff"]["handler_runtime"]
            .as_str()
            .unwrap_or("unknown");
        return format!(
            "[planning_decision: {planning}] [execution_path: {path}] accepted as {task_id} -> {handler}"
        );
    }

    raw.to_string()
}

fn render(history: &[Message]) -> Result<(), String> {
    print!("\x1b[2J\x1b[H");
    println!("\x1b[1;38;5;122massistant-runtime shell\x1b[0m");
    println!("\x1b[38;5;244mforeground chat shell for runtime testing\x1b[0m");
    println!();
    for line in panel("Conversation", history_lines(history), 104, "\x1b[38;5;117m") {
        println!("{line}");
    }
    io::stdout()
        .flush()
        .map_err(|e| format!("failed to flush shell render: {e}"))
}

fn run_codex(workspace: &PathBuf, prompt: &str) -> Result<String, String> {
    if let Ok(mock) = env::var("ASSISTANT_RUNTIME_CODEX_MOCK_RESPONSE") {
        return Ok(mock);
    }

    let output_path = temp_output_path("assistant-runtime-shell-codex");
    let output = Command::new("codex")
        .args([
            "exec",
            "--skip-git-repo-check",
            "--color",
            "never",
            "-C",
            workspace
                .to_str()
                .ok_or("workspace path is not valid utf-8")?,
            "-o",
            output_path
                .to_str()
                .ok_or("output path is not valid utf-8")?,
            prompt,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("failed to run local codex exec: {e}"))?;

    if !output.status.success() {
        let _ = std::fs::remove_file(&output_path);
        return Err(format!(
            "local codex exec failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let reply = std::fs::read_to_string(&output_path)
        .map_err(|e| format!("failed to read codex reply: {e}"))?;
    let _ = std::fs::remove_file(&output_path);
    let reply = reply.trim();
    if reply.is_empty() {
        return Err("local codex exec returned an empty reply".into());
    }
    Ok(reply.to_string())
}

fn temp_output_path(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or(0);
    env::temp_dir().join(format!("{prefix}-{nonce}.txt"))
}

fn history_lines(history: &[Message]) -> Vec<String> {
    history
        .iter()
        .rev()
        .take(14)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|message| format!("{}: {}", message.role, message.text))
        .collect()
}

fn panel(title: &str, body: Vec<String>, width: usize, color: &str) -> Vec<String> {
    let inner = width.saturating_sub(2);
    let mut lines = Vec::new();
    lines.push(format!("{color}┌{:─<inner$}┐\x1b[0m", "", inner = inner));
    lines.push(format!(
        "{color}│\x1b[1m{title:<inner$}\x1b[0m{color}│\x1b[0m",
        inner = inner
    ));
    lines.push(format!("{color}├{:─<inner$}┤\x1b[0m", "", inner = inner));
    for entry in body {
        let clipped = compact(&entry, inner);
        lines.push(format!(
            "{color}│\x1b[0m{clipped:<inner$}{color}│\x1b[0m",
            inner = inner
        ));
    }
    lines.push(format!("{color}└{:─<inner$}┘\x1b[0m", "", inner = inner));
    lines
}

fn compact(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_string();
    }
    let mut clipped = text.chars().take(width.saturating_sub(1)).collect::<String>();
    clipped.push('…');
    clipped
}
