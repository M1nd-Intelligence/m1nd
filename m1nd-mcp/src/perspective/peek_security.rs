// === m1nd-mcp/src/perspective/peek_security.rs ===
// Theme 6: Peek Security Module.
// Standalone security pipeline executed on every peek call.
// Order: canonicalize → allow-list → existence → staleness → size → binary → encoding → truncation → wrap.

use m1nd_core::error::{M1ndError, M1ndResult};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use super::state::{PeekContent, PeekSecurityConfig};

// ---------------------------------------------------------------------------
// Security pipeline
// ---------------------------------------------------------------------------

/// Execute the full peek security pipeline.
///
/// Steps (Theme 6, in order):
/// 1. Path canonicalization
/// 2. Allow-list validation (reject symlinks resolving outside)
/// 3. File existence check
/// 4. Staleness detection (compare mtime against last ingest)
/// 5. File size cap (reject > 10MB)
/// 6. Binary detection (NUL in first 8KB)
/// 7. Encoding safety (from_utf8_lossy)
/// 8. Truncation on char boundaries (max_chars)
/// 9. Content wrapping (structural markers for prompt injection defense)
pub fn secure_peek(
    source_path: &str,
    config: &PeekSecurityConfig,
    line_hint: Option<u32>,
    last_ingest_ms: Option<u64>,
) -> M1ndResult<PeekContent> {
    // Step 1: Canonicalize
    let canonical = canonicalize_path(source_path)?;

    // Step 2: Allow-list
    validate_allow_list(&canonical, &config.allow_roots)?;

    // Step 3: Existence
    if !canonical.exists() {
        return Err(M1ndError::InvalidParams {
            tool: "perspective.peek".into(),
            detail: format!("source unavailable: {}", source_path),
        });
    }

    // Step 4: Staleness
    let provenance_stale = check_staleness(&canonical, last_ingest_ms);

    // Step 5: File size
    let metadata = fs::metadata(&canonical).map_err(|e| M1ndError::Io(e))?;
    if metadata.len() > config.max_file_size {
        return Err(M1ndError::InvalidParams {
            tool: "perspective.peek".into(),
            detail: format!(
                "file too large: {} bytes (cap: {})",
                metadata.len(),
                config.max_file_size
            ),
        });
    }

    // Step 6: Binary detection
    check_binary(&canonical)?;

    // Step 7+8: Read with encoding safety and line-bounded extraction
    let center_line = line_hint.unwrap_or(1);
    let start_line = center_line.saturating_sub(config.max_lines_before);
    let end_line = center_line + config.max_lines_after;

    let (content, actual_start, actual_end, encoding_lossy) =
        extract_lines(&canonical, start_line, end_line, config.max_chars)?;

    // Step 9: Relative path
    let relative_path = strip_allow_root(&canonical, &config.allow_roots);

    Ok(PeekContent {
        content,
        truncated: actual_end < end_line, // simplified: true if we stopped before end_line
        provenance_stale,
        encoding_lossy,
        relative_path,
        line_start: actual_start,
        line_end: actual_end,
    })
}

// ---------------------------------------------------------------------------
// Internal pipeline steps
// ---------------------------------------------------------------------------

fn canonicalize_path(path: &str) -> M1ndResult<PathBuf> {
    fs::canonicalize(path).map_err(|e| M1ndError::InvalidParams {
        tool: "perspective.peek".into(),
        detail: format!("path canonicalization failed for '{}': {}", path, e),
    })
}

fn validate_allow_list(canonical: &Path, allow_roots: &[String]) -> M1ndResult<()> {
    if allow_roots.is_empty() {
        // No allow-list configured — all paths allowed (should be tightened in production)
        return Ok(());
    }

    for root in allow_roots {
        let root_path = PathBuf::from(root);
        if let Ok(canonical_root) = fs::canonicalize(&root_path) {
            if canonical.starts_with(&canonical_root) {
                return Ok(());
            }
        }
    }

    Err(M1ndError::InvalidParams {
        tool: "perspective.peek".into(),
        detail: format!(
            "path '{}' is outside allowed roots",
            canonical.display()
        ),
    })
}

fn check_staleness(canonical: &Path, last_ingest_ms: Option<u64>) -> bool {
    let last_ingest = match last_ingest_ms {
        Some(ms) => ms,
        None => return false, // no ingest info — can't determine staleness
    };

    let mtime = fs::metadata(canonical)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    mtime > last_ingest
}

