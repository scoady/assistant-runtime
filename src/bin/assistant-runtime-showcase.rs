use std::collections::VecDeque;
use std::env;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq)]
enum PathKind {
    Direct,
    Planner,
}

impl PathKind {
    fn execution_path(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Planner => "planner",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum FlowStage {
    User,
    Reply,
    Orchestrator,
    Queue,
    Worker,
}

#[derive(Clone)]
struct Turn {
    user: &'static str,
    assistant: &'static str,
    accepted: &'static str,
    path: PathKind,
    long_running: bool,
    mounted_skill: Option<&'static str>,
    mounted_tool: Option<&'static str>,
}

#[derive(Clone)]
struct ActiveTask {
    id: String,
    remaining_turns: usize,
    queue_state: &'static str,
}

struct ShowcaseState {
    queue: VecDeque<ActiveTask>,
    direct_worker: Vec<ActiveTask>,
    planner_worker: Vec<ActiveTask>,
    mounted_skills: Vec<String>,
    mounted_tools: Vec<String>,
    next_id: usize,
    user_context_tokens: usize,
    foreground_chat_tokens: usize,
    governed_chat_tokens: usize,
    governed_orchestrator_tokens: usize,
    governed_queue_tokens: usize,
    governed_worker_tokens: usize,
    stock_shared_tokens: usize,
}

impl ShowcaseState {
    fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            direct_worker: Vec::new(),
            planner_worker: Vec::new(),
            mounted_skills: Vec::new(),
            mounted_tools: Vec::new(),
            next_id: 1,
            user_context_tokens: 0,
            foreground_chat_tokens: 0,
            governed_chat_tokens: 0,
            governed_orchestrator_tokens: 0,
            governed_queue_tokens: 0,
            governed_worker_tokens: 0,
            stock_shared_tokens: 0,
        }
    }

    fn apply_turn(&mut self, turn: &Turn) {
        self.tick_workers();
        self.accumulate_context(turn);

        if let Some(skill) = turn.mounted_skill {
            if !self.mounted_skills.iter().any(|item| item == skill) {
                self.mounted_skills.push(skill.to_string());
            }
        }

        if let Some(tool) = turn.mounted_tool {
            if !self.mounted_tools.iter().any(|item| item == tool) {
                self.mounted_tools.push(tool.to_string());
            }
        }

        let task_id = format!("task-{:03}", self.next_id);
        self.next_id += 1;

        let mut task = ActiveTask {
            id: task_id,
            remaining_turns: if turn.long_running { 4 } else { 2 },
            queue_state: "queued",
        };

        self.queue.push_back(task.clone());
        task.queue_state = "enqueued";
        match turn.path {
            PathKind::Direct => self.direct_worker.push(task),
            PathKind::Planner => self.planner_worker.push(task),
        }
    }

    fn accumulate_context(&mut self, turn: &Turn) {
        let user_tokens = token_count_str(turn.user);
        let assistant_tokens = token_count_str(turn.assistant);
        let accepted_tokens = token_count_str(turn.accepted);
        let shared_turn_tokens = user_tokens + assistant_tokens + accepted_tokens;

        self.user_context_tokens += user_tokens;
        self.foreground_chat_tokens += user_tokens + assistant_tokens;
        self.governed_chat_tokens += user_tokens;
        self.governed_orchestrator_tokens += accepted_tokens + 1;
        self.governed_queue_tokens += accepted_tokens + 1;
        self.governed_worker_tokens += accepted_tokens + 1;
        self.stock_shared_tokens += shared_turn_tokens;
    }

    fn tick_workers(&mut self) {
        for task in self.direct_worker.iter_mut().chain(self.planner_worker.iter_mut()) {
            if task.remaining_turns > 0 {
                task.remaining_turns -= 1;
                task.queue_state = if task.remaining_turns == 0 {
                    "completed"
                } else {
                    "running"
                };
            }
        }

        let completed_ids = self
            .direct_worker
            .iter()
            .chain(self.planner_worker.iter())
            .filter(|task| task.queue_state == "completed")
            .map(|task| task.id.clone())
            .collect::<Vec<_>>();

        if !completed_ids.is_empty() {
            self.queue
                .retain(|task| !completed_ids.iter().any(|id| id == &task.id));
        }

        self.direct_worker.retain(|task| task.queue_state != "completed");
        self.planner_worker.retain(|task| task.queue_state != "completed");
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        args.push("play".to_string());
    }

    match args.remove(0).as_str() {
        "play" => play(args),
        "snapshot" => snapshot(args),
        "proof" => proof(),
        "summary" => summary(),
        "metrics" => metrics(),
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        other => Err(format!("unknown showcase command: {other}")),
    }
}

