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
.images figure { cursor: zoom-in; }
.entry.filtered { display: none; }
.toolbar { display: flex; flex-wrap: wrap; gap: 8px 16px; align-items: center; margin-bottom: 16px; }
.toolbar input[type=search] { padding: 6px 10px; border: 1px solid var(--line); border-radius: 6px; background: var(--card); color: var(--fg); min-width: 260px; }
.toolbar kbd { font: 11px/1 ui-monospace, monospace; padding: 2px 5px; border: 1px solid var(--line); border-radius: 4px; color: var(--muted); }
dialog#viewer { width: 100vw; height: 100vh; max-width: none; max-height: none; margin: 0; padding: 0; border: 0; background: var(--bg); color: var(--fg); }
dialog#viewer::backdrop { background: #000c; }
#viewer .bar { display: flex; flex-wrap: wrap; gap: 8px 16px; align-items: center; padding: 10px 16px; border-bottom: 1px solid var(--line); }
#viewer .bar code { font-size: 13px; }
#viewer .bar button { padding: 4px 10px; border: 1px solid var(--line); border-radius: 6px; background: var(--card); color: var(--fg); cursor: pointer; }
#viewer .bar button.on { border-color: var(--fg); font-weight: 600; }
#viewer .bar label { color: var(--muted); font-size: 12px; display: inline-flex; gap: 6px; align-items: center; }
#viewer .stage { position: relative; height: calc(100vh - 53px); overflow: auto; padding: 16px; }
#viewer .stack { position: relative; display: inline-block; }
#viewer .stack img { display: block; max-width: none; border: 1px solid var(--line); }
#viewer .stack img.over { position: absolute; left: 0; top: 0; }
"#;

