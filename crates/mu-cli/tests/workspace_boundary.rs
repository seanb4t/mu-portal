//! Rules of the workspace: members, dependency edges, license, lints and
//! the shared dependency table. `ALLOWED_EDGES` is the single source of the
//! edge rule.

use std::fs;
use std::path::{Path, PathBuf};

use cargo_metadata::MetadataCommand;

use support::TreeCopy;

mod support;

/// Each member and the members that it can depend on.
const ALLOWED_EDGES: &[(&str, &[&str])] = &[
    ("mu-proto", &[]),
    ("mu-core", &["mu-proto"]),
    ("mu-tui", &["mu-proto"]),
    ("mu-cli", &["mu-proto", "mu-core", "mu-tui"]),
];

const LICENSE: &str = "Apache-2.0";

/// The package fields that each member takes from `[workspace.package]`.
const INHERITED_PACKAGE_FIELDS: [&str; 2] = ["rust-version", "publish"];

const DEPENDENCY_TABLES: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

#[derive(Debug, PartialEq, Eq)]
enum Violation {
    UnlistedMember {
        member: String,
    },
    ForbiddenEdge {
        from: String,
        to: String,
    },
    SpikeDependency {
        from: String,
        to: String,
    },
    OutsidePathDependency {
        from: String,
        to: String,
    },
    License {
        member: String,
        found: Option<String>,
    },
    NotInherited {
        member: String,
        field: &'static str,
    },
    PublishableWorkspace,
    DependencyNotFromWorkspace {
        member: String,
        key: String,
    },
}

fn allowed(from: &str) -> Option<&'static [&'static str]> {
    ALLOWED_EDGES
        .iter()
        .find(|(member, _)| *member == from)
        .map(|(_, to)| *to)
}

/// Returns each rule that the workspace at `root` breaks.
fn violations(root: &Path) -> Vec<Violation> {
    let metadata = MetadataCommand::new()
        .manifest_path(root.join("Cargo.toml"))
        .no_deps()
        .exec()
        .expect("cargo metadata reads the workspace");
    let spikes = metadata.workspace_root.join("spikes");
    let members = metadata.workspace_packages();
    let member_directories: Vec<_> = members
        .iter()
        .filter_map(|member| member.manifest_path.parent())
        .collect();
    let is_member = |name: &str| {
        allowed(name).is_some() || members.iter().any(|member| member.name.as_str() == name)
    };

    let mut found = Vec::new();
    if !workspace_is_private(metadata.workspace_root.as_std_path()) {
        found.push(Violation::PublishableWorkspace);
    }
    for package in &members {
        let member = package.name.to_string();
        let allowed_targets = allowed(&member);
        if allowed_targets.is_none() {
            found.push(Violation::UnlistedMember {
                member: member.clone(),
            });
        }

        // Declared dependencies: every kind, optional ones and renamed ones.
        for dependency in &package.dependencies {
            let to = dependency.name.clone();
            // The path decides first: a crate outside the workspace can carry a member name.
            let outside_path = dependency
                .path
                .as_ref()
                .filter(|path| !member_directories.contains(&path.as_path()));
            if let Some(path) = outside_path {
                if path.starts_with(&spikes) {
                    found.push(Violation::SpikeDependency {
                        from: member.clone(),
                        to,
                    });
                } else {
                    found.push(Violation::OutsidePathDependency {
                        from: member.clone(),
                        to,
                    });
                }
            } else if is_member(&to) {
                // A member is a member whatever its source: path, git or registry.
                if !allowed_targets.is_some_and(|targets| targets.contains(&to.as_str())) {
                    found.push(Violation::ForbiddenEdge {
                        from: member.clone(),
                        to,
                    });
                }
            }
        }

        if package.license.as_deref() != Some(LICENSE) {
            found.push(Violation::License {
                member: member.clone(),
                found: package.license.clone(),
            });
        }

        let manifest = read_manifest(package.manifest_path.as_std_path());
        // The manifest decides: the metadata cannot tell an inherited value from a local one.
        let package = manifest.get("package");
        let fields = std::iter::once(("lints", manifest.get("lints"))).chain(
            INHERITED_PACKAGE_FIELDS
                .into_iter()
                .map(|field| (field, package.and_then(|package| package.get(field)))),
        );
        for (field, value) in fields {
            if !inherits_from_workspace(value) {
                found.push(Violation::NotInherited {
                    member: member.clone(),
                    field,
                });
            }
        }

        for (key, entry) in dependency_entries(&manifest) {
            let from_workspace = entry
                .get("workspace")
                .and_then(toml::Value::as_bool)
                .unwrap_or(false);
            let package_name = entry
                .get("package")
                .and_then(toml::Value::as_str)
                .unwrap_or(key);
            // A path dependency and a dependency on a member have their own rules above.
            let has_own_rule = entry.get("path").is_some() || is_member(package_name);
            if !from_workspace && !has_own_rule {
                found.push(Violation::DependencyNotFromWorkspace {
                    member: member.clone(),
                    key: key.clone(),
                });
            }
        }
    }
    found
}

