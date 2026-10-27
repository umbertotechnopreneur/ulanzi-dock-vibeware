/* VBWR B
 *
 * Project: UlanziDock VibeWare version
 * Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
 * Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
 *
 * VibeWare: Human intent, AI execution, and plenty of tokens
 * Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
 *
 * Created with AI: OpenAI Codex; transactional persistence; 2026-10-27
 * Modified with AI: OpenAI Codex; comments and readability; 2026-10-27
 * Human guidance: Umberto Giacobbi; AGENTS.md; readability/documentation request
 * Evidence: docs/CODE-REVIEW.md and current working-tree changes
 *
 * Copyright (c) 2026 Umberto Giacobbi
 * SPDX-License-Identifier: MIT
 * License: MIT - see LICENSE
 *
 * VBWR E */

//! Serialize cooperating processes around one state file and publish complete writes.
//! Transactions keep a sidecar OS lock, verify snapshots, and preserve a backup. Atomic
//! replacement prevents partial files; it cannot make an external editor honor the lock.
//! Callers own JSON/YAML validation and decide which fields may change.

use anyhow::{ensure, Context, Result};
use std::{
    fs::{self, File, OpenOptions, TryLockError},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

/// Exclusive cooperating-process lock for one canonical destination path.
/// Keep this guard alive across the complete read/modify/write operation.
pub struct Transaction {
    path: PathBuf,
    // Closing this handle releases the OS lock, including after a process crash.
    _lock: File,
}

impl Transaction {
    /// Resolve `path` and acquire its persistent sidecar lock, waiting at most two seconds.
    /// The parent must exist. Lock/canonicalization errors leave the target untouched.
    pub fn begin(path: &Path) -> Result<Self> {
        let path = match fs::canonicalize(path) {
            Ok(path) => path,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let absolute = std::path::absolute(path)?;
                let parent = absolute
                    .parent()
                    .context("state file needs a parent directory")?;
                fs::canonicalize(parent)?.join(
                    absolute
                        .file_name()
                        .context("state file needs a filename")?,
                )
            }
            Err(error) => return Err(error).context("resolving state file"),
        };
        let mut name = path.as_os_str().to_owned();
        name.push(".lock");
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(PathBuf::from(name))
            .context("opening state lock")?;
        let started = Instant::now();
        loop {
            match lock.try_lock() {
                Ok(()) => return Ok(Self { path, _lock: lock }),
                Err(TryLockError::WouldBlock) if started.elapsed() < Duration::from_secs(2) => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(TryLockError::WouldBlock) => {
                    anyhow::bail!("{} is busy in another process; try again", path.display())
                }
                Err(TryLockError::Error(error)) => return Err(error).context("locking state file"),
            }
        }
    }

    /// Return the canonical destination used by this transaction and its lock.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Read a complete current snapshot; an absent destination is `None`.
    /// Other I/O errors propagate rather than being mistaken for a new file.
    pub fn read(&self) -> Result<Option<Vec<u8>>> {
        match fs::read(&self.path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error).with_context(|| format!("reading {}", self.path.display())),
        }
    }

    /// Replace only if the target still matches `expected`; `None` means absent.
    /// Stage and flush `bytes`, save the previous bytes as `.gui.bak`, then publish.
    /// Snapshot, staging or replacement errors preserve a readable destination.
    pub fn replace(&self, expected: Option<&[u8]>, bytes: &[u8]) -> Result<()> {
        ensure!(
            self.read()?.as_deref() == expected,
            "{} changed before saving",
            self.path.display()
        );
        if expected == Some(bytes) {
            return Ok(());
        }
        let staged = StagedFile::new(&self.path, bytes)?;
        if let Some(original) = expected {
            let mut backup = self.path.as_os_str().to_owned();
            backup.push(".gui.bak");
            let backup = PathBuf::from(backup);
            StagedFile::new(&backup, original)?.replace(&backup)?;
        }
        // Editors outside this application may not honor its lock.
        ensure!(
            self.read()?.as_deref() == expected,
            "{} changed before saving",
            self.path.display()
        );
        staged.replace(&self.path)
    }

    /// Publish a complete new file without ever replacing an existing destination.
    pub fn create(&self, bytes: &[u8]) -> Result<bool> {
        if self.read()?.is_some() {
            return Ok(false);
        }
        let staged = StagedFile::new(&self.path, bytes)?;
        // The sibling temporary is on the same volume. Linking is an atomic
        // create-if-absent operation on Windows and Unix; unsupported filesystems fail closed.
        match fs::hard_link(&staged.path, &self.path) {
            Ok(()) => Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
            Err(error) => Err(error).context("publishing new state file without replacement"),
        }
    }
}

/// Create `path` with complete `bytes`, failing if another writer already owns the name.
/// Parent creation and content validation remain the caller's responsibility.
pub fn create_new(path: &Path, bytes: &[u8]) -> Result<()> {
    ensure!(
        Transaction::begin(path)?.create(bytes)?,
        "{} already exists; preserving it",
        path.display()
    );
    Ok(())
}

