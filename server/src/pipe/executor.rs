use crate::pipe::IntentClassifier;
use crate::tools::Calculator;
use crate::types::{AppError, AppResult, ExecutionPlan, IntentType, TaskResponse};

pub struct Executor;

impl Executor {
    pub fn execute(plan: ExecutionPlan) -> AppResult<TaskResponse> {
        let first_step = plan
            .steps
            .into_iter()
            .next()
            .ok_or_else(|| AppError::ExecutionError("Execution plan has no steps".to_string()))?;

        match first_step.action {
            IntentType::EvaluateExpression { expr } => {
                let (value, _is_exact, formatted) = Calculator::eval_expression(&expr)?;
                Ok(TaskResponse {
                    success: true,
                    query: plan.original_query,
                    intent: "EvaluateExpression".to_string(),
                    output: formatted,
                    numeric_result: Some(value),
                    solutions: None,
                })
            }
            IntentType::SolveSingleEquation { lhs, rhs } => {
                let (entries, summary) = Calculator::solve_single_equation(&lhs, &rhs)?;
                let first_val = entries.first().map(|e| e.value);
                Ok(TaskResponse {
                    success: true,
                    query: plan.original_query,
                    intent: "SolveEquation".to_string(),
                    output: summary,
                    numeric_result: first_val,
                    solutions: Some(entries),
                })
            }
            IntentType::SolveSystem { equations } => {
                let (entries, summary) = Calculator::solve_system(&equations)?;
                let first_val = entries.first().map(|e| e.value);
                Ok(TaskResponse {
                    success: true,
                    query: plan.original_query,
                    intent: "SolveMultivariateSystem".to_string(),
                    output: summary,
                    numeric_result: first_val,
                    solutions: Some(entries),
                })
            }
            IntentType::SymbolicDiff { expr, var } => {
                let output = Calculator::symbolic_derivative(&expr, var)?;
                Ok(TaskResponse {
                    success: true,
                    query: plan.original_query,
                    intent: "SymbolicCalculus".to_string(),
                    output,
                    numeric_result: None,
                    solutions: None,
                })
            }
            IntentType::DefiniteIntegral {
                expr,
                var,
                lower,
                upper,
            } => {
                let (val, output) = Calculator::definite_integral(&expr, var, lower, upper)?;
                Ok(TaskResponse {
                    success: true,
                    query: plan.original_query,
                    intent: "DefiniteIntegral".to_string(),
                    output,
                    numeric_result: Some(val),
                    solutions: None,
                })
            }
            IntentType::BooleanSat { formula } => {
                let output = Calculator::solve_boolean_sat(&formula)?;
                Ok(TaskResponse {
                    success: true,
                    query: plan.original_query,
                    intent: "BooleanSAT".to_string(),
                    output,
                    numeric_result: None,
                    solutions: None,
                })
            }
            IntentType::BatchProcess { items } => {
                let mut outputs = Vec::new();
                let mut all_solutions = Vec::new();
                for item in items {
                    let sub_intent = IntentClassifier::classify(&item)?;
                    let sub_plan = crate::pipe::Planner::build_plan(&item, sub_intent)?;
                    let sub_res = Self::execute(sub_plan)?;
                    outputs.push(format!("[{}] {}", sub_res.intent, sub_res.output));
                    if let Some(sols) = sub_res.solutions {
                        all_solutions.extend(sols);
                    }
                }
                Ok(TaskResponse {
                    success: true,
                    query: plan.original_query,
                    intent: "BatchProcess".to_string(),
                    output: outputs.join(" | "),
                    numeric_result: all_solutions.first().map(|s| s.value),
                    solutions: Some(all_solutions),
                })
            }
            IntentType::Echo { message } => Ok(TaskResponse {
                success: true,
                query: plan.original_query,
                intent: "Echo".to_string(),
                output: message,
                numeric_result: None,
                solutions: None,
            }),
            IntentType::HealthCheck => Ok(TaskResponse {
                success: true,
                query: plan.original_query,
                intent: "HealthCheck".to_string(),
                output: "PONG".to_string(),
                numeric_result: None,
                solutions: None,
            }),
        }
    }
}