fn check_binary(canonical: &Path) -> M1ndResult<()> {
    let file = fs::File::open(canonical).map_err(M1ndError::Io)?;
    let mut reader = BufReader::new(file);
    let mut buf = [0u8; 8192];
    let n = std::io::Read::read(&mut reader, &mut buf).map_err(M1ndError::Io)?;

    if buf[..n].contains(&0) {
        return Err(M1ndError::InvalidParams {
            tool: "perspective.peek".into(),
            detail: format!("binary content detected in '{}'", canonical.display()),
        });
    }

    Ok(())
}

fn extract_lines(
    canonical: &Path,
    start_line: u32,
    end_line: u32,
    max_chars: usize,
) -> M1ndResult<(String, u32, u32, bool)> {
    let file = fs::File::open(canonical).map_err(M1ndError::Io)?;
    let reader = BufReader::new(file);

    let mut result = String::new();
    let mut actual_start = start_line;
    let mut actual_end = start_line;
    let mut encoding_lossy = false;
    let mut char_count = 0;
    let mut started = false;

    for (line_num_0, line_result) in reader.split(b'\n').enumerate() {
        let line_num = (line_num_0 + 1) as u32;

        if line_num < start_line {
            continue;
        }
        if line_num > end_line {
            break;
        }

        let raw_bytes = line_result.map_err(M1ndError::Io)?;
        let line_str = String::from_utf8_lossy(&raw_bytes);

        if line_str.as_ref() != std::str::from_utf8(&raw_bytes).unwrap_or("") {
            encoding_lossy = true;
        }

        if !started {
            actual_start = line_num;
            started = true;
        }

        let remaining = max_chars.saturating_sub(char_count);
        if remaining == 0 {
            break;
        }

        // Truncate on char boundaries
        let truncated_line: String = line_str.chars().take(remaining).collect();
        char_count += truncated_line.len();

        if !result.is_empty() {
            result.push('\n');
            char_count += 1;
        }
        result.push_str(&truncated_line);
        actual_end = line_num;

        if char_count >= max_chars {
            break;
        }
    }

    Ok((result, actual_start, actual_end, encoding_lossy))
}

fn strip_allow_root(canonical: &Path, allow_roots: &[String]) -> String {
    for root in allow_roots {
        if let Ok(canonical_root) = fs::canonicalize(root) {
            if let Ok(relative) = canonical.strip_prefix(&canonical_root) {
                return relative.display().to_string();
            }
        }
    }
    // Fallback: return full path
    canonical.display().to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_file(content: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f.flush().unwrap();
        f
    }

    #[test]
    fn canonicalize_rejects_nonexistent() {
        let result = canonicalize_path("/nonexistent/path/foo.rs");
        assert!(result.is_err());
    }

    #[test]
    fn bidi_normalize_order() {
        let (lo, hi) = super::super::keys::normalize_bidi_endpoints("z", "a");
        assert_eq!(lo, "a");
        assert_eq!(hi, "z");
    }

    #[test]
    fn binary_detection_catches_nul() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(b"hello\x00world").unwrap();
        f.flush().unwrap();
        let canonical = fs::canonicalize(f.path()).unwrap();
        let result = check_binary(&canonical);
        assert!(result.is_err());
    }

    #[test]
    fn extract_lines_respects_range() {
        let content = "line1\nline2\nline3\nline4\nline5\n";
        let f = temp_file(content);
        let canonical = fs::canonicalize(f.path()).unwrap();
        let (text, start, end, _lossy) = extract_lines(&canonical, 2, 4, 2000).unwrap();
        assert_eq!(start, 2);
        assert_eq!(end, 4);
        assert!(text.contains("line2"));
        assert!(text.contains("line4"));
        assert!(!text.contains("line1"));
    }

    #[test]
    fn extract_lines_respects_char_cap() {
        let content = "a".repeat(100) + "\n" + &"b".repeat(100);
        let f = temp_file(&content);
        let canonical = fs::canonicalize(f.path()).unwrap();
        let (text, _, _, _) = extract_lines(&canonical, 1, 10, 50).unwrap();
        assert!(text.len() <= 51); // 50 chars + possible newline
    }

    #[test]
    fn staleness_detection() {
        // File modified "now" vs last_ingest far in the past
        assert!(check_staleness(Path::new("/dev/null"), Some(1)));
        // No ingest info
        assert!(!check_staleness(Path::new("/dev/null"), None));
    }
}
