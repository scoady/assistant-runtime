#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanningDecision {
    Yes,
    No,
}

impl PlanningDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Yes => "yes",
            Self::No => "no",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionPath {
    Planner,
    Direct,
}

impl ExecutionPath {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Planner => "planner",
            Self::Direct => "direct",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanningStatus {
    pub planning_decision: PlanningDecision,
    pub execution_path: ExecutionPath,
}

impl PlanningStatus {
    pub fn format_default(self) -> String {
        format!(
            "[planning_decision: {}] [execution_path: {}]",
            self.planning_decision.as_str(),
            self.execution_path.as_str()
        )
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlanningContext {
    pub explicit_plan: bool,
    pub multi_step: bool,
    pub ordering_matters: bool,
    pub has_stop_conditions: bool,
    pub defines_success_up_front: bool,
    pub mutates_real_state: bool,
    pub has_dependencies: bool,
    pub needs_durable_record: bool,
}

pub fn evaluate_planning_decision(context: PlanningContext) -> PlanningDecision {
    if context.explicit_plan
        || context.multi_step
        || context.ordering_matters
        || context.has_stop_conditions
        || context.defines_success_up_front
        || context.mutates_real_state
        || context.has_dependencies
        || context.needs_durable_record
    {
        PlanningDecision::Yes
    } else {
        PlanningDecision::No
    }
}

pub fn evaluate_planning_status(context: PlanningContext) -> PlanningStatus {
    let planning_decision = evaluate_planning_decision(context);
    let execution_path = match planning_decision {
        PlanningDecision::Yes => ExecutionPath::Planner,
        PlanningDecision::No => ExecutionPath::Direct,
    };
    PlanningStatus {
        planning_decision,
        execution_path,
    }
}

pub fn apply_planning_flag(context: &mut PlanningContext, flag: &str) -> Result<(), String> {
    match flag {
        "--plan" => context.explicit_plan = true,
        "--multi-step" => context.multi_step = true,
        "--ordering-matters" => context.ordering_matters = true,
        "--has-stop-conditions" => context.has_stop_conditions = true,
        "--defines-success-up-front" => context.defines_success_up_front = true,
        "--mutates-real-state" => context.mutates_real_state = true,
        "--has-dependencies" => context.has_dependencies = true,
        "--needs-durable-record" => context.needs_durable_record = true,
        other => return Err(format!("unknown planning status flag: {other}")),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        evaluate_planning_decision, evaluate_planning_status, ExecutionPath, PlanningContext,
        PlanningDecision,
    };

    #[test]
    fn planning_decision_exercises_every_state() {
        let no = evaluate_planning_decision(PlanningContext::default());
        assert_eq!(no, PlanningDecision::No);

        let yes_cases = [
            PlanningContext {
                explicit_plan: true,
                ..PlanningContext::default()
            },
            PlanningContext {
                multi_step: true,
                ..PlanningContext::default()
            },
            PlanningContext {
                ordering_matters: true,
                ..PlanningContext::default()
            },
            PlanningContext {
                has_stop_conditions: true,
                ..PlanningContext::default()
            },
            PlanningContext {
                defines_success_up_front: true,
                ..PlanningContext::default()
            },
            PlanningContext {
                mutates_real_state: true,
                ..PlanningContext::default()
            },
            PlanningContext {
                has_dependencies: true,
                ..PlanningContext::default()
            },
            PlanningContext {
                needs_durable_record: true,
                ..PlanningContext::default()
            },
        ];

        for context in yes_cases {
            let decision = evaluate_planning_decision(context);
            assert_eq!(decision, PlanningDecision::Yes);
        }
    }

    #[test]
    fn default_status_format_uses_execution_path_language() {
        let direct = evaluate_planning_status(PlanningContext::default());
        assert_eq!(direct.planning_decision, PlanningDecision::No);
        assert_eq!(direct.execution_path, ExecutionPath::Direct);
        assert_eq!(
            direct.format_default(),
            "[planning_decision: no] [execution_path: direct]"
        );

        let planner = evaluate_planning_status(PlanningContext {
            explicit_plan: true,
            ..PlanningContext::default()
        });
        assert_eq!(planner.planning_decision, PlanningDecision::Yes);
        assert_eq!(planner.execution_path, ExecutionPath::Planner);
        assert_eq!(
            planner.format_default(),
            "[planning_decision: yes] [execution_path: planner]"
        );
    }
}
