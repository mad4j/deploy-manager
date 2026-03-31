use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::sync::Mutex;

/// Manages per-action progress bars.
pub struct ProgressTracker {
    multi: MultiProgress,
    bars: Mutex<HashMap<String, ProgressBar>>,
    overall: ProgressBar,
    total: u64,
}

impl ProgressTracker {
    /// Create a new tracker for `total` actions.
    pub fn new(total: usize) -> Self {
        let multi = MultiProgress::new();

        let overall_style = ProgressStyle::with_template(
            "{spinner:.cyan} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} actions {msg}",
        )
        .unwrap()
        .progress_chars("##-");

        let overall = multi.add(ProgressBar::new(total as u64));
        overall.set_style(overall_style);
        overall.set_message("running");

        Self {
            multi,
            bars: Mutex::new(HashMap::new()),
            overall,
            total: total as u64,
        }
    }

    /// Register an action as started; creates and displays its progress bar.
    pub fn start_action(&self, name: &str) {
        let style = ProgressStyle::with_template(
            "  {spinner:.green} {msg}",
        )
        .unwrap();

        let bar = self.multi.add(ProgressBar::new_spinner());
        bar.set_style(style);
        bar.set_message(format!("▶  {}", name));
        bar.enable_steady_tick(std::time::Duration::from_millis(100));

        self.bars.lock().unwrap().insert(name.to_string(), bar);
    }

    /// Mark an action as finished.  `success` controls the display symbol.
    pub fn finish_action(&self, name: &str, success: bool) {
        let bars = self.bars.lock().unwrap();
        if let Some(bar) = bars.get(name) {
            let symbol = if success { "✔" } else { "✘" };
            bar.finish_with_message(format!("{} {}", symbol, name));
        }
        drop(bars);
        self.overall.inc(1);
        if self.overall.position() == self.total {
            self.overall.finish_with_message("done");
        }
    }
}
