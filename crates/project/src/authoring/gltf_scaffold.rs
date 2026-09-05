//! Scene look L5b (plan `look/05b`): the scaffold that writes the authoring
//! records for a glTF file (`cargo run -p xtask -- gltf-scaffold`).

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use super::gltf::{GltfDocument, GltfError, GltfImageUse, sibling_path};
use super::{ProjectAuthoringError, read_file, safe_join};
use next_contracts::canonical::sha256;

/// The records of one glTF file as a JSON document to paste into the
/// authoring manifest: `referenced_sources` (the entries the manifest
/// lacks), `render_records` (`texture-png`, `material`, `mesh-gltf`) and
/// a `placements` table (node, mesh, primitive, the mesh and material
/// asset ids). Asset ids are `[asset_base + n; 16]`, textures first, then
/// materials, then the meshes of the default scene in traversal order.
pub fn scaffold_gltf(
    project_directory: &Path,
    relative_path: &str,
    asset_base: u8,
) -> Result<String, ProjectAuthoringError> {
    let manifest_bytes = read_file(&safe_join(
        project_directory,
        super::PROJECT_AUTHORING_MANIFEST_FILE,
    )?)?;
    let manifest: serde_json::Value = serde_json::from_slice(&manifest_bytes)?;
    let mut existing_ids = Vec::new();
    collect_asset_ids(&manifest, &mut existing_ids);
    let provenance = &manifest["provenance"];
    let license = provenance["license_id"]
        .as_str()
        .unwrap_or("CC0-1.0")
        .to_owned();
    let source_identity = provenance["source_identity"]
        .as_str()
        .unwrap_or("urn:nextengine:project")
        .to_owned();
    let declared: Vec<String> = provenance["referenced_sources"]
        .as_array()
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| entry["relative_path"].as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    let notice_path = provenance["referenced_sources"]
        .as_array()
        .and_then(|entries| entries.first())
        .and_then(|entry| entry["notice_path"].as_str())
        .unwrap_or("NOTICE")
        .to_owned();

    let gltf_bytes = read_file(&safe_join(project_directory, relative_path)?)?;
    let document = GltfDocument::parse(&gltf_bytes, |uri| {
        let sibling = sibling_path(relative_path, uri)
            .ok_or_else(|| GltfError::UndeclaredSource(uri.to_owned()))?;
        safe_join(project_directory, &sibling)
            .and_then(|path| read_file(&path))
            .map_err(|_| GltfError::UndeclaredSource(sibling))
    })
    .map_err(ProjectAuthoringError::Gltf)?;

    // Sources: the document, its external files.
    let mut source_files = vec![relative_path.to_owned()];
    for file in document.external_files() {
        let sibling = sibling_path(relative_path, file).ok_or_else(|| {
            ProjectAuthoringError::Gltf(GltfError::UndeclaredSource(file.clone()))
        })?;
        if !source_files.contains(&sibling) {
            source_files.push(sibling);
        }
    }

    // Materials and the images they bind, by use.
    let scene_meshes = document
        .scene_meshes()
        .map_err(ProjectAuthoringError::Gltf)?;
    let mut material_indices = Vec::new();
    for scene_mesh in &scene_meshes {
        for primitive in 0..scene_mesh.primitive_count {
            if let Some(material) = document
                .primitive_material(scene_mesh.mesh, primitive)
                .map_err(ProjectAuthoringError::Gltf)?
                && !material_indices.contains(&material)
            {
                material_indices.push(material);
            }
        }
    }
    let mut summaries = Vec::new();
    let mut texture_keys: Vec<(usize, GltfImageUse, String)> = Vec::new();
    for material in &material_indices {
        let summary = document
            .material_summary(*material)
            .map_err(ProjectAuthoringError::Gltf)?;
        for (slot, image) in [
            (GltfImageUse::BaseColor, &summary.base_color_image),
            (
                GltfImageUse::MetallicRoughness,
                &summary.metallic_roughness_image,
            ),
            (GltfImageUse::Normal, &summary.normal_image),
        ] {
            if let Some((index, file)) = image {
                let key = (*index, slot, file.clone());
                if !texture_keys.contains(&key) {
                    texture_keys.push(key);
                }
            }
        }
        summaries.push(summary);
    }

    // Ids.
    let mesh_count: usize = scene_meshes.iter().map(|mesh| mesh.primitive_count).sum();
    let total = texture_keys.len() + summaries.len() + mesh_count;
    let last = usize::from(asset_base)
        .checked_add(total)
        .filter(|last| *last <= 0x100)
        .ok_or(ProjectAuthoringError::InvalidValue)?;
    let ids: Vec<String> = (usize::from(asset_base)..last)
        .map(|byte| format!("{byte:02x}").repeat(16))
        .collect();
    if let Some(collision) = ids.iter().find(|id| existing_ids.contains(id)) {
        return Err(ProjectAuthoringError::HashMismatch(format!(
            "asset id {collision} from base 0x{asset_base:02x} is already in the manifest"
        )));
    }
    let mut next_id = ids.iter();
    let mut texture_ids: BTreeMap<(usize, GltfImageUse), String> = BTreeMap::new();
    for (index, slot, _) in &texture_keys {
        texture_ids.insert((*index, *slot), next_id.next().expect("counted").clone());
    }
    let material_ids: Vec<String> = summaries
        .iter()
        .map(|_| next_id.next().expect("counted").clone())
        .collect();

    let mut out = String::new();
    out.push_str("{\n  \"referenced_sources\": [\n");
    let mut first = true;
    for file in &source_files {
        if declared.contains(file) {
            continue;
        }
        let bytes = read_file(&safe_join(project_directory, file)?)?;
        let hash = hex(&sha256(&bytes));
        let stem = relative_path
            .rsplit('/')
            .next()
            .unwrap_or(relative_path)
            .rsplit_once('.')
            .map_or(relative_path, |(stem, _)| stem);
        let file_name = file.rsplit('/').next().unwrap_or(file);
        if !first {
            out.push_str(",\n");
        }
        first = false;
        let _ = write!(
            out,
            "    {{\n      \"logical_source\": \"{source_identity}:gltf:{stem}:{file_name}\",\n      \"relative_path\": \"{file}\",\n      \"sha256\": \"{hash}\",\n      \"license_expression\": \"{license}\",\n      \"notice_path\": \"{notice_path}\",\n      \"source_span\": {{ \"relative_path\": \"{file}\", \"line\": 1, \"column\": 1 }}\n    }}"
        );
    }
    out.push_str("\n  ],\n  \"render_records\": [\n");
    let mut first = true;
    for (index, slot, file) in &texture_keys {
        let id = &texture_ids[&(*index, *slot)];
        let path = sibling_path(relative_path, file).expect("checked");
        let color_space = match slot {
            GltfImageUse::BaseColor => "srgb",
            GltfImageUse::MetallicRoughness | GltfImageUse::Normal => "linear",
        };
        if !first {
            out.push_str(",\n");
        }
        first = false;
        let _ = write!(
            out,
            "    {{\n      \"kind\": \"texture-png\",\n      \"asset_id\": \"{id}\",\n      \"record_revision\": 1,\n      \"relative_path\": \"{path}\",\n      \"color_space\": \"{color_space}\",\n      \"alpha\": \"opaque\",\n      \"mip_levels\": \"full\",\n      \"source_span\": {{ \"relative_path\": \"{path}\", \"line\": 1, \"column\": 1 }}\n    }}"
        );
    }
    for (summary, id) in summaries.iter().zip(&material_ids) {
        let base = summary
            .base_color_image
            .as_ref()
            .map(|(index, _)| texture_ids[&(*index, GltfImageUse::BaseColor)].clone());
        let base_line = match &base {
            Some(id) => format!("      \"texture_asset_id\": \"{id}\",\n"),
            None => {
                "      \"texture_asset_id\": \"<a base-colour texture asset id>\",\n".to_owned()
            }
        };
        let mut maps = String::new();
        if let Some((index, _)) = &summary.metallic_roughness_image {
            let _ = writeln!(
                maps,
                "      \"metallic_roughness_texture_asset_id\": \"{}\",",
                texture_ids[&(*index, GltfImageUse::MetallicRoughness)]
            );
        }
        if let Some((index, _)) = &summary.normal_image {
            let _ = writeln!(
                maps,
                "      \"normal_texture_asset_id\": \"{}\",",
                texture_ids[&(*index, GltfImageUse::Normal)]
            );
        }
        let [r, g, b, a] = summary.base_color_rgba_u16;
        let [er, eg, eb] = summary.emissive_rgb_u16;
        if !first {
            out.push_str(",\n");
        }
        first = false;
        let _ = write!(
            out,
            "    {{\n      \"kind\": \"material\",\n      \"asset_id\": \"{id}\",\n      \"record_revision\": 1,\n{base_line}{maps}      \"uv_scale\": 1.0,\n      \"base_color_rgba_u16\": [{r}, {g}, {b}, {a}],\n      \"metallic_u16\": {},\n      \"roughness_u16\": {},\n      \"emissive_rgb_u16\": [{er}, {eg}, {eb}],\n      \"double_sided\": {},\n      \"source_span\": {{ \"relative_path\": \"{relative_path}\", \"line\": 1, \"column\": 1 }}\n    }}",
            summary.metallic_u16, summary.roughness_u16, summary.double_sided
        );
    }
    let mut placements = Vec::new();
    for scene_mesh in &scene_meshes {
        for primitive in 0..scene_mesh.primitive_count {
            let id = next_id.next().expect("counted");
            let material = document
                .primitive_material(scene_mesh.mesh, primitive)
                .map_err(ProjectAuthoringError::Gltf)?;
            let double_sided = material
                .and_then(|material| material_indices.iter().position(|index| *index == material))
                .is_some_and(|position| summaries[position].double_sided);
            if !first {
                out.push_str(",\n");
            }
            first = false;
            let _ = write!(
                out,
                "    {{\n      \"kind\": \"mesh-gltf\",\n      \"asset_id\": \"{id}\",\n      \"record_revision\": 1,\n      \"relative_path\": \"{relative_path}\",\n      \"mesh\": {},\n      \"primitive\": {primitive},\n      \"node\": {},\n      \"double_sided\": {double_sided},\n      \"source_span\": {{ \"relative_path\": \"{relative_path}\", \"line\": 1, \"column\": 1 }}\n    }}",
                scene_mesh.mesh, scene_mesh.node
            );
            let material_id = material
                .and_then(|material| material_indices.iter().position(|index| *index == material))
                .map(|position| material_ids[position].clone());
            placements.push((
                scene_mesh.node,
                scene_mesh.node_name.clone(),
                scene_mesh.mesh,
                scene_mesh.mesh_name.clone(),
                primitive,
                id.clone(),
                material_id,
            ));
        }
    }
    out.push_str("\n  ],\n  \"placements\": [\n");
    for (position, (node, node_name, mesh, mesh_name, primitive, mesh_id, material_id)) in
        placements.iter().enumerate()
    {
        if position > 0 {
            out.push_str(",\n");
        }
        let _ = write!(
            out,
            "    {{ \"node\": {node}, \"node_name\": {}, \"mesh\": {mesh}, \"mesh_name\": {}, \"primitive\": {primitive}, \"mesh_asset_id\": \"{mesh_id}\", \"material_asset_id\": {} }}",
            json_string(node_name.as_deref()),
            json_string(mesh_name.as_deref()),
            json_string(material_id.as_deref())
        );
    }
    out.push_str("\n  ]\n}\n");
    Ok(out)
}

