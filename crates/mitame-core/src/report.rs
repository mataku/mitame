use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use mitame_contract::{Entry, ResultFile, Status};

use crate::error::{Error, Result};
use crate::layout::Layout;

const STYLE: &str = r#"
:root { color-scheme: light dark; --bg: #fff; --fg: #1a1a1a; --muted: #666; --line: #e3e3e3; --card: #fafafa;
  --changed: #c62828; --added: #2e7d32; --removed: #ef6c00; --mismatch: #6a1b9a; --error: #b71c1c; --unchanged: #616161; }
@media (prefers-color-scheme: dark) { :root { --bg: #121212; --fg: #e8e8e8; --muted: #9a9a9a; --line: #2c2c2c; --card: #1c1c1c; } }
* { box-sizing: border-box; }
body { margin: 0; padding: 24px; font: 14px/1.5 system-ui, sans-serif; background: var(--bg); color: var(--fg); }
h1 { font-size: 20px; margin: 0 0 4px; }
.meta { color: var(--muted); margin-bottom: 16px; }
.summary { display: flex; flex-wrap: wrap; gap: 8px; margin-bottom: 24px; }
.summary a { padding: 6px 12px; border: 1px solid var(--line); border-radius: 999px; text-decoration: none; color: var(--fg); background: var(--card); }
.summary a b { margin-right: 6px; }
.summary a.off { opacity: .45; }
section h2 { font-size: 16px; margin: 24px 0 8px; text-transform: capitalize; }
.entry { border: 1px solid var(--line); border-radius: 8px; padding: 12px; margin-bottom: 12px; background: var(--card); }
.entry.hidden { display: none; }
.entry header { display: flex; flex-wrap: wrap; gap: 8px 16px; align-items: baseline; margin-bottom: 8px; }
.entry header code { font-size: 13px; }
.status { font-weight: 600; text-transform: uppercase; font-size: 11px; letter-spacing: .04em; }
.status.changed { color: var(--changed); } .status.added { color: var(--added); } .status.removed { color: var(--removed); }
.status.mismatch { color: var(--mismatch); } .status.error { color: var(--error); } .status.unchanged { color: var(--unchanged); }
.detail { color: var(--muted); }
.images { display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 12px; }
figure { margin: 0; }
figcaption { color: var(--muted); font-size: 12px; margin-bottom: 4px; }
img { max-width: 100%; height: auto; border: 1px solid var(--line); background:
  repeating-conic-gradient(#8882 0 25%, transparent 0 50%) 0 0 / 16px 16px; }
"#;

const SCRIPT: &str = r#"
function localize(el, prefix) {
  var d = new Date(el.dataset.utc.replace(' UTC', 'Z').replace(' ', 'T'));
  if (!isNaN(d)) { el.textContent = prefix + d.toLocaleString(); el.title = el.dataset.utc; }
}
document.querySelectorAll('.generated').forEach(function (el) { localize(el, ''); });
document.querySelectorAll('.captured').forEach(function (el) { localize(el, 'captured '); });
document.querySelectorAll('.summary a').forEach(function (a) {
  a.addEventListener('click', function (e) {
    e.preventDefault();
    a.classList.toggle('off');
    var status = a.dataset.status;
    document.querySelectorAll('.entry[data-status="' + status + '"]').forEach(function (el) {
      el.classList.toggle('hidden', a.classList.contains('off'));
    });
  });
});
"#;

pub fn write_html(layout: &Layout, result: &ResultFile) -> Result<PathBuf> {
    let report_dir = layout.report_dir();
    fs::create_dir_all(&report_dir).map_err(|e| Error::io(&report_dir, e))?;
    for entry in result.results.iter().filter(|e| shows_images(e)) {
        for (side, path) in [("baseline", &entry.baseline), ("current", &entry.current)] {
            if let Some(rel) = path {
                let src = layout.root.join(rel);
                let dst = report_dir.join(side).join(format!("{}.png", entry.id));
                if let Some(parent) = dst.parent() {
                    fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
                }
                fs::copy(&src, &dst).map_err(|e| Error::io(&dst, e))?;
            }
        }
    }
    let html = render(result);
    let path = report_dir.join("index.html");
    fs::write(&path, html).map_err(|e| Error::io(&path, e))?;
    Ok(path)
}

fn shows_images(entry: &Entry) -> bool {
    entry.status != Status::Unchanged || entry.diff_pixels.unwrap_or(0) > 0
}

pub fn render(result: &ResultFile) -> String {
    render_at(result, &timestamp())
}

fn timestamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let (y, m, d) = civil_from_days(days as i64);
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02} UTC",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

pub fn render_at(result: &ResultFile, generated_at: &str) -> String {
    let mut out = String::new();
    let s = &result.summary;
    let _ = write!(
        out,
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>mitame report</title><style>{STYLE}</style></head><body>"
    );
    let _ = write!(
        out,
        "<h1>mitame report</h1><div class=\"meta\">profile <code>{}</code> · {} screenshots · generated <span class=\"generated\" data-utc=\"{2}\">{2}</span></div>",
        escape(&result.profile),
        result.results.len(),
        escape(generated_at)
    );
    out.push_str("<nav class=\"summary\">");
    for (status, count) in [
        (Status::Changed, s.changed),
        (Status::Added, s.added),
        (Status::Removed, s.removed),
        (Status::Mismatch, s.mismatch),
        (Status::Error, s.error),
        (Status::Unchanged, s.unchanged),
    ] {
        let name = status_name(status);
        let all_hidden = status == Status::Unchanged
            && count > 0
            && result
                .results
                .iter()
                .all(|e| e.status != Status::Unchanged || e.diff_pixels.unwrap_or(0) == 0);
        let off = if all_hidden { " class=\"off\"" } else { "" };
        let with_diffs = if status == Status::Unchanged {
            result
                .results
                .iter()
                .filter(|e| e.status == Status::Unchanged && e.diff_pixels.unwrap_or(0) > 0)
                .count()
        } else {
            0
        };
        let suffix = if with_diffs > 0 {
            format!(" <small>({with_diffs} with diffs)</small>")
        } else {
            String::new()
        };
        let _ = write!(
            out,
            "<a href=\"#{name}\" data-status=\"{name}\"{off}><b>{count}</b>{name}{suffix}</a>"
        );
    }
    out.push_str("</nav>");

    for status in [
        Status::Changed,
        Status::Added,
        Status::Removed,
        Status::Mismatch,
        Status::Error,
        Status::Unchanged,
    ] {
        let mut entries: Vec<&Entry> = result
            .results
            .iter()
            .filter(|e| e.status == status)
            .collect();
        if entries.is_empty() {
            continue;
        }
        entries.sort_by(|a, b| {
            b.diff_pixels
                .unwrap_or(0)
                .cmp(&a.diff_pixels.unwrap_or(0))
                .then_with(|| a.id.cmp(&b.id))
        });
        let name = status_name(status);
        let _ = write!(
            out,
            "<section id=\"{name}\"><h2>{name} ({})</h2>",
            entries.len()
        );
        for entry in entries {
            render_entry(&mut out, entry);
        }
        out.push_str("</section>");
    }
    let _ = write!(out, "<script>{SCRIPT}</script></body></html>");
    out
}

fn render_entry(out: &mut String, entry: &Entry) {
    let name = status_name(entry.status);
    let hidden = if entry.status == Status::Unchanged && entry.diff_pixels.unwrap_or(0) == 0 {
        " hidden"
    } else {
        ""
    };
    let _ = write!(
        out,
        "<article class=\"entry{hidden}\" data-status=\"{name}\"><header><span class=\"status {name}\">{name}</span><code>{}</code>",
        escape(&entry.id)
    );
    if let (Some(ratio), Some(pixels)) = (entry.diff_ratio, entry.diff_pixels) {
        let _ = write!(
            out,
            "<span class=\"detail\">{:.3}% · {pixels} px</span>",
            ratio * 100.0
        );
    }
    if let Some(msg) = &entry.message {
        let _ = write!(out, "<span class=\"detail\">{}</span>", escape(msg));
    }
    if let Some(at) = &entry.captured_at {
        let _ = write!(
            out,
            "<span class=\"detail captured\" data-utc=\"{0}\">captured {0}</span>",
            escape(at)
        );
    }
    out.push_str("</header>");
    if shows_images(entry) {
        out.push_str("<div class=\"images\">");
        let copied = |side: &str| format!("{side}/{}.png", entry.id);
        let images = [
            (
                "baseline",
                entry.baseline.as_ref().map(|_| copied("baseline")),
            ),
            ("current", entry.current.as_ref().map(|_| copied("current"))),
            ("diff", entry.diff.as_deref().map(report_relative)),
        ];
        for (label, path) in images {
            if let Some(p) = path {
                let _ = write!(
                    out,
                    "<figure><figcaption>{label}</figcaption><a href=\"{0}\"><img src=\"{0}\" alt=\"{label}\" loading=\"lazy\"></a></figure>",
                    escape(&p)
                );
            }
        }
        out.push_str("</div>");
    }
    out.push_str("</article>");
}

fn report_relative(root_relative: &str) -> String {
    root_relative
        .strip_prefix("report/")
        .unwrap_or(root_relative)
        .to_string()
}

fn status_name(status: Status) -> &'static str {
    match status {
        Status::Changed => "changed",
        Status::Added => "added",
        Status::Removed => "removed",
        Status::Mismatch => "mismatch",
        Status::Error => "error",
        Status::Unchanged => "unchanged",
    }
}

fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use mitame_contract::{Summary, SCHEMA_VERSION};

    #[test]
    fn renders_entries_with_paths_relative_to_report_dir() {
        let result = ResultFile {
            schema_version: SCHEMA_VERSION,
            profile: "default".into(),
            summary: Summary {
                changed: 1,
                unchanged: 1,
                ..Default::default()
            },
            results: vec![
                Entry {
                    id: "flutter/a/x__theme=dark".into(),
                    status: Status::Changed,
                    diff_ratio: Some(0.25),
                    diff_pixels: Some(4),
                    baseline: Some("baseline/default/flutter/a/x__theme=dark.png".into()),
                    current: Some("current/default/flutter/a/x__theme=dark.png".into()),
                    diff: Some("report/diff/flutter/a/x__theme=dark.png".into()),
                    message: None,
                    captured_at: Some("2026-09-21T10:00:00Z".into()),
                },
                Entry {
                    id: "flutter/a/<y>".into(),
                    status: Status::Unchanged,
                    diff_ratio: Some(0.0),
                    diff_pixels: Some(0),
                    baseline: None,
                    current: None,
                    diff: None,
                    message: None,
                    captured_at: None,
                },
            ],
        };
        let html = render(&result);
        assert!(html.contains("src=\"baseline/flutter/a/x__theme=dark.png\""));
        assert!(html.contains("src=\"current/flutter/a/x__theme=dark.png\""));
        assert!(!html.contains("../"));
        assert!(html.contains("src=\"diff/flutter/a/x__theme=dark.png\""));
        assert!(html.contains("25.000% · 4 px"));
        assert!(html.contains("0.000% · 0 px"));
        assert!(html.contains("captured 2026-09-21T10:00:00Z"));
        assert!(html.contains("flutter/a/&lt;y&gt;"));
        assert!(!html.contains("<y>"));
    }
}
