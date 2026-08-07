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
///
/// `rename` itself is atomic, but on Unix the *directory entry* it produces
/// is not guaranteed durable until the containing directory is fsynced --
/// without that, a crash or power loss immediately after a successful
/// rename can still lose the rename on some filesystems/journal orderings.
/// Every caller here writes config or downloaded-model state that must
/// survive exactly that failure mode, so the parent directory is fsynced
/// after the rename. This has no Windows equivalent (NTFS's journal makes
/// the metadata update durable as part of the rename itself), so it's a
/// no-op there. A failure to fsync the directory is reported as an error
/// rather than swallowed: the rename already landed as far as any reader
/// can tell, but the caller's durability guarantee did not, and silently
/// downgrading "durable" to "probably fine" is exactly the kind of silent
/// fallback this codebase's dependency-management rules forbid.
pub fn replace_file_atomically(tmp_path: &Path, target_path: &Path) -> Result<(), String> {
    fs::rename(tmp_path, target_path).map_err(|error| {
        format!(
            "atomically replace {} with {}: {error}",
            target_path.display(),
            tmp_path.display()
        )
    })?;

    #[cfg(unix)]
    {
        let parent = target_path.parent().ok_or_else(|| {
            format!(
                "{} has no parent directory to fsync after rename",
                target_path.display()
            )
        })?;
        let dir = fs::File::open(parent).map_err(|error| {
            format!(
                "open directory {} to fsync after replacing {}: {error}",
                parent.display(),
                target_path.display()
            )
        })?;
        dir.sync_all().map_err(|error| {
            format!(
                "fsync directory {} after replacing {}: {error}",
                parent.display(),
                target_path.display()
            )
        })?;
    }

    Ok(())
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

    // CR3 P2.3.2 note: the two tests above already exercise the added
    // parent-directory fsync on every successful call (both `.unwrap()` the
    // result), so a regression that broke the fsync step -- wrong path,
    // wrong open mode, etc. -- would already fail them. A dedicated third
    // test was considered and dropped: there is no portable, non-flaky way
    // to assert fsync *actually reached disk* from a unit test, and a test
    // that only re-asserts "the call still succeeds" would not add coverage
    // beyond what's already here.
}
