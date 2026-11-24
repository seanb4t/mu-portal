//! A snapshot of a configuration directory: each relative path with its bytes, or with the
//! error that the walk met there.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt as _;
use std::path::Path;

use walkdir::WalkDir;

use super::error::ConfigError;

/// The directory of the client layers, at the top of the configuration directory.
pub const CLIENTS_DIR: &str = "clients";

/// What a snapshot holds at one path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnapEntry {
    /// A regular file, with its bytes.
    File(Vec<u8>),
    /// An entry that is not a regular file and not a directory, such as a FIFO. It is not read.
    NotRegular,
    /// An entry that the walk could not read, stat or list.
    Error {
        /// The kind of the error, as text, such as `permission denied`.
        kind: String,
        /// The whole error, as text.
        text: String,
    },
}

impl SnapEntry {
    fn of_io(error: &io::Error) -> Self {
        Self::Error {
            kind: error.kind().to_string(),
            text: error.to_string(),
        }
    }
}

/// The files of a configuration directory, read once.
///
/// The snapshot keeps every entry under [`CLIENTS_DIR`], each file outside it whose name ends
/// in `.toml` in any case, and each entry that the walk could not read. A path has `/` between
/// its parts, and the directory of the client layers is always `clients`, in any case on disk.
/// The walk follows a link itself, and a link to a directory on the path from the root to the
/// link is an error of the link. It reads only regular files.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DirSnapshot {
    entries: BTreeMap<String, SnapEntry>,
    top_files: BTreeSet<String>,
}

/// The device and the inode of a directory.
type DirId = (u64, u64);

impl DirSnapshot {
    /// Reads the directory `root`. A missing directory gives an empty snapshot.
    ///
    /// # Errors
    ///
    /// A [`ConfigError`] that names `root`, when it exists and cannot be read.
    pub fn take(root: &Path) -> Result<Self, ConfigError> {
        let mut snapshot = Self::default();
        let metadata = match fs::metadata(root) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(snapshot),
            Err(error) => return Err(ConfigError::new(root.display(), error.to_string())),
        };
        let mut walker = Walker {
            root,
            snapshot: &mut snapshot,
        };
        walker.walk(root, &[], &[(metadata.dev(), metadata.ino())])?;
        Ok(snapshot)
    }

    /// Each path with its entry, in the order of the paths.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &SnapEntry)> {
        self.entries
            .iter()
            .map(|(path, entry)| (path.as_str(), entry))
    }

    /// The entry at `path`.
    #[must_use]
    pub fn get(&self, path: &str) -> Option<&SnapEntry> {
        self.entries.get(path)
    }

    /// Each entry of the directory of the client layers, the directory itself included.
    pub fn clients(&self) -> impl Iterator<Item = (&str, &SnapEntry)> {
        self.entries().filter(|(path, _)| is_under_clients(path))
    }

    /// Each entry that is outside the directory of the client layers and is no top file.
    pub fn world_entries(&self) -> impl Iterator<Item = (&str, &SnapEntry)> {
        self.entries()
            .filter(|(path, _)| !is_under_clients(path) && !self.top_files.contains(*path))
    }

    /// The entry of [`SERVER_FILE`](super::SERVER_FILE), if the directory holds one.
    #[must_use]
    pub fn server_file(&self) -> Option<&SnapEntry> {
        self.entries.get(super::SERVER_FILE)
    }
}

/// Whether `path` is the directory of the client layers or under it.
fn is_under_clients(path: &str) -> bool {
    path == CLIENTS_DIR
        || path
            .strip_prefix(CLIENTS_DIR)
            .is_some_and(|rest| rest.starts_with('/'))
}

struct Walker<'a> {
    root: &'a Path,
    snapshot: &'a mut DirSnapshot,
}