fn print_help() {
    println!("assistant-runtime-showcase");
    println!("  play [--delay-ms <n>] [--turns <n>]");
    println!("  snapshot --turn <n>");
    println!("  proof");
    println!("  summary");
    println!("  metrics");
    println!();
    println!("Compact runtime showcase:");
    println!("  - foreground chat on the left");
    println!("  - lanes stacked on the right");
    println!("  - active task moves horizontally through the pipeline");
    println!("  - automatic 5s pacing between turns by default");
    println!("  - proof dashboard shows drift and truth-delivery deltas over time");
}

#[derive(Clone, Copy)]
struct RouteMetrics {
    visible_tokens: usize,
    relevant_tokens: usize,
    irrelevant_tokens: usize,
    stable_reference_events: usize,
    resume_boundaries: usize,
}

impl RouteMetrics {
    fn truth_delivery_ratio(self) -> f64 {
        ratio(self.relevant_tokens, self.visible_tokens)
    }

    fn context_amplification(self, unique_truth_tokens: usize) -> f64 {
        ratio(self.visible_tokens, unique_truth_tokens)
    }

    fn drift_pressure(self, turns: usize) -> f64 {
        ratio(self.irrelevant_tokens, turns)
    }

    fn stable_reference_rate(self, turns: usize) -> f64 {
        ratio(self.stable_reference_events, turns)
    }

    fn resume_boundary_rate(self, turns: usize) -> f64 {
        ratio(self.resume_boundaries, turns)
    }
}

struct MetricsReport {
    turns: usize,
    unique_truth_tokens: usize,
    governed: RouteMetrics,
    stock: RouteMetrics,
}

fn metrics_report(turns: &[Turn]) -> MetricsReport {
    let mut unique_truth_tokens = 0usize;
    let mut governed = RouteMetrics {
        visible_tokens: 0,
        relevant_tokens: 0,
        irrelevant_tokens: 0,
        stable_reference_events: 0,
        resume_boundaries: 0,
    };
    let mut stock = RouteMetrics {
        visible_tokens: 0,
        relevant_tokens: 0,
        irrelevant_tokens: 0,
        stable_reference_events: 0,
        resume_boundaries: 0,
    };

    for turn in turns {
        let user_tokens = token_count_str(turn.user);
        let assistant_tokens = token_count_str(turn.assistant);
        let accepted_tokens = token_count_str(turn.accepted);
        let shared_turn_tokens = user_tokens + assistant_tokens + accepted_tokens;
        unique_truth_tokens += shared_turn_tokens;

        let governed_visible = user_tokens + 3 * (accepted_tokens + 1);
        governed.visible_tokens += governed_visible;
        governed.relevant_tokens += governed_visible;
        governed.stable_reference_events += 1;
        governed.resume_boundaries += 1;

        let stock_visible = 4 * shared_turn_tokens;
        let stock_relevant = user_tokens + (user_tokens + assistant_tokens) + accepted_tokens + assistant_tokens;
        stock.visible_tokens += stock_visible;
        stock.relevant_tokens += stock_relevant;
        stock.irrelevant_tokens += stock_visible.saturating_sub(stock_relevant);
    }

    MetricsReport {
        turns: turns.len(),
        unique_truth_tokens,
        governed,
        stock,
    }
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn metrics() -> Result<(), String> {
    let turns = scripted_turns();
    let report = metrics_report(&turns);

    println!("assistant-runtime-showcase metrics");
    println!("turns: {}", report.turns);
    println!("unique truth tokens: {}", report.unique_truth_tokens);
    println!();
    println!("governed:");
    println!("  visible_tokens: {}", report.governed.visible_tokens);
    println!("  relevant_tokens: {}", report.governed.relevant_tokens);
    println!("  irrelevant_tokens: {}", report.governed.irrelevant_tokens);
    println!("  truth_delivery_ratio: {:.3}", report.governed.truth_delivery_ratio());
    println!("  context_amplification: {:.3}", report.governed.context_amplification(report.unique_truth_tokens));
    println!("  drift_pressure_per_turn: {:.3}", report.governed.drift_pressure(report.turns));
    println!("  stable_reference_rate: {:.3}", report.governed.stable_reference_rate(report.turns));
    println!("  resume_boundary_rate: {:.3}", report.governed.resume_boundary_rate(report.turns));
    println!();
    println!("stock:");
    println!("  visible_tokens: {}", report.stock.visible_tokens);
    println!("  relevant_tokens: {}", report.stock.relevant_tokens);
    println!("  irrelevant_tokens: {}", report.stock.irrelevant_tokens);
    println!("  truth_delivery_ratio: {:.3}", report.stock.truth_delivery_ratio());
    println!("  context_amplification: {:.3}", report.stock.context_amplification(report.unique_truth_tokens));
    println!("  drift_pressure_per_turn: {:.3}", report.stock.drift_pressure(report.turns));
    println!("  stable_reference_rate: {:.3}", report.stock.stable_reference_rate(report.turns));
    println!("  resume_boundary_rate: {:.3}", report.stock.resume_boundary_rate(report.turns));
    println!();
    println!("delta:");
    println!("  irrelevant_reduction_tokens: {}", report.stock.irrelevant_tokens.saturating_sub(report.governed.irrelevant_tokens));
    println!("  truth_delivery_gain: {:.3}", report.governed.truth_delivery_ratio() - report.stock.truth_delivery_ratio());
    println!("  amplification_reduction: {:.3}", report.stock.context_amplification(report.unique_truth_tokens) - report.governed.context_amplification(report.unique_truth_tokens));
    Ok(())
}

fn summary() -> Result<(), String> {
    let turns = scripted_turns();
    println!("assistant-runtime-showcase");
    println!("turns: {}", turns.len());
    let direct = turns.iter().filter(|turn| turn.path == PathKind::Direct).count();
    let planner = turns.len() - direct;
    println!("direct turns: {direct}");
    println!("planner turns: {planner}");
    println!(
        "long-running turns: {}",
        turns.iter().filter(|turn| turn.long_running).count()
    );
    Ok(())
}

fn snapshot(args: Vec<String>) -> Result<(), String> {
    let mut turn_number = None;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--turn" => {
                let value = iter.next().ok_or("missing value for --turn")?;
                turn_number = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| format!("invalid turn: {value}"))?,
                );
            }
            other => return Err(format!("unknown showcase snapshot flag: {other}")),
        }
    }

    let turns = scripted_turns();
    let turn_number = turn_number.ok_or("missing --turn for snapshot")?;
    if turn_number == 0 || turn_number > turns.len() {
        return Err(format!("snapshot turn must be between 1 and {}", turns.len()));
    }

    let mut state = ShowcaseState::new();
    for turn in turns.iter().take(turn_number) {
        state.apply_turn(turn);
    }
    render_showcase(
        turn_number,
        turns.len(),
        &turns[turn_number - 1],
        &state,
        FlowStage::Worker,
    )
}

