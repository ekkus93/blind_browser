use std::collections::{HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use tauri::Manager;
use uuid::Uuid;

use crate::commands::{current_timestamp_ms, normalized_origin, ToolError};

const HANDLE_PREFIX: &str = "img_";
const HANDLE_HEX_LEN: usize = 32;
const DEFAULT_TTL_MS: u64 = 15 * 60 * 1_000;
const DEFAULT_MAX_COUNT: usize = 32;
const DEFAULT_MAX_BYTES: u64 = 128 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ImageHandle(String);

impl ImageHandle {
    fn generate() -> Self {
        Self(format!("{HANDLE_PREFIX}{}", Uuid::new_v4().simple()))
    }

    fn parse(value: &str) -> Result<Self, ToolError> {
        let valid = value.len() == HANDLE_PREFIX.len() + HANDLE_HEX_LEN
            && value.starts_with(HANDLE_PREFIX)
            && value[HANDLE_PREFIX.len()..]
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
        if valid {
            Ok(Self(value.to_owned()))
        } else {
            Err(error(
                "invalid_image_handle",
                "image handle must be an application-generated lowercase opaque identifier",
                false,
            ))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ImageContext {
    page_id: String,
    origin: Option<String>,
    generation: u64,
}

#[derive(Debug, Clone)]
struct ImageRecord {
    path: PathBuf,
    context: ImageContext,
    created_ms: u64,
    expires_ms: u64,
    size: u64,
    sha256: String,
}

#[derive(Debug)]
pub(super) struct ImageCache {
    records: HashMap<ImageHandle, ImageRecord>,
    ttl_ms: u64,
    max_count: usize,
    max_bytes: u64,
}

impl Default for ImageCache {
    fn default() -> Self {
        Self {
            records: HashMap::new(),
            ttl_ms: DEFAULT_TTL_MS,
            max_count: DEFAULT_MAX_COUNT,
            max_bytes: DEFAULT_MAX_BYTES,
        }
    }
}

impl ImageCache {
    fn persist(
        &mut self,
        root: &Path,
        context: ImageContext,
        bytes: &[u8],
        now_ms: u64,
    ) -> Result<String, ToolError> {
        if self.max_count == 0 || self.max_bytes == 0 {
            return Err(error(
                "image_cache_capacity_failed",
                "private screenshot cache limits must be positive",
                false,
            ));
        }
        let size = u64::try_from(bytes.len())
            .map_err(|_| error("screenshot_too_large", "screenshot size overflow", false))?;
        if size > self.max_bytes {
            return Err(error(
                "screenshot_too_large",
                "screenshot exceeds the bounded private cache capacity",
                false,
            ));
        }

        let root = prepare_root(root)?;
        self.remove_orphans(&root)?;
        self.cleanup_expired(now_ms)?;
        self.make_room(size)?;

        let handle = ImageHandle::generate();
        let (path, mut file) = create_private_file(&root)?;
        if let Err(io_error) = file.write_all(bytes).and_then(|_| file.sync_all()) {
            let _ = fs::remove_file(&path);
            return Err(io_tool_error("screenshot_write_failed", io_error));
        }
        drop(file);

        let path = verify_file(&root, &path)?;
        self.records.insert(
            handle.clone(),
            ImageRecord {
                path,
                context,
                created_ms: now_ms,
                expires_ms: now_ms.saturating_add(self.ttl_ms),
                size,
                sha256: format!("{:x}", Sha256::digest(bytes)),
            },
        );
        Ok(handle.0)
    }

    fn resolve(
        &mut self,
        root: &Path,
        raw: &str,
        current: &ImageContext,
        now_ms: u64,
    ) -> Result<PathBuf, ToolError> {
        let handle = ImageHandle::parse(raw)?;
        let record = self.records.get(&handle).cloned().ok_or_else(|| {
            error(
                "unknown_image_handle",
                "image handle is not registered in this application session",
                false,
            )
        })?;
        if now_ms >= record.expires_ms {
            self.remove(&handle)?;
            return Err(error(
                "expired_image_handle",
                "image handle expired before OCR use",
                false,
            ));
        }
        if record.context.page_id != current.page_id {
            return Err(error(
                "cross_page_image_handle",
                "image handle belongs to another page",
                false,
            ));
        }
        if record.context.origin != current.origin {
            return Err(error(
                "cross_origin_image_handle",
                "image handle belongs to another origin",
                false,
            ));
        }
        if record.context.generation != current.generation {
            return Err(error(
                "stale_image_handle",
                "image handle belongs to an older page generation",
                false,
            ));
        }

        let root = prepare_root(root)?;
        let path = verify_file(&root, &record.path)?;
        let metadata =
            fs::metadata(&path).map_err(|e| io_tool_error("image_cache_read_failed", e))?;
        if metadata.len() != record.size || digest_file(&path)? != record.sha256 {
            return Err(error(
                "image_cache_integrity_mismatch",
                "registered screenshot changed after capture",
                false,
            ));
        }
        Ok(path)
    }

    fn cleanup_expired(&mut self, now_ms: u64) -> Result<(), ToolError> {
        let handles: Vec<_> = self
            .records
            .iter()
            .filter(|(_, r)| now_ms >= r.expires_ms)
            .map(|(h, _)| h.clone())
            .collect();
        for handle in handles {
            self.remove(&handle)?;
        }
        Ok(())
    }

    fn make_room(&mut self, incoming: u64) -> Result<(), ToolError> {
        while self.records.len() >= self.max_count
            || self.total_bytes().saturating_add(incoming) > self.max_bytes
        {
            let oldest = self
                .records
                .iter()
                .min_by_key(|(_, r)| r.created_ms)
                .map(|(h, _)| h.clone())
                .ok_or_else(|| {
                    error(
                        "image_cache_capacity_failed",
                        "cache capacity is invalid",
                        false,
                    )
                })?;
            self.remove(&oldest)?;
        }
        Ok(())
    }

    fn total_bytes(&self) -> u64 {
        self.records.values().map(|r| r.size).sum()
    }

    fn remove(&mut self, handle: &ImageHandle) -> Result<(), ToolError> {
        let Some(record) = self.records.get(handle) else {
            return Ok(());
        };
        match fs::symlink_metadata(&record.path) {
            Ok(m) if m.is_dir() => {
                return Err(error(
                    "image_cache_cleanup_failed",
                    "registered screenshot unexpectedly became a directory",
                    false,
                ))
            }
            Ok(_) => fs::remove_file(&record.path)
                .map_err(|e| io_tool_error("image_cache_cleanup_failed", e))?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(io_tool_error("image_cache_cleanup_failed", e)),
        }
        self.records.remove(handle);
        Ok(())
    }

    fn remove_orphans(&self, root: &Path) -> Result<(), ToolError> {
        let registered: HashSet<&Path> = self.records.values().map(|r| r.path.as_path()).collect();
        for entry in fs::read_dir(root).map_err(|e| io_tool_error("image_cache_scan_failed", e))? {
            let path = entry
                .map_err(|e| io_tool_error("image_cache_scan_failed", e))?
                .path();
            if registered.contains(path.as_path()) {
                continue;
            }
            let metadata = fs::symlink_metadata(&path)
                .map_err(|e| io_tool_error("image_cache_scan_failed", e))?;
            if metadata.is_file() || metadata.file_type().is_symlink() {
                fs::remove_file(path)
                    .map_err(|e| io_tool_error("image_cache_cleanup_failed", e))?;
            }
        }
        Ok(())
    }
}

impl super::AppCore {
    pub(super) fn persist_screenshot_image(
        &mut self,
        page_id: String,
        origin: Option<String>,
        generation: u64,
        bytes: &[u8],
    ) -> Result<String, ToolError> {
        let root = self.screenshot_cache_root()?;
        self.image_cache.persist(
            &root,
            ImageContext {
                page_id,
                origin,
                generation,
            },
            bytes,
            current_timestamp_ms(),
        )
    }

    pub(super) fn resolve_screenshot_image(&mut self, raw: &str) -> Result<PathBuf, ToolError> {
        let context = ImageContext {
            page_id: self.state.current_page_id.clone().ok_or_else(|| {
                error(
                    "no_active_page",
                    "OCR image resolution requires an active page",
                    false,
                )
            })?,
            origin: normalized_origin(
                self.state
                    .current_page
                    .as_ref()
                    .and_then(|p| p.url.as_deref()),
            ),
            generation: self.state.page_generation,
        };
        let root = self.screenshot_cache_root()?;
        self.image_cache
            .resolve(&root, raw, &context, current_timestamp_ms())
    }

    fn screenshot_cache_root(&self) -> Result<PathBuf, ToolError> {
        self.app_handle
            .path()
            .app_cache_dir()
            .map(|p| p.join("screenshots"))
            .map_err(|e| io_tool_error("resolve_app_cache_dir_failed", e))
    }
}

fn prepare_root(root: &Path) -> Result<PathBuf, ToolError> {
    fs::create_dir_all(root).map_err(|e| io_tool_error("create_screenshot_dir_failed", e))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(root, fs::Permissions::from_mode(0o700))
            .map_err(|e| io_tool_error("secure_screenshot_dir_failed", e))?;
    }
    fs::canonicalize(root).map_err(|e| io_tool_error("canonicalize_screenshot_dir_failed", e))
}

fn create_private_file(root: &Path) -> Result<(PathBuf, File), ToolError> {
    for _ in 0..8 {
        let path = root.join(format!("shot_{}.png", Uuid::new_v4().simple()));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(io_tool_error("screenshot_file_create_failed", e)),
        }
    }
    Err(error(
        "screenshot_file_collision",
        "private screenshot filename allocation exhausted its retry budget",
        true,
    ))
}

fn verify_file(root: &Path, path: &Path) -> Result<PathBuf, ToolError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|e| io_tool_error("image_cache_metadata_failed", e))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(error(
            "image_cache_unsafe_file_type",
            "registered screenshot must remain a non-symlink regular file",
            false,
        ));
    }
    let path =
        fs::canonicalize(path).map_err(|e| io_tool_error("canonicalize_screenshot_failed", e))?;
    if !path.starts_with(root) {
        return Err(error(
            "image_cache_containment_violation",
            "registered screenshot escaped the canonical cache root",
            false,
        ));
    }
    Ok(path)
}

