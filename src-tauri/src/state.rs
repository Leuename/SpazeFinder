use crate::scan::Node;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct ScanResult {
    pub root_path: PathBuf,
    pub root: Node,
}

#[derive(Default)]
pub struct AppState {
    pub tree: Mutex<Option<ScanResult>>,
}
