use super::*;

impl WorldNavigationCatalogV1 {
    pub fn validate(&self) -> Result<(), WorldPopulationContractError> {
        if self.schema_version != WORLD_POPULATION_SCHEMA_VERSION
            || self.topology_revision == 0
            || self.tiles.is_empty()
            || self.nodes.is_empty()
            || self.edges.is_empty()
            || !strictly_sorted(&self.tiles)
            || !strictly_sorted(&self.nodes)
            || !strictly_sorted(&self.edges)
        {
            return Err(WorldPopulationContractError::NavigationContentInvalid);
        }
        let tiles = self
            .tiles
            .iter()
            .map(|tile| (tile.tile_id.as_str(), tile))
            .collect::<BTreeMap<_, _>>();
        if tiles.len() != self.tiles.len()
            || self
                .tiles
                .windows(2)
                .any(|pair| pair[0].region_id >= pair[1].region_id)
        {
            return Err(WorldPopulationContractError::NavigationContentInvalid);
        }
        let nodes = self
            .nodes
            .iter()
            .map(|node| (node.node_id.as_str(), node))
            .collect::<BTreeMap<_, _>>();
        let chunk_ids = self
            .nodes
            .iter()
            .map(|node| node.chunk_id.as_str())
            .collect::<BTreeSet<_>>();
        if nodes.len() != self.nodes.len()
            || chunk_ids.len() != self.nodes.len()
            || self.nodes.iter().any(|node| {
                node.node_id != node.chunk_id || !tiles.contains_key(node.tile_id.as_str())
            })
        {
            return Err(WorldPopulationContractError::NavigationContentInvalid);
        }
        for tile in &self.tiles {
            let node_ids = self
                .nodes
                .iter()
                .filter(|node| node.tile_id == tile.tile_id)
                .map(|node| &node.node_id)
                .collect::<Vec<_>>();
            if node_ids.is_empty()
                || tile.tile_revision
                    != navigation_tile_revision(&tile.tile_id, &tile.region_id, &node_ids)?
            {
                return Err(WorldPopulationContractError::NavigationContentInvalid);
            }
        }
        if self.edges.iter().any(|edge| {
            edge.node_low >= edge.node_high
                || edge.cost == 0
                || !nodes.contains_key(edge.node_low.as_str())
                || !nodes.contains_key(edge.node_high.as_str())
        }) {
            return Err(WorldPopulationContractError::NavigationContentInvalid);
        }

        let mut visited = BTreeSet::new();
        let first = self
            .nodes
            .first()
            .ok_or(WorldPopulationContractError::NavigationContentInvalid)?;
        let mut pending = vec![first.node_id.clone()];
        while let Some(node_id) = pending.pop() {
            if !visited.insert(node_id.clone()) {
                continue;
            }
            for edge in &self.edges {
                let neighbour = if edge.node_low == node_id {
                    Some(&edge.node_high)
                } else if edge.node_high == node_id {
                    Some(&edge.node_low)
                } else {
                    None
                };
                if let Some(neighbour) = neighbour {
                    pending.push(neighbour.clone());
                }
            }
        }
        if visited.len() != self.nodes.len() {
            return Err(WorldPopulationContractError::NavigationContentInvalid);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let tiles = encode_sequence(
            self.tiles
                .iter()
                .map(|tile| {
                    encode_struct([
                        field_text(1, &tile.tile_id),
                        field_text(2, &tile.region_id),
                        field_hash(3, tile.tile_revision),
                    ])
                })
                .collect::<Result<Vec<_>, _>>()?,
        )?;
        let nodes = encode_sequence(
            self.nodes
                .iter()
                .map(|node| {
                    encode_struct([
                        field_text(1, &node.node_id),
                        field_text(2, &node.tile_id),
                        field_text(3, &node.chunk_id),
                    ])
                })
                .collect::<Result<Vec<_>, _>>()?,
        )?;
        let edges = encode_sequence(
            self.edges
                .iter()
                .map(|edge| {
                    encode_struct([
                        field_text(1, &edge.node_low),
                        field_text(2, &edge.node_high),
                        field_u32(3, edge.cost),
                        field_u8(4, edge.capability as u8),
                    ])
                })
                .collect::<Result<Vec<_>, _>>()?,
        )?;
        encode_canonical_segment(
            WORLD_NAVIGATION_CATALOG_OWNER_ID,
            WORLD_NAVIGATION_CATALOG_SCHEMA_ID,
            WORLD_NAVIGATION_CATALOG_SEGMENT_ID,
            [
                field_u16(1, self.schema_version),
                field_id(2, self.catalog_asset_id),
                field_u64(3, self.topology_revision),
                CanonicalField::new(4, CANONICAL_TYPE_SEQUENCE, tiles),
                CanonicalField::new(5, CANONICAL_TYPE_SEQUENCE, nodes),
                CanonicalField::new(6, CANONICAL_TYPE_SEQUENCE, edges),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, WorldPopulationContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            WORLD_NAVIGATION_CATALOG_OWNER_ID,
            WORLD_NAVIGATION_CATALOG_SCHEMA_ID,
            WORLD_NAVIGATION_CATALOG_SEGMENT_ID,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_U64),
                (4, CANONICAL_TYPE_SEQUENCE),
                (5, CANONICAL_TYPE_SEQUENCE),
                (6, CANONICAL_TYPE_SEQUENCE),
            ],
        )?;
        let tiles = decode_sequence(field(&segment, 4)?, limits)?
            .into_iter()
            .map(|payload| {
                let fields = decode_struct(&payload, limits)?;
                require_fields(
                    &fields,
                    &[
                        (1, CANONICAL_TYPE_UTF8_NFC),
                        (2, CANONICAL_TYPE_UTF8_NFC),
                        (3, CANONICAL_TYPE_HASH256),
                    ],
                )?;
                Ok(WorldNavigationTileV1 {
                    tile_id: read_schema_id(nested_field(&fields, 1)?)?,
                    region_id: read_schema_id(nested_field(&fields, 2)?)?,
                    tile_revision: read_hash(nested_field(&fields, 3)?)?,
                })
            })
            .collect::<Result<Vec<_>, WorldPopulationContractError>>()?;
        let nodes = decode_sequence(field(&segment, 5)?, limits)?
            .into_iter()
            .map(|payload| {
                let fields = decode_struct(&payload, limits)?;
                require_fields(
                    &fields,
                    &[
                        (1, CANONICAL_TYPE_UTF8_NFC),
                        (2, CANONICAL_TYPE_UTF8_NFC),
                        (3, CANONICAL_TYPE_UTF8_NFC),
                    ],
                )?;
                Ok(WorldNavigationNodeV1 {
                    node_id: read_schema_id(nested_field(&fields, 1)?)?,
                    tile_id: read_schema_id(nested_field(&fields, 2)?)?,
                    chunk_id: read_schema_id(nested_field(&fields, 3)?)?,
                })
            })
            .collect::<Result<Vec<_>, WorldPopulationContractError>>()?;
        let edges = decode_sequence(field(&segment, 6)?, limits)?
            .into_iter()
            .map(|payload| {
                let fields = decode_struct(&payload, limits)?;
                require_fields(
                    &fields,
                    &[
                        (1, CANONICAL_TYPE_UTF8_NFC),
                        (2, CANONICAL_TYPE_UTF8_NFC),
                        (3, CANONICAL_TYPE_U32),
                        (4, CANONICAL_TYPE_U8),
                    ],
                )?;
                Ok(WorldNavigationEdgeV1 {
                    node_low: read_schema_id(nested_field(&fields, 1)?)?,
                    node_high: read_schema_id(nested_field(&fields, 2)?)?,
                    cost: read_u32(nested_field(&fields, 3)?)?,
                    capability: NavigationCapabilityV1::from_tag(read_u8(nested_field(
                        &fields, 4,
                    )?)?)?,
                })
            })
            .collect::<Result<Vec<_>, WorldPopulationContractError>>()?;
        let value = Self {
            schema_version: read_u16(field(&segment, 1)?)?,
            catalog_asset_id: AssetId::from_bytes(read_exact(field(&segment, 2)?)?),
            topology_revision: read_u64(field(&segment, 3)?)?,
            tiles,
            nodes,
            edges,
        };
        value.validate()?;
        if value.canonical_bytes()? != bytes {
            return Err(WorldPopulationContractError::NonCanonicalEncoding);
        }
        Ok(value)
    }