fn digest_file(path: &Path) -> Result<String, ToolError> {
    let mut file = File::open(path).map_err(|e| io_tool_error("image_cache_read_failed", e))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|e| io_tool_error("image_cache_read_failed", e))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn error(code: &str, message: &str, retryable: bool) -> ToolError {
    ToolError {
        code: code.to_owned(),
        message: message.to_owned(),
        retryable,
        details: None,
    }
}

fn io_tool_error(code: &str, source: impl std::fmt::Display) -> ToolError {
    ToolError {
        code: code.to_owned(),
        message: "private screenshot cache operation failed".to_owned(),
        retryable: true,
        details: Some(serde_json::json!({ "reason": source.to_string() })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn ctx(page: &str, origin: &str, generation: u64) -> ImageContext {
        ImageContext {
            page_id: page.to_owned(),
            origin: Some(origin.to_owned()),
            generation,
        }
    }

    fn cache(ttl_ms: u64, max_count: usize, max_bytes: u64) -> ImageCache {
        ImageCache {
            records: HashMap::new(),
            ttl_ms,
            max_count,
            max_bytes,
        }
    }

    #[test]
    fn strict_handles_reject_path_and_encoding_tricks() {
        for bad in [
            "../x",
            "/tmp/x",
            r"C:\\x",
            r"\\\\server\\share",
            "img_%2e%2e%2fx",
            "img_..%5cx",
            "img_／etc／passwd",
            "img_..∕x",
            "img_0000000000004000800000000000000g",
            "IMG_00000000000040008000000000000001",
            " img_00000000000040008000000000000001",
        ] {
            assert_eq!(
                ImageHandle::parse(bad).unwrap_err().code,
                "invalid_image_handle"
            );
        }
        let generated = ImageHandle::generate();
        assert!(ImageHandle::parse(&generated.0).is_ok());
    }

    #[test]
    fn same_context_resolves_but_cross_page_origin_and_generation_fail() {
        let dir = TempDir::new().unwrap();
        let expected = ctx("p1", "https://example.com", 1);
        let mut cache = cache(1000, 4, 1024);
        let handle = cache
            .persist(dir.path(), expected.clone(), b"png", 10)
            .unwrap();
        let path = cache.resolve(dir.path(), &handle, &expected, 11).unwrap();
        assert!(!path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains(&handle));
        assert_eq!(
            cache
                .resolve(
                    dir.path(),
                    &handle,
                    &ctx("p2", "https://example.com", 1),
                    11
                )
                .unwrap_err()
                .code,
            "cross_page_image_handle"
        );
        assert_eq!(
            cache
                .resolve(
                    dir.path(),
                    &handle,
                    &ctx("p1", "https://other.example", 1),
                    11
                )
                .unwrap_err()
                .code,
            "cross_origin_image_handle"
        );
        assert_eq!(
            cache
                .resolve(
                    dir.path(),
                    &handle,
                    &ctx("p1", "https://example.com", 2),
                    11
                )
                .unwrap_err()
                .code,
            "stale_image_handle"
        );
    }

    #[test]
    fn unknown_expired_and_tampered_records_fail_closed() {
        let dir = TempDir::new().unwrap();
        let expected = ctx("p1", "https://example.com", 1);
        let mut cache = cache(10, 4, 1024);
        assert_eq!(
            cache
                .resolve(
                    dir.path(),
                    "img_00000000000040008000000000000001",
                    &expected,
                    1
                )
                .unwrap_err()
                .code,
            "unknown_image_handle"
        );
        let expired = cache
            .persist(dir.path(), expected.clone(), b"old", 10)
            .unwrap();
        assert_eq!(
            cache
                .resolve(dir.path(), &expired, &expected, 20)
                .unwrap_err()
                .code,
            "expired_image_handle"
        );
        let tampered = cache
            .persist(dir.path(), expected.clone(), b"original", 30)
            .unwrap();
        let key = ImageHandle::parse(&tampered).unwrap();
        fs::write(&cache.records[&key].path, b"modified").unwrap();
        assert_eq!(
            cache
                .resolve(dir.path(), &tampered, &expected, 31)
                .unwrap_err()
                .code,
            "image_cache_integrity_mismatch"
        );
    }

    #[test]
    fn count_and_byte_limits_evict_oldest_file_and_record() {
        let dir = TempDir::new().unwrap();
        let expected = ctx("p1", "https://example.com", 1);
        let mut cache = cache(1000, 2, 8);
        let first = cache
            .persist(dir.path(), expected.clone(), b"1111", 1)
            .unwrap();
        let first_key = ImageHandle::parse(&first).unwrap();
        let first_path = cache.records[&first_key].path.clone();
        cache
            .persist(dir.path(), expected.clone(), b"2222", 2)
            .unwrap();
        cache.persist(dir.path(), expected, b"3333", 3).unwrap();
        assert!(!cache.records.contains_key(&first_key));
        assert!(!first_path.exists());
        assert_eq!(cache.total_bytes(), 8);
    }

    #[test]
    fn restart_orphans_are_removed_before_capture() {
        let dir = TempDir::new().unwrap();
        let orphan = dir.path().join("orphan.png");
        fs::write(&orphan, b"old").unwrap();
        cache(1000, 4, 1024)
            .persist(dir.path(), ctx("p1", "https://example.com", 1), b"new", 1)
            .unwrap();
        assert!(!orphan.exists());
    }

    #[cfg(unix)]
    #[test]
    fn private_permissions_and_symlink_replacement_are_enforced() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let dir = TempDir::new().unwrap();
        let outside_dir = TempDir::new().unwrap();
        let outside = outside_dir.path().join("outside.png");
        fs::write(&outside, b"outside").unwrap();
        let expected = ctx("p1", "https://example.com", 1);
        let mut cache = cache(1000, 4, 1024);
        let handle = cache
            .persist(dir.path(), expected.clone(), b"private", 1)
            .unwrap();
        let key = ImageHandle::parse(&handle).unwrap();
        let path = cache.records[&key].path.clone();
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        fs::remove_file(&path).unwrap();
        symlink(outside, path).unwrap();
        assert_eq!(
            cache
                .resolve(dir.path(), &handle, &expected, 2)
                .unwrap_err()
                .code,
            "image_cache_unsafe_file_type"
        );
    }
}
