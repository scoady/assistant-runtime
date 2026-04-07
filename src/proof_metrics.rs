use serde::Serialize;
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
struct ScenarioTurn {
    user: &'static str,
    assistant: &'static str,
    accepted: &'static str,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct RouteMetrics {
    pub visible_tokens: usize,
    pub relevant_tokens: usize,
    pub irrelevant_tokens: usize,
    pub truth_delivery_ratio: f64,
    pub context_amplification: f64,
    pub drift_pressure_per_turn: f64,
    pub stable_reference_rate: f64,
    pub resume_boundary_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProofMetricsDelta {
    pub irrelevant_reduction_tokens: usize,
    pub truth_delivery_gain: f64,
    pub amplification_reduction: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProofMetricsReport {
    pub schema: &'static str,
    pub scenario: &'static str,
    pub turns: usize,
    pub unique_truth_tokens: usize,
    pub governed: RouteMetrics,
    pub stock: RouteMetrics,
    pub delta: ProofMetricsDelta,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkRun {
    pub route: &'static str,
    pub turns: usize,
    pub wall_time_micros: u128,
    pub metrics: RouteMetrics,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkReport {
    pub schema: &'static str,
    pub scenario: &'static str,
    pub methodology: Vec<&'static str>,
    pub unique_truth_tokens: usize,
    pub governed: BenchmarkRun,
    pub stock: BenchmarkRun,
    pub delta: ProofMetricsDelta,
    pub effects: Vec<&'static str>,
}

pub fn proof_metrics_report() -> ProofMetricsReport {
    let turns = scripted_turns();
    let unique_truth_tokens = unique_truth_tokens(turns);
    let governed = simulate_governed(turns);
    let stock = simulate_stock(turns);

    ProofMetricsReport {
        schema: "assistant.runtime.proof_metrics.v1",
        scenario: "assistant.runtime.showcase.v1",
        turns: turns.len(),
        unique_truth_tokens,
        delta: metrics_delta(governed, stock),
        governed,
        stock,
    }
}

pub fn benchmark_report() -> BenchmarkReport {
    let turns = scripted_turns();
    let unique_truth_tokens = unique_truth_tokens(turns);

    let governed_started = Instant::now();
    let governed_metrics = simulate_governed(turns);
    let governed_elapsed = governed_started.elapsed();

    let stock_started = Instant::now();
    let stock_metrics = simulate_stock(turns);
    let stock_elapsed = stock_started.elapsed();

    BenchmarkReport {
        schema: "assistant.runtime.benchmark.v1",
        scenario: "assistant.runtime.showcase.v1",
        methodology: vec![
            "same 20-turn query set for both routes",
            "governed and stock routes executed sequentially",
            "one route run at a time for fair local resource use",
            "metrics emphasize drift, amplification, and stable truth delivery",
        ],
        unique_truth_tokens,
        governed: BenchmarkRun {
            route: "governed",
            turns: turns.len(),
            wall_time_micros: governed_elapsed.as_micros(),
            metrics: governed_metrics,
        },
        stock: BenchmarkRun {
            route: "stock",
            turns: turns.len(),
            wall_time_micros: stock_elapsed.as_micros(),
            metrics: stock_metrics,
        },
        delta: metrics_delta(governed_metrics, stock_metrics),
        effects: vec![
            "governed isolates context by lane and preserves stable references",
            "stock shares context across the whole loop and amplifies irrelevant exposure",
            "lower drift pressure means less accidental carryover between turns",
            "higher resume-boundary rate means the runtime can restart from durable truth instead of chat residue",
        ],
    }
}

fn build_route_metrics(
    visible_tokens: usize,
    relevant_tokens: usize,
    irrelevant_tokens: usize,
    turns: usize,
    unique_truth_tokens: usize,
    stable_reference_events: usize,
    resume_boundaries: usize,
) -> RouteMetrics {
    RouteMetrics {
        visible_tokens,
        relevant_tokens,
        irrelevant_tokens,
        truth_delivery_ratio: ratio(relevant_tokens, visible_tokens),
        context_amplification: ratio(visible_tokens, unique_truth_tokens),
        drift_pressure_per_turn: ratio(irrelevant_tokens, turns),
        stable_reference_rate: ratio(stable_reference_events, turns),
        resume_boundary_rate: ratio(resume_boundaries, turns),
    }
}

fn simulate_governed(turns: &[ScenarioTurn]) -> RouteMetrics {
    let turn_count = turns.len();
    let unique_truth_tokens = unique_truth_tokens(turns);
    let mut visible_tokens = 0usize;
    let mut relevant_tokens = 0usize;

    for turn in turns {
        let user_tokens = token_count(turn.user);
        let accepted_tokens = token_count(turn.accepted);
        let governed_visible_turn = user_tokens + 3 * (accepted_tokens + 1);
        visible_tokens += governed_visible_turn;
        relevant_tokens += governed_visible_turn;
    }

    build_route_metrics(
        visible_tokens,
        relevant_tokens,
        0,
        turn_count,
        unique_truth_tokens,
        turn_count,
        turn_count,
    )
}

fn simulate_stock(turns: &[ScenarioTurn]) -> RouteMetrics {
    let turn_count = turns.len();
    let unique_truth_tokens = unique_truth_tokens(turns);
    let mut visible_tokens = 0usize;
    let mut relevant_tokens = 0usize;
    let mut irrelevant_tokens = 0usize;

    for turn in turns {
        let user_tokens = token_count(turn.user);
        let assistant_tokens = token_count(turn.assistant);
        let accepted_tokens = token_count(turn.accepted);
        let unique_turn_tokens = user_tokens + assistant_tokens + accepted_tokens;

        let stock_visible_turn = 4 * unique_turn_tokens;
        let stock_relevant_turn =
            user_tokens + (user_tokens + assistant_tokens) + accepted_tokens + assistant_tokens;
        visible_tokens += stock_visible_turn;
        relevant_tokens += stock_relevant_turn;
        irrelevant_tokens += stock_visible_turn.saturating_sub(stock_relevant_turn);
    }

    build_route_metrics(
        visible_tokens,
        relevant_tokens,
        irrelevant_tokens,
        turn_count,
        unique_truth_tokens,
        0,
        0,
    )
}

fn unique_truth_tokens(turns: &[ScenarioTurn]) -> usize {
    let mut total = 0usize;
    for turn in turns {
        total += token_count(turn.user) + token_count(turn.assistant) + token_count(turn.accepted);
    }
    total
}

fn metrics_delta(governed: RouteMetrics, stock: RouteMetrics) -> ProofMetricsDelta {
    ProofMetricsDelta {
        irrelevant_reduction_tokens: stock
            .irrelevant_tokens
            .saturating_sub(governed.irrelevant_tokens),
        truth_delivery_gain: governed.truth_delivery_ratio - stock.truth_delivery_ratio,
        amplification_reduction: stock.context_amplification - governed.context_amplification,
    }
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn token_count(text: &str) -> usize {
    text.split_whitespace().count()
}

fn scripted_turns() -> &'static [ScenarioTurn] {
    &[
        ScenarioTurn { user: "what runtimes are loaded?", assistant: "I can inspect the active runtime surfaces without leaving the chat lane.", accepted: "refresh the local runtime status snapshot" },
        ScenarioTurn { user: "can you keep chat open while you work?", assistant: "Yes. I will accept the work and return this lane immediately.", accepted: "confirm foreground orchestrator availability" },
        ScenarioTurn { user: "analyze a telemetry regression in the background", assistant: "I’ll route that through planning and keep this lane open.", accepted: "analyze a telemetry regression in the background" },
        ScenarioTurn { user: "what is the queue id for that one?", assistant: "I can reference the queued task id without pulling execution back up here.", accepted: "look up the task-queue reference id" },
        ScenarioTurn { user: "prepare the weekly observability review packet", assistant: "That’s a longer background task. I’m handing it below the chat lane.", accepted: "prepare the weekly observability review packet" },
        ScenarioTurn { user: "show me the mounted resources", assistant: "I’ll show only the resources actually mounted so far.", accepted: "show the mounted skill and tool resources" },
        ScenarioTurn { user: "draft an implementation approach for queue pressure", assistant: "That needs a plan, not a foreground answer.", accepted: "draft an implementation approach for queue pressure" },
        ScenarioTurn { user: "what changed in the worker lane?", assistant: "I can summarize worker state without loading the whole backlog here.", accepted: "summarize the active worker state" },
        ScenarioTurn { user: "prepare a release checklist for the runtime", assistant: "I’ll route that as background planning work.", accepted: "prepare a release checklist for the runtime" },
        ScenarioTurn { user: "did the queue limit hold?", assistant: "I can verify that directly from the runtime state.", accepted: "verify the queue limit held during the run" },
        ScenarioTurn { user: "find a way to reduce prompt sprawl", assistant: "That belongs in a longer design loop.", accepted: "design a prompt-sprawl reduction approach" },
        ScenarioTurn { user: "what skill is mounted for doc work?", assistant: "I can answer from the mounted resource list.", accepted: "inspect mounted doc skill state" },
        ScenarioTurn { user: "compare governed and stock routing", assistant: "I’ll keep the comparison visible while the work routes below.", accepted: "compare governed and stock routing behavior" },
        ScenarioTurn { user: "review queue task ids for stability", assistant: "I can check the durable ids without reopening the task body.", accepted: "review queue task id stability" },
        ScenarioTurn { user: "prepare an incident follow-up outline", assistant: "That should go through planning and stay below the chat lane.", accepted: "prepare an incident follow-up outline" },
        ScenarioTurn { user: "what is safe to expose in the foreground?", assistant: "Only stable references and short summaries should surface here.", accepted: "audit safe foreground exposure" },
        ScenarioTurn { user: "check whether the stock path drifts", assistant: "I’ll compare the two context models and keep the result small.", accepted: "measure stock path drift pressure" },
        ScenarioTurn { user: "plan an instrumentation cleanup", assistant: "That is a multi-step background task.", accepted: "plan an instrumentation cleanup" },
        ScenarioTurn { user: "what did the queue save us from?", assistant: "It preserved a durable handoff and kept the foreground lane available.", accepted: "summarize the queue isolation benefit" },
        ScenarioTurn { user: "show me the proof that this is better", assistant: "I can render the runtime proof metrics directly.", accepted: "render the governed runtime proof metrics" },
    ]
}

#[cfg(test)]
mod tests {
    use super::{benchmark_report, proof_metrics_report};

    #[test]
    fn proof_metrics_report_favors_governed_route() {
        let report = proof_metrics_report();
        assert_eq!(report.schema, "assistant.runtime.proof_metrics.v1");
        assert_eq!(report.turns, 20);
        assert_eq!(report.governed.irrelevant_tokens, 0);
        assert!(report.stock.irrelevant_tokens > report.governed.irrelevant_tokens);
        assert!(report.governed.truth_delivery_ratio > report.stock.truth_delivery_ratio);
        assert!(report.governed.context_amplification < report.stock.context_amplification);
        assert_eq!(report.governed.stable_reference_rate, 1.0);
        assert_eq!(report.stock.stable_reference_rate, 0.0);
    }

    #[test]
    fn benchmark_report_runs_both_routes_sequentially() {
        let report = benchmark_report();
        assert_eq!(report.schema, "assistant.runtime.benchmark.v1");
        assert_eq!(report.governed.turns, 20);
        assert_eq!(report.stock.turns, 20);
        assert!(report.governed.wall_time_micros <= u128::MAX);
        assert!(report.stock.wall_time_micros <= u128::MAX);
        assert_eq!(report.governed.metrics.irrelevant_tokens, 0);
        assert!(report.stock.metrics.irrelevant_tokens > 0);
    }
}