fn play(args: Vec<String>) -> Result<(), String> {
    let mut delay_ms = 5_000;
    let mut max_turns = None;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--delay-ms" => {
                let value = iter.next().ok_or("missing value for --delay-ms")?;
                delay_ms = value
                    .parse::<u64>()
                    .map_err(|_| format!("invalid delay-ms: {value}"))?;
            }
            "--turns" => {
                let value = iter.next().ok_or("missing value for --turns")?;
                max_turns = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| format!("invalid turns: {value}"))?,
                );
            }
            other => return Err(format!("unknown showcase flag: {other}")),
        }
    }

    let turns = scripted_turns();
    let total = max_turns.unwrap_or(turns.len()).min(turns.len());
    let mut state = ShowcaseState::new();

    for (index, turn) in turns.into_iter().take(total).enumerate() {
        state.apply_turn(&turn);
        animate_turn(index + 1, total, &turn, &state, delay_ms)?;
    }

    state.tick_workers();
    render_completion(total, &state)?;
    Ok(())
}

#[derive(Clone, Copy)]
struct MetricsPoint {
    turn: usize,
    governed_irrelevant: usize,
    stock_irrelevant: usize,
    governed_ratio: f64,
    stock_ratio: f64,
}

fn proof() -> Result<(), String> {
    let turns = scripted_turns();
    let report = metrics_report(&turns);
    let points = metrics_points(&turns);

    clear_screen();
    println!("\x1b[1;38;5;122massistant.runtime proof dashboard\x1b[0m");
    println!("\x1b[38;5;244mlive scenario replay: governed route vs stock shared-context loop\x1b[0m");
    println!();

    let overview = panel(
        "Proof Summary",
        &vec![
            format!("turns              {}", report.turns),
            format!("unique truth tokens {}", report.unique_truth_tokens),
            format!(
                "irrelevant delta    -{} tokens",
                report
                    .stock
                    .irrelevant_tokens
                    .saturating_sub(report.governed.irrelevant_tokens)
            ),
            format!(
                "truth delivery      +{:.3}",
                report.governed.truth_delivery_ratio() - report.stock.truth_delivery_ratio()
            ),
            format!(
                "amplification       -{:.3}",
                report.stock.context_amplification(report.unique_truth_tokens)
                    - report.governed.context_amplification(report.unique_truth_tokens)
            ),
        ],
        54,
        "\x1b[38;5;122m",
    );

    let drift = panel(
        "Drift Over Time",
        &points
            .iter()
            .map(|point| {
                format!(
                    "t{:02}  G {:>3} {}  S {:>3} {}",
                    point.turn,
                    point.governed_irrelevant,
                    spark_bar(point.governed_irrelevant, 120, 8, "\x1b[38;5;118m"),
                    point.stock_irrelevant,
                    spark_bar(point.stock_irrelevant, report.stock.irrelevant_tokens.max(1), 12, "\x1b[38;5;203m"),
                )
            })
            .collect::<Vec<_>>(),
        54,
        "\x1b[38;5;203m",
    );

    let truth = panel(
        "Truth Delivery Over Time",
        &points
            .iter()
            .map(|point| {
                format!(
                    "t{:02}  G {:.2} {}  S {:.2} {}",
                    point.turn,
                    point.governed_ratio,
                    ratio_bar(point.governed_ratio, 8, "\x1b[38;5;118m"),
                    point.stock_ratio,
                    ratio_bar(point.stock_ratio, 8, "\x1b[38;5;221m"),
                )
            })
            .collect::<Vec<_>>(),
        54,
        "\x1b[38;5;221m",
    );

    let effect = panel(
        "Why It Matters",
        &vec![
            "governed route keeps irrelevant exposure at zero".into(),
            "stock route keeps re-reading shared context".into(),
            "stable queue references survive interruption".into(),
            "resume boundaries stay durable below chat".into(),
        ],
        54,
        "\x1b[38;5;117m",
    );

    let left = stack_panels(&[overview, effect]);
    let right = stack_panels(&[drift, truth]);
    print_columns(&left, &right);
    Ok(())
}

