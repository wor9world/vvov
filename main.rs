// server/src/main.rs — SvelteKit Scaffolder (standalone)
//
//   cargo run                          # scaffold → E:/vvov/frontend
//   cargo run -- E:/vvov/myapp         # custom path
//   cargo run -- E:/vvov/myapp 1.7.0   # custom path + sv version
//   cargo run -- --force               # delete existing first
//   cargo run -- --dry-run             # print commands only

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{exit, Command};

const DEFAULT_TARGET: &str = "E:/vvov/app";
const DEFAULT_SV_VERSION: &str = "latest";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut dry_run = false;
    let mut force = false;
    let mut target: Option<String> = None;
    let mut version: Option<String> = None;

    for arg in &args {
        match arg.as_str() {
            "-h" | "--help" => { print_help(); return; }
            "-n" | "--dry-run" => dry_run = true,
            "-f" | "--force" => force = true,
            s if s.starts_with('-') => {
                eprintln!("unknown flag: {}", s);
                exit(1);
            }
            s => {
                if target.is_none() { target = Some(s.to_string()); }
                else if version.is_none() { version = Some(s.to_string()); }
                else { eprintln!("too many arguments"); exit(1); }
            }
        }
    }

    let target = target.unwrap_or_else(|| DEFAULT_TARGET.to_string());
    let version = version.unwrap_or_else(|| DEFAULT_SV_VERSION.to_string());

    if let Err(e) = scaffold(&target, &version, dry_run, force) {
        eprintln!("\n❌ {}", e);
        exit(1);
    }
}

fn scaffold(target: &str, sv_version: &str, dry_run: bool, force: bool) -> Result<(), String> {
    let path = PathBuf::from(target);
    let parent = path.parent()
        .ok_or("invalid target path")?
        .to_path_buf();
    let name = path.file_name()
        .and_then(|s| s.to_str())
        .ok_or("invalid project name")?
        .to_string();

    println!("🎯 target  : {}", path.display());
    println!("🎯 version : sv@{}", sv_version);
    println!();

    if path.exists() {
        if force {
            println!("🗑  removing existing: {}", path.display());
            if !dry_run {
                fs::remove_dir_all(&path).map_err(|e| format!("remove failed: {}", e))?;
            }
        } else {
            return Err(format!(
                "target exists: {}\n   use --force to delete and recreate",
                path.display()
            ));
        }
    }

    let sv_args: Vec<String> = vec![
        "x".into(), format!("sv@{}", sv_version), "create".into(), name.clone(),
        "--template".into(), "minimal".into(),
        "--types".into(), "ts".into(),
        "--add".into(), "prettier".into(),
        "tailwindcss=plugins:none".into(),
        "sveltekit-adapter=adapter:bun".into(),
        "experimental=features:async,remoteFunctions".into(),
        "--install".into(), "bun".into(),
    ];
    let add_args: Vec<String> = vec![
        "add".into(), "-d".into(),
        "@lucide/svelte".into(), "ai".into(),
        "@ai-sdk/svelte".into(), "zod".into(),
    ];

    if dry_run {
        println!("[dry-run] bun {}", sv_args.join(" "));
        println!("[dry-run] cd {} && bun {}", path.display(), add_args.join(" "));
        return Ok(());
    }

    fs::create_dir_all(&parent).map_err(|e| format!("mkdir failed: {}", e))?;

    println!("▶ bun x sv@{} create {}", sv_version, name);
    let status = Command::new("bun")
        .args(&sv_args)
        .current_dir(&parent)
        .status()
        .map_err(|e| format!("spawn bun failed (installed?): {}", e))?;
    if !status.success() {
        return Err(format!("sv create failed ({})", status));
    }

    println!("\n▶ bun add -d @lucide/svelte ai @ai-sdk/svelte zod");
    let status = Command::new("bun")
        .args(&add_args)
        .current_dir(&path)
        .status()
        .map_err(|e| format!("spawn bun failed: {}", e))?;
    if !status.success() {
        return Err(format!("bun add failed ({})", status));
    }

    println!("\n✅ done → {}", path.display());
    println!("   next: cd {} && bun run dev", path.display());
    Ok(())
}

fn print_help() {
    println!(r#"SvelteKit Scaffolder

Usage:
  cargo run [target] [sv_version] [flags]

Args:
  target       path (default: {})
  sv_version   svelte CLI version (default: {})

Flags:
  -f, --force      delete target if exists
  -n, --dry-run    print commands, don't execute
  -h, --help       show this help

Examples:
  cargo run
  cargo run -- E:/vvov/myapp
  cargo run -- E:/vvov/myapp 1.7.0
  cargo run -- --force
  cargo run -- --dry-run
"#, DEFAULT_TARGET, DEFAULT_SV_VERSION);
}