    pub fn revision(&self) -> Result<ContentHash, WorldPopulationContractError> {
        self.validate()?;
        Ok(domain_hash(
            WORLD_NAVIGATION_CATALOG_SEGMENT_ID,
            &self.canonical_bytes()?,
        ))
    }

    #[must_use]
    pub fn node(&self, node_id: &SchemaId) -> Option<&WorldNavigationNodeV1> {
        self.nodes
            .binary_search_by(|node| node.node_id.cmp(node_id))
            .ok()
            .and_then(|index| self.nodes.get(index))
    }

    #[must_use]
    pub fn region_for_node(&self, node_id: &SchemaId) -> Option<&SchemaId> {
        let node = self.node(node_id)?;
        self.tiles
            .binary_search_by(|tile| tile.tile_id.cmp(&node.tile_id))
            .ok()
            .and_then(|index| self.tiles.get(index))
            .map(|tile| &tile.region_id)
    }
}

pub fn navigation_tile_revision(
    tile_id: &SchemaId,
    region_id: &SchemaId,
    node_ids: &[&SchemaId],
) -> Result<ContentHash, WorldPopulationContractError> {
    if node_ids.is_empty() || node_ids.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(WorldPopulationContractError::NavigationContentInvalid);
    }
    let mut bytes = Vec::new();
    append_text(&mut bytes, tile_id.as_str())?;
    append_text(&mut bytes, region_id.as_str())?;
    bytes.extend_from_slice(
        &u32::try_from(node_ids.len())
            .map_err(|_| WorldPopulationContractError::NavigationContentInvalid)?
            .to_le_bytes(),
    );
    for node_id in node_ids {
        append_text(&mut bytes, node_id.as_str())?;
    }
    Ok(domain_hash("nextengine.navigation-tile.v1", &bytes))
}