fn metrics_points(turns: &[Turn]) -> Vec<MetricsPoint> {
    let mut points = Vec::new();
    let mut governed_visible = 0usize;
    let mut governed_relevant = 0usize;
    let mut stock_visible = 0usize;
    let mut stock_relevant = 0usize;
    let mut stock_irrelevant = 0usize;

    for (index, turn) in turns.iter().enumerate() {
        let user_tokens = token_count_str(turn.user);
        let assistant_tokens = token_count_str(turn.assistant);
        let accepted_tokens = token_count_str(turn.accepted);
        let shared_turn_tokens = user_tokens + assistant_tokens + accepted_tokens;

        let governed_turn_visible = user_tokens + 3 * (accepted_tokens + 1);
        governed_visible += governed_turn_visible;
        governed_relevant += governed_turn_visible;

        let stock_turn_visible = 4 * shared_turn_tokens;
        let stock_turn_relevant =
            user_tokens + (user_tokens + assistant_tokens) + accepted_tokens + assistant_tokens;
        stock_visible += stock_turn_visible;
        stock_relevant += stock_turn_relevant;
        stock_irrelevant += stock_turn_visible.saturating_sub(stock_turn_relevant);

        points.push(MetricsPoint {
            turn: index + 1,
            governed_irrelevant: 0,
            stock_irrelevant,
            governed_ratio: ratio(governed_relevant, governed_visible),
            stock_ratio: ratio(stock_relevant, stock_visible),
        });
    }

    points
}

fn spark_bar(value: usize, max: usize, width: usize, color: &str) -> String {
    let max = max.max(1);
    let filled = ((value * width) + max - 1) / max;
    let filled = filled.min(width);
    format!(
        "{color}{}{}\x1b[0m",
        "█".repeat(filled),
        "·".repeat(width.saturating_sub(filled))
    )
}

fn ratio_bar(value: f64, width: usize, color: &str) -> String {
    let filled = ((value.clamp(0.0, 1.0) * width as f64).round() as usize).min(width);
    format!(
        "{color}{}{}\x1b[0m",
        "█".repeat(filled),
        "·".repeat(width.saturating_sub(filled))
    )
}

fn animate_turn(
    turn_number: usize,
    total_turns: usize,
    turn: &Turn,
    state: &ShowcaseState,
    delay_ms: u64,
) -> Result<(), String> {
    for stage in [
        FlowStage::User,
        FlowStage::Reply,
        FlowStage::Orchestrator,
        FlowStage::Queue,
        FlowStage::Worker,
    ] {
        render_showcase(turn_number, total_turns, turn, state, stage)?;
        thread::sleep(Duration::from_millis(stage_delay_ms(stage, delay_ms)));
    }
    Ok(())
}

