//! Helpers that the integration tests share.

#![allow(dead_code, reason = "each test binary uses a part of this module")]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

use tempfile::TempDir;

/// The root of the repository.
pub fn repo_root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

/// The standard output of a finished command, as text.
pub fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout is UTF-8")
}

/// The standard error of a finished command, as text.
pub fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr is UTF-8")
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// A copy of a workspace in a temporary directory that a test can break in one place.
pub struct TreeCopy {
    dir: TempDir,
}

impl TreeCopy {
    /// Copies every file under `from`.
    pub fn of_dir(from: &Path) -> Self {
        let dir = TempDir::new().unwrap();
        copy_tree(from, dir.path());
        Self { dir }
    }

    /// The root of the copy.
    pub fn root(&self) -> &Path {
        self.dir.path()
    }

    fn manifest(member: &str) -> PathBuf {
        Path::new("crates").join(member).join("Cargo.toml")
    }

    /// Adds text to the end of the manifest of a member.
    pub fn append(self, member: &str, text: &str) -> Self {
        let manifest = Self::manifest(member);
        self.append_file(&manifest, text)
    }

    /// Replaces text in the manifest of a member. The text must be there.
    pub fn replace(self, member: &str, from: &str, to: &str) -> Self {
        let manifest = Self::manifest(member);
        self.replace_in_file(&manifest, from, to)
    }

    /// Adds text to the end of a file, which this creates if it is absent.
    pub fn append_file(self, relative: &Path, text: &str) -> Self {
        let path = self.root().join(relative);
        let mut content = fs::read_to_string(&path).unwrap_or_default();
        content.push_str(text);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
        self
    }

    /// Replaces text in a file. The text must be there.
    pub fn replace_in_file(self, relative: &Path, from: &str, to: &str) -> Self {
        let path = self.root().join(relative);
        let content = fs::read_to_string(&path).unwrap();
        assert!(
            content.contains(from),
            "{} holds {from:?}",
            relative.display()
        );
        fs::write(path, content.replace(from, to)).unwrap();
        self
    }

    /// Adds a crate at `relative` with the manifest of the member `mu-proto` and a new name.
    pub fn add_crate(self, relative: &str, name: &str) -> Self {
        let root = self.root().join(relative);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/lib.rs"), "").unwrap();
        let manifest = fs::read_to_string(self.root().join(Self::manifest("mu-proto")))
            .unwrap()
            .replace("mu-proto", name);
        fs::write(root.join("Cargo.toml"), manifest).unwrap();
        self
    }
}