fn json_string(value: Option<&str>) -> String {
    match value {
        Some(value) => serde_json::to_string(value).unwrap_or_else(|_| "null".to_owned()),
        None => "null".to_owned(),
    }
}

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

fn collect_asset_ids(value: &serde_json::Value, out: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, entry) in map {
                if key.ends_with("asset_id")
                    && let Some(id) = entry.as_str()
                {
                    out.push(id.to_owned());
                }
                collect_asset_ids(entry, out);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_asset_ids(item, out);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::scaffold_gltf;
    use std::path::Path;

    fn reference_project() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../projects/reference-alpha")
    }

    #[test]
    fn tank_scaffold_names_its_records_in_order() {
        // The tank's records live in the manifest from base 0x50, so a
        // fresh base is scaffolded and compared.
        let fragment = scaffold_gltf(&reference_project(), "assets/models/water_tank.gltf", 0x10)
            .expect("scaffold");
        let value: serde_json::Value = serde_json::from_str(&fragment).expect("fragment parses");
        // The files are declared already, so no source entries.
        assert_eq!(
            value["referenced_sources"].as_array().map(Vec::len),
            Some(0)
        );
        let records = value["render_records"].as_array().expect("records");
        assert_eq!(records.len(), 11);
        let kinds: Vec<&str> = records
            .iter()
            .map(|record| record["kind"].as_str().expect("kind"))
            .collect();
        assert_eq!(
            kinds,
            [
                "texture-png",
                "texture-png",
                "texture-png",
                "material",
                "mesh-gltf",
                "mesh-gltf",
                "mesh-gltf",
                "mesh-gltf",
                "mesh-gltf",
                "mesh-gltf",
                "mesh-gltf"
            ]
        );
        let ids: Vec<&str> = records
            .iter()
            .map(|record| record["asset_id"].as_str().expect("id"))
            .collect();
        let expected: Vec<String> = (0x10_u32..0x1b)
            .map(|byte| format!("{byte:02x}").repeat(16))
            .collect();
        assert_eq!(ids, expected);
        assert_eq!(records[0]["color_space"], "srgb");
        assert_eq!(records[1]["color_space"], "linear");
        assert_eq!(records[2]["color_space"], "linear");
        assert_eq!(records[3]["texture_asset_id"], ids[0]);
        assert_eq!(records[3]["metallic_roughness_texture_asset_id"], ids[1]);
        assert_eq!(records[3]["normal_texture_asset_id"], ids[2]);
        assert_eq!(records[3]["double_sided"], false);
        assert_eq!(records[4]["node"], 0);
        assert_eq!(records[5]["node"], 1);
        assert_eq!(records[7]["mesh"], 2);
        let placements = value["placements"].as_array().expect("placements");
        assert_eq!(placements.len(), 7);
        assert_eq!(placements[1]["node_name"], "tank_lid");
        assert!(
            placements
                .iter()
                .all(|placement| placement["material_asset_id"] == ids[3])
        );
    }

    #[test]
    fn colliding_base_is_refused() {
        let error = scaffold_gltf(&reference_project(), "assets/models/water_tank.gltf", 0x50)
            .expect_err("collides");
        assert!(
            error.to_string().contains("already in the manifest"),
            "{error}"
        );
    }
}