fn read_manifest(path: &Path) -> toml::Table {
    fs::read_to_string(path)
        .expect("the manifest is readable")
        .parse()
        .expect("the manifest is TOML")
}

/// True for a value of the form `{ workspace = true }`.
fn inherits_from_workspace(value: Option<&toml::Value>) -> bool {
    value
        .and_then(|value| value.get("workspace"))
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

/// True if `[workspace.package]` holds `publish = false`. A list of registries is not private.
fn workspace_is_private(root: &Path) -> bool {
    read_manifest(&root.join("Cargo.toml"))
        .get("workspace")
        .and_then(|workspace| workspace.get("package"))
        .and_then(|package| package.get("publish"))
        .and_then(toml::Value::as_bool)
        == Some(false)
}

/// Every entry of every dependency table of a manifest, target tables included.
fn dependency_entries(manifest: &toml::Table) -> Vec<(&String, &toml::Value)> {
    let mut scopes = vec![manifest];
    if let Some(targets) = manifest.get("target").and_then(toml::Value::as_table) {
        scopes.extend(targets.values().filter_map(toml::Value::as_table));
    }
    scopes
        .into_iter()
        .flat_map(|scope| DEPENDENCY_TABLES.iter().filter_map(|name| scope.get(*name)))
        .filter_map(toml::Value::as_table)
        .flatten()
        .collect()
}

fn base_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/base")
}

fn fixture() -> TreeCopy {
    TreeCopy::of_dir(&base_fixture())
}

fn forbidden(from: &str, to: &str) -> Vec<Violation> {
    vec![Violation::ForbiddenEdge {
        from: from.to_owned(),
        to: to.to_owned(),
    }]
}

fn not_inherited(member: &str, field: &'static str) -> Vec<Violation> {
    vec![Violation::NotInherited {
        member: member.to_owned(),
        field,
    }]
}

fn not_from_workspace(member: &str, key: &str) -> Vec<Violation> {
    vec![Violation::DependencyNotFromWorkspace {
        member: member.to_owned(),
        key: key.to_owned(),
    }]
}

#[test]
fn real_workspace_has_no_violation() {
    assert_eq!(violations(support::repo_root()), vec![]);
}

#[test]
fn allowed_edges_have_no_violation() {
    assert_eq!(violations(&base_fixture()), vec![]);
}

#[test]
fn optional_tui_to_core_fails() {
    let fixture = fixture().append(
        "mu-tui",
        "mu-core = { path = \"../mu-core\", optional = true }\n",
    );
    assert_eq!(violations(fixture.root()), forbidden("mu-tui", "mu-core"));
}

#[test]
fn build_tui_to_core_fails() {
    let fixture = fixture().append(
        "mu-tui",
        "\n[build-dependencies]\nmu-core = { path = \"../mu-core\" }\n",
    );
    assert_eq!(violations(fixture.root()), forbidden("mu-tui", "mu-core"));
}

#[test]
fn dev_tui_to_core_fails() {
    let fixture = fixture().append(
        "mu-tui",
        "\n[dev-dependencies]\nmu-core = { path = \"../mu-core\" }\n",
    );
    assert_eq!(violations(fixture.root()), forbidden("mu-tui", "mu-core"));
}

#[test]
fn renamed_tui_to_core_fails() {
    let fixture = fixture().append(
        "mu-tui",
        "engine = { path = \"../mu-core\", package = \"mu-core\" }\n",
    );
    assert_eq!(violations(fixture.root()), forbidden("mu-tui", "mu-core"));
}

#[test]
fn target_tui_to_core_fails() {
    let fixture = fixture().append(
        "mu-tui",
        "\n[target.'cfg(unix)'.dependencies]\nmu-core = { path = \"../mu-core\" }\n",
    );
    assert_eq!(violations(fixture.root()), forbidden("mu-tui", "mu-core"));
}

#[test]
fn git_tui_to_core_fails() {
    let fixture = fixture().append(
        "mu-tui",
        "mu-core = { git = \"https://example.invalid/mu.git\" }\n",
    );
    assert_eq!(violations(fixture.root()), forbidden("mu-tui", "mu-core"));
}

#[test]
fn local_dependency_version_fails() {
    let fixture = fixture().append("mu-tui", "anyhow = \"1\"\n");
    assert_eq!(
        violations(fixture.root()),
        not_from_workspace("mu-tui", "anyhow")
    );
}

#[test]
fn local_dev_dependency_version_fails() {
    let fixture = fixture().append("mu-tui", "\n[dev-dependencies]\nanyhow = \"1\"\n");
    assert_eq!(
        violations(fixture.root()),
        not_from_workspace("mu-tui", "anyhow")
    );
}

#[test]
fn local_target_dependency_version_fails() {
    let fixture = fixture().append(
        "mu-tui",
        "\n[target.'cfg(unix)'.build-dependencies]\nanyhow = \"1\"\n",
    );
    assert_eq!(
        violations(fixture.root()),
        not_from_workspace("mu-tui", "anyhow")
    );
}

