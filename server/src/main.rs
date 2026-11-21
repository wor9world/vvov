use server::{api::create_router, config::Settings, core::Orchestrator};
use std::{
    env,
    io::{self, BufRead, Write},
    sync::Arc,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::from_env();
    let orchestrator = Arc::new(Orchestrator::new(settings.clone()));

    let args: Vec<String> = env::args().skip(1).collect();
    if !args.is_empty() {
        let query = args.join(" ");
        match orchestrator.ask(&query) {
            Ok(res) => {
                println!("\n[Intent] : {}", res.intent);
                println!("[Result] : {}\n", res.output);
            }
            Err(e) => eprintln!("\n[Error]  : {}\n", e),
        }
        return Ok(());
    }

    let bind_addr = settings.bind_address();
    let app = create_router(orchestrator.clone());
    let listener = TcpListener::bind(&bind_addr).await?;

    println!("================================================================================");
    println!("  WOR ULTIMATE NEURO-SYMBOLIC ENGINE (v0.2.0)");
    println!("  - Web Chat UI : http://{}", bind_addr);
    println!("  - [Batch]     : ทดสอบ  9.8-9.11   23x+100=491   3^x=x^9");
    println!("  - [Rational]  : 3x+y=1, y=0                 --> x = 1/3 (≈ 0.3333333333)");
    println!("  - [NonLinear] : x^2+y^2=25, x-y=1           --> Multi-root Jacobian Newton");
    println!("  - [Calculus]  : diff: 4x^3 - 5x^2 + 2x - 9  |  int: x^2, 0, 3");
    println!("  - [Logic SAT] : sat: (A & B) -> (A | C)");
    println!("================================================================================\n");

    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    let repl_orchestrator = orchestrator.clone();
    tokio::task::spawn_blocking(move || {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        loop {
            print!("wor> ");
            let _ = stdout.flush();

            let mut line = String::new();
            match stdin.lock().read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit") {
                        println!("Goodbye!");
                        break;
                    }
                    match repl_orchestrator.ask(trimmed) {
                        Ok(res) => println!("  => [{}] {}\n", res.intent, res.output),
                        Err(err) => println!("  [!] {}\n", err),
                    }
                }
                Err(_) => break,
            }
        }
    })
    .await?;

    Ok(())
}