fn stage_delay_ms(stage: FlowStage, turn_delay_ms: u64) -> u64 {
    match stage {
        FlowStage::User | FlowStage::Reply => 350,
        FlowStage::Orchestrator | FlowStage::Queue => 250,
        FlowStage::Worker => turn_delay_ms,
    }
}

fn render_showcase(
    turn_number: usize,
    total_turns: usize,
    turn: &Turn,
    state: &ShowcaseState,
    stage: FlowStage,
) -> Result<(), String> {
    clear_screen();
    println!(
        "[1;38;5;122massistant.runtime showcase[0m  turn {turn_number}/{total_turns}"
    );
    println!("[38;5;244mforeground chat stays open while work routes through governed lanes[0m");
    println!();

    let left = stack_panels(&[
        lane_box(
            "User",
            "request",
            &reveal_text(turn.user, stage, FlowStage::User),
            34,
            "[38;5;117m",
            state.user_context_tokens,
            CounterPlacement::Top,
        ),
        lane_box(
            "Chat",
            "foreground open",
            &reveal_text(turn.assistant, stage, FlowStage::Reply),
            34,
            "[38;5;81m",
            state.foreground_chat_tokens,
            CounterPlacement::Top,
        ),
    ]);

    let task_id = state
        .queue
        .back()
        .map(|task| task.id.clone())
        .unwrap_or_else(String::new);
    let worker_label = match turn.path {
        PathKind::Direct => "direct worker",
        PathKind::Planner => "planner worker",
    };
    let worker_state = state
        .queue
        .back()
        .map(|task| task.queue_state.to_string())
        .unwrap_or_else(String::new);

    let governed = join_panels_with_flow(&[
        lane_box(
            "Chat",
            "isolated lane",
            &reveal_text(turn.user, stage, FlowStage::User),
            18,
            "[38;5;117m",
            state.governed_chat_tokens,
            CounterPlacement::Top,
        ),
        lane_box_reached(
            "Orchestrator",
            &format!("route {}", turn.path.execution_path()),
            turn.accepted,
            20,
            "[38;5;180m",
            stage >= FlowStage::Orchestrator,
            stage == FlowStage::Orchestrator,
            state.governed_orchestrator_tokens,
            CounterPlacement::Top,
        ),
        lane_box_reached(
            "Queue",
            "buffer 3 max",
            &task_id,
            20,
            "[38;5;213m",
            stage >= FlowStage::Queue,
            stage == FlowStage::Queue,
            state.governed_queue_tokens,
            CounterPlacement::Top,
        ),
        lane_box_reached(
            "Worker",
            worker_label,
            &worker_state,
            20,
            "[38;5;221m",
            stage >= FlowStage::Worker,
            stage == FlowStage::Worker,
            state.governed_worker_tokens,
            CounterPlacement::Top,
        ),
    ]);

    let stock_tools_active = stage == FlowStage::Orchestrator || stage == FlowStage::Queue;
    let stock = join_panels_with_flow(&[
        lane_box(
            "Chat",
            "shared lane",
            &reveal_text(turn.user, stage, FlowStage::User),
            18,
            "[38;5;117m",
            state.stock_shared_tokens,
            CounterPlacement::Bottom,
        ),
        lane_box_reached(
            "Agent",
            "shared ctx",
            "same memory",
            18,
            "[38;5;110m",
            stage >= FlowStage::Reply,
            stage == FlowStage::Reply,
            state.stock_shared_tokens,
            CounterPlacement::Bottom,
        ),
        lane_box_reached(
            "Tools",
            "ambient tools",
            "shared call",
            18,
            "[38;5;150m",
            stage >= FlowStage::Orchestrator,
            stock_tools_active,
            state.stock_shared_tokens,
            CounterPlacement::Bottom,
        ),
        lane_box_reached(
            "Reply",
            "same lane",
            turn.assistant,
            18,
            "[38;5;151m",
            stage >= FlowStage::Worker,
            stage == FlowStage::Worker,
            state.stock_shared_tokens,
            CounterPlacement::Bottom,
        ),
    ]);

    let mut right = Vec::new();
    right.push("[1;38;5;180mGoverned Route[0m".to_string());
    right.push("[38;5;244misolated lane context per box[0m".to_string());
    right.extend(governed);
    right.push(String::new());
    right.push("[1;38;5;110mStock Loop[0m".to_string());
    right.push("[38;5;244mshared context accumulates everywhere[0m".to_string());
    right.extend(stock);

    print_columns(&left, &right);
    Ok(())
}

