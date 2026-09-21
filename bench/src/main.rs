use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

use clap::Parser;

const GOLDENS_PER_FILE: usize = 10;

#[derive(Parser)]
#[command(
    name = "mitame-bench",
    about = "Benchmark mitame against flutter_test's LocalFileComparator"
)]
struct Args {
    #[arg(long, default_value_t = 200)]
    goldens: usize,
    #[arg(long, default_value = "390x844")]
    size: String,
    #[arg(long, default_value_t = default_jobs())]
    jobs: usize,
    #[arg(long, default_value_t = 3)]
    runs: usize,
    #[arg(long, default_value = "flutter")]
    flutter: String,
    #[arg(long)]
    mitame: Option<PathBuf>,
    #[arg(long)]
    fixture: Option<PathBuf>,
    #[arg(long)]
    keep: bool,
}

fn default_jobs() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

struct Row {
    config: &'static str,
    flutter_s: f64,
    compare_s: f64,
    flutter_exit: i32,
    changed: Option<u64>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let root = repo_root();
    let (width, height) = parse_size(&args.size)?;
    let fixture = args
        .fixture
        .clone()
        .unwrap_or_else(|| root.join("bench").join("fixture"));
    let mitame = args
        .mitame
        .clone()
        .unwrap_or_else(|| root.join("target").join("release").join("mitame"));

    if !args.keep && fixture.exists() {
        fs::remove_dir_all(&fixture)?;
    }
    if !fixture.exists() {
        generate(&fixture, &root, args.goldens, width, height)?;
    }
    if !mitame.exists() {
        let status = Command::new("cargo")
            .args(["build", "--release", "-p", "mitame-cli"])
            .current_dir(&root)
            .status()?;
        if !status.success() {
            return Err("cargo build failed".into());
        }
    }

    let jobs = args.jobs.to_string();
    let flutter_test: Vec<&str> = vec![args.flutter.as_str(), "test", "-j", &jobs];
    let env = |mode: &str, variant: &str| -> BTreeMap<String, String> {
        BTreeMap::from([
            ("BENCH_MODE".to_string(), mode.to_string()),
            ("BENCH_VARIANT".to_string(), variant.to_string()),
            ("MITAME_PROFILE".to_string(), "default".to_string()),
        ])
    };
    let mut no_golden = env("stock", "a");
    no_golden.insert("BENCH_SKIP_GOLDEN".to_string(), "1".to_string());

    println!(
        "goldens={} size={width}x{height} jobs={} runs={}",
        args.goldens, args.jobs, args.runs
    );
    exec(
        &[args.flutter.as_str(), "pub", "get"],
        &fixture,
        &BTreeMap::new(),
        true,
    )?;
    println!("warm-up");
    exec(&flutter_test, &fixture, &no_golden, true)?;
    println!("prepare stock goldens");
    let mut update = flutter_test.clone();
    update.push("--update-goldens");
    exec(&update, &fixture, &env("stock", "a"), true)?;
    println!("prepare mitame baseline");
    exec(&flutter_test, &fixture, &env("mitame", "a"), true)?;
    exec(
        &[mitame.to_str().unwrap(), "compare", "--update"],
        &fixture,
        &BTreeMap::new(),
        true,
    )?;

    let configs: [(&str, BTreeMap<String, String>, bool, bool); 5] = [
        ("no goldens (compile + render only)", no_golden, false, true),
        (
            "stock, all goldens identical",
            env("stock", "a"),
            false,
            true,
        ),
        (
            "stock, all goldens differ (tests fail)",
            env("stock", "b"),
            false,
            false,
        ),
        (
            "mitame, all goldens identical",
            env("mitame", "a"),
            true,
            true,
        ),
        ("mitame, all goldens differ", env("mitame", "b"), true, true),
    ];