impl Walker<'_> {
    /// Walks `directory`, whose parts relative to the root are `prefix`. `above` holds the
    /// directories from the root to `directory`, `directory` included.
    fn walk(
        &mut self,
        directory: &Path,
        prefix: &[String],
        above: &[DirId],
    ) -> Result<(), ConfigError> {
        let walk = WalkDir::new(directory)
            .follow_links(false)
            .min_depth(1)
            .sort_by_file_name();
        for item in walk {
            let entry = match item {
                Ok(entry) => entry,
                Err(error) => {
                    let path = error.path().unwrap_or(directory);
                    let parts = self.parts(directory, prefix, path);
                    let io = error
                        .into_io_error()
                        .unwrap_or_else(|| io::Error::other("the walk failed"));
                    if parts.is_empty() {
                        return Err(ConfigError::new(directory.display(), io.to_string()));
                    }
                    self.keep(&parts, SnapEntry::of_io(&io));
                    continue;
                }
            };
            let parts = self.parts(directory, prefix, entry.path());
            let file_type = entry.file_type();
            if file_type.is_symlink() {
                self.link(entry.path(), directory, &parts, above)?;
            } else if file_type.is_file() {
                self.file(entry.path(), &parts);
            } else if !file_type.is_dir() {
                self.other(&parts);
            }
        }
        Ok(())
    }

    /// Follows the link at `path`.
    fn link(
        &mut self,
        path: &Path,
        directory: &Path,
        parts: &[String],
        above: &[DirId],
    ) -> Result<(), ConfigError> {
        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(error) => {
                self.keep(parts, SnapEntry::of_io(&error));
                return Ok(());
            }
        };
        if metadata.is_file() {
            self.file(path, parts);
        } else if metadata.is_dir() {
            let mut chain = above.to_vec();
            for ancestor in path.ancestors().skip(1) {
                if ancestor == directory || !ancestor.starts_with(directory) {
                    break;
                }
                if let Ok(metadata) = fs::metadata(ancestor) {
                    chain.push((metadata.dev(), metadata.ino()));
                }
            }
            let target = (metadata.dev(), metadata.ino());
            if chain.contains(&target) {
                self.keep(
                    parts,
                    SnapEntry::Error {
                        kind: "loop".to_owned(),
                        text: "the link points to a directory that holds it".to_owned(),
                    },
                );
                return Ok(());
            }
            chain.push(target);
            self.walk(path, parts, &chain)?;
        } else {
            self.other(parts);
        }
        Ok(())
    }

    /// Reads the regular file at `path`, if the snapshot keeps it.
    fn file(&mut self, path: &Path, parts: &[String]) {
        if !Self::kept(parts) {
            return;
        }
        if let [name] = parts {
            match is_top_file(self.root, name) {
                Ok(true) => {
                    self.snapshot.top_files.insert(name.clone());
                }
                Ok(false) => {}
                Err(error) => {
                    self.keep(parts, SnapEntry::of_io(&error));
                    return;
                }
            }
        }
        let entry = match fs::read(path) {
            Ok(bytes) => SnapEntry::File(bytes),
            Err(error) => SnapEntry::of_io(&error),
        };
        self.keep(parts, entry);
    }

    /// Keeps an entry that is neither a file nor a directory, if the snapshot keeps its path.
    fn other(&mut self, parts: &[String]) {
        if Self::kept(parts) {
            self.keep(parts, SnapEntry::NotRegular);
        }
    }

    /// Whether the snapshot keeps a file at `parts`: every file under the directory of the
    /// client layers, and a file whose name ends in `.toml` in any case.
    fn kept(parts: &[String]) -> bool {
        parts.first().is_some_and(|first| first == CLIENTS_DIR)
            || parts
                .last()
                .is_some_and(|name| name.to_ascii_lowercase().ends_with(".toml"))
    }

    fn keep(&mut self, parts: &[String], entry: SnapEntry) {
        self.snapshot.entries.insert(parts.join("/"), entry);
    }

    /// The parts of `path`, which is under `directory`, relative to the root. A top-level
    /// directory that is the directory of the client layers in another case is named
    /// [`CLIENTS_DIR`].
    fn parts(&self, directory: &Path, prefix: &[String], path: &Path) -> Vec<String> {
        let mut parts = prefix.to_vec();
        if let Ok(rest) = path.strip_prefix(directory) {
            parts.extend(
                rest.components()
                    .map(|part| part.as_os_str().to_string_lossy().into_owned()),
            );
        }
        if let Some(first) = parts.first_mut()
            && first != CLIENTS_DIR
            && first.eq_ignore_ascii_case(CLIENTS_DIR)
            && same_file::is_same_file(self.root.join(&*first), self.root.join(CLIENTS_DIR))
                .unwrap_or(false)
        {
            CLIENTS_DIR.clone_into(first);
        }
        parts
    }
}

/// Whether the top-level file `name` is [`SERVER_FILE`](super::SERVER_FILE): the same name in
/// any case, and the same file. On a file system that ignores case, `Server.toml` is that
/// file.
fn is_top_file(root: &Path, name: &str) -> io::Result<bool> {
    let file = super::SERVER_FILE;
    if !name.eq_ignore_ascii_case(file) {
        return Ok(false);
    }
    if name == file {
        return Ok(true);
    }
    match same_file::is_same_file(root.join(name), root.join(file)) {
        Ok(same) => Ok(same),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}
