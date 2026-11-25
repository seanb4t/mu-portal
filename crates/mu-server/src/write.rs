//! The writes of the configuration store: a layout file as a whole, and one field of a device
//! layer. Each write goes to a temporary file in the target directory, and a rename puts it in
//! place, so a reader never sees a part of a file.

use std::fs;
use std::io::Write as _;
use std::path::Path;

use toml_edit::{Array, DocumentMut, Item, Table, value};

/// Makes `dir` if it is missing.
///
/// # Errors
///
/// The text of the error for a path that exists and is no directory, and for a directory that
/// the server cannot make. It names `shown`, the path relative to the configuration directory.
pub fn ensure_dir(dir: &Path, shown: &str) -> Result<(), String> {
    // A link to a directory is a directory here, as it is for the reader of the layers.
    match fs::metadata(dir) {
        Ok(found) if found.is_dir() => Ok(()),
        Ok(_) => Err(format!("{shown} is not a directory")),
        Err(_) => fs::create_dir_all(dir).map_err(|error| format!("{shown}: {error}")),
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
    let dir = target
        .parent()
        .ok_or_else(|| failed(&"the path has no directory"))?;
    let mut file = tempfile::NamedTempFile::new_in(dir).map_err(|error| failed(&error))?;
    file.write_all(text.as_bytes())
        .map_err(|error| failed(&error))?;
    file.persist(target).map_err(|error| failed(&error.error))?;
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
    let mut file: DocumentMut = text.parse().map_err(|error| format!("{error}"))?;
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
