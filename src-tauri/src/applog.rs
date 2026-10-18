// Minimal application logger.
//
// The app previously initialised no `log` backend at all, so every
// `log::info!` / `warn!` / `error!` in the codebase - including the
// diagnostics that explain a failing local-ASR startup - was silently
// discarded. That made runtime failures indistinguishable from each other.
//
// Writes each record to stderr (so `npm run dev` shows it) and appends to
// %LOCALAPPDATA%\Fluence\fluence.log (so a packaged build can be diagnosed
// after the fact). No new dependency: uses `log` and std only.
//
// Rotates by truncation once the file passes MAX_BYTES, keeping the newest
// records, so a long-running session cannot grow it without bound.

use log::{Level, LevelFilter, Metadata, Record};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Cap the on-disk log at ~2 MB.
const MAX_BYTES: u64 = 2 * 1024 * 1024;

struct Sink {
    file: Option<Mutex<File>>,
}

static SINK: OnceLock<Sink> = OnceLock::new();

impl log::Log for Sink {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = format!(
            "[{}] {:<5} {}: {}\n",
            timestamp(),
            record.level(),
            record.target(),
            record.args()
        );

        // stderr: inherited by the sidecars, so spawn failures surface live.
        let _ = std::io::stderr().write_all(line.as_bytes());

        if let Some(file) = &self.file {
            if let Ok(mut f) = file.lock() {
                let _ = f.write_all(line.as_bytes());
                let _ = f.flush();
            }
        }
    }

    fn flush(&self) {
        if let Some(file) = &self.file {
            if let Ok(mut f) = file.lock() {
                let _ = f.flush();
            }
        }
    }
}

/// UTC `HH:MM:SS`, computed without pulling in a date library.
fn timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let day = secs % 86_400;
    format!("{:02}:{:02}:{:02}", day / 3600, (day % 3600) / 60, day % 60)
}

fn log_path() -> Option<std::path::PathBuf> {
    let base = dirs::data_local_dir()?;
    let dir = base.join("Fluence");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir.join("fluence.log"))
}

/// Install the global logger. Safe to call once; later calls are no-ops.
///
/// `FLUENCE_LOG` overrides the level, e.g. `FLUENCE_LOG=debug`.
pub fn init() {
    let level = std::env::var("FLUENCE_LOG")
        .ok()
        .and_then(|v| v.parse::<LevelFilter>().ok())
        .unwrap_or(LevelFilter::Info);

    let file = log_path().and_then(|p| {
        // Truncate rather than append once oversized, keeping the log bounded.
        let needs_reset = std::fs::metadata(&p)
            .map(|m| m.len() > MAX_BYTES)
            .unwrap_or(false);
        OpenOptions::new()
            .create(true)
            .write(true)
            .append(!needs_reset)
            .truncate(needs_reset)
            .open(&p)
            .ok()
    });

    let sink = Sink {
        file: file.map(Mutex::new),
    };

    if SINK.set(sink).is_ok() {
        let _ = log::set_boxed_logger(Box::new(SINK.get().expect("sink just set")));
        log::set_max_level(level);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Proves a record actually reaches the on-disk sink. Guards the failure
    /// mode where init() runs (file is created) but writes never land, which
    /// leaves the log empty and is indistinguishable from "nothing logged".
    #[test]
    fn record_reaches_the_file_sink() {
        init();
        log::set_max_level(LevelFilter::Trace);
        log::error!("applog self-test sentinel 12345");

        let path = log_path().expect("log path");
        let contents = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(
            contents.contains("applog self-test sentinel 12345"),
            "log record did not reach {}\n--- file ---\n{}",
            path.display(),
            contents
        );
    }
}
