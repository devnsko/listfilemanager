use serde::Serialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Serialize)]
struct MountPoint {
    path: String,
    label: String,
}

#[derive(Serialize)]
struct FileEntry {
    // absolute path
    path: String,
    // path relative to the provided root
    relative_path: String,
    size: u64,
}

fn canonical_within(root: &Path, candidate: &Path) -> Result<PathBuf, String> {
    let root = root
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize root: {e}"))?;
    let cand = candidate
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize path: {e}"))?;
    if cand.starts_with(&root) {
        Ok(cand)
    } else {
        Err("Path escapes selected root".to_string())
    }
}

#[tauri::command]
fn list_candidate_mounts() -> Result<Vec<MountPoint>, String> {
    let mut mounts: Vec<MountPoint> = Vec::new();

    #[cfg(target_os = "linux")]
    {
        let user = env::var("USER").unwrap_or_default();
        let candidates: [&str; 3] = [
            "/media",
            "/run/media", // usually /run/media/$USER/<label>
            "/mnt",
        ];

        for base in candidates.iter() {
            let base_path = if *base == "/run/media" && !user.is_empty() {
                Path::new(base).join(&user)
            } else {
                PathBuf::from(base)
            };

            if base_path.is_dir() {
                if let Ok(entries) = fs::read_dir(&base_path) {
                    for e in entries.flatten() {
                        let p = e.path();
                        if p.is_dir() {
                            let label = p
                                .file_name()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_else(|| p.display().to_string());
                            mounts.push(MountPoint {
                                path: p.display().to_string(),
                                label,
                            });
                        }
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        // On other OSes, just return empty list for now.
    }

    Ok(mounts)
}

#[tauri::command]
fn list_files(root: &str) -> Result<Vec<FileEntry>, String> {
    let root_path = PathBuf::from(root);
    let root_canon = root_path
        .canonicalize()
        .map_err(|e| format!("Invalid root: {e}"))?;

    let mut result: Vec<FileEntry> = Vec::new();
    let mut stack: Vec<PathBuf> = vec![root_canon.clone()];

    while let Some(dir) = stack.pop() {
        let read_dir = fs::read_dir(&dir).map_err(|e| format!("Failed to read dir {}: {e}", dir.display()))?;
        for entry in read_dir.flatten() {
            let p = entry.path();
            let rel = p.strip_prefix(&root_canon).unwrap_or(&p);
            if p.is_dir() {
                stack.push(p);
            } else if p.is_file() {
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                result.push(FileEntry {
                    path: p.display().to_string(),
                    relative_path: rel.display().to_string(),
                    size,
                });
            }
        }
    }
    // Sort by relative_path for stable display
    result.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(result)
}

#[tauri::command]
fn rename_file(root: &str, relative_path: &str, new_name: &str) -> Result<(), String> {
    let root = PathBuf::from(root);
    let abs_path = canonical_within(&root, &root.join(relative_path))?;
    if !abs_path.is_file() {
        return Err("Target is not a file".into());
    }
    let parent = abs_path.parent().ok_or("File has no parent directory")?;
    let new_path = parent.join(new_name);
    // Ensure destination stays within root
    let _ = parent; // parent is already within root since abs_path is
    // For rename, we can't canonicalize non-existent dest; instead, check that parent starts with root
    let root_canon = PathBuf::from(root).canonicalize().map_err(|e| format!("Invalid root: {e}"))?;
    if !parent.starts_with(&root_canon) {
        return Err("Destination escapes root".into());
    }
    fs::rename(&abs_path, &new_path).map_err(|e| format!("Rename failed: {e}"))?;
    Ok(())
}

#[tauri::command]
fn delete_file(root: &str, relative_path: &str) -> Result<(), String> {
    let root = PathBuf::from(root);
    let abs_path = canonical_within(&root, &root.join(relative_path))?;
    if abs_path.is_file() {
        fs::remove_file(&abs_path).map_err(|e| format!("Delete failed: {e}"))?;
        Ok(())
    } else {
        Err("Only files can be deleted with this action".into())
    }
}

#[tauri::command]
fn move_file(root: &str, from_relative: &str, to_relative_dir: &str, create_dir: bool) -> Result<(), String> {
    let root = PathBuf::from(root);
    let root_canon = root.canonicalize().map_err(|e| format!("Invalid root: {e}"))?;
    
    let src_abs = canonical_within(&root, &root.join(from_relative))?;
    if !src_abs.is_file() {
        return Err("Source is not a file".into());
    }
    
    // Handle empty or root-relative paths
    let to_relative_dir = to_relative_dir.trim();
    let dest_dir = if to_relative_dir.is_empty() || to_relative_dir == "/" || to_relative_dir == "." {
        // Move to root directory
        root_canon.clone()
    } else {
        // Remove leading slash if present
        let clean_path = to_relative_dir.trim_start_matches('/');
        root.join(clean_path)
    };
    
    // Validate destination is within root
    let dest_canon = if dest_dir.exists() {
        canonical_within(&root, &dest_dir)?
    } else {
        // For non-existent paths, validate the parent
        if create_dir {
            let parent = dest_dir.parent().unwrap_or(&root);
            if parent.exists() {
                canonical_within(&root, parent)?;
            }
            fs::create_dir_all(&dest_dir).map_err(|e| format!("Failed to create dir: {e}"))?;
            dest_dir.canonicalize().map_err(|e| format!("Failed to validate created dir: {e}"))?
        } else {
            return Err("Destination directory does not exist".into());
        }
    };
    
    let file_name = src_abs
        .file_name()
        .ok_or("Source file has no name")?
        .to_os_string();
    let dest_abs = dest_canon.join(file_name);
    
    fs::rename(&src_abs, &dest_abs).map_err(|e| format!("Move failed: {e}"))?;
    Ok(())
}

#[tauri::command]
fn create_folder(root: &str, relative_dir: &str) -> Result<(), String> {
    let root = PathBuf::from(root);
    let target = root.join(relative_dir);
    // Ensure target is within root (can't canonicalize new path before it's created, so validate parent)
    let parent = target.parent().unwrap_or(&root);
    let _ = canonical_within(&root, parent)?;
    fs::create_dir_all(&target).map_err(|e| format!("Create folder failed: {e}"))?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            list_candidate_mounts,
            list_files,
            rename_file,
            delete_file,
            move_file,
            create_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
