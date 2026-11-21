use crate::types::{AppResult, ExecutionPlan, IntentType, PlanStep};

pub struct Planner;

impl Planner {
    pub fn build_plan(original_query: &str, intent: IntentType) -> AppResult<ExecutionPlan> {
        let description = match &intent {
            IntentType::EvaluateExpression { expr } => {
                format!("Evaluate arithmetic expression '{}'", expr)
            }
            IntentType::SolveSingleEquation { lhs, rhs } => {
                format!("Solve equation '{} = {}' with exact/rational/approx distinction", lhs, rhs)
            }
            IntentType::SolveSystem { equations } => {
                format!("Solve multivariate system of {} equations (Linear/Newton-Jacobian)", equations.len())
            }
            IntentType::SymbolicDiff { expr, var } => {
                format!("Compute symbolic derivative d/d{} of '{}'", var, expr)
            }
            IntentType::DefiniteIntegral { expr, var, lower, upper } => {
                format!("Compute definite integral of '{}' d{} from {} to {}", expr, var, lower, upper)
            }
            IntentType::BooleanSat { formula } => {
                format!("Verify propositional logic & SAT for '{}'", formula)
            }
            IntentType::BatchProcess { items } => {
                format!("Execute batch of {} tasks", items.len())
            }
            IntentType::Echo { .. } => "Echo message payload back to caller".to_string(),
            IntentType::HealthCheck => "Perform system readiness check".to_string(),
        };

        let step = PlanStep {
            step_id: 1,
            description,
            action: intent,
        };

        Ok(ExecutionPlan {
            original_query: original_query.to_string(),
            steps: vec![step],
        })
    }
}