/// A flushed sibling temporary file; dropping it removes only this owned path.
struct StagedFile {
    path: PathBuf,
}

impl StagedFile {
    /// Stage `bytes` beside `destination` using create-new, collision-resistant names.
    /// A write or flush failure closes the handle before cleanup, including on Windows.
    fn new(destination: &Path, bytes: &[u8]) -> Result<Self> {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        for _ in 0..16 {
            let mut name = destination.as_os_str().to_owned();
            name.push(format!(
                ".gui-{}-{stamp}-{}.tmp",
                std::process::id(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            let path = PathBuf::from(name);
            let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => file,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error).context("creating staged state file"),
            };
            let staged = Self { path };
            let result = file.write_all(bytes).and_then(|()| file.sync_all());
            // Close first so cleanup also works after a write failure on Windows.
            drop(file);
            result.context("writing and flushing staged state")?;
            return Ok(staged);
        }
        anyhow::bail!("could not allocate a unique staging file")
    }

    /// Atomically publish this staged file over `destination` on the same filesystem.
    /// Errors leave this guard responsible for removing its uncommitted temporary.
    fn replace(&self, destination: &Path) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::ffi::OsStrExt;
            use windows_sys::Win32::Storage::FileSystem::{
                MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
            };
            let source: Vec<_> = self.path.as_os_str().encode_wide().chain(Some(0)).collect();
            let destination: Vec<_> = destination
                .as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect();
            ensure!(
                unsafe {
                    MoveFileExW(
                        source.as_ptr(),
                        destination.as_ptr(),
                        MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
                    )
                } != 0,
                "replacing state: {}",
                std::io::Error::last_os_error()
            );
            Ok(())
        }
        #[cfg(not(target_os = "windows"))]
        fs::rename(&self.path, destination).context("replacing state")
    }
}

impl Drop for StagedFile {
    /// Remove only resources owned by this guard; cleanup failures cannot mask the result.
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Result<Self> {
            static SEQUENCE: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "ulanzi-persistence-fixture-{}-{}-{}",
                std::process::id(),
                SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path)?;
            Ok(Self(path))
        }
        fn file(&self) -> PathBuf {
            self.0.join("state.json")
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            // Only direct files in the unique directory created by this fixture.
            if let Ok(entries) = fs::read_dir(&self.0) {
                for entry in entries.flatten() {
                    if entry.file_type().is_ok_and(|kind| kind.is_file()) {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
            let _ = fs::remove_dir(&self.0);
        }
    }

    #[test]
    fn new_files_preserve_the_winner_and_replacements_keep_backups() -> Result<()> {
        let fixture = Fixture::new()?;
        let transaction = Transaction::begin(&fixture.file())?;
        assert!(transaction.create(b"original")?);
        assert!(!transaction.create(b"other seed")?);
        assert_eq!(transaction.read()?.as_deref(), Some(b"original".as_slice()));
        assert!(transaction.replace(Some(b"stale"), b"bad").is_err());
        transaction.replace(Some(b"original"), b"updated")?;
        assert_eq!(fs::read(fixture.0.join("state.json.gui.bak"))?, b"original");
        assert_eq!(transaction.read()?.as_deref(), Some(b"updated".as_slice()));
        Ok(())
    }

    #[test]
    fn process_transactions_do_not_lose_updates() -> Result<()> {
        let fixture = Fixture::new()?;
        create_new(&fixture.file(), b"0")?;
        let executable = std::env::current_exe()?;
        let mut children = Vec::new();
        for _ in 0..3 {
            children.push(
                std::process::Command::new(&executable)
                    .args([
                        "--exact",
                        "persistence::tests::transaction_worker",
                        "--nocapture",
                    ])
                    .env("ULANZI_TEST_TRANSACTION_PATH", fixture.file())
                    .spawn()?,
            );
        }
        // Join every child before fixture cleanup, even if an earlier child failed.
        let results: Vec<_> = children.iter_mut().map(|child| child.wait()).collect();
        for result in results {
            ensure!(result?.success(), "transaction worker failed");
        }
        assert_eq!(fs::read_to_string(fixture.file())?, "24");
        Ok(())
    }

    #[test]
    fn transaction_worker() -> Result<()> {
        let Some(path) = std::env::var_os("ULANZI_TEST_TRANSACTION_PATH") else {
            return Ok(());
        };
        let path = PathBuf::from(path);
        ensure!(
            path.file_name().is_some_and(|name| name == "state.json")
                && path
                    .parent()
                    .and_then(Path::file_name)
                    .is_some_and(|name| name
                        .to_string_lossy()
                        .starts_with("ulanzi-persistence-fixture-")),
            "worker requires an isolated fixture"
        );
        for _ in 0..8 {
            let transaction = Transaction::begin(&path)?;
            let original = transaction.read()?.context("missing counter")?;
            let count: u32 = std::str::from_utf8(&original)?.parse()?;
            std::thread::sleep(Duration::from_millis(3));
            transaction.replace(Some(&original), (count + 1).to_string().as_bytes())?;
        }
        Ok(())
    }
}