const SCRIPT: &str = r#"
function localize(el, prefix) {
  var d = new Date(el.dataset.utc.replace(' UTC', 'Z').replace(' ', 'T'));
  if (!isNaN(d)) { el.textContent = prefix + d.toLocaleString(); el.title = el.dataset.utc; }
}
document.querySelectorAll('.generated').forEach(function (el) { localize(el, ''); });
document.querySelectorAll('.captured').forEach(function (el) { localize(el, 'captured '); });
var filter = document.getElementById('filter');
filter.addEventListener('input', function () {
  var q = filter.value.trim().toLowerCase();
  document.querySelectorAll('.entry').forEach(function (el) {
    el.classList.toggle('filtered', q !== '' && el.dataset.id.toLowerCase().indexOf(q) < 0);
  });
});
var viewer = document.getElementById('viewer');
var stage = viewer.querySelector('.stage');
var base = document.getElementById('v-base');
var over = document.getElementById('v-over');
var state = { entry: null, mode: 'current', zoom: 'fit', opacity: 0.5 };
function srcFor(mode) { return state.entry.dataset[mode] || ''; }
function applyZoom(img) {
  if (state.zoom === 'fit') { img.style.width = ''; img.style.maxWidth = '100%'; return; }
  img.style.maxWidth = 'none';
  img.style.width = (img.naturalWidth * Number(state.zoom)) + 'px';
}
function paint() {
  var e = state.entry;
  if (!e) { return; }
  viewer.querySelector('code').textContent = e.dataset.id;
  viewer.querySelectorAll('button[data-mode]').forEach(function (b) {
    b.classList.toggle('on', b.dataset.mode === state.mode);
    b.disabled = b.dataset.mode === 'onion' ? !(e.dataset.baseline && e.dataset.current) : !e.dataset[b.dataset.mode];
  });
  viewer.querySelectorAll('button[data-zoom]').forEach(function (b) { b.classList.toggle('on', b.dataset.zoom === state.zoom); });
  var onion = state.mode === 'onion';
  base.src = onion ? srcFor('baseline') : srcFor(state.mode);
  over.src = onion ? srcFor('current') : '';
  over.style.display = onion ? 'block' : 'none';
  over.style.opacity = state.opacity;
  base.onload = function () { applyZoom(base); if (onion) { over.style.width = base.style.width; over.style.maxWidth = base.style.maxWidth; } };
  if (base.complete) { base.onload(); }
}
function openViewer(entry, mode) {
  state.entry = entry;
  state.mode = mode || 'current';
  if (state.mode === 'diff' && !entry.dataset.diff) { state.mode = 'current'; }
  paint();
  if (!viewer.open) { viewer.showModal(); }
  history.replaceState(null, '', '#view=' + encodeURIComponent(entry.dataset.id));
}
function closeViewer() { viewer.close(); history.replaceState(null, '', location.pathname); }
function step(delta) {
  var visible = Array.prototype.filter.call(document.querySelectorAll('.entry'), function (el) {
    return !el.classList.contains('hidden') && !el.classList.contains('filtered') && (el.dataset.baseline || el.dataset.current);
  });
  var i = visible.indexOf(state.entry);
  var next = visible[i + delta];
  if (next) { openViewer(next, state.mode); }
}
document.querySelectorAll('.images figure').forEach(function (fig) {
  fig.addEventListener('click', function (e) {
    e.preventDefault();
    openViewer(fig.closest('.entry'), fig.dataset.mode);
  });
});
viewer.querySelectorAll('button[data-mode]').forEach(function (b) {
  b.addEventListener('click', function () { state.mode = b.dataset.mode; paint(); });
});
viewer.querySelectorAll('button[data-zoom]').forEach(function (b) {
  b.addEventListener('click', function () { state.zoom = b.dataset.zoom; paint(); });
});
viewer.querySelector('input[type=range]').addEventListener('input', function (ev) {
  state.opacity = Number(ev.target.value); over.style.opacity = state.opacity;
});
viewer.querySelector('button.close').addEventListener('click', closeViewer);
viewer.addEventListener('cancel', function (e) { e.preventDefault(); closeViewer(); });
document.addEventListener('keydown', function (e) {
  if (!viewer.open) { return; }
  var modes = { '1': 'baseline', '2': 'current', '3': 'diff', '4': 'onion' };
  if (modes[e.key]) { state.mode = modes[e.key]; paint(); }
  else if (e.key === 'ArrowRight' || e.key === 'j') { step(1); }
  else if (e.key === 'ArrowLeft' || e.key === 'k') { step(-1); }
});
var hash = location.hash.match(/^#view=(.+)$/);
if (hash) {
  var target = document.querySelector('.entry[data-id="' + decodeURIComponent(hash[1]).replace(/"/g, '\"') + '"]');
  if (target) { openViewer(target, 'current'); }
}
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
        "<h1>mitame report</h1><div class=\"meta\">profile <code>{}</code> · {} screenshots · generated <span class=\"generated\" data-utc=\"{2}\">{2}</span>{3}</div>",
        escape(&result.profile),
        result.results.len(),
        escape(generated_at),
        result
            .mitame_version
            .as_deref()
            .map(|v| format!(" · mitame {}", escape(v)))
            .unwrap_or_default()
    );
    out.push_str("<div class=\"toolbar\"><input id=\"filter\" type=\"search\" placeholder=\"filter by id\" autocomplete=\"off\"><span class=\"detail\">click an image to open the viewer · <kbd>1</kbd> baseline <kbd>2</kbd> current <kbd>3</kbd> diff <kbd>4</kbd> onion · <kbd>←</kbd> <kbd>→</kbd> entries · <kbd>Esc</kbd> close</span></div>");
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
    out.push_str(
        "<dialog id=\"viewer\"><div class=\"bar\"><code></code><button data-mode=\"baseline\">baseline</button><button data-mode=\"current\">current</button><button data-mode=\"diff\">diff</button><button data-mode=\"onion\">onion</button><label>opacity <input type=\"range\" min=\"0\" max=\"1\" step=\"0.05\" value=\"0.5\"></label><button data-zoom=\"fit\">fit</button><button data-zoom=\"1\">100%</button><button data-zoom=\"2\">200%</button><button class=\"close\">close</button></div><div class=\"stage\"><div class=\"stack\"><img id=\"v-base\" alt=\"\"><img id=\"v-over\" class=\"over\" alt=\"\"></div></div></dialog>",
    );
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
    let copied = |side: &str| format!("{side}/{}.png", entry.id);
    let baseline = entry.baseline.as_ref().map(|_| copied("baseline"));
    let current = entry.current.as_ref().map(|_| copied("current"));
    let diff = entry.diff.as_deref().map(report_relative);
    let attr = |v: &Option<String>| v.as_deref().map(escape).unwrap_or_default();
    let _ = write!(
        out,
        "<article class=\"entry{hidden}\" data-status=\"{name}\" data-id=\"{}\" data-baseline=\"{}\" data-current=\"{}\" data-diff=\"{}\"><header><span class=\"status {name}\">{name}</span><code>{}</code>",
        escape(&entry.id),
        if shows_images(entry) { attr(&baseline) } else { String::new() },
        if shows_images(entry) { attr(&current) } else { String::new() },
        attr(&diff),
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
        let images = [("baseline", baseline), ("current", current), ("diff", diff)];
        for (label, path) in images {
            if let Some(p) = path {
                let _ = write!(
                    out,
                    "<figure data-mode=\"{label}\"><figcaption>{label}</figcaption><a href=\"{0}\"><img src=\"{0}\" alt=\"{label}\" loading=\"lazy\"></a></figure>",
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
            mitame_version: None,
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
        assert!(html.contains("id=\"filter\""));
        assert!(html.contains("<dialog id=\"viewer\""));
        assert!(html.contains("data-id=\"flutter/a/x__theme=dark\" data-baseline=\"baseline/flutter/a/x__theme=dark.png\" data-current=\"current/flutter/a/x__theme=dark.png\" data-diff=\"diff/flutter/a/x__theme=dark.png\""));
        assert!(html.contains(
            "data-id=\"flutter/a/&lt;y&gt;\" data-baseline=\"\" data-current=\"\" data-diff=\"\""
        ));
        assert!(html.contains("flutter/a/&lt;y&gt;"));
        assert!(!html.contains("<y>"));
    }
}
