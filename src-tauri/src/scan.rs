use rayon::prelude::*;
use serde::Serialize;
use std::cmp::Reverse;
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
    children.sort_by_key(|c| Reverse(c.size));
    let size = children.iter().map(|c| c.size).sum();
    Node { name, size, is_dir: true, children }
}

pub fn find_node<'a>(node: &'a Node, rel: &[String]) -> Option<&'a Node> {
    match rel.split_first() {
        None => Some(node),
        Some((head, tail)) => {
            node.children.iter().find(|c| c.name == *head).and_then(|c| find_node(c, tail))
        }
    }
}

pub fn remove_node(node: &mut Node, rel: &[String]) -> Option<Node> {
    let (head, tail) = rel.split_first()?;
    if tail.is_empty() {
        let idx = node.children.iter().position(|c| c.name == *head)?;
        let removed = node.children.remove(idx);
        node.size -= removed.size;
        Some(removed)
    } else {
        let child = node.children.iter_mut().find(|c| c.name == *head)?;
        let removed = remove_node(child, tail)?;
        node.size -= removed.size;
        Some(removed)
    }
}

pub fn rename_node(node: &mut Node, rel: &[String], new_name: &str) -> bool {
    let Some((head, tail)) = rel.split_first() else { return false };
    let Some(child) = node.children.iter_mut().find(|c| c.name == *head) else { return false };
    if tail.is_empty() {
        child.name = new_name.to_string();
        true
    } else {
        rename_node(child, tail, new_name)
    }
}

pub fn insert_node(node: &mut Node, dest_rel: &[String], new: Node) -> bool {
    node.size += new.size;
    match dest_rel.split_first() {
        None => {
            node.children.push(new);
            node.children.sort_by_key(|c| Reverse(c.size));
            true
        }
        Some((head, tail)) => {
            match node.children.iter_mut().find(|c| c.name == *head) {
                Some(child) => insert_node(child, tail, new),
                None => false, // caller guarantees dest exists via find_node
            }
        }
    }
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

    fn scanned() -> (tempfile::TempDir, Node) {
        let dir = fixture();
        let prog = Progress::default();
        let root = scan(dir.path(), &prog);
        (dir, root)
    }

    fn rel(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn find_node_walks_path() {
        let (_d, root) = scanned();
        assert_eq!(find_node(&root, &[]).unwrap().size, 1502);
        assert_eq!(find_node(&root, &rel(&["sub", "mid.dat"])).unwrap().size, 500);
        assert!(find_node(&root, &rel(&["nope"])).is_none());
    }

    #[test]
    fn remove_node_updates_ancestor_sizes() {
        let (_d, mut root) = scanned();
        let removed = remove_node(&mut root, &rel(&["sub", "mid.dat"])).unwrap();
        assert_eq!(removed.size, 500);
        assert_eq!(root.size, 1002);
        assert_eq!(find_node(&root, &rel(&["sub"])).unwrap().size, 0);
    }

    #[test]
    fn rename_node_keeps_size() {
        let (_d, mut root) = scanned();
        assert!(rename_node(&mut root, &rel(&["big.bin"]), "huge.bin"));
        assert!(find_node(&root, &rel(&["huge.bin"])).is_some());
        assert_eq!(root.size, 1502);
    }

    #[test]
    fn move_via_remove_then_insert() {
        let (_d, mut root) = scanned();
        let node = remove_node(&mut root, &rel(&["big.bin"])).unwrap();
        assert!(insert_node(&mut root, &rel(&["sub"]), node));
        assert_eq!(root.size, 1502);
        let sub = find_node(&root, &rel(&["sub"])).unwrap();
        assert_eq!(sub.size, 1500);
        assert_eq!(sub.children[0].name, "big.bin"); // re-sorted, biggest first
    }
}
