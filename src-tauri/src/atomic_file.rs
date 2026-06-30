use std::fs;
use std::path::Path;

/// Atomically replaces `target_path` with the file at `tmp_path`.
///
/// On Unix-like systems, `std::fs::rename` atomically replaces the destination
/// even when it already exists. On Windows, Rust's standard library calls
/// `MoveFileExW` with `MOVEFILE_REPLACE_EXISTING`, which gives the same
/// cross-platform replace semantics.
///
/// Both paths must be on the same filesystem; callers should place the temp
/// file in the same directory as the target.
pub fn replace_file_atomically(tmp_path: &Path, target_path: &Path) -> Result<(), String> {
    fs::rename(tmp_path, target_path).map_err(|error| {
        format!(
            "atomically replace {} with {}: {error}",
            target_path.display(),
            tmp_path.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_file_atomically_creates_target() {
        let dir = tempfile::tempdir().unwrap();
        let tmp = dir.path().join("data.tmp");
        let target = dir.path().join("data.final");

        fs::write(&tmp, b"hello").unwrap();
        replace_file_atomically(&tmp, &target).unwrap();

        assert_eq!(fs::read(&target).unwrap(), b"hello");
        assert!(!tmp.exists(), "temp file should be gone after replace");
    }

    #[test]
    fn replace_file_atomically_replaces_existing_target() {
        let dir = tempfile::tempdir().unwrap();
        let tmp = dir.path().join("data.tmp");
        let target = dir.path().join("data.final");

        fs::write(&target, b"old content").unwrap();
        fs::write(&tmp, b"new content").unwrap();
        replace_file_atomically(&tmp, &target).unwrap();

        assert_eq!(fs::read(&target).unwrap(), b"new content");
        assert!(!tmp.exists(), "temp file should be gone after replace");
    }
}
