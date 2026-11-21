use crate::core::Orchestrator;
use crate::types::{AppResult, TaskRequest, TaskResponse};
use axum::{
    extract::State,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::sync::Arc;

pub fn create_router(orchestrator: Arc<Orchestrator>) -> Router {
    Router::new()
        .route("/", get(chat_ui_handler))
        .route("/health", get(health_handler))
        .route("/api/v1/execute", post(execute_handler))
        .with_state(orchestrator)
}

async fn health_handler(State(orchestrator): State<Arc<Orchestrator>>) -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": orchestrator.settings().app_name,
    }))
}

async fn execute_handler(
    State(orchestrator): State<Arc<Orchestrator>>,
    Json(payload): Json<TaskRequest>,
) -> AppResult<Json<TaskResponse>> {
    let response = orchestrator.process(payload)?;
    Ok(Json(response))
}

async fn chat_ui_handler() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html>
<html lang="th">
<head>
<meta charset="UTF-8">
<title>WOR Ultimate Neuro-Symbolic Math & Logic Console</title>
<style>
  body { background: #0f172a; color: #f8fafc; font-family: system-ui, sans-serif; max-width: 860px; margin: 36px auto; padding: 0 20px; }
  h2 { color: #38bdf8; margin-bottom: 4px; }
  .sub { color: #94a3b8; font-size: 0.9rem; margin-bottom: 16px; }
  #log { background: #1e293b; border: 1px solid #334155; border-radius: 10px; padding: 16px; min-height: 320px; max-height: 480px; overflow-y: auto; margin-bottom: 14px; }
  .msg { margin: 8px 0; padding: 10px 12px; border-radius: 8px; line-height: 1.5; font-family: monospace; }
  .user { background: #0284c7; color: #fff; text-align: right; }
  .bot { background: #0f172a; border-left: 4px solid #22c55e; color: #e2e8f0; }
  .err { background: #450a0a; border-left: 4px solid #ef4444; color: #fca5a5; }
  form { display: flex; gap: 8px; }
  input { flex: 1; padding: 12px; border-radius: 8px; border: 1px solid #475569; background: #1e293b; color: #fff; font-size: 1rem; }
  button { padding: 12px 20px; border: none; border-radius: 8px; background: #38bdf8; color: #0f172a; font-weight: bold; cursor: pointer; }
  .chips { display: flex; gap: 8px; flex-wrap: wrap; margin-bottom: 12px; }
  .chip { background: #334155; color: #cbd5e1; padding: 6px 10px; border-radius: 6px; font-size: 0.82rem; cursor: pointer; }
  .chip:hover { background: #475569; }
</style>
</head>
<body>
  <h2>WOR Ultimate Neuro-Symbolic Engine</h2>
  <div class="sub">Exact Rational (p/q) | '=' vs '≈' | Linear & Non-Linear Multivariate | Calculus | Boolean SAT</div>
  <div class="chips">
    <span class="chip" onclick="sendPreset('ทดสอบ  9.8-9.11   23x+100=491   3^x=x^9')">Batch: 9.8-9.11 | 23x+100=491 | 3^x=x^9</span>
    <span class="chip" onclick="sendPreset('3x+y=1, y=0')">Exact Rational: 3x+y=1, y=0</span>
    <span class="chip" onclick="sendPreset('x+y+z=6, 2x-y+z=3, x+2y-z=2')">Linear 3x3: x, y, z</span>
    <span class="chip" onclick="sendPreset('x^2+y^2=25, x-y=1')">Non-Linear 2D: x^2+y^2=25, x-y=1</span>
    <span class="chip" onclick="sendPreset('diff: 4x^3 - 5x^2 + 2x - 9')">Symbolic Diff: d/dx</span>
    <span class="chip" onclick="sendPreset('int: x^2, 0, 3')">Integral: ∫ x^2 [0..3]</span>
    <span class="chip" onclick="sendPreset('sat: (A & B) -> (A | C)')">SAT Tautology</span>
    <span class="chip" onclick="sendPreset('sat: (A | B) & (!A & !B)')">SAT Contradiction</span>
  </div>
  <div id="log"></div>
  <form onsubmit="sendQuery(event)">
    <input id="q" placeholder="พิมพ์โจทย์ได้เลย เช่น x^2+y^2=25, x-y=1 หรือ diff: 4x^3-5x^2 หรือ sat: (A&B)->A" autofocus />
    <button type="submit">รันคำสั่ง</button>
  </form>
<script>
const log = document.getElementById('log');
const input = document.getElementById('q');
function append(text, cls) {
  const d = document.createElement('div');
  d.className = 'msg ' + cls;
  d.textContent = text;
  log.appendChild(d);
  log.scrollTop = log.scrollHeight;
}
function sendPreset(txt) { input.value = txt; sendQuery(new Event('submit')); }
async function sendQuery(e) {
  e.preventDefault();
  const q = input.value.trim();
  if (!q) return;
  append('> ' + q, 'user');
  input.value = '';
  try {
    const res = await fetch('/api/v1/execute', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ query: q })
    });
    const data = await res.json();
    if (data.success) {
      append('[' + data.intent + '] ' + data.output, 'bot');
    } else {
      append('Error: ' + data.error, 'err');
    }
  } catch (err) {
    append('Network Error: ' + err.message, 'err');
  }
}
</script>
</body>
</html>"#)
}