#[test]
fn renamed_local_dependency_version_fails() {
    let fixture = fixture().append(
        "mu-tui",
        "errors = { package = \"anyhow\", version = \"1\" }\n",
    );
    assert_eq!(
        violations(fixture.root()),
        not_from_workspace("mu-tui", "errors")
    );
}

#[test]
fn core_to_tui_fails() {
    let fixture = fixture().append("mu-core", "mu-tui = { path = \"../mu-tui\" }\n");
    assert_eq!(violations(fixture.root()), forbidden("mu-core", "mu-tui"));
}

#[test]
fn proto_dependency_fails() {
    let fixture = fixture().append(
        "mu-proto",
        "\n[dependencies]\nmu-core = { path = \"../mu-core\" }\n",
    );
    assert_eq!(violations(fixture.root()), forbidden("mu-proto", "mu-core"));
}

#[test]
fn spike_dependency_fails() {
    let fixture = fixture()
        .add_crate("spikes/probe", "probe")
        .append("mu-core", "probe = { path = \"../../spikes/probe\" }\n");
    assert_eq!(
        violations(fixture.root()),
        vec![Violation::SpikeDependency {
            from: "mu-core".to_owned(),
            to: "probe".to_owned(),
        }]
    );
}

#[test]
fn spike_named_like_member_fails() {
    let fixture = fixture()
        .add_crate("spikes/mu-core", "mu-core")
        .append("mu-tui", "mu-core = { path = \"../../spikes/mu-core\" }\n");
    assert_eq!(
        violations(fixture.root()),
        vec![Violation::SpikeDependency {
            from: "mu-tui".to_owned(),
            to: "mu-core".to_owned(),
        }]
    );
}

#[test]
fn outside_path_dependency_fails() {
    let fixture = fixture()
        .add_crate("outside/helper", "helper")
        .append("mu-core", "helper = { path = \"../../outside/helper\" }\n");
    assert_eq!(
        violations(fixture.root()),
        vec![Violation::OutsidePathDependency {
            from: "mu-core".to_owned(),
            to: "helper".to_owned(),
        }]
    );
}

#[test]
fn unlisted_member_fails() {
    let fixture = fixture().add_crate("crates/mu-extra", "mu-extra");
    assert_eq!(
        violations(fixture.root()),
        vec![Violation::UnlistedMember {
            member: "mu-extra".to_owned(),
        }]
    );
}

#[test]
fn missing_license_fails() {
    let fixture = fixture().replace("mu-core", "license.workspace = true\n", "");
    assert_eq!(
        violations(fixture.root()),
        vec![Violation::License {
            member: "mu-core".to_owned(),
            found: None,
        }]
    );
}

#[test]
fn wrong_license_fails() {
    let fixture = fixture().replace(
        "mu-core",
        "license.workspace = true\n",
        "license = \"MIT\"\n",
    );
    assert_eq!(
        violations(fixture.root()),
        vec![Violation::License {
            member: "mu-core".to_owned(),
            found: Some("MIT".to_owned()),
        }]
    );
}

#[test]
fn missing_lints_fails() {
    let fixture = fixture().replace("mu-core", "[lints]\nworkspace = true\n", "");
    assert_eq!(
        violations(fixture.root()),
        not_inherited("mu-core", "lints")
    );
}

#[test]
fn local_lints_fail() {
    let fixture = fixture().replace(
        "mu-core",
        "[lints]\nworkspace = true\n",
        "[lints.rust]\nunsafe_code = \"forbid\"\n",
    );
    assert_eq!(
        violations(fixture.root()),
        not_inherited("mu-core", "lints")
    );
}

#[test]
fn missing_rust_version_fails() {
    let fixture = fixture().replace("mu-core", "rust-version.workspace = true\n", "");
    assert_eq!(
        violations(fixture.root()),
        not_inherited("mu-core", "rust-version")
    );
}

#[test]
fn local_rust_version_fails() {
    let fixture = fixture().replace(
        "mu-core",
        "rust-version.workspace = true\n",
        "rust-version = \"1.90\"\n",
    );
    assert_eq!(
        violations(fixture.root()),
        not_inherited("mu-core", "rust-version")
    );
}

#[test]
fn missing_publish_fails() {
    let fixture = fixture().replace("mu-core", "publish.workspace = true\n", "");
    assert_eq!(
        violations(fixture.root()),
        not_inherited("mu-core", "publish")
    );
}

#[test]
fn local_publish_fails() {
    let fixture = fixture().replace("mu-core", "publish.workspace = true\n", "publish = false\n");
    assert_eq!(
        violations(fixture.root()),
        not_inherited("mu-core", "publish")
    );
}

#[test]
fn publishable_workspace_fails() {
    // Cargo itself refuses a workspace with no `publish`, because each member inherits it.
    for publish in ["publish = true\n", "publish = [\"crates-io\"]\n"] {
        let fixture =
            fixture().replace_in_file(Path::new("Cargo.toml"), "publish = false\n", publish);
        assert_eq!(
            violations(fixture.root()),
            vec![Violation::PublishableWorkspace],
            "with {publish:?}"
        );
    }
}
