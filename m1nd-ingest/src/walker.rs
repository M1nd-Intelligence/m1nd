// === crates/m1nd-ingest/src/walker.rs ===

use m1nd_core::error::{M1ndError, M1ndResult};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// DirectoryWalker — file discovery
// FM-ING-004 fix: binary detection (NUL byte in first 8KB).
// Replaces: ingest.py CodebaseIngestor._walk_directory()
// ---------------------------------------------------------------------------

/// A discovered file with metadata.
#[derive(Clone, Debug)]
pub struct DiscoveredFile {
    pub path: PathBuf,
    pub relative_path: String,
    pub extension: Option<String>,
    pub size_bytes: u64,
    pub last_modified: f64,
    /// Number of git commits touching this file (0 if not in a git repo).
    pub commit_count: u32,
    /// Timestamp of most recent git commit for this file (0.0 if unavailable).
    pub last_commit_time: f64,
}

/// Result of directory walking including co-change commit groups.
#[derive(Clone, Debug, Default)]
pub struct WalkResult {
    pub files: Vec<DiscoveredFile>,
    /// Each inner Vec is a group of relative_paths that changed together in one commit.
    pub commit_groups: Vec<Vec<String>>,
}

/// Directory walker with skip rules and binary file detection.
/// Replaces: ingest.py SKIP_DIRS, SKIP_FILES, and walk logic
pub struct DirectoryWalker {
    skip_dirs: Vec<String>,
    skip_files: Vec<String>,
}

impl DirectoryWalker {
    pub fn new(skip_dirs: Vec<String>, skip_files: Vec<String>) -> Self {
        Self {
            skip_dirs,
            skip_files,
        }
    }

    /// Walk directory and return all non-binary, non-skipped files.
    /// FM-ING-004 fix: checks first 8KB for NUL bytes to detect binary files.
    /// Replaces: ingest.py directory walking logic
    pub fn walk(&self, root: &Path) -> M1ndResult<WalkResult> {
        use walkdir::WalkDir;

        if !root.exists() {
            return Err(M1ndError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Root directory not found: {}", root.display()),
            )));
        }

        let mut files = Vec::new();
        let root_canonical = root.canonicalize().map_err(M1ndError::Io)?;

        for entry in WalkDir::new(&root_canonical)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                if e.file_type().is_dir() {
                    let name = e.file_name().to_string_lossy();
                    // Skip hidden dirs and configured skip dirs
                    if name.starts_with('.') && name != "." {
                        return false;
                    }
                    return !self.skip_dirs.iter().any(|s| name == s.as_str());
                }
                true
            })
        {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue, // permission error, broken symlink, etc.
            };

            if !entry.file_type().is_file() {
                continue;
            }

            let file_name = entry.file_name().to_string_lossy();

            // Skip configured file patterns
            if self.skip_files.iter().any(|s| file_name == s.as_str()) {
                continue;
            }

            // Skip hidden files
            if file_name.starts_with('.') {
                continue;
            }

            let path = entry.path().to_path_buf();

            // FM-ING-004: skip binary files
            match Self::is_binary(&path) {
                Ok(true) => continue,
                Ok(false) => {}
                Err(_) => continue, // can't read -> skip
            }

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            let relative_path = path
                .strip_prefix(&root_canonical)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            let extension = path.extension().map(|e| e.to_string_lossy().to_string());

            let last_modified = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0);

            files.push(DiscoveredFile {
                path,
                relative_path,
                extension,
                size_bytes: metadata.len(),
                last_modified,
                commit_count: 0,
                last_commit_time: 0.0,
            });
        }

        // Enrich with git history if available, and collect commit groups
        let commit_groups = Self::enrich_with_git(&root_canonical, &mut files);

        Ok(WalkResult {
            files,
            commit_groups,
        })
    }

    /// Enrich discovered files with git history (commit count + last commit time).
    /// Runs `git log --format='%at' --name-only` once and distributes to files.
    /// Also collects commit groups: files that changed together in the same commit.
    /// Gracefully returns empty groups if not in a git repo.
    fn enrich_with_git(root: &Path, files: &mut [DiscoveredFile]) -> Vec<Vec<String>> {
        use std::collections::HashMap;
        use std::process::Command;

        // Run git log to get all commits with timestamps and affected files
        let output = match Command::new("git")
            .args(["log", "--format=%at", "--name-only", "--diff-filter=ACDMR"])
            .current_dir(root)
            .output()
        {
            Ok(o) if o.status.success() => o,
            _ => return Vec::new(), // Not a git repo or git not available
        };

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse: alternating timestamp lines and file path lines
        let mut file_stats: HashMap<String, (u32, f64)> = HashMap::new(); // path -> (count, last_time)
        let mut current_timestamp = 0.0f64;
        let mut commit_groups: Vec<Vec<String>> = Vec::new();
        let mut current_group: Vec<String> = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                // Empty line separates commits; flush current group
                if current_group.len() >= 2 {
                    commit_groups.push(std::mem::take(&mut current_group));
                } else {
                    current_group.clear();
                }
                continue;
            }
            // Is this a timestamp line? (all digits)
            if line.chars().all(|c| c.is_ascii_digit()) && line.len() >= 8 {
                // New commit: flush previous group if any
                if current_group.len() >= 2 {
                    commit_groups.push(std::mem::take(&mut current_group));
                } else {
                    current_group.clear();
                }
                current_timestamp = line.parse::<f64>().unwrap_or(0.0);
            } else if current_timestamp > 0.0 {
                // File path line
                let entry = file_stats.entry(line.to_string()).or_insert((0, 0.0));
                entry.0 += 1; // commit count
                if current_timestamp > entry.1 {
                    entry.1 = current_timestamp; // latest commit time
                }
                current_group.push(line.to_string());
            }
        }
        // Flush final group
        if current_group.len() >= 2 {
            commit_groups.push(current_group);
        }

        // Apply to discovered files
        for file in files.iter_mut() {
            if let Some(&(count, last_time)) = file_stats.get(&file.relative_path) {
                file.commit_count = count;
                file.last_commit_time = last_time;
                // Override last_modified with git time if available
                if last_time > 0.0 {
                    file.last_modified = last_time;
                }
            }
        }

        commit_groups
    }

    /// Check if a file is binary (NUL byte in first 8KB).
    /// FM-ING-004 fix.
    pub fn is_binary(path: &Path) -> M1ndResult<bool> {
        use std::io::Read;
        let mut file = std::fs::File::open(path).map_err(M1ndError::Io)?;
        let mut buf = [0u8; 8192];
        let n = file.read(&mut buf).map_err(M1ndError::Io)?;
        Ok(buf[..n].contains(&0))
    }
}
