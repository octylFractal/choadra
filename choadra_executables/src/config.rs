use std::path::PathBuf;

use directories_next::ProjectDirs;
use once_cell::sync::Lazy;

pub static APP_CONFIG: Lazy<PathBuf> = Lazy::new(|| {
    ProjectDirs::from("net.octyl", "choadra", "choadra_executables")
        .expect("Failed to find project dirs")
        .config_dir()
        .to_owned()
});
