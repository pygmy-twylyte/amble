use std::env;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

/// Cached path to the directory containing the engine's runtime data files.
static DATA_ROOT: LazyLock<PathBuf> = LazyLock::new(detect_data_root);

/// Construct a data path relative to the resolved data root.
pub fn data_path(relative: impl AsRef<Path>) -> PathBuf {
    DATA_ROOT.join(relative)
}

/// Resolve the most likely location of the runtime data directory.
fn detect_data_root() -> PathBuf {
    let mut candidates = Vec::new();

    // Common layouts: workspace root and flattened `data/`.
    candidates.push(PathBuf::from("amble_engine/data"));
    candidates.push(PathBuf::from("data"));

    if let Ok(exe_path) = env::current_exe()
        && let Some(dir) = exe_path.parent() {
            candidates.push(dir.join("amble_engine/data"));
            candidates.push(dir.join("data"));

            if let Some(parent) = dir.parent() {
                candidates.push(parent.join("amble_engine/data"));
                candidates.push(parent.join("data"));
            }
        }

    candidates
        .into_iter()
        .find(|candidate| candidate.is_dir())
        .unwrap_or_else(|| PathBuf::from("amble_engine/data"))
}
