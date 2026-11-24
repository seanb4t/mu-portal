//! Helpers that the integration tests share.

#![allow(dead_code, reason = "each test binary uses a part of this module")]

use std::ffi::{OsStr, OsString};
use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

pub mod process;

/// The root of the repository.
pub fn repo_root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

/// The changes to the environment of each `mu` that a test starts, in the shape
/// `(name, Some(value))` to set a variable and `(name, None)` to remove one. They remove
/// `MU_SOCKET`, `MU_SERVER` and `MU_DEVICE` and point `MU_CONFIG` at an empty directory, so
/// that the socket, the server, the device and the configuration of the developer never reach
/// a test. They also remove each variable whose name starts with `OTEL_`, so that the telemetry of the developer never reaches a test.
pub fn test_env() -> Vec<(OsString, Option<OsString>)> {
    let mut changes: Vec<(OsString, Option<OsString>)> = vec![
        ("MU_SOCKET".into(), None),
        ("MU_SERVER".into(), None),
        ("MU_DEVICE".into(), None),
        (
            "MU_CONFIG".into(),
            Some(empty_config_dir().into_os_string()),
        ),
    ];
    changes.extend(
        std::env::vars_os()
            .map(|(name, _)| name)
            .filter(|name| name.as_encoded_bytes().starts_with(b"OTEL_"))
            .map(|name| (name, None)),
    );
    changes
}

/// An empty directory under the temporary directory of the target, for `MU_CONFIG`.
pub fn empty_config_dir() -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("empty-config");
    fs::create_dir_all(&dir).expect("the empty configuration directory is created");
    dir
}

/// Runs the built binary with `args` to its exit.
pub fn mu(args: &[&str]) -> Output {
    mu_with(args, &[])
}

/// Runs the built binary with `args` to its exit, with [`test_env`], then `env`.
pub fn mu_with(args: &[&str], env: &[(&str, Option<&str>)]) -> Output {
    mu_command(args, env).output().expect("the mu binary runs")
}

/// The built binary with `args`, with [`test_env`], then `env`. Each `(name, Some(value))`
/// sets a variable and each `(name, None)` removes one.
pub fn mu_command(args: &[&str], env: &[(&str, Option<&str>)]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mu"));
    command.args(args);
    apply_test_env(&mut command);
    apply_env(&mut command, env);
    command
}

/// Applies [`test_env`] to `command`.
pub fn apply_test_env(command: &mut Command) {
    apply_env(command, &test_env());
}

/// Applies `changes` to the environment of `command`: `(name, Some(value))` sets a variable
/// and `(name, None)` removes one.
pub fn apply_env<K: AsRef<OsStr>, V: AsRef<OsStr>>(
    command: &mut Command,
    changes: &[(K, Option<V>)],
) {
    for (name, value) in changes {
        match value {
            Some(value) => command.env(name, value),
            None => command.env_remove(name),
        };
    }
}

/// Runs the built binary with `args` to its exit on the runtime of the test, with
/// [`test_env`], then `env`.
pub async fn mu_async(args: &[&str], env: &[(&str, Option<&str>)]) -> Output {
    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_mu"));
    command.args(args);
    apply_test_env(command.as_std_mut());
    apply_env(command.as_std_mut(), env);
    command.output().await.expect("the mu binary runs")
}

/// The game file of the base directory of the world-config spec.
pub const BASE_GAME: &str = "extends = \"tinymux\"\nhost = \"mux.example.org\"\nport = 4201\n";

/// A configuration directory of worlds that a test writes.
pub struct ConfigDir {
    path: PathBuf,
    _temp: Option<TempDir>,
}

impl ConfigDir {
    /// An empty directory in a new temporary directory.
    pub fn empty() -> Self {
        let temp = tempfile::tempdir().unwrap();
        Self {
            path: temp.path().to_path_buf(),
            _temp: Some(temp),
        }
    }

    /// The base directory of the spec in a new temporary directory.
    pub fn base() -> Self {
        Self::empty().with_base()
    }

    /// The directory `relative` under `root`, which the caller keeps.
    pub fn at(root: &Path, relative: &str) -> Self {
        let path = root.join(relative);
        fs::create_dir_all(&path).unwrap();
        Self { path, _temp: None }
    }

