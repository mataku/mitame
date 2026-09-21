use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use mitame_contract::{ResultFile, Sidecar};
use mitame_core::{approve, compare, Config, Layout};

const EXIT_OK: u8 = 0;
const EXIT_DIFF: u8 = 1;
const EXIT_ERROR: u8 = 2;

#[derive(Parser)]
#[command(
    name = "mitame",
    version,
    about = "Visual regression testing for Flutter, Android, and iOS"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Args)]
struct Common {
    #[arg(long, global = true, default_value = "mitame.toml")]
    config: PathBuf,
    #[arg(long, global = true)]
    root: Option<PathBuf>,
    #[arg(long, global = true, env = "MITAME_PROFILE")]
    profile: Option<String>,
}

#[derive(Subcommand)]
enum Command {
    Compare {
        #[command(flatten)]
        common: Common,
    },
    Approve {
        #[command(flatten)]
        common: Common,
        ids: Vec<String>,
    },
    Report {
        #[command(flatten)]
        common: Common,
    },
    Test {
        #[command(flatten)]
        common: Common,
        #[arg(long, env = "MITAME_FLUTTER")]
        flutter: Option<PathBuf>,
        #[arg(long)]
        keep_current: bool,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    Run {
        #[command(flatten)]
        common: Common,
        #[arg(long)]
        keep_current: bool,
        #[arg(required = true, trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>,
    },
    Schema {
        #[arg(long, default_value = "schema")]
        out: PathBuf,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(EXIT_ERROR)
        }
    }
}

fn run() -> Result<u8, Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Command::Compare { common } => {
            let (config, layout) = resolve(&common)?;
            run_compare(&config, &layout)
        }
        Command::Test {
            common,
            flutter,
            keep_current,
            args,
        } => {
            let (config, layout) = resolve(&common)?;
            let flutter = resolve_flutter(flutter, Path::new("."));
            let mut command = vec![flutter.to_string_lossy().into_owned(), "test".to_string()];
            command.extend(args);
            run_capture(&config, &layout, &command, keep_current)
        }
        Command::Run {
            common,
            keep_current,
            command,
        } => {
            let (config, layout) = resolve(&common)?;
            run_capture(&config, &layout, &command, keep_current)
        }
        Command::Report { common } => {
            let (_, layout) = resolve(&common)?;
            let text = std::fs::read_to_string(layout.result_path())?;
            let result: ResultFile = serde_json::from_str(&text)?;
            let path = mitame_core::write_html(&layout, &result)?;
            println!("report: {}", path.display());
            Ok(EXIT_OK)
        }
        Command::Approve { common, ids } => {
            let (_, layout) = resolve(&common)?;
            let outcome = approve(&layout, &ids)?;
            for id in &outcome.copied {
                println!("approved {id}");
            }
            for id in &outcome.deleted {
                println!("deleted  {id}");
            }
            if !outcome.unchanged.is_empty() {
                println!(
                    "unchanged {} (already in baseline)",
                    outcome.unchanged.len()
                );
            }
            println!("baseline: {}", layout.baseline_dir().display());
            Ok(EXIT_OK)
        }
        Command::Schema { out } => {
            std::fs::create_dir_all(&out)?;
            let sidecar = schemars::schema_for!(Sidecar);
            let result = schemars::schema_for!(ResultFile);
            std::fs::write(
                out.join("sidecar.schema.json"),
                serde_json::to_string_pretty(&sidecar)?,
            )?;
            std::fs::write(
                out.join("result.schema.json"),
                serde_json::to_string_pretty(&result)?,
            )?;
            println!("wrote {}", out.display());
            Ok(EXIT_OK)
        }
    }
}

fn run_capture(
    config: &Config,
    layout: &Layout,
    command: &[String],
    keep_current: bool,
) -> Result<u8, Box<dyn std::error::Error>> {
    let output_dir = std::path::absolute(layout.root.join("current"))?;
    println!("using {}", command.join(" "));
    if !keep_current {
        let dir = layout.current_dir();
        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
        }
    }
    let run_id = format!(
        "{}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
        std::process::id()
    );
    let vars = [
        (
            "MITAME_OUTPUT_DIR",
            output_dir.to_string_lossy().into_owned(),
        ),
        ("MITAME_PROFILE", layout.profile.clone()),
        ("MITAME_RUN_ID", run_id),
    ];
    let mut child = std::process::Command::new(&command[0]);
    child.args(&command[1..]);
    for (key, value) in &vars {
        child.env(key, value);
        child.env(format!("TEST_RUNNER_{key}"), value);
    }
    let status = child
        .status()
        .map_err(|e| format!("failed to run {}: {e}", command[0]))?;
    let code = run_compare(config, layout)?;
    if !status.success() {
        eprintln!(
            "{} exited with {status}; the report covers the captures that succeeded",
            command[0]
        );
        return Ok(EXIT_ERROR);
    }
    Ok(code)
}

