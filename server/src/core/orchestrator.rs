use crate::config::Settings;
use crate::pipe::{Executor, IntentClassifier, Planner};
use crate::types::{AppResult, TaskRequest, TaskResponse};

#[derive(Debug, Clone)]
pub struct Orchestrator {
    settings: Settings,
}

impl Orchestrator {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    pub fn process(&self, request: TaskRequest) -> AppResult<TaskResponse> {
        let intent = IntentClassifier::classify(&request.query)?;
        let plan = Planner::build_plan(&request.query, intent)?;
        Executor::execute(plan)
    }

    pub fn ask(&self, query: &str) -> AppResult<TaskResponse> {
        self.process(TaskRequest {
            query: query.to_string(),
        })
    }
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new(Settings::default())
    }
}