    /// A directory with the world `alpha/aria` of the type `tinymux` on `127.0.0.1:port`, and
    /// `character` as `characters/alpha/aria.toml`.
    pub fn tcp_world(port: u16, character: &str) -> Self {
        let game = format!("extends = \"tinymux\"\nhost = \"127.0.0.1\"\nport = {port}\n");
        Self::empty()
            .file("games/alpha.toml", &game)
            .file("characters/alpha/aria.toml", character)
    }

    /// Adds the base files: `games/alpha.toml` with [`BASE_GAME`], and an empty
    /// `characters/alpha/aria.toml`.
    pub fn with_base(self) -> Self {
        self.file("games/alpha.toml", BASE_GAME)
            .file("characters/alpha/aria.toml", "")
    }

    /// Writes `text` to the file at `path`, relative to the directory.
    pub fn file(self, path: &str, text: &str) -> Self {
        let path = self.path.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
        self
    }

    /// The path of the directory.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// The standard output of a finished command, as text.
pub fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout is UTF-8")
}

/// The standard error of a finished command, as text.
pub fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr is UTF-8")
}

/// Asserts that `output` failed with status 1, and that stderr holds each of `names`.
#[track_caller]
pub fn fails_naming(output: &Output, names: &[&str]) {
    let error = stderr(output);
    assert_eq!(output.status.code(), Some(1), "stderr: {error}");
    for name in names {
        assert!(error.contains(name), "stderr names {name}: {error}");
    }
}

/// The text of a file of the repository, by its path from the repository root.
pub fn repo_file(relative: &str) -> String {
    let path = repo_root().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{relative} is readable: {error}"))
}

/// Each level-two heading of a Markdown page with the text of its section, read with a Markdown
/// parser. Code spans keep their backticks. A line break and the end of each block give a line end.
pub fn h2_sections(text: &str) -> Vec<(String, String)> {
    use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

    let mut sections: Vec<(String, String)> = Vec::new();
    let mut heading: Option<String> = None;
    let options = Options::ENABLE_YAML_STYLE_METADATA_BLOCKS | Options::ENABLE_TABLES;
    for event in Parser::new_ext(text, options) {
        match event {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H2,
                ..
            }) => heading = Some(String::new()),
            Event::End(TagEnd::Heading(HeadingLevel::H2)) => {
                sections.push((heading.take().unwrap_or_default(), String::new()));
            }
            Event::Text(text) | Event::Code(text) if heading.is_some() => {
                heading.get_or_insert_default().push_str(&text);
            }
            event => {
                let Some((_, body)) = sections.last_mut() else {
                    continue;
                };
                match event {
                    Event::Text(text) => body.push_str(&text),
                    Event::Code(code) => {
                        body.push('`');
                        body.push_str(&code);
                        body.push('`');
                    }
                    Event::SoftBreak
                    | Event::HardBreak
                    | Event::End(
                        TagEnd::Paragraph
                        | TagEnd::Item
                        | TagEnd::List(_)
                        | TagEnd::CodeBlock
                        | TagEnd::BlockQuote(_)
                        | TagEnd::TableCell
                        | TagEnd::Heading(_)
                        | TagEnd::HtmlBlock,
                    ) => body.push('\n'),
                    _ => {}
                }
            }
        }
    }
    sections
}

/// The text of each level-two heading of a Markdown page, in order.
pub fn h2_headings(text: &str) -> Vec<String> {
    h2_sections(text)
        .into_iter()
        .map(|(heading, _)| heading)
        .collect()
}

/// The metadata of the Cargo workspace of `manifest`, without its dependencies.
pub fn workspace_metadata(manifest: &Path) -> cargo_metadata::Metadata {
    cargo_metadata::MetadataCommand::new()
        .manifest_path(manifest)
        .no_deps()
        .exec()
        .expect("cargo metadata reads the workspace")
}

/// Every file under `dir`, at any depth.
pub fn files_under(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            files.extend(files_under(&entry.path()));
        } else {
            files.push(entry.path());
        }
    }
    files
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for file in files_under(from) {
        let target = to.join(file.strip_prefix(from).unwrap());
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::copy(&file, target).unwrap();
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

/// The permission bits of `path`.
pub fn mode(path: &Path) -> u32 {
    fs::metadata(path).unwrap().permissions().mode() & 0o777
}
