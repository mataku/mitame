use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use mitame_contract::{ResultFile, Sidecar};
use mitame_core::{compare, update_baseline, Config, Layout, CONFIG_TEMPLATE};

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
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    #[arg(long, global = true)]
    root: Option<PathBuf>,
    #[arg(long, global = true, env = "MITAME_PROFILE")]
    profile: Option<String>,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Write a mitame.toml with the default settings into the current directory")]
    Init {
        #[arg(long)]
        force: bool,
    },
    #[command(about = "Run the test command with capture enabled and write .mitame/current/")]
    Capture {
        #[command(flatten)]
        common: Common,
        #[command(flatten)]
        capture: CaptureArgs,
    },
    #[command(about = "Compare current captures against the baseline and write the report")]
    Compare {
        #[command(flatten)]
        common: Common,
        #[command(flatten)]
        update: UpdateArgs,
    },
    #[command(about = "Capture, then compare; the report is written even when the command fails")]
    Run {
        #[command(flatten)]
        common: Common,
        #[command(flatten)]
        capture: CaptureArgs,
        #[command(flatten)]
        update: UpdateArgs,
    },
    #[command(about = "Rewrite the HTML report from an existing result.json")]
    Report {
        #[command(flatten)]
        common: Common,
    },
    #[command(about = "Write the JSON schemas for the sidecar and result.json")]
    Schema {
        #[arg(long, default_value = "schema")]
        out: PathBuf,
    },
}

#[derive(Args)]
struct CaptureArgs {
    #[arg(long, env = "MITAME_FLUTTER")]
    flutter: Option<PathBuf>,
    #[arg(long)]
    keep_current: bool,
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

#[derive(Args)]
struct UpdateArgs {
    #[arg(long)]
    update: bool,
    #[arg(long, requires = "update")]
    prune: bool,
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
        Command::Init { force } => {
            let path = Path::new("mitame.toml");
            if path.exists() && !force {
                return Err("mitame.toml already exists; pass --force to overwrite it".into());
            }
            let mut text = CONFIG_TEMPLATE.to_string();
            if let Some(command) = detect_capture_command(Path::new(".")) {
                let rendered = command
                    .iter()
                    .map(|c| format!("{c:?}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                text = text.replacen("command = []", &format!("command = [{rendered}]"), 1);
                println!("detected test command: {}", command.join(" "));
            } else {
                println!("no test command detected; set [capture] command in mitame.toml");
            }
            std::fs::write(path, text)?;
            println!("wrote {}", path.display());
            println!("commit .mitame/baseline/ and add .mitame/current/ and .mitame/report/ to .gitignore");
            Ok(EXIT_OK)
        }
        Command::Capture { common, capture } => {
            let (config, layout, project) = resolve(&common)?;
            let command = build_command(&config, &capture, &project)?;
            let ok = run_capture(&layout, &command, capture.keep_current)?;
            Ok(if ok { EXIT_OK } else { EXIT_ERROR })
        }
        Command::Compare { common, update } => {
            let (config, layout, _) = resolve(&common)?;
            run_compare(&config, &layout, &update)
        }
        Command::Run {
            common,
            capture,
            update,
        } => {
            let (config, layout, project) = resolve(&common)?;
            let command = build_command(&config, &capture, &project)?;
            let ok = run_capture(&layout, &command, capture.keep_current)?;
            let code = run_compare(&config, &layout, &update)?;
            Ok(if ok { code } else { EXIT_ERROR })
        }
        Command::Report { common } => {
            let (_, layout, _) = resolve(&common)?;
            let text = std::fs::read_to_string(layout.result_path())?;
            let result: ResultFile = serde_json::from_str(&text)?;
            let path = mitame_core::write_html(&layout, &result)?;
            println!("report: {}", path.display());
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

fn build_command(
    config: &Config,
    capture: &CaptureArgs,
    project: &Path,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut command = config.capture.command.clone();
    command.extend(capture.args.iter().cloned());
    if command.is_empty() {
        return Err(
            "no test command: set [capture] command in mitame.toml or pass one after --".into(),
        );
    }
    if command[0] == "flutter" {
        command[0] = resolve_flutter(capture.flutter.clone(), project)
            .to_string_lossy()
            .into_owned();
    }
    Ok(command)
}

fn run_capture(
    layout: &Layout,
    command: &[String],
    keep_current: bool,
) -> Result<bool, Box<dyn std::error::Error>> {
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
    if !status.success() {
        eprintln!(
            "{} exited with {status}; captures written before that are kept",
            command[0]
        );
    }
    Ok(status.success())
}

fn run_compare(
    config: &Config,
    layout: &Layout,
    update: &UpdateArgs,
) -> Result<u8, Box<dyn std::error::Error>> {
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
    if update.update {
        let applied = update_baseline(layout, &outcome.result, update.prune)?;
        for id in &applied.updated {
            println!("updated  {id}");
        }
        for id in &applied.added {
            println!("added    {id}");
        }
        for id in &applied.deleted {
            println!("deleted  {id}");
        }
        println!("baseline: {}", layout.baseline_dir().display());
    }
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

fn detect_capture_command(dir: &Path) -> Option<Vec<&'static str>> {
    if dir.join("pubspec.yaml").is_file() {
        return Some(vec!["flutter", "test"]);
    }
    if dir.join("settings.gradle.kts").is_file() || dir.join("settings.gradle").is_file() {
        return Some(vec!["./gradlew", "test", "--rerun"]);
    }
    None
}

fn find_project_dir(start: &Path) -> Option<PathBuf> {
    if start.join("mitame.toml").is_file() || start.join(".mitame").is_dir() {
        return Some(start.to_path_buf());
    }
    start
        .ancestors()
        .skip(1)
        .find(|dir| {
            dir.join("mitame.toml").is_file() || dir.join(".mitame").join("baseline").is_dir()
        })
        .map(Path::to_path_buf)
}

fn resolve(common: &Common) -> Result<(Config, Layout, PathBuf), Box<dyn std::error::Error>> {
    let here = PathBuf::from(".");
    let project = match &common.config {
        Some(path) => path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map(Path::to_path_buf)
            .unwrap_or(here),
        None => {
            let cwd = std::env::current_dir()?;
            match find_project_dir(&cwd) {
                Some(dir) if dir != cwd => {
                    println!("project: {}", dir.display());
                    dir
                }
                _ => here,
            }
        }
    };
    let config_path = common
        .config
        .clone()
        .unwrap_or_else(|| join_in(&project, "mitame.toml"));
    let config = Config::load_or_default(&config_path)?;
    let root = common
        .root
        .clone()
        .unwrap_or_else(|| join_in(&project, &config.paths.root));
    let profile = common
        .profile
        .clone()
        .or_else(|| config.profile.clone())
        .unwrap_or_else(|| "default".to_string());
    Ok((config, Layout::new(root, profile), project))
}

fn join_in(project: &Path, child: &str) -> PathBuf {
    if project == Path::new(".") {
        PathBuf::from(child)
    } else {
        project.join(child)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_dir_is_the_nearest_ancestor_with_config_or_layout() {
        let dir = tempfile::tempdir().unwrap();
        let outer = dir.path().join("outer");
        let inner = outer.join("inner");
        let leaf = inner.join("a").join("b");
        std::fs::create_dir_all(&leaf).unwrap();
        std::fs::write(outer.join("mitame.toml"), "").unwrap();
        assert_eq!(find_project_dir(&leaf), Some(outer.clone()));
        std::fs::create_dir_all(inner.join(".mitame").join("report")).unwrap();
        assert_eq!(find_project_dir(&leaf), Some(outer.clone()));
        assert_eq!(find_project_dir(&inner), Some(inner.clone()));
        std::fs::create_dir_all(inner.join(".mitame").join("baseline")).unwrap();
        assert_eq!(find_project_dir(&leaf), Some(inner.clone()));
        assert_eq!(find_project_dir(&outer), Some(outer));
    }

    #[test]
    fn capture_command_is_detected_from_project_files() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(detect_capture_command(dir.path()), None);
        std::fs::write(dir.path().join("settings.gradle.kts"), "").unwrap();
        assert_eq!(
            detect_capture_command(dir.path()),
            Some(vec!["./gradlew", "test", "--rerun"])
        );
        std::fs::write(dir.path().join("pubspec.yaml"), "").unwrap();
        assert_eq!(
            detect_capture_command(dir.path()),
            Some(vec!["flutter", "test"])
        );
    }

    #[test]
    fn join_in_keeps_paths_bare_for_the_current_directory() {
        assert_eq!(join_in(Path::new("."), ".mitame"), PathBuf::from(".mitame"));
        assert_eq!(
            join_in(Path::new("/p"), ".mitame"),
            PathBuf::from("/p/.mitame")
        );
    }

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