    let mut rows = Vec::new();
    for (name, flutter_env, compare, check_flutter) in configs {
        let mut flutter_times = Vec::new();
        let mut compare_times = Vec::new();
        let mut flutter_exit = 0;
        let mut changed = None;
        for _ in 0..args.runs {
            let _ = fs::remove_dir_all(fixture.join(".mitame").join("current"));
            for entry in fs::read_dir(fixture.join("test"))?.flatten() {
                let _ = fs::remove_dir_all(entry.path().join("failures"));
            }
            let (t_flutter, code) = exec(&flutter_test, &fixture, &flutter_env, check_flutter)?;
            flutter_exit = code;
            let mut t_compare = 0.0;
            if compare {
                let (t, _) = exec(
                    &[mitame.to_str().unwrap(), "compare"],
                    &fixture,
                    &BTreeMap::new(),
                    false,
                )?;
                t_compare = t;
                let text =
                    fs::read_to_string(fixture.join(".mitame").join("report").join("result.json"))?;
                let json: serde_json::Value = serde_json::from_str(&text)?;
                changed = json["summary"]["changed"].as_u64();
            }
            flutter_times.push(t_flutter);
            compare_times.push(t_compare);
        }
        let row = Row {
            config: name,
            flutter_s: median(&mut flutter_times),
            compare_s: median(&mut compare_times),
            flutter_exit,
            changed,
        };
        println!(
            "{}: flutter {:.2}s compare {:.2}s exit {} changed {}",
            row.config,
            row.flutter_s,
            row.compare_s,
            row.flutter_exit,
            row.changed.map(|c| c.to_string()).unwrap_or_default()
        );
        rows.push(row);
    }

    println!();
    println!("| configuration | flutter test (s) | mitame compare (s) | total (s) | flutter exit | changed |");
    println!("| --- | ---: | ---: | ---: | ---: | ---: |");
    for row in &rows {
        println!(
            "| {} | {:.2} | {:.2} | {:.2} | {} | {} |",
            row.config,
            row.flutter_s,
            row.compare_s,
            row.flutter_s + row.compare_s,
            row.flutter_exit,
            row.changed.map(|c| c.to_string()).unwrap_or_default()
        );
    }
    Ok(())
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn parse_size(s: &str) -> Result<(u32, u32), String> {
    let (w, h) = s
        .split_once('x')
        .ok_or_else(|| format!("size must be WxH, got {s}"))?;
    Ok((
        w.parse().map_err(|e| format!("{e}"))?,
        h.parse().map_err(|e| format!("{e}"))?,
    ))
}

fn generate(
    fixture: &Path,
    root: &Path,
    goldens: usize,
    width: u32,
    height: u32,
) -> std::io::Result<()> {
    let adapter = root.join("adapters").join("flutter");
    write(
        &fixture.join("pubspec.yaml"),
        &include_str!("templates/pubspec.yaml")
            .replace("__ADAPTER_PATH__", adapter.to_str().unwrap()),
    )?;
    write(&fixture.join("mitame.toml"), "[compare]\nthreshold = 0.0\n")?;
    write(
        &fixture.join("lib").join("bench_widget.dart"),
        include_str!("templates/bench_widget.dart"),
    )?;
    write(
        &fixture.join("test").join("flutter_test_config.dart"),
        include_str!("templates/flutter_test_config.dart"),
    )?;
    let files = goldens.div_ceil(GOLDENS_PER_FILE);
    for f in 0..files {
        let mut body = String::from(include_str!("templates/test_header.dart"));
        for j in 0..GOLDENS_PER_FILE {
            let index = f * GOLDENS_PER_FILE + j;
            if index >= goldens {
                break;
            }
            body.push_str(
                &include_str!("templates/test_case.dart")
                    .replace("__INDEX__", &index.to_string())
                    .replace("__GOLDEN__", &j.to_string())
                    .replace("__WIDTH__", &width.to_string())
                    .replace("__HEIGHT__", &height.to_string()),
            );
        }
        body.push_str("}\n");
        write(
            &fixture
                .join("test")
                .join(format!("g{f}"))
                .join("bench_test.dart"),
            &body,
        )?;
    }
    Ok(())
}

fn write(path: &Path, text: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, text)
}

fn exec(
    cmd: &[&str],
    cwd: &Path,
    env: &BTreeMap<String, String>,
    check: bool,
) -> Result<(f64, i32), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let status = Command::new(cmd[0])
        .args(&cmd[1..])
        .current_dir(cwd)
        .envs(env)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    let elapsed = start.elapsed().as_secs_f64();
    let code = status.code().unwrap_or(-1);
    if check && !status.success() {
        let _ = Command::new(cmd[0])
            .args(&cmd[1..])
            .current_dir(cwd)
            .envs(env)
            .status();
        return Err(format!("command failed: {}", cmd.join(" ")).into());
    }
    Ok((elapsed, code))
}

fn median(values: &mut [f64]) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = values.len();
    if n == 0 {
        0.0
    } else if n % 2 == 1 {
        values[n / 2]
    } else {
        (values[n / 2 - 1] + values[n / 2]) / 2.0
    }
}
