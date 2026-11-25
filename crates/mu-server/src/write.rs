//! The writes of the configuration store: a layout file as a whole, and one field of a device
//! layer. Each write goes to a temporary file in the target directory, and a rename puts it in
//! place, so a reader never sees a part of a file.

use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

use toml_edit::{Array, DocumentMut, Item, Table, value};

/// Why a directory of the store is not there for a write.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DirError {
    /// The path exists and is no directory.
    NotADirectory(String),
    /// The server could not read the path or make the directory.
    Failed(String),
}

/// Makes `dir` if it is missing.
///
/// # Errors
///
/// A [`DirError`] for a path that exists and is no directory, and for a directory that the
/// server cannot read or make. Its text names `shown`, the path relative to the configuration
/// directory.
pub fn ensure_dir(dir: &Path, shown: &str) -> Result<(), DirError> {
    let failed = |error: io::Error| DirError::Failed(format!("{shown}: {error}"));
    // A link to a directory is a directory here, as it is for the reader of the layers.
    match fs::metadata(dir) {
        Ok(found) if found.is_dir() => Ok(()),
        Ok(_) => Err(DirError::NotADirectory(format!(
            "{shown} is not a directory"
        ))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            fs::create_dir_all(dir).map_err(failed)
        }
        Err(error) => Err(failed(error)),
    }
}

/// The file that a write to `target` replaces: the file that a link names, or `target`. The
/// write follows a link only to a regular file.
///
/// The error is the text of the reason, with no path of the file system in it.
fn resolved(target: &Path) -> Result<PathBuf, String> {
    match fs::symlink_metadata(target) {
        Ok(found) if found.file_type().is_symlink() => {
            let named = fs::canonicalize(target)
                .map_err(|_| "is a link to nothing, so the server does not write it".to_owned())?;
            match fs::metadata(&named) {
                Ok(found) if found.is_file() => Ok(named),
                _ => Err(
                    "is a link to something that is no regular file, so the server does not write it"
                        .to_owned(),
                ),
            }
        }
        _ => Ok(target.to_path_buf()),
    }
}

/// Writes `text` to `target` as a whole.
///
/// # Errors
///
/// The text of the error of the write, with `shown`, the path relative to the configuration
/// directory.
pub fn write_whole(target: &Path, shown: &str, text: &str) -> Result<(), String> {
    let failed = |error: &dyn std::fmt::Display| format!("{shown}: {error}");
    // A link stays a link: the write replaces the file that it names.
    let target = resolved(target).map_err(|error| failed(&error))?;
    let dir = target
        .parent()
        .ok_or_else(|| failed(&"the path has no directory"))?;
    // The error of `tempfile` holds the path of the directory, which can be outside the
    // configuration directory for a linked file. The kind of the error says enough.
    let mut file = tempfile::NamedTempFile::new_in(dir)
        .map_err(|error| failed(&io::Error::from(error.kind())))?;
    file.write_all(text.as_bytes())
        .map_err(|error| failed(&error))?;
    // A file that the write replaces keeps its permissions.
    if let Ok(old) = fs::metadata(&target) {
        fs::set_permissions(file.path(), old.permissions()).map_err(|error| failed(&error))?;
    }
    // The text is on the disk before the rename, so a crash leaves the old file or the new one.
    file.as_file().sync_all().map_err(|error| failed(&error))?;
    file.persist(&target)
        .map_err(|error| failed(&error.error))?;
    Ok(())
}

/// The text of a layout file with `lines`, one line of the plan on each line of the file.
#[must_use]
pub fn layout_text(lines: &[String]) -> String {
    let mut place = Array::new();
    for line in lines {
        place.push(line.as_str());
    }
    for item in place.iter_mut() {
        item.decor_mut().set_prefix("\n  ");
    }
    place.set_trailing_comma(true);
    place.set_trailing("\n");
    let mut file = DocumentMut::new();
    file["place"] = value(place);
    file.to_string()
}

/// The text of the error for `text` that is no TOML: the line, the column and the reason. It
/// holds no part of `text`, because a line of a layer can hold what the player keeps private.
fn not_toml(text: &str, error: &toml_edit::TomlError) -> String {
    let Some(span) = error.span() else {
        return error.message().to_owned();
    };
    let before = text.get(..span.start).unwrap_or(text);
    let line = before.matches('\n').count() + 1;
    let column = before
        .rsplit('\n')
        .next()
        .map_or(0, |last| last.chars().count())
        + 1;
    format!("line {line}, column {column}: {}", error.message())
}

/// Whether the layer `text` holds a value at `path`, which is as in [`edit_field`].
///
/// # Errors
///
/// The text of the error for a text that is no TOML.
pub fn has_field(text: &str, path: &[&str]) -> Result<bool, String> {
    let file: DocumentMut = text.parse().map_err(|error| not_toml(text, &error))?;
    Ok(match path {
        [field] => file.get(field).is_some(),
        [table, key] => file
            .get(table)
            .and_then(Item::as_table_like)
            .is_some_and(|entries| entries.contains_key(key)),
        _ => false,
    })
}

/// One change of a device layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldEdit {
    /// Set the value at this path.
    Set(String),
    /// Remove the value at this path.
    Remove,
}

/// The text of a layer after `edit` at `path`, which is a field such as `layout` or a table and
/// a key such as `["world-layouts", "alpha/aria"]`. Each other line of `text` stays.
///
/// # Errors
///
/// The text of the error for a text that is no TOML, and for a path whose table is a value.
pub fn edit_field(text: &str, path: &[&str], edit: &FieldEdit) -> Result<String, String> {
    let mut file: DocumentMut = text.parse().map_err(|error| not_toml(text, &error))?;
    match path {
        [field] => match edit {
            FieldEdit::Set(new) => file[*field] = value(new.as_str()),
            FieldEdit::Remove => {
                file.remove(field);
            }
        },
        [table, key] => {
            let not_a_table = || format!("{table} is not a table");
            match edit {
                FieldEdit::Set(new) => {
                    let entry = file.entry(table).or_insert(Item::Table(Table::new()));
                    let entries = entry.as_table_like_mut().ok_or_else(not_a_table)?;
                    entries.insert(key, value(new.as_str()));
                }
                FieldEdit::Remove => {
                    if let Some(entry) = file.get_mut(table) {
                        let entries = entry.as_table_like_mut().ok_or_else(not_a_table)?;
                        entries.remove(key);
                        if entries.is_empty() {
                            file.remove(table);
                        }
                    }
                }
            }
        }
        _ => return Err("a field path has one or two parts".to_owned()),
    }
    Ok(file.to_string())
}
