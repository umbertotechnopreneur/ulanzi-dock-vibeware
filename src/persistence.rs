/* VBWR B
 *
 * Project: UlanziDeck
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
    destination: PathBuf,
    #[cfg(target_os = "linux")]
    directory: File,
    // Closing this handle releases the OS lock, including after a process crash.
    _lock: File,
}

impl Transaction {
    /// Resolve `path` and acquire its persistent sidecar lock, waiting at most two seconds.
    /// The parent must exist. Lock/canonicalization errors leave the target untouched.
    pub fn begin(path: &Path) -> Result<Self> {
        Self::begin_with_timeout(path, Duration::from_secs(2))
    }

    // Tests can allow slower disks more time without changing the user-facing timeout.
    fn begin_with_timeout(path: &Path, timeout: Duration) -> Result<Self> {
        #[cfg(target_os = "linux")]
        let (path, directory, destination) = {
            let absolute = std::path::absolute(path)?;
            let name = absolute.file_name().context("state file needs a filename")?;
            let parent = fs::canonicalize(absolute.parent().context("state file needs a parent")?)?;
            let directory = linux::open_directory(&parent)?;
            let destination = linux::fd_path(&directory).join(name);
            (parent.join(name), directory, destination)
        };
        #[cfg(not(target_os = "linux"))]
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
        #[cfg(not(target_os = "linux"))]
        let destination = path.clone();
        let mut name = destination.as_os_str().to_owned();
        name.push(".lock");
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options
                .mode(0o600)
                .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_NOCTTY);
        }
        let lock = options
            .open(PathBuf::from(name))
            .context("opening state lock")?;
        #[cfg(target_os = "linux")]
        ensure!(
            lock.metadata()?.is_file(),
            "state lock must be a regular file"
        );
        let started = Instant::now();
        loop {
            match lock.try_lock() {
                Ok(()) => {
                    return Ok(Self {
                        path,
                        destination,
                        #[cfg(target_os = "linux")]
                        directory,
                        _lock: lock,
                    })
                }
                Err(TryLockError::WouldBlock) if started.elapsed() < timeout => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(TryLockError::WouldBlock) => {
                    anyhow::bail!("{} is busy in another process; try again", path.display())
                }
                Err(TryLockError::Error(error)) => return Err(error).context("locking state file"),
            }
        }
    }

    /// Return the stable profile path that callers can keep after the lock drops.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Read a complete current snapshot; an absent destination is `None`.
    /// Other I/O errors propagate rather than being mistaken for a new file.
    pub fn read(&self) -> Result<Option<Vec<u8>>> {
        read_optional(&self.destination)
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
        let staged = StagedFile::new(self, &self.destination, bytes)?;
        if let Some(original) = expected {
            let mut backup = self.destination.as_os_str().to_owned();
            backup.push(".gui.bak");
            let backup = PathBuf::from(backup);
            StagedFile::new(self, &backup, original)?.replace(&backup)?;
        }
        // Editors outside this application may not honor its lock.
        ensure!(
            self.read()?.as_deref() == expected,
            "{} changed before saving",
            self.path.display()
        );
        staged.replace(&self.destination)
    }

    /// Publish a complete new file without ever replacing an existing destination.
    pub fn create(&self, bytes: &[u8]) -> Result<bool> {
        if self.read()?.is_some() {
            return Ok(false);
        }
        let staged = StagedFile::new(self, &self.destination, bytes)?;
        // The sibling temporary is on the same volume. Linking is an atomic
        // create-if-absent operation on Windows and Unix; unsupported filesystems fail closed.
        match fs::hard_link(&staged.path, &self.destination) {
            Ok(()) => Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
            Err(error) => Err(error).context("publishing new state file without replacement"),
        }
    }
}

/// Read a profile input. Linux opens only regular files, without following
/// a final symlink or waiting for a writer when a FIFO occupies the expected name.
pub fn read(path: &Path) -> std::io::Result<Vec<u8>> {
    #[cfg(target_os = "linux")]
    {
        linux::read_regular(path)
    }
    #[cfg(not(target_os = "linux"))]
    {
        fs::read(path)
    }
}

/// Only a missing file becomes `None`. Propagate all other read errors.
pub fn read_optional(path: &Path) -> Result<Option<Vec<u8>>> {
    match read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("reading {}", path.display())),
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::{
        io::{self, Read},
        os::{
            fd::AsRawFd,
            unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt},
        },
    };

    // A procfs fd path pins the directory inode across all name lookups, including
    // rename and hard_link. Callers retain the descriptor until the operation ends.
    pub(super) fn fd_path(directory: &File) -> PathBuf {
        PathBuf::from(format!("/proc/self/fd/{}", directory.as_raw_fd()))
    }

    pub(super) fn open_directory(path: &Path) -> io::Result<File> {
        OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_PATH | libc::O_DIRECTORY | libc::O_NOFOLLOW)
            .open(path)
    }

    pub(super) fn read_regular(path: &Path) -> io::Result<Vec<u8>> {
        let mut file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_NOCTTY)
            .open(path)?;
        if !file.metadata()?.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "expected a regular file",
            ));
        }
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    // Never grant group/other access, even briefly, or increase an existing owner's mode.
    pub(super) fn private_mode(path: &Path) -> io::Result<u32> {
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.is_file() => Ok(metadata.mode() & 0o600),
            Ok(_) => Ok(0o600),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(0o600),
            Err(error) => Err(error),
        }
    }

    pub(super) fn private_staging(directory: &File) -> Result<Option<File>> {
        let metadata = directory.metadata()?;
        // SAFETY: geteuid takes no arguments and cannot fail.
        let uid = unsafe { libc::geteuid() };
        if metadata.uid() == uid && metadata.mode() & 0o022 == 0 {
            return Ok(None);
        }
        // Peers with write access to the profile could swap an ordinary sibling
        // temporary before publication. Use a private subdirectory on the same FS.
        let path = fd_path(directory).join(format!(".ulanzi-stage-{uid}"));
        match fs::DirBuilder::new().mode(0o700).create(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error).context("creating private state staging directory"),
        }
        let staging = open_directory(&path).context("opening private state staging directory")?;
        let metadata = staging.metadata()?;
        ensure!(
            metadata.uid() == uid && metadata.mode() & 0o777 == 0o700,
            "state staging directory must be owned by the current user and have mode 0700"
        );
        Ok(Some(staging))
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
    #[cfg(target_os = "linux")]
    _private_directory: Option<File>,
}

impl StagedFile {
    /// Stage `bytes` beside `destination` using create-new, collision-resistant names.
    /// A write or flush failure closes the handle before cleanup, including on Windows.
    fn new(_transaction: &Transaction, destination: &Path, bytes: &[u8]) -> Result<Self> {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(target_os = "linux")]
        let (destination, private_directory) = {
            use std::os::unix::fs::OpenOptionsExt;
            let mode = linux::private_mode(&_transaction.destination)?;
            options.mode(mode & linux::private_mode(destination)?);
            let private_directory = linux::private_staging(&_transaction.directory)?;
            let path = match &private_directory {
                Some(dir) => linux::fd_path(dir)
                    .join(destination.file_name().context("missing state filename")?),
                None => destination.to_owned(),
            };
            (path, private_directory)
        };
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
            let mut file = match options.open(&path) {
                Ok(file) => file,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error).context("creating staged state file"),
            };
            let staged = Self {
                path,
                #[cfg(target_os = "linux")]
                _private_directory: private_directory,
            };
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
            #[cfg(target_os = "linux")]
            {
                use std::os::unix::fs::DirBuilderExt;
                fs::DirBuilder::new().mode(0o700).create(&path)?;
            }
            #[cfg(not(target_os = "linux"))]
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

    #[cfg(target_os = "linux")]
    #[test]
    fn redirected_state_cannot_overwrite_another_file() -> Result<()> {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let fixture = Fixture::new()?;
        let external = Fixture::new()?;
        let source = fixture.file();
        let backup = fixture.0.join("state.json.gui.bak");
        let outside = external.file();
        fs::write(&outside, b"other user data")?;
        let transaction = Transaction::begin(&source)?;
        symlink(&outside, &source)?;
        let redirected_read = transaction.read();
        let redirected_write = transaction.replace(Some(b"other user data"), b"new page");
        fs::remove_file(&source)?;
        assert!(redirected_read.is_err());
        assert!(redirected_write.is_err());

        fs::write(&source, b"old page")?;
        fs::set_permissions(&source, fs::Permissions::from_mode(0o400))?;
        symlink(&outside, &backup)?;
        transaction.replace(Some(b"old page"), b"new page")?;
        assert_eq!(fs::read(&outside)?, b"other user data");
        assert_eq!(fs::read(&backup)?, b"old page");
        for file in [&source, &backup] {
            assert_eq!(fs::metadata(file)?.permissions().mode() & 0o777, 0o400);
        }
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
            let transaction = Transaction::begin_with_timeout(&path, Duration::from_secs(30))?;
            let original = transaction.read()?.context("missing counter")?;
            let count: u32 = std::str::from_utf8(&original)?.parse()?;
            std::thread::sleep(Duration::from_millis(3));
            transaction.replace(Some(&original), (count + 1).to_string().as_bytes())?;
        }
        Ok(())
    }
}