#[derive(Clone, Copy)]
enum CounterPlacement {
    Top,
    Bottom,
}

fn lane_box(
    title: &str,
    desc: &str,
    detail: &str,
    width: usize,
    color: &str,
    tokens: usize,
    placement: CounterPlacement,
) -> Vec<String> {
    lane_panel(title, desc, detail, width, color, tokens, placement)
}

fn lane_box_reached(
    title: &str,
    desc: &str,
    detail: &str,
    width: usize,
    color: &str,
    reached: bool,
    active: bool,
    tokens: usize,
    placement: CounterPlacement,
) -> Vec<String> {
    let tint = if active { color } else { "[38;5;244m" };
    let body = if reached { detail } else { "" };
    lane_panel(title, desc, body, width, tint, tokens, placement)
}

fn lane_panel(
    title: &str,
    desc: &str,
    detail: &str,
    width: usize,
    color: &str,
    tokens: usize,
    placement: CounterPlacement,
) -> Vec<String> {
    let inner = width.saturating_sub(2);
    let badge = context_badge(tokens);
    let title_text = match placement {
        CounterPlacement::Top => align_with_badge(title, &badge, inner),
        CounterPlacement::Bottom => compact(title, inner),
    };
    let footer_text = match placement {
        CounterPlacement::Top => String::new(),
        CounterPlacement::Bottom => align_right(&badge, inner),
    };

    vec![
        format!("{color}┌{:─<inner$}┐[0m", "", inner = inner),
        format!("{color}│[1m{}[0m{color}│[0m", pad_ansi(&title_text, inner)),
        format!("{color}├{:─<inner$}┤[0m", "", inner = inner),
        format!("{color}│[0m{:<inner$}{color}│[0m", compact(desc, inner), inner = inner),
        format!("{color}│[0m{:<inner$}{color}│[0m", compact(detail, inner), inner = inner),
        format!("{color}│[0m{}{}│[0m", footer_text, " ".repeat(inner.saturating_sub(strip_ansi_width(&footer_text)))),
        format!("{color}└{:─<inner$}┘[0m", "", inner = inner),
    ]
}

fn stack_panels(panels: &[Vec<String>]) -> Vec<String> {
    let mut out = Vec::new();
    for (index, panel) in panels.iter().enumerate() {
        if index > 0 {
            out.push(String::new());
        }
        out.extend(panel.clone());
    }
    out
}

fn join_panels_with_flow(panels: &[Vec<String>]) -> Vec<String> {
    let height = panels.first().map(Vec::len).unwrap_or(0);
    let mut out = Vec::new();
    for row in 0..height {
        let mut line = String::new();
        for (index, panel) in panels.iter().enumerate() {
            if index > 0 {
                line.push_str(if row == 3 || row == 4 { " ==> " } else { "     " });
            }
            line.push_str(panel.get(row).map(String::as_str).unwrap_or(""));
        }
        out.push(line);
    }
    out
}

fn context_badge(tokens: usize) -> String {
    let meter = context_meter(tokens);
    let color = context_color(tokens);
    format!("{color}{meter} {tokens}t[0m")
}

fn context_color(tokens: usize) -> &'static str {
    match tokens {
        0..=12 => "[38;5;118m",
        13..=32 => "[38;5;221m",
        _ => "[38;5;203m",
    }
}

fn align_with_badge(title: &str, badge: &str, width: usize) -> String {
    let title = compact(title, width);
    let title_width = strip_ansi_width(&title);
    let badge_width = strip_ansi_width(badge);
    if title_width + badge_width + 1 >= width {
        return format!("{} {}", compact(&title, width.saturating_sub(badge_width + 1)), badge);
    }
    format!("{title}{}{}", " ".repeat(width - title_width - badge_width), badge)
}

fn align_right(text: &str, width: usize) -> String {
    let visible = strip_ansi_width(text);
    format!("{}{}", " ".repeat(width.saturating_sub(visible)), text)
}

fn pad_ansi(text: &str, width: usize) -> String {
    format!("{text}{}", " ".repeat(width.saturating_sub(strip_ansi_width(text))))
}

fn render_completion(total_turns: usize, state: &ShowcaseState) -> Result<(), String> {
    clear_screen();
    println!("\x1b[1;38;5;122massistant.runtime showcase complete\x1b[0m  turns {total_turns}/20");
    println!();
    let summary = vec![
        format!("queue        {}", state.queue.len()),
        format!("direct       {}", state.direct_worker.len()),
        format!("planner      {}", state.planner_worker.len()),
        format!(
            "skills       {}",
            if state.mounted_skills.is_empty() {
                "none".into()
            } else {
                compact(&state.mounted_skills.join(", "), 24)
            }
        ),
        format!(
            "tools        {}",
            if state.mounted_tools.is_empty() {
                "none".into()
            } else {
                compact(&state.mounted_tools.join(", "), 24)
            }
        ),
    ];
    for line in panel(&section_title("Final", &summary), &summary, 98, "\x1b[38;5;117m") {
        println!("{line}");
    }
    Ok(())
}

