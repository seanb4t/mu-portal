//! The merge of named layers with `figment2`, with the source of each field.

use std::collections::BTreeMap;

use figment2::Figment;
use figment2::providers::{Named, Serialized};
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Merges `layers` in order, so a later layer wins, and a table merges key by key. An unset
/// field of a layer must serialize as nothing, so that it keeps the value of an earlier layer.
///
/// It gives the merged value, and for each path of `paths` that the merged value holds, the
/// place in `layers` of the last layer that set it. A path has `.` between its parts.
///
/// # Errors
///
/// The text of the error of the merge.
pub fn merge_layers<T: Serialize + DeserializeOwned>(
    layers: &[&T],
    paths: &[String],
) -> Result<(T, BTreeMap<String, usize>), String> {
    let mut figment = Figment::new();
    for (place, layer) in layers.iter().enumerate() {
        figment = figment.merge(Named::new(place.to_string(), Serialized::defaults(layer)));
    }
    let merged: T = figment.extract().map_err(|error| error.to_string())?;
    // The tag of a merged list is the tag of the first list, so the tags do not give the layer
    // of a list. Each layer alone says whether it sets a path.
    let alone: Vec<Figment> = layers
        .iter()
        .map(|layer| Figment::new().merge(Serialized::defaults(layer)))
        .collect();
    let mut sources = BTreeMap::new();
    for path in paths {
        if figment.find_value(path).is_ok()
            && let Some(place) = alone
                .iter()
                .rposition(|layer| layer.find_value(path).is_ok())
        {
            sources.insert(path.clone(), place);
        }
    }
    Ok((merged, sources))
}