fn run_compare(config: &Config, layout: &Layout) -> Result<u8, Box<dyn std::error::Error>> {
    let outcome = compare(config, layout)?;
    let s = &outcome.result.summary;
    println!(
        "profile {}: unchanged {}, changed {}, added {}, removed {}, mismatch {}, error {}",
        layout.profile, s.unchanged, s.changed, s.added, s.removed, s.mismatch, s.error
    );
    let mut listed: Vec<&mitame_contract::Entry> = outcome
        .result
        .results
        .iter()
        .filter(|e| e.status != mitame_contract::Status::Unchanged)
        .collect();
    listed.sort_by(|a, b| {
        b.diff_pixels
            .unwrap_or(0)
            .cmp(&a.diff_pixels.unwrap_or(0))
            .then_with(|| a.id.cmp(&b.id))
    });
    for entry in listed {
        {
            let detail = entry
                .message
                .clone()
                .or_else(|| match (entry.diff_pixels, entry.diff_ratio) {
                    (Some(px), Some(r)) => Some(format!("{px} px ({:.3}%)", r * 100.0)),
                    _ => None,
                })
                .unwrap_or_default();
            println!(
                "  {:<9} {} {}",
                format!("{:?}", entry.status).to_lowercase(),
                entry.id,
                detail
            );
        }
    }
    println!(
        "report: {}",
        layout.report_dir().join("index.html").display()
    );
    Ok(if outcome.errored {
        EXIT_ERROR
    } else if outcome.failed {
        EXIT_DIFF
    } else {
        EXIT_OK
    })
}

fn resolve_flutter(explicit: Option<PathBuf>, project: &Path) -> PathBuf {
    if let Some(path) = explicit {
        return path;
    }
    if let Some(path) = flutter_from_fvmrc(project, fvm_cache_dir().as_deref()) {
        return path;
    }
    let symlink = project
        .join(".fvm")
        .join("flutter_sdk")
        .join("bin")
        .join("flutter");
    if symlink.exists() {
        return symlink;
    }
    PathBuf::from("flutter")
}

fn fvm_cache_dir() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("FVM_CACHE_PATH") {
        return Some(PathBuf::from(path));
    }
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join("fvm"))
}

fn flutter_from_fvmrc(project: &Path, cache: Option<&Path>) -> Option<PathBuf> {
    let text = std::fs::read_to_string(project.join(".fvmrc")).ok()?;
    let json: serde_json::Value = serde_json::from_str(&text).ok()?;
    let version = json.get("flutter")?.as_str()?;
    let candidate = cache?
        .join("versions")
        .join(version)
        .join("bin")
        .join("flutter");
    candidate.exists().then_some(candidate)
}

fn resolve(common: &Common) -> Result<(Config, Layout), Box<dyn std::error::Error>> {
    let config = Config::load_or_default(&common.config)?;
    let root = common
        .root
        .clone()
        .unwrap_or_else(|| PathBuf::from(&config.paths.root));
    let profile = common
        .profile
        .clone()
        .or_else(|| config.profile.clone())
        .unwrap_or_else(|| "default".to_string());
    Ok((config, Layout::new(root, profile)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fvmrc_resolves_to_cached_version_only_when_present() {
        let dir = tempfile::tempdir().unwrap();
        let cache = dir.path().join("cache");
        let cached = cache.join("versions").join("3.35.2").join("bin");
        std::fs::create_dir_all(&cached).unwrap();
        std::fs::write(cached.join("flutter"), "").unwrap();
        let project = dir.path().join("project");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::write(project.join(".fvmrc"), r#"{"flutter": "3.35.2"}"#).unwrap();
        assert_eq!(
            flutter_from_fvmrc(&project, Some(&cache)),
            Some(cached.join("flutter"))
        );
        std::fs::write(project.join(".fvmrc"), r#"{"flutter": "9.9.9"}"#).unwrap();
        assert_eq!(flutter_from_fvmrc(&project, Some(&cache)), None);
    }
}