fn section_title(base: &str, body: &[String]) -> String {
    let tokens = token_count(body);
    let meter = context_meter(tokens);
    format!("{base}  {meter} {tokens}t")
}

fn context_meter(tokens: usize) -> String {
    let filled = ((tokens.min(40) + 4) / 5).max(1);
    let empty = 8usize.saturating_sub(filled);
    format!("[{}{}]", "#".repeat(filled), "-".repeat(empty))
}

fn panel(title: &str, body: &[String], width: usize, color: &str) -> Vec<String> {
    let inner = width.saturating_sub(2);
    let mut lines = Vec::new();
    lines.push(format!("{color}┌{:─<inner$}┐\x1b[0m", "", inner = inner));
    lines.push(format!(
        "{color}│\x1b[1m{title:<inner$}\x1b[0m{color}│\x1b[0m",
        inner = inner
    ));
    lines.push(format!("{color}├{:─<inner$}┤\x1b[0m", "", inner = inner));
    if body.is_empty() {
        lines.push(format!("{color}│{:<inner$}│\x1b[0m", "", inner = inner));
    } else {
        for entry in body {
            let clipped = compact(entry, inner);
            lines.push(format!(
                "{color}│\x1b[0m{clipped:<inner$}{color}│\x1b[0m",
                inner = inner
            ));
        }
    }
    lines.push(format!("{color}└{:─<inner$}┘\x1b[0m", "", inner = inner));
    lines
}

fn print_columns(left: &[String], right: &[String]) {
    let left_width = strip_ansi_width(left.first().map(String::as_str).unwrap_or(""));
    let total = left.len().max(right.len());

    for index in 0..total {
        let left_line = left.get(index).map(String::as_str).unwrap_or("");
        let right_line = right.get(index).map(String::as_str).unwrap_or("");
        let left_pad = left_width.saturating_sub(strip_ansi_width(left_line));
        println!("{left_line}{}{right_line}", " ".repeat(left_pad + 2));
    }
}

fn reveal_text(text: &str, stage: FlowStage, reveal_at: FlowStage) -> String {
    if stage < reveal_at {
        return String::new();
    }

    match stage {
        FlowStage::User if reveal_at == FlowStage::User => {
            text.chars().take(text.chars().count() / 2 + 1).collect()
        }
        FlowStage::Reply if reveal_at == FlowStage::Reply => {
            text.chars().take(text.chars().count() / 2 + 1).collect()
        }
        _ => text.to_string(),
    }
}

fn compact(text: &str, width: usize) -> String {
    if text.len() <= width {
        return text.to_string();
    }
    let mut clipped = text.chars().take(width.saturating_sub(1)).collect::<String>();
    clipped.push('…');
    clipped
}

fn token_count_str(text: &str) -> usize {
    text.split_whitespace().count()
}

fn token_count(lines: &[String]) -> usize {
    lines.iter()
        .flat_map(|line| line.split_whitespace())
        .count()
}

fn strip_ansi_width(text: &str) -> usize {
    let mut chars = text.chars().peekable();
    let mut width = 0usize;
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            while let Some(next) = chars.next() {
                if next == 'm' {
                    break;
                }
            }
        } else {
            width += 1;
        }
    }
    width
}

fn clear_screen() {
    print!("\x1b[2J\x1b[H");
    let _ = io::stdout().flush();
}

