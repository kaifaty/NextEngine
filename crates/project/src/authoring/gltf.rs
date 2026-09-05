//! Scene look L5b (plan `look/05b`): glTF 2.0 sources for the `mesh-gltf`
//! authoring record and the scaffold command.
//!
//! The reader covers the subset the neutral mesh record can carry: the
//! JSON form with external or `data:` buffers and the GLB container;
//! triangle primitives with `POSITION`, `NORMAL`, `TANGENT`, `TEXCOORD_0`
//! and `u8`/`u16`/`u32` indices; the default scene's node transforms
//! baked into the vertices. Everything the record cannot carry is refused
//! with a diagnostic rather than approximated.

use std::collections::BTreeMap;

use serde::Deserialize;

const GLB_MAGIC: u32 = 0x4654_6C67;
const GLB_CHUNK_JSON: u32 = 0x4E4F_534A;
const GLB_CHUNK_BIN: u32 = 0x004E_4942;
const MICROMETRES_PER_METRE: f64 = 1_000_000.0;
const Q16_ONE: f64 = 65_536.0;
const SNORM16_ONE: f64 = 32_767.0;
const MODE_TRIANGLES: u32 = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GltfError {
    /// The JSON document or the GLB container does not parse.
    Malformed(String),
    /// A required extension the reader does not implement.
    RequiredExtension(String),
    /// A buffer or image URI that is not among the declared sources.
    UndeclaredSource(String),
    /// A buffer, buffer view or accessor is out of its container.
    AccessorOutOfBounds(usize),
    /// A sparse accessor.
    SparseAccessor(usize),
    /// A primitive topology other than triangles.
    Topology(u32),
    /// A missing attribute the record needs.
    MissingAttribute(&'static str),
    /// An attribute in a component type or element type the reader does
    /// not accept for that semantic.
    AttributeFormat(&'static str),
    /// An index beyond the vertex count.
    IndexOutOfRange,
    /// A mesh, primitive, node, material, texture or image index beyond
    /// its array.
    OutOfRange(&'static str, usize),
    /// A node that the default scene does not reach.
    NodeNotInScene(usize),
    /// A vertex that the quantisation cannot hold.
    Quantisation(&'static str),
    /// A degenerate normal, tangent or transform.
    Degenerate(&'static str),
    /// An image the scaffold cannot turn into a `texture-png` record.
    Image(String),
}

impl std::fmt::Display for GltfError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malformed(detail) => write!(formatter, "malformed glTF: {detail}"),
            Self::RequiredExtension(name) => {
                write!(formatter, "required glTF extension not supported: {name}")
            }
            Self::UndeclaredSource(uri) => {
                write!(
                    formatter,
                    "glTF source is not a declared referenced source: {uri}"
                )
            }
            Self::AccessorOutOfBounds(index) => {
                write!(formatter, "glTF accessor {index} leaves its buffer")
            }
            Self::SparseAccessor(index) => {
                write!(formatter, "glTF accessor {index} is sparse (not supported)")
            }
            Self::Topology(mode) => {
                write!(formatter, "glTF primitive mode {mode} is not triangles")
            }
            Self::MissingAttribute(name) => {
                write!(formatter, "glTF primitive lacks {name}")
            }
            Self::AttributeFormat(name) => {
                write!(formatter, "glTF attribute {name} has an unsupported format")
            }
            Self::IndexOutOfRange => write!(formatter, "glTF index beyond the vertex count"),
            Self::OutOfRange(what, index) => {
                write!(formatter, "glTF {what} index {index} is out of range")
            }
            Self::NodeNotInScene(index) => {
                write!(formatter, "glTF node {index} is not in the default scene")
            }
            Self::Quantisation(what) => {
                write!(
                    formatter,
                    "glTF {what} does not fit the neutral quantisation"
                )
            }
            Self::Degenerate(what) => write!(formatter, "glTF {what} is degenerate"),
            Self::Image(detail) => write!(formatter, "glTF image not importable: {detail}"),
        }
    }
}

impl std::error::Error for GltfError {}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GltfJson {
    #[serde(default)]
    extensions_required: Vec<String>,
    #[serde(default)]
    buffers: Vec<GltfBuffer>,
    #[serde(default)]
    buffer_views: Vec<GltfBufferView>,
    #[serde(default)]
    accessors: Vec<GltfAccessor>,
    #[serde(default)]
    meshes: Vec<GltfMesh>,
    #[serde(default)]
    nodes: Vec<GltfNode>,
    #[serde(default)]
    scenes: Vec<GltfScene>,
    #[serde(default)]
    scene: Option<usize>,
    #[serde(default)]
    materials: Vec<GltfMaterial>,
    #[serde(default)]
    textures: Vec<GltfTexture>,
    #[serde(default)]
    images: Vec<GltfImage>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GltfBuffer {
    byte_length: usize,
    #[serde(default)]
    uri: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GltfBufferView {
    buffer: usize,
    #[serde(default)]
    byte_offset: usize,
    byte_length: usize,
    #[serde(default)]
    byte_stride: Option<usize>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GltfAccessor {
    #[serde(default)]
    buffer_view: Option<usize>,
    #[serde(default)]
    byte_offset: usize,
    component_type: u32,
    #[serde(default)]
    normalized: bool,
    count: usize,
    #[serde(rename = "type")]
    element_type: String,
    #[serde(default)]
    sparse: Option<serde_json::Value>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GltfMesh {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    primitives: Vec<GltfPrimitive>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GltfPrimitive {
    #[serde(default)]
    attributes: BTreeMap<String, usize>,
    #[serde(default)]
    indices: Option<usize>,
    #[serde(default)]
    material: Option<usize>,
    #[serde(default = "default_mode")]
    mode: u32,
}

const fn default_mode() -> u32 {
    MODE_TRIANGLES
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GltfNode {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    mesh: Option<usize>,
    #[serde(default)]
    children: Vec<usize>,
    #[serde(default)]
    matrix: Option<[f64; 16]>,
    #[serde(default)]
    translation: Option<[f64; 3]>,
    #[serde(default)]
    rotation: Option<[f64; 4]>,
    #[serde(default)]
    scale: Option<[f64; 3]>,
}

#[derive(Debug, Default, Deserialize)]
struct GltfScene {
    #[serde(default)]
    nodes: Vec<usize>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GltfMaterial {
    #[serde(default)]
    pub(crate) name: Option<String>,
    #[serde(default)]
    pub(crate) pbr_metallic_roughness: GltfPbr,
    #[serde(default)]
    pub(crate) normal_texture: Option<GltfTextureInfo>,
    #[serde(default)]
    pub(crate) emissive_factor: [f64; 3],
    #[serde(default)]
    pub(crate) double_sided: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GltfPbr {
    #[serde(default = "default_base_color")]
    pub(crate) base_color_factor: [f64; 4],
    #[serde(default = "default_one")]
    pub(crate) metallic_factor: f64,
    #[serde(default = "default_one")]
    pub(crate) roughness_factor: f64,
    #[serde(default)]
    pub(crate) base_color_texture: Option<GltfTextureInfo>,
    #[serde(default)]
    pub(crate) metallic_roughness_texture: Option<GltfTextureInfo>,
}

impl Default for GltfPbr {
    fn default() -> Self {
        Self {
            base_color_factor: default_base_color(),
            metallic_factor: 1.0,
            roughness_factor: 1.0,
            base_color_texture: None,
            metallic_roughness_texture: None,
        }
    }
}

const fn default_base_color() -> [f64; 4] {
    [1.0, 1.0, 1.0, 1.0]
}

const fn default_one() -> f64 {
    1.0
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GltfTextureInfo {
    pub(crate) index: usize,
    #[serde(default)]
    pub(crate) tex_coord: u32,
}

#[derive(Debug, Default, Deserialize)]
struct GltfTexture {
    #[serde(default)]
    source: Option<usize>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GltfImage {
    #[serde(default)]
    uri: Option<String>,
    #[serde(default)]
    mime_type: Option<String>,
    #[serde(default)]
    buffer_view: Option<usize>,
}

/// A parsed document with its buffers resolved.
pub(crate) struct GltfDocument {
    json: GltfJson,
    buffers: Vec<Vec<u8>>,
    /// The external files the document names (buffers, then images),
    /// as URIs relative to the document, percent-decoded.
    external_files: Vec<String>,
}

/// One primitive's geometry in the neutral quantisation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GltfGeometry {
    pub(crate) positions_micrometres: Vec<[i64; 3]>,
    pub(crate) normals_snorm16: Option<Vec<[i16; 3]>>,
    pub(crate) tangents_snorm16: Option<Vec<([i16; 3], i8)>>,
    pub(crate) uv0_q16: Vec<[i32; 2]>,
    pub(crate) indices: Vec<u32>,
    pub(crate) bounds_min: [i64; 3],
    pub(crate) bounds_max: [i64; 3],
}

/// A mesh-bearing node of the default scene, in traversal order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GltfSceneMesh {
    pub(crate) node: usize,
    pub(crate) node_name: Option<String>,
    pub(crate) mesh: usize,
    pub(crate) mesh_name: Option<String>,
    pub(crate) primitive_count: usize,
}

/// How a material uses an image (the scaffold's colour-space rule).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GltfImageUse {
    BaseColor,
    MetallicRoughness,
    Normal,
}

impl GltfDocument {
    /// Parses a `.gltf` or `.glb` file; `resolve` returns the bytes of an
    /// external URI (relative to the document) or refuses it.
    pub(crate) fn parse(
        bytes: &[u8],
        mut resolve: impl FnMut(&str) -> Result<Vec<u8>, GltfError>,
    ) -> Result<Self, GltfError> {
        let (json_bytes, bin_chunk) = if bytes.len() >= 12 && read_u32(bytes, 0) == GLB_MAGIC {
            parse_glb(bytes)?
        } else {
            (bytes.to_vec(), None)
        };
        let json: GltfJson = serde_json::from_slice(&json_bytes)
            .map_err(|error| GltfError::Malformed(error.to_string()))?;
        if let Some(name) = json.extensions_required.first() {
            return Err(GltfError::RequiredExtension(name.clone()));
        }
        let mut buffers = Vec::with_capacity(json.buffers.len());
        let mut external_files = Vec::new();
        let mut bin_chunk = bin_chunk;
        for buffer in &json.buffers {
            let data = match &buffer.uri {
                None => bin_chunk.take().ok_or_else(|| {
                    GltfError::Malformed("buffer without uri or BIN chunk".into())
                })?,
                Some(uri) => match data_uri(uri) {
                    Some(payload) => decode_base64(payload)?,
                    None => {
                        let file = percent_decode(uri);
                        external_files.push(file.clone());
                        resolve(&file)?
                    }
                },
            };
            if data.len() < buffer.byte_length {
                return Err(GltfError::Malformed(
                    "buffer shorter than byteLength".into(),
                ));
            }
            buffers.push(data);
        }
        for image in &json.images {
            if let Some(uri) = &image.uri
                && data_uri(uri).is_none()
            {
                external_files.push(percent_decode(uri));
            }
        }
        Ok(Self {
            json,
            buffers,
            external_files,
        })
    }

    /// The external files the document names, in order (buffers, then
    /// images), percent-decoded and relative to the document.
    pub(crate) fn external_files(&self) -> &[String] {
        &self.external_files
    }

    pub(crate) fn materials(&self) -> &[GltfMaterial] {
        &self.json.materials
    }

    pub(crate) fn primitive_material(
        &self,
        mesh: usize,
        primitive: usize,
    ) -> Result<Option<usize>, GltfError> {
        Ok(self.primitive(mesh, primitive)?.material)
    }

    /// The image behind a texture index, as an external PNG URI.
    pub(crate) fn texture_image_uri(&self, texture: usize) -> Result<(usize, String), GltfError> {
        let texture_record = self
            .json
            .textures
            .get(texture)
            .ok_or(GltfError::OutOfRange("texture", texture))?;
        let image_index = texture_record
            .source
            .ok_or_else(|| GltfError::Image(format!("texture {texture} has no source")))?;
        let image = self
            .json
            .images
            .get(image_index)
            .ok_or(GltfError::OutOfRange("image", image_index))?;
        if image.buffer_view.is_some() {
            return Err(GltfError::Image(format!(
                "image {image_index} is embedded in a buffer view; only external PNG files are importable"
            )));
        }
        let uri = image
            .uri
            .as_ref()
            .ok_or_else(|| GltfError::Image(format!("image {image_index} has no uri")))?;
        if data_uri(uri).is_some() {
            return Err(GltfError::Image(format!(
                "image {image_index} is a data URI; only external PNG files are importable"
            )));
        }
        let file = percent_decode(uri);
        let is_png = image.mime_type.as_deref().map_or_else(
            || file.to_ascii_lowercase().ends_with(".png"),
            |mime| mime == "image/png",
        );
        if !is_png {
            return Err(GltfError::Image(format!(
                "image {image_index} ({file}) is not a PNG; only external PNG files are importable"
            )));
        }
        Ok((image_index, file))
    }

    /// The mesh-bearing nodes of the default scene, depth first in the
    /// scene's order.
    pub(crate) fn scene_meshes(&self) -> Result<Vec<GltfSceneMesh>, GltfError> {
        let mut result = Vec::new();
        let mut stack = self.scene_roots()?;
        stack.reverse();
        let mut visited = vec![false; self.json.nodes.len()];
        while let Some(index) = stack.pop() {
            let node = self
                .json
                .nodes
                .get(index)
                .ok_or(GltfError::OutOfRange("node", index))?;
            if std::mem::replace(&mut visited[index], true) {
                return Err(GltfError::Malformed(format!(
                    "node {index} is reached twice"
                )));
            }
            if let Some(mesh) = node.mesh {
                let mesh_record = self
                    .json
                    .meshes
                    .get(mesh)
                    .ok_or(GltfError::OutOfRange("mesh", mesh))?;
                result.push(GltfSceneMesh {
                    node: index,
                    node_name: node.name.clone(),
                    mesh,
                    mesh_name: mesh_record.name.clone(),
                    primitive_count: mesh_record.primitives.len(),
                });
            }
            for child in node.children.iter().rev() {
                stack.push(*child);
            }
        }
        Ok(result)
    }

    /// One primitive's geometry, the given node's global transform baked.
    pub(crate) fn geometry(
        &self,
        mesh: usize,
        primitive: usize,
        node: Option<usize>,
    ) -> Result<GltfGeometry, GltfError> {
        let primitive_record = self.primitive(mesh, primitive)?;
        if primitive_record.mode != MODE_TRIANGLES {
            return Err(GltfError::Topology(primitive_record.mode));
        }
        let model = match node {
            Some(node) => self.global_transform(node)?,
            None => IDENTITY,
        };
        let normal_matrix = normal_matrix(&model)?;
        let mirrored = determinant3(&model) < 0.0;

        let position_accessor = *primitive_record
            .attributes
            .get("POSITION")
            .ok_or(GltfError::MissingAttribute("POSITION"))?;
        let positions = self.read_floats(position_accessor, 3, "POSITION")?;
        let vertex_count = positions.len();
        let mut positions_micrometres = Vec::with_capacity(vertex_count);
        let mut bounds_min = [i64::MAX; 3];
        let mut bounds_max = [i64::MIN; 3];
        for position in &positions {
            let world = transform_point(&model, position);
            let mut quantised = [0_i64; 3];
            for axis in 0..3 {
                let value = (world[axis] * MICROMETRES_PER_METRE).round();
                if !value.is_finite() || value.abs() > 9.0e15 {
                    return Err(GltfError::Quantisation("position"));
                }
                quantised[axis] = value as i64;
                bounds_min[axis] = bounds_min[axis].min(quantised[axis]);
                bounds_max[axis] = bounds_max[axis].max(quantised[axis]);
            }
            positions_micrometres.push(quantised);
        }
        if vertex_count == 0 {
            return Err(GltfError::MissingAttribute("POSITION"));
        }

        let normals_snorm16 = match primitive_record.attributes.get("NORMAL") {
            Some(accessor) => {
                let normals = self.read_floats(*accessor, 3, "NORMAL")?;
                if normals.len() != vertex_count {
                    return Err(GltfError::AttributeFormat("NORMAL"));
                }
                let mut quantised = Vec::with_capacity(vertex_count);
                for normal in &normals {
                    let world =
                        transform_direction(&normal_matrix, &[normal[0], normal[1], normal[2]]);
                    quantised.push(snorm16_unit(&world, "normal")?);
                }
                Some(quantised)
            }
            None => None,
        };

        let tangents_snorm16 = match primitive_record.attributes.get("TANGENT") {
            Some(accessor) => {
                let tangents = self.read_floats(*accessor, 4, "TANGENT")?;
                if tangents.len() != vertex_count {
                    return Err(GltfError::AttributeFormat("TANGENT"));
                }
                let mut quantised = Vec::with_capacity(vertex_count);
                for tangent in &tangents {
                    let world = transform_direction(&model, &[tangent[0], tangent[1], tangent[2]]);
                    let mut handedness: i8 = if tangent[3] < 0.0 { -1 } else { 1 };
                    if mirrored {
                        handedness = -handedness;
                    }
                    quantised.push((snorm16_unit(&world, "tangent")?, handedness));
                }
                Some(quantised)
            }
            None => None,
        };

        let uv0_q16 = match primitive_record.attributes.get("TEXCOORD_0") {
            Some(accessor) => {
                let uvs = self.read_floats(*accessor, 2, "TEXCOORD_0")?;
                if uvs.len() != vertex_count {
                    return Err(GltfError::AttributeFormat("TEXCOORD_0"));
                }
                let mut quantised = Vec::with_capacity(vertex_count);
                for uv in &uvs {
                    let mut q = [0_i32; 2];
                    for axis in 0..2 {
                        let value = (uv[axis] * Q16_ONE).round();
                        if !value.is_finite() || value.abs() > f64::from(i32::MAX) {
                            return Err(GltfError::Quantisation("texcoord"));
                        }
                        q[axis] = value as i32;
                    }
                    quantised.push(q);
                }
                quantised
            }
            None => Vec::new(),
        };

        let indices = match primitive_record.indices {
            Some(accessor) => self.read_indices(accessor)?,
            None => {
                (0..u32::try_from(vertex_count).map_err(|_| GltfError::IndexOutOfRange)?).collect()
            }
        };
        if indices.is_empty() || indices.len() % 3 != 0 {
            return Err(GltfError::Malformed(
                "index count is not a multiple of three".into(),
            ));
        }
        if indices
            .iter()
            .any(|index| usize::try_from(*index).map_or(true, |index| index >= vertex_count))
        {
            return Err(GltfError::IndexOutOfRange);
        }
        Ok(GltfGeometry {
            positions_micrometres,
            normals_snorm16,
            tangents_snorm16,
            uv0_q16,
            indices,
            bounds_min,
            bounds_max,
        })
    }

    fn primitive(&self, mesh: usize, primitive: usize) -> Result<&GltfPrimitive, GltfError> {
        let mesh_record = self
            .json
            .meshes
            .get(mesh)
            .ok_or(GltfError::OutOfRange("mesh", mesh))?;
        mesh_record
            .primitives
            .get(primitive)
            .ok_or(GltfError::OutOfRange("primitive", primitive))
    }

    fn scene_roots(&self) -> Result<Vec<usize>, GltfError> {
        if self.json.scenes.is_empty() {
            // No scene: every node without a parent is a root.
            let mut is_child = vec![false; self.json.nodes.len()];
            for node in &self.json.nodes {
                for child in &node.children {
                    let slot = is_child
                        .get_mut(*child)
                        .ok_or(GltfError::OutOfRange("node", *child))?;
                    *slot = true;
                }
            }
            return Ok((0..self.json.nodes.len())
                .filter(|index| !is_child[*index])
                .collect());
        }
        let scene = self.json.scene.unwrap_or(0);
        let scene_record = self
            .json
            .scenes
            .get(scene)
            .ok_or(GltfError::OutOfRange("scene", scene))?;
        Ok(scene_record.nodes.clone())
    }

    fn global_transform(&self, node: usize) -> Result<[f64; 16], GltfError> {
        if node >= self.json.nodes.len() {
            return Err(GltfError::OutOfRange("node", node));
        }
        let mut parent = vec![None; self.json.nodes.len()];
        let mut reached = vec![false; self.json.nodes.len()];
        let mut stack = self.scene_roots()?;
        for root in &stack {
            if *root >= self.json.nodes.len() {
                return Err(GltfError::OutOfRange("node", *root));
            }
            reached[*root] = true;
        }
        while let Some(index) = stack.pop() {
            for child in &self.json.nodes[index].children {
                if *child >= self.json.nodes.len() {
                    return Err(GltfError::OutOfRange("node", *child));
                }
                if !reached[*child] {
                    reached[*child] = true;
                    parent[*child] = Some(index);
                    stack.push(*child);
                }
            }
        }
        if !reached[node] {
            return Err(GltfError::NodeNotInScene(node));
        }
        let mut chain = vec![node];
        let mut cursor = node;
        while let Some(up) = parent[cursor] {
            chain.push(up);
            cursor = up;
        }
        let mut global = IDENTITY;
        for index in chain.iter().rev() {
            global = multiply(&global, &local_transform(&self.json.nodes[*index]));
        }
        Ok(global)
    }

    fn accessor_bytes(
        &self,
        index: usize,
        element_size: usize,
    ) -> Result<(&[u8], usize), GltfError> {
        let accessor = self
            .json
            .accessors
            .get(index)
            .ok_or(GltfError::OutOfRange("accessor", index))?;
        if accessor.sparse.is_some() {
            return Err(GltfError::SparseAccessor(index));
        }
        let view_index = accessor
            .buffer_view
            .ok_or(GltfError::AccessorOutOfBounds(index))?;
        let view = self
            .json
            .buffer_views
            .get(view_index)
            .ok_or(GltfError::OutOfRange("bufferView", view_index))?;
        let buffer = self
            .buffers
            .get(view.buffer)
            .ok_or(GltfError::OutOfRange("buffer", view.buffer))?;
        let stride = view.byte_stride.unwrap_or(element_size);
        if stride < element_size {
            return Err(GltfError::AccessorOutOfBounds(index));
        }
        let view_end = view
            .byte_offset
            .checked_add(view.byte_length)
            .ok_or(GltfError::AccessorOutOfBounds(index))?;
        if view_end > buffer.len() {
            return Err(GltfError::AccessorOutOfBounds(index));
        }
        let needed = if accessor.count == 0 {
            0
        } else {
            stride
                .checked_mul(accessor.count - 1)
                .and_then(|value| value.checked_add(element_size))
                .ok_or(GltfError::AccessorOutOfBounds(index))?
        };
        let start = view
            .byte_offset
            .checked_add(accessor.byte_offset)
            .ok_or(GltfError::AccessorOutOfBounds(index))?;
        let end = start
            .checked_add(needed)
            .ok_or(GltfError::AccessorOutOfBounds(index))?;
        if end > view_end {
            return Err(GltfError::AccessorOutOfBounds(index));
        }
        Ok((&buffer[start..end], stride))
    }

    fn read_floats(
        &self,
        index: usize,
        components: usize,
        name: &'static str,
    ) -> Result<Vec<[f64; 4]>, GltfError> {
        let accessor = self
            .json
            .accessors
            .get(index)
            .ok_or(GltfError::OutOfRange("accessor", index))?;
        let expected_type = match components {
            2 => "VEC2",
            3 => "VEC3",
            4 => "VEC4",
            _ => return Err(GltfError::AttributeFormat(name)),
        };
        if accessor.element_type != expected_type {
            return Err(GltfError::AttributeFormat(name));
        }
        let (component_size, kind) = match (accessor.component_type, accessor.normalized) {
            (5126, _) => (4, FloatKind::F32),
            (5121, true) => (1, FloatKind::U8),
            (5123, true) => (2, FloatKind::U16),
            (5120, true) => (1, FloatKind::I8),
            (5122, true) => (2, FloatKind::I16),
            _ => return Err(GltfError::AttributeFormat(name)),
        };
        if !matches!(kind, FloatKind::F32) && components != 2 {
            // Normalized integers are only admitted for texture coordinates.
            return Err(GltfError::AttributeFormat(name));
        }
        let element_size = component_size * components;
        let count = accessor.count;
        let (bytes, stride) = self.accessor_bytes(index, element_size)?;
        let mut values = Vec::with_capacity(count);
        for element in 0..count {
            let base = element * stride;
            let mut value = [0.0_f64; 4];
            for (component, slot) in value.iter_mut().enumerate().take(components) {
                let offset = base + component * component_size;
                *slot = match kind {
                    FloatKind::F32 => f64::from(f32::from_le_bytes([
                        bytes[offset],
                        bytes[offset + 1],
                        bytes[offset + 2],
                        bytes[offset + 3],
                    ])),
                    FloatKind::U8 => f64::from(bytes[offset]) / 255.0,
                    FloatKind::U16 => {
                        f64::from(u16::from_le_bytes([bytes[offset], bytes[offset + 1]])) / 65_535.0
                    }
                    FloatKind::I8 => (f64::from(bytes[offset] as i8) / 127.0).max(-1.0),
                    FloatKind::I16 => {
                        (f64::from(i16::from_le_bytes([bytes[offset], bytes[offset + 1]]))
                            / 32_767.0)
                            .max(-1.0)
                    }
                };
                if !slot.is_finite() {
                    return Err(GltfError::AttributeFormat(name));
                }
            }
            values.push(value);
        }
        Ok(values)
    }

    fn read_indices(&self, index: usize) -> Result<Vec<u32>, GltfError> {
        let accessor = self
            .json
            .accessors
            .get(index)
            .ok_or(GltfError::OutOfRange("accessor", index))?;
        if accessor.element_type != "SCALAR" {
            return Err(GltfError::AttributeFormat("indices"));
        }
        let component_size = match accessor.component_type {
            5121 => 1,
            5123 => 2,
            5125 => 4,
            _ => return Err(GltfError::AttributeFormat("indices")),
        };
        let count = accessor.count;
        let (bytes, stride) = self.accessor_bytes(index, component_size)?;
        let mut values = Vec::with_capacity(count);
        for element in 0..count {
            let offset = element * stride;
            values.push(match component_size {
                1 => u32::from(bytes[offset]),
                2 => u32::from(u16::from_le_bytes([bytes[offset], bytes[offset + 1]])),
                _ => u32::from_le_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ]),
            });
        }
        Ok(values)
    }
}

#[derive(Clone, Copy)]
enum FloatKind {
    F32,
    U8,
    U16,
    I8,
    I16,
}

fn parse_glb(bytes: &[u8]) -> Result<(Vec<u8>, Option<Vec<u8>>), GltfError> {
    let version = read_u32(bytes, 4);
    if version != 2 {
        return Err(GltfError::Malformed(format!("GLB version {version}")));
    }
    let length = usize::try_from(read_u32(bytes, 8))
        .map_err(|_| GltfError::Malformed("GLB length".into()))?;
    if length > bytes.len() || length < 12 {
        return Err(GltfError::Malformed("GLB length beyond the file".into()));
    }
    let mut cursor = 12;
    let mut json = None;
    let mut bin = None;
    while cursor + 8 <= length {
        let chunk_length = usize::try_from(read_u32(bytes, cursor))
            .map_err(|_| GltfError::Malformed("GLB chunk length".into()))?;
        let chunk_type = read_u32(bytes, cursor + 4);
        let start = cursor + 8;
        let end = start
            .checked_add(chunk_length)
            .ok_or_else(|| GltfError::Malformed("GLB chunk length".into()))?;
        if end > length {
            return Err(GltfError::Malformed("GLB chunk beyond the file".into()));
        }
        let chunk = bytes[start..end].to_vec();
        match chunk_type {
            GLB_CHUNK_JSON if json.is_none() => json = Some(chunk),
            GLB_CHUNK_BIN if bin.is_none() => bin = Some(chunk),
            _ => {}
        }
        cursor = end;
    }
    let json = json.ok_or_else(|| GltfError::Malformed("GLB without a JSON chunk".into()))?;
    Ok((json, bin))
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn data_uri(uri: &str) -> Option<&str> {
    let rest = uri.strip_prefix("data:")?;
    let (_, payload) = rest.split_once(";base64,")?;
    Some(payload)
}

pub(crate) fn decode_base64(text: &str) -> Result<Vec<u8>, GltfError> {
    let mut output = Vec::with_capacity(text.len() / 4 * 3);
    let mut accumulator: u32 = 0;
    let mut bits = 0;
    let mut padding = 0;
    for byte in text.bytes() {
        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' => {
                padding += 1;
                continue;
            }
            b' ' | b'\n' | b'\r' | b'\t' => continue,
            _ => return Err(GltfError::Malformed("base64 payload".into())),
        };
        if padding > 0 {
            return Err(GltfError::Malformed("base64 payload".into()));
        }
        accumulator = (accumulator << 6) | u32::from(value);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            output.push(((accumulator >> bits) & 0xff) as u8);
        }
    }
    Ok(output)
}

fn percent_decode(uri: &str) -> String {
    let bytes = uri.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).ok();
            if let Some(value) = hex.and_then(|hex| u8::from_str_radix(hex, 16).ok()) {
                output.push(value);
                index += 3;
                continue;
            }
        }
        output.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&output).into_owned()
}

const IDENTITY: [f64; 16] = [
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];

/// Column-major `4 x 4`, as glTF stores it: element `(row, column)` at
/// `column * 4 + row`.
fn multiply(a: &[f64; 16], b: &[f64; 16]) -> [f64; 16] {
    let mut out = [0.0; 16];
    for column in 0..4 {
        for row in 0..4 {
            let mut sum = 0.0;
            for k in 0..4 {
                sum += a[k * 4 + row] * b[column * 4 + k];
            }
            out[column * 4 + row] = sum;
        }
    }
    out
}

fn local_transform(node: &GltfNode) -> [f64; 16] {
    if let Some(matrix) = node.matrix {
        return matrix;
    }
    let t = node.translation.unwrap_or([0.0; 3]);
    let [x, y, z, w] = node.rotation.unwrap_or([0.0, 0.0, 0.0, 1.0]);
    let s = node.scale.unwrap_or([1.0; 3]);
    let rotation = [
        1.0 - 2.0 * (y * y + z * z),
        2.0 * (x * y + z * w),
        2.0 * (x * z - y * w),
        0.0,
        2.0 * (x * y - z * w),
        1.0 - 2.0 * (x * x + z * z),
        2.0 * (y * z + x * w),
        0.0,
        2.0 * (x * z + y * w),
        2.0 * (y * z - x * w),
        1.0 - 2.0 * (x * x + y * y),
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    ];
    let mut out = rotation;
    for column in 0..3 {
        for row in 0..3 {
            out[column * 4 + row] *= s[column];
        }
    }
    out[12] = t[0];
    out[13] = t[1];
    out[14] = t[2];
    out
}

fn transform_point(m: &[f64; 16], p: &[f64; 4]) -> [f64; 3] {
    [
        m[0] * p[0] + m[4] * p[1] + m[8] * p[2] + m[12],
        m[1] * p[0] + m[5] * p[1] + m[9] * p[2] + m[13],
        m[2] * p[0] + m[6] * p[1] + m[10] * p[2] + m[14],
    ]
}

fn transform_direction(m: &[f64; 16], d: &[f64; 3]) -> [f64; 3] {
    [
        m[0] * d[0] + m[4] * d[1] + m[8] * d[2],
        m[1] * d[0] + m[5] * d[1] + m[9] * d[2],
        m[2] * d[0] + m[6] * d[1] + m[10] * d[2],
    ]
}

fn determinant3(m: &[f64; 16]) -> f64 {
    m[0] * (m[5] * m[10] - m[9] * m[6]) - m[4] * (m[1] * m[10] - m[9] * m[2])
        + m[8] * (m[1] * m[6] - m[5] * m[2])
}

/// The inverse transpose of the upper `3 x 3`, in the same `4 x 4` layout.
fn normal_matrix(m: &[f64; 16]) -> Result<[f64; 16], GltfError> {
    let det = determinant3(m);
    if !det.is_finite() || det.abs() < 1.0e-18 {
        return Err(GltfError::Degenerate("node transform"));
    }
    // Cofactors of the upper 3x3 (column-major): inverse transpose =
    // cofactor matrix / det.
    let a = |row: usize, column: usize| m[column * 4 + row];
    let cofactor = |r: usize, c: usize| {
        let rows: [usize; 2] = match r {
            0 => [1, 2],
            1 => [0, 2],
            _ => [0, 1],
        };
        let columns: [usize; 2] = match c {
            0 => [1, 2],
            1 => [0, 2],
            _ => [0, 1],
        };
        let minor = a(rows[0], columns[0]) * a(rows[1], columns[1])
            - a(rows[0], columns[1]) * a(rows[1], columns[0]);
        if (r + c).is_multiple_of(2) {
            minor
        } else {
            -minor
        }
    };
    let mut out = IDENTITY;
    for row in 0..3 {
        for column in 0..3 {
            out[column * 4 + row] = cofactor(row, column) / det;
        }
    }
    Ok(out)
}

fn snorm16_unit(v: &[f64; 3], what: &'static str) -> Result<[i16; 3], GltfError> {
    let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if !length.is_finite() || length < 1.0e-9 {
        return Err(GltfError::Degenerate(what));
    }
    let mut out = [0_i16; 3];
    for axis in 0..3 {
        let value = (v[axis] / length * SNORM16_ONE).round();
        out[axis] = value.clamp(-SNORM16_ONE, SNORM16_ONE) as i16;
    }
    Ok(out)
}

/// The scaffold's view of a material: the factors in `u16`, the image
/// per slot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GltfMaterialSummary {
    pub(crate) name: Option<String>,
    pub(crate) base_color_rgba_u16: [u16; 4],
    pub(crate) metallic_u16: u16,
    pub(crate) roughness_u16: u16,
    pub(crate) emissive_rgb_u16: [u16; 3],
    pub(crate) double_sided: bool,
    /// `(image index, file)` per slot when the material binds a texture.
    pub(crate) base_color_image: Option<(usize, String)>,
    pub(crate) metallic_roughness_image: Option<(usize, String)>,
    pub(crate) normal_image: Option<(usize, String)>,
}

impl GltfDocument {
    pub(crate) fn material_summary(&self, index: usize) -> Result<GltfMaterialSummary, GltfError> {
        let material = self
            .materials()
            .get(index)
            .ok_or(GltfError::OutOfRange("material", index))?;
        let pbr = &material.pbr_metallic_roughness;
        let image = |info: &Option<GltfTextureInfo>| -> Result<Option<(usize, String)>, GltfError> {
            match info {
                None => Ok(None),
                Some(info) => {
                    if info.tex_coord != 0 {
                        return Err(GltfError::Image(format!(
                            "material {index} binds a texture at TEXCOORD_{}; only set 0 is importable",
                            info.tex_coord
                        )));
                    }
                    self.texture_image_uri(info.index).map(Some)
                }
            }
        };
        Ok(GltfMaterialSummary {
            name: material.name.clone(),
            base_color_rgba_u16: [
                unit_u16(pbr.base_color_factor[0]),
                unit_u16(pbr.base_color_factor[1]),
                unit_u16(pbr.base_color_factor[2]),
                unit_u16(pbr.base_color_factor[3]),
            ],
            metallic_u16: unit_u16(pbr.metallic_factor),
            roughness_u16: unit_u16(pbr.roughness_factor),
            emissive_rgb_u16: [
                unit_u16(material.emissive_factor[0]),
                unit_u16(material.emissive_factor[1]),
                unit_u16(material.emissive_factor[2]),
            ],
            double_sided: material.double_sided,
            base_color_image: image(&pbr.base_color_texture)?,
            metallic_roughness_image: image(&pbr.metallic_roughness_texture)?,
            normal_image: image(&material.normal_texture)?,
        })
    }
}

fn unit_u16(value: f64) -> u16 {
    (value.clamp(0.0, 1.0) * 65_535.0).round() as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad_buffer() -> Vec<u8> {
        // Four vertices: position (f32 x3), normal (f32 x3), tangent
        // (f32 x4), uv (f32 x2); then six u16 indices.
        let positions: [[f32; 3]; 4] = [
            [-0.5, 0.0, -0.5],
            [0.5, 0.0, -0.5],
            [0.5, 0.0, 0.5],
            [-0.5, 0.0, 0.5],
        ];
        let mut bytes = Vec::new();
        for p in positions {
            for v in p {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
        }
        for _ in 0..4 {
            for v in [0.0_f32, 1.0, 0.0] {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
        }
        for _ in 0..4 {
            for v in [1.0_f32, 0.0, 0.0, 1.0] {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
        }
        for uv in [[0.0_f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]] {
            for v in uv {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
        }
        for index in [0_u16, 2, 1, 0, 3, 2] {
            bytes.extend_from_slice(&index.to_le_bytes());
        }
        bytes
    }

    fn encode_base64(bytes: &[u8]) -> String {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for chunk in bytes.chunks(3) {
            let mut value = 0_u32;
            for (i, byte) in chunk.iter().enumerate() {
                value |= u32::from(*byte) << (16 - 8 * i);
            }
            for i in 0..4 {
                if i <= chunk.len() {
                    out.push(ALPHABET[((value >> (18 - 6 * i)) & 63) as usize] as char);
                } else {
                    out.push('=');
                }
            }
        }
        out
    }

    fn quad_json(buffer_uri: Option<&str>, node_extra: &str, mode: u32) -> String {
        let buffer = match buffer_uri {
            Some(uri) => format!(r#"{{"byteLength":140,"uri":"{uri}"}}"#),
            None => r#"{"byteLength":140}"#.to_owned(),
        };
        format!(
            r#"{{
  "asset": {{"version": "2.0"}},
  "buffers": [{buffer}],
  "bufferViews": [
    {{"buffer": 0, "byteOffset": 0, "byteLength": 48}},
    {{"buffer": 0, "byteOffset": 48, "byteLength": 48}},
    {{"buffer": 0, "byteOffset": 96, "byteLength": 64}},
    {{"buffer": 0, "byteOffset": 160, "byteLength": 32}},
    {{"buffer": 0, "byteOffset": 192, "byteLength": 12}}
  ],
  "accessors": [
    {{"bufferView": 0, "componentType": 5126, "count": 4, "type": "VEC3"}},
    {{"bufferView": 1, "componentType": 5126, "count": 4, "type": "VEC3"}},
    {{"bufferView": 2, "componentType": 5126, "count": 4, "type": "VEC4"}},
    {{"bufferView": 3, "componentType": 5126, "count": 4, "type": "VEC2"}},
    {{"bufferView": 4, "componentType": 5123, "count": 6, "type": "SCALAR"}}
  ],
  "meshes": [{{"name": "quad", "primitives": [{{"attributes": {{"POSITION": 0, "NORMAL": 1, "TANGENT": 2, "TEXCOORD_0": 3}}, "indices": 4, "mode": {mode}}}]}}],
  "nodes": [{{"name": "root", "mesh": 0{node_extra}}}],
  "scenes": [{{"nodes": [0]}}],
  "scene": 0
}}"#
        )
    }

    fn quad_json_with_uri(uri: &str) -> String {
        quad_json(Some(uri), "", MODE_TRIANGLES).replace("\"byteLength\":140", "\"byteLength\":204")
    }

    fn data_document(node_extra: &str, mode: u32) -> GltfDocument {
        let uri = format!(
            "data:application/octet-stream;base64,{}",
            encode_base64(&quad_buffer())
        );
        let json = quad_json(Some(&uri), node_extra, mode)
            .replace("\"byteLength\":140", "\"byteLength\":204");
        GltfDocument::parse(json.as_bytes(), |_| {
            Err(GltfError::UndeclaredSource("x".into()))
        })
        .expect("parses")
    }

    #[test]
    fn quad_decodes_from_a_data_uri() {
        let document = data_document("", MODE_TRIANGLES);
        let geometry = document.geometry(0, 0, Some(0)).expect("geometry");
        assert_eq!(
            geometry.positions_micrometres,
            vec![
                [-500_000, 0, -500_000],
                [500_000, 0, -500_000],
                [500_000, 0, 500_000],
                [-500_000, 0, 500_000]
            ]
        );
        assert_eq!(
            geometry.normals_snorm16.as_deref(),
            Some(&[[0, 32_767, 0]; 4][..])
        );
        assert_eq!(
            geometry.tangents_snorm16.as_deref(),
            Some(&[([32_767, 0, 0], 1); 4][..])
        );
        assert_eq!(
            geometry.uv0_q16,
            vec![[0, 0], [65_536, 0], [65_536, 65_536], [0, 65_536]]
        );
        assert_eq!(geometry.indices, vec![0, 2, 1, 0, 3, 2]);
        assert_eq!(geometry.bounds_min, [-500_000, 0, -500_000]);
        assert_eq!(geometry.bounds_max, [500_000, 0, 500_000]);
        let meshes = document.scene_meshes().expect("scene");
        assert_eq!(meshes.len(), 1);
        assert_eq!(meshes[0].mesh_name.as_deref(), Some("quad"));
    }

    #[test]
    fn glb_decodes_identically() {
        let json =
            quad_json(None, "", MODE_TRIANGLES).replace("\"byteLength\":140", "\"byteLength\":204");
        let mut json_bytes = json.into_bytes();
        while !json_bytes.len().is_multiple_of(4) {
            json_bytes.push(b' ');
        }
        let mut bin = quad_buffer();
        while !bin.len().is_multiple_of(4) {
            bin.push(0);
        }
        let mut glb = Vec::new();
        glb.extend_from_slice(&GLB_MAGIC.to_le_bytes());
        glb.extend_from_slice(&2_u32.to_le_bytes());
        let total = 12 + 8 + json_bytes.len() + 8 + bin.len();
        glb.extend_from_slice(&(total as u32).to_le_bytes());
        glb.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
        glb.extend_from_slice(&GLB_CHUNK_JSON.to_le_bytes());
        glb.extend_from_slice(&json_bytes);
        glb.extend_from_slice(&(bin.len() as u32).to_le_bytes());
        glb.extend_from_slice(&GLB_CHUNK_BIN.to_le_bytes());
        glb.extend_from_slice(&bin);
        let document = GltfDocument::parse(&glb, |_| Err(GltfError::UndeclaredSource("x".into())))
            .expect("glb");
        let from_glb = document.geometry(0, 0, Some(0)).expect("geometry");
        let from_json = data_document("", MODE_TRIANGLES)
            .geometry(0, 0, Some(0))
            .expect("geometry");
        assert_eq!(from_glb, from_json);
    }

    #[test]
    fn node_transform_bakes_positions_and_normals() {
        // Translate 1 m along x, rotate 90 degrees about y, scale 2.
        let half = std::f64::consts::FRAC_1_SQRT_2;
        let extra = format!(
            r#", "translation": [1, 0, 0], "rotation": [0, {half}, 0, {half}], "scale": [2, 2, 2]"#
        );
        let document = data_document(&extra, MODE_TRIANGLES);
        let geometry = document.geometry(0, 0, Some(0)).expect("geometry");
        // Vertex 1 at (0.5, 0, -0.5): scaled (1, 0, -1), rotated about y
        // by 90 degrees -> (-1, 0, -1), translated -> (0, 0, -1).
        assert_eq!(geometry.positions_micrometres[1], [0, 0, -1_000_000]);
        assert_eq!(
            geometry.normals_snorm16.as_deref().map(|n| n[0]),
            Some([0, 32_767, 0])
        );
        // The tangent (+x) rotates to -z.
        assert_eq!(
            geometry.tangents_snorm16.as_deref().map(|t| t[0]),
            Some(([0, 0, -32_767], 1))
        );
        let without_node = document.geometry(0, 0, None).expect("geometry");
        assert_eq!(
            without_node.positions_micrometres[1],
            [500_000, 0, -500_000]
        );
    }

    #[test]
    fn mirrored_transform_flips_the_tangent_handedness() {
        let document = data_document(r#", "scale": [-1, 1, 1]"#, MODE_TRIANGLES);
        let geometry = document.geometry(0, 0, Some(0)).expect("geometry");
        assert_eq!(
            geometry.tangents_snorm16.as_deref().map(|t| t[0]),
            Some(([-32_767, 0, 0], -1))
        );
        assert_eq!(
            geometry.normals_snorm16.as_deref().map(|n| n[0]),
            Some([0, 32_767, 0])
        );
    }

    #[test]
    fn refusals_name_their_reason() {
        assert_eq!(
            data_document("", 1).geometry(0, 0, Some(0)).unwrap_err(),
            GltfError::Topology(1)
        );
        let uri = format!(
            "data:application/octet-stream;base64,{}",
            encode_base64(&quad_buffer())
        );
        let json = quad_json_with_uri(&uri);
        let no_position = json.replace("\"POSITION\": 0, ", "");
        let document =
            GltfDocument::parse(no_position.as_bytes(), |_| unreachable!()).expect("parses");
        assert_eq!(
            document.geometry(0, 0, Some(0)).unwrap_err(),
            GltfError::MissingAttribute("POSITION")
        );
        let required = json.replace(
            "\"asset\"",
            "\"extensionsRequired\": [\"KHR_draco_mesh_compression\"], \"asset\"",
        );
        assert_eq!(
            GltfDocument::parse(required.as_bytes(), |_| unreachable!()).err(),
            Some(GltfError::RequiredExtension(
                "KHR_draco_mesh_compression".into()
            ))
        );
        let sparse = json.replace(
            "{\"bufferView\": 0, \"componentType\": 5126, \"count\": 4, \"type\": \"VEC3\"}",
            "{\"bufferView\": 0, \"componentType\": 5126, \"count\": 4, \"type\": \"VEC3\", \"sparse\": {\"count\": 1}}",
        );
        let document = GltfDocument::parse(sparse.as_bytes(), |_| unreachable!()).expect("parses");
        assert_eq!(
            document.geometry(0, 0, Some(0)).unwrap_err(),
            GltfError::SparseAccessor(0)
        );
        let mut broken = quad_buffer();
        broken[192] = 9;
        let uri = format!(
            "data:application/octet-stream;base64,{}",
            encode_base64(&broken)
        );
        let document = GltfDocument::parse(quad_json_with_uri(&uri).as_bytes(), |_| unreachable!())
            .expect("parses");
        assert_eq!(
            document.geometry(0, 0, Some(0)).unwrap_err(),
            GltfError::IndexOutOfRange
        );
        let external = quad_json_with_uri("missing.bin");
        assert_eq!(
            GltfDocument::parse(external.as_bytes(), |uri| Err(GltfError::UndeclaredSource(
                uri.to_owned()
            )))
            .err(),
            Some(GltfError::UndeclaredSource("missing.bin".into()))
        );
        assert_eq!(
            data_document("", MODE_TRIANGLES)
                .geometry(0, 0, Some(3))
                .unwrap_err(),
            GltfError::OutOfRange("node", 3)
        );
    }

    #[test]
    fn base64_and_percent_decoding() {
        assert_eq!(decode_base64("aGVsbG8=").expect("decodes"), b"hello");
        assert_eq!(
            decode_base64("aGVsbG8gd29ybGQ=").expect("decodes"),
            b"hello world"
        );
        assert!(decode_base64("a*").is_err());
        assert_eq!(percent_decode("water%20tank.bin"), "water tank.bin");
        assert_eq!(percent_decode("plain.bin"), "plain.bin");
    }
}

/// Reads a `mesh-gltf` source: the file and every external buffer or image
/// it names must be declared referenced sources of the project.
pub(super) fn load_document(
    project_directory: &std::path::Path,
    relative_path: &str,
    referenced_sources: &[String],
) -> Result<GltfDocument, super::ProjectAuthoringError> {
    use super::ProjectAuthoringError;
    if !referenced_sources.iter().any(|path| path == relative_path) {
        return Err(ProjectAuthoringError::MissingReference(
            relative_path.to_owned(),
        ));
    }
    let bytes = super::read_file(&super::safe_join(project_directory, relative_path)?)?;
    let mut failure: Option<ProjectAuthoringError> = None;
    let document = GltfDocument::parse(&bytes, |uri| {
        let sibling = sibling_path(relative_path, uri)
            .ok_or_else(|| GltfError::UndeclaredSource(uri.to_owned()))?;
        if !referenced_sources.contains(&sibling) {
            return Err(GltfError::UndeclaredSource(sibling));
        }
        match super::safe_join(project_directory, &sibling).and_then(|path| super::read_file(&path))
        {
            Ok(bytes) => Ok(bytes),
            Err(error) => {
                failure = Some(error);
                Err(GltfError::UndeclaredSource(sibling))
            }
        }
    });
    match document {
        Ok(document) => {
            // Every image the document names must be declared too, so the
            // composition lock covers the whole asset.
            for file in document.external_files() {
                let sibling = sibling_path(relative_path, file)
                    .ok_or_else(|| GltfError::UndeclaredSource(file.clone()))
                    .map_err(ProjectAuthoringError::Gltf)?;
                if !referenced_sources.contains(&sibling) {
                    return Err(ProjectAuthoringError::Gltf(GltfError::UndeclaredSource(
                        sibling,
                    )));
                }
            }
            Ok(document)
        }
        Err(error) => Err(failure.unwrap_or(ProjectAuthoringError::Gltf(error))),
    }
}

/// A URI relative to the glTF file, as a project-relative path with `/`
/// separators; `..` and absolute URIs are refused.
pub(crate) fn sibling_path(gltf_relative_path: &str, uri: &str) -> Option<String> {
    if uri.starts_with('/') || uri.contains("://") || uri.starts_with("data:") {
        return None;
    }
    let directory = gltf_relative_path
        .rsplit_once('/')
        .map_or("", |(directory, _)| directory);
    let mut segments: Vec<&str> = if directory.is_empty() {
        Vec::new()
    } else {
        directory.split('/').collect()
    };
    for segment in uri.split('/') {
        match segment {
            "" | "." => {}
            ".." => return None,
            other => segments.push(other),
        }
    }
    if segments.is_empty() {
        return None;
    }
    Some(segments.join("/"))
}

#[cfg(test)]
mod path_tests {
    use super::sibling_path;

    #[test]
    fn sibling_paths_stay_under_the_document_directory() {
        assert_eq!(
            sibling_path("assets/models/tank.gltf", "tank.bin").as_deref(),
            Some("assets/models/tank.bin")
        );
        assert_eq!(
            sibling_path("assets/models/tank.gltf", "./maps/a.png").as_deref(),
            Some("assets/models/maps/a.png")
        );
        assert_eq!(
            sibling_path("tank.gltf", "tank.bin").as_deref(),
            Some("tank.bin")
        );
        assert_eq!(sibling_path("assets/models/tank.gltf", "../x.bin"), None);
        assert_eq!(sibling_path("assets/models/tank.gltf", "/etc/x.bin"), None);
    }
}
