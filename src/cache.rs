use std::fs;
use std::path::PathBuf;

use crate::models::Map;

fn cache_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("rhythia-rp-finder")
        .join("maps.json")
}

pub fn load() -> Option<Vec<Map>> {
    let data = fs::read_to_string(cache_path()).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save(maps: &[Map]) {
    let path = cache_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(data) = serde_json::to_string(maps) {
        let _ = fs::write(path, data);
    }
}