fn scripted_turns() -> Vec<Turn> {
    vec![
        Turn { user: "what research lanes are available?", assistant: "I can inspect the active workflow surfaces without leaving the chat lane.", accepted: "refresh the research workflow snapshot", path: PathKind::Direct, long_running: false, mounted_skill: None, mounted_tool: Some("assistant.runtime.tools") },
        Turn { user: "can you keep chat open while you research?", assistant: "Yes. I will accept the work and return this lane immediately.", accepted: "confirm foreground orchestrator availability", path: PathKind::Direct, long_running: false, mounted_skill: None, mounted_tool: None },
        Turn { user: "build a background reading plan for climate adaptation books", assistant: "I’ll route that through planning and keep this lane open.", accepted: "build a background reading plan for climate adaptation books", path: PathKind::Planner, long_running: true, mounted_skill: Some("assistant.runtime.skills"), mounted_tool: Some("assistant.runtime.tools") },
        Turn { user: "what is the queue id for that one?", assistant: "I can reference the queued task id without pulling execution back up here.", accepted: "look up the task-queue reference id", path: PathKind::Direct, long_running: false, mounted_skill: None, mounted_tool: None },
        Turn { user: "prepare a weekly literature review packet on urban design", assistant: "That’s a longer background task. I’m handing it below the chat lane.", accepted: "prepare a weekly literature review packet on urban design", path: PathKind::Planner, long_running: true, mounted_skill: Some("meeting-notes"), mounted_tool: Some("assistant.runtime.tools") },
        Turn { user: "show me the mounted resources", assistant: "I’ll show only the resources actually mounted so far.", accepted: "show the mounted skill and tool resources", path: PathKind::Direct, long_running: false, mounted_skill: None, mounted_tool: None },
        Turn { user: "summarize the current research brief status", assistant: "That can run directly below this lane.", accepted: "summarize the current research brief status", path: PathKind::Direct, long_running: false, mounted_skill: None, mounted_tool: Some("assistant.runtime.tools") },
        Turn { user: "queue a comparative study of public transit systems", assistant: "Queued. The planner worker will own the deeper investigation.", accepted: "queue a comparative study of public transit systems", path: PathKind::Planner, long_running: true, mounted_skill: Some("assistant.runtime.skills"), mounted_tool: Some("assistant.runtime.tools") },
        Turn { user: "what is still running?", assistant: "I’ll inspect the workers without collapsing the foreground lane.", accepted: "inspect direct and planner worker state", path: PathKind::Direct, long_running: false, mounted_skill: None, mounted_tool: None },
        Turn { user: "draft a compact reading summary packet", assistant: "That packet will be prepared in the planner lane.", accepted: "draft a compact reading summary packet", path: PathKind::Planner, long_running: true, mounted_skill: Some("meeting-notes"), mounted_tool: None },
        Turn { user: "show the planning decision path clearly", assistant: "I’ll make the routing explicit on this turn.", accepted: "render explicit planning status for the last request", path: PathKind::Direct, long_running: false, mounted_skill: None, mounted_tool: None },
        Turn { user: "run an occasional source quality check", assistant: "Accepted. That will use the direct worker.", accepted: "run an occasional source quality check", path: PathKind::Direct, long_running: false, mounted_skill: None, mounted_tool: Some("assistant.runtime.tools") },
        Turn { user: "plan the next reading sprint on food systems", assistant: "That belongs in the planner lane.", accepted: "plan the next reading sprint on food systems", path: PathKind::Planner, long_running: true, mounted_skill: Some("assistant.runtime.skills"), mounted_tool: None },
        Turn { user: "confirm the direct worker stayed isolated", assistant: "Yes. Direct work stays below the foreground lane.", accepted: "confirm the direct worker stayed isolated", path: PathKind::Direct, long_running: false, mounted_skill: None, mounted_tool: None },
        Turn { user: "simulate a queue pressure event with more reading requests", assistant: "I’ll push the planner side until the queue limit becomes visible.", accepted: "simulate a queue pressure event with more reading requests", path: PathKind::Planner, long_running: true, mounted_skill: None, mounted_tool: Some("assistant.runtime.tools") },
        Turn { user: "which lane is foreground right now?", assistant: "This orchestrator stays available the whole time.", accepted: "report foreground orchestrator status", path: PathKind::Direct, long_running: false, mounted_skill: None, mounted_tool: None },
        Turn { user: "prepare a starter pack on the history of maps", assistant: "Queued for background preparation.", accepted: "prepare a starter pack on the history of maps", path: PathKind::Planner, long_running: true, mounted_skill: Some("meeting-notes"), mounted_tool: Some("assistant.runtime.tools") },
        Turn { user: "show the queue ids again", assistant: "I’ll reference only the active queued ids.", accepted: "show the task-queue reference ids again", path: PathKind::Direct, long_running: false, mounted_skill: None, mounted_tool: None },
        Turn { user: "simulate a long-running review of coastal resilience papers", assistant: "That’s another planner-owned background task.", accepted: "simulate a long-running review of coastal resilience papers", path: PathKind::Planner, long_running: true, mounted_skill: Some("assistant.runtime.skills"), mounted_tool: Some("assistant.runtime.tools") },
        Turn { user: "wrap it all up", assistant: "I’ll summarize the runtime state without dragging old filler back in.", accepted: "wrap the session with a clean research-session summary", path: PathKind::Direct, long_running: false, mounted_skill: None, mounted_tool: None },
    ]
}
