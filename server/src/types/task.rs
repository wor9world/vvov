use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RootEntry {
    pub variable: String,
    pub value: f64,
    pub is_exact: bool,
    pub rational_repr: Option<String>,
}

impl RootEntry {
    pub fn symbol(&self) -> &'static str {
        if self.is_exact { "=" } else { "≈" }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IntentType {
    EvaluateExpression { expr: String },
    SolveSingleEquation { lhs: String, rhs: String },
    SolveSystem { equations: Vec<(String, String)> },
    SymbolicDiff { expr: String, var: char },
    DefiniteIntegral { expr: String, var: char, lower: f64, upper: f64 },
    BooleanSat { formula: String },
    BatchProcess { items: Vec<String> },
    Echo { message: String },
    HealthCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequest {
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub step_id: usize,
    pub description: String,
    pub action: IntentType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub original_query: String,
    pub steps: Vec<PlanStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub success: bool,
    pub query: String,
    pub intent: String,
    pub output: String,
    pub numeric_result: Option<f64>,
    pub solutions: Option<Vec<RootEntry>>,
}