use std::path::PathBuf;
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
    about = "Visual regression testing for Flutter"
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
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
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
        Command::Test { common, args } => {
            let (config, layout) = resolve(&common)?;
            let output_dir = std::path::absolute(layout.root.join("current"))?;
            let status = std::process::Command::new("flutter")
                .arg("test")
                .args(&args)
                .env("MITAME_OUTPUT_DIR", &output_dir)
                .env("MITAME_PROFILE", &layout.profile)
                .status()?;
            if !status.success() {
                eprintln!("flutter test exited with {status}");
                return Ok(EXIT_ERROR);
            }
            run_compare(&config, &layout)
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

fn run_compare(config: &Config, layout: &Layout) -> Result<u8, Box<dyn std::error::Error>> {
    let outcome = compare(config, layout)?;
    let s = &outcome.result.summary;
    println!(
        "profile {}: unchanged {}, changed {}, added {}, removed {}, mismatch {}, error {}",
        layout.profile, s.unchanged, s.changed, s.added, s.removed, s.mismatch, s.error
    );
    for entry in &outcome.result.results {
        if entry.status != mitame_contract::Status::Unchanged {
            let detail = entry
                .message
                .clone()
                .or_else(|| entry.diff_ratio.map(|r| format!("diff_ratio {r:.4}")))
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
