use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let target = args.first().map(String::as_str).unwrap_or("E:/vvov/frontend");
    let sv_version = args.get(1).map(String::as_str).unwrap_or("1.7.0");

    println!("▶ scaffold → {target} (sv@{sv_version})");

    let status = Command::new("bun")
        .args([
            "x", &format!("sv@latest"), "create",
            std::path::Path::new(target).file_name().unwrap().to_str().unwrap(),
            "--template", "minimal", "--types", "ts",
            "--add", "prettier", "tailwindcss=plugins:none",
            "sveltekit-adapter=adapter:bun",
            "experimental=features:async,remoteFunctions",
            "--install", "bun",
        ])
        .current_dir(std::path::Path::new(target).parent().unwrap())
        .status()?;

    if !status.success() {
        return Err("sv create failed".into());
    }
    println!("✅ done");
    Ok(())
}