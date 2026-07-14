use rayon::prelude::*;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Serialize, Clone)]
pub struct Node {
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    #[serde(skip)]
    pub children: Vec<Node>,
}

#[derive(Default)]
pub struct Progress {
    pub files: AtomicU64,
    pub bytes: AtomicU64,
    pub denied: AtomicU64,
}

pub fn scan(path: &Path, prog: &Progress) -> Node {
    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());
    let entries = match fs::read_dir(path) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect::<Vec<_>>(),
        Err(_) => {
            prog.denied.fetch_add(1, Ordering::Relaxed);
            return Node { name, size: 0, is_dir: true, children: vec![] };
        }
    };
    let mut children: Vec<Node> = entries
        .par_iter()
        .filter_map(|entry| {
            let ft = entry.file_type().ok()?;
            if ft.is_symlink() {
                return None; // avoid cycles and double-counting
            }
            if ft.is_dir() {
                Some(scan(&entry.path(), prog))
            } else {
                let size = entry.metadata().ok().map(|m| m.len()).unwrap_or(0);
                prog.files.fetch_add(1, Ordering::Relaxed);
                prog.bytes.fetch_add(size, Ordering::Relaxed);
                Some(Node {
                    name: entry.file_name().to_string_lossy().into_owned(),
                    size,
                    is_dir: false,
                    children: vec![],
                })
            }
        })
        .collect();
    children.sort_by(|a, b| b.size.cmp(&a.size));
    let size = children.iter().map(|c| c.size).sum();
    Node { name, size, is_dir: true, children }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("big.bin"), vec![0u8; 1000]).unwrap();
        fs::write(dir.path().join("small.txt"), b"hi").unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub").join("mid.dat"), vec![0u8; 500]).unwrap();
        dir
    }

    #[test]
    fn scan_sums_sizes_recursively() {
        let dir = fixture();
        let prog = Progress::default();
        let root = scan(dir.path(), &prog);
        assert_eq!(root.size, 1502);
        assert!(root.is_dir);
        let sub = root.children.iter().find(|c| c.name == "sub").unwrap();
        assert_eq!(sub.size, 500);
        assert!(sub.is_dir);
    }

    #[test]
    fn scan_sorts_children_size_desc() {
        let dir = fixture();
        let prog = Progress::default();
        let root = scan(dir.path(), &prog);
        let names: Vec<&str> = root.children.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["big.bin", "sub", "small.txt"]);
    }

    #[test]
    fn scan_counts_progress() {
        let dir = fixture();
        let prog = Progress::default();
        scan(dir.path(), &prog);
        assert_eq!(prog.files.load(Ordering::Relaxed), 3);
        assert_eq!(prog.bytes.load(Ordering::Relaxed), 1502);
    }
}
