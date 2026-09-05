use serde::Deserialize;

pub(super) const AUTHORING_FORMAT_V7: &str = "nextengine.project-authoring.v7";

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProjectAuthoringManifestV7 {
    pub format: String,
    pub project: AuthoringProjectV2,
    pub provenance: AuthoringProvenanceV1,
    pub partition: AuthoringPartitionV1,
    pub records: Vec<AuthoringNeutralRecordV1>,
    pub render_records: Vec<AuthoringRenderRecordV1>,
    pub text_catalogs: Vec<AuthoringTextCatalogV1>,
    pub audio_records: Vec<AuthoringAudioRecordV1>,
    #[serde(default)]
    pub neutral_animation_catalogs: Vec<AuthoringAnimationCatalogReferenceV1>,
    pub body_schema_asset: AuthoringBodySchemaAssetV1,
    pub world_routine_catalog: Option<AuthoringWorldRoutineCatalogV1>,
    pub world_routine_interaction_binding: Option<AuthoringWorldRoutineInteractionBindingV1>,
    pub world_navigation_catalog: AuthoringWorldNavigationCatalogV1,
    pub world_population_catalog: AuthoringWorldPopulationCatalogV1,
    pub agent_cognition_catalog: AuthoringAgentCognitionCatalogV1,
    pub world_activity_catalog: AuthoringWorldActivityCatalogV1,
    pub root_asset_ids: Vec<String>,
    pub allowed_presentation_targets: Vec<AuthoringPresentationTargetV1>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringBodySchemaAssetV1 {
    pub asset_id: String,
    pub record_revision: u32,
    pub profile_id: String,
    pub compiler_profile_id: String,
    #[serde(default)]
    pub functional_anatomy_profile_id: Option<String>,
    pub source_span: AuthoringSourceSpanV1,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringWorldActivityCatalogV1 {
    pub schema_version: u16,
    pub catalog_asset_id: String,
    pub commitment_id: String,
    pub work_id: String,
    pub workplace_node_id: String,
    pub work_duration_ticks: u64,
    pub employer_character_id: String,
    pub seller_character_id: String,
    pub worker_inventory_id: String,
    pub seller_inventory_id: String,
    pub food_item_id: String,
    pub currency_resource_id: String,
    pub hunger_resource_id: String,
    pub satiety_resource_id: String,
    pub wage_amount: i32,
    pub food_price: i32,
    pub hunger_restore_amount: i32,
    pub satiety_gain_amount: i32,
    pub listener_trust_q16: u32,
    pub social_action_id: String,
    pub await_activity_action_id: String,
    pub settlement_action_id: String,
    pub social_ready_fact_id: String,
    pub activity_ready_fact_id: String,
    pub settlement_ready_fact_id: String,
    pub work_topic_id: String,
    pub work_claim_predicate_id: String,
    pub work_claim_value_id: String,
    pub work_claim_cited_belief_id: String,
    pub work_exchange_tick: u64,
    pub work_exchange_expiry_tick: u64,
    pub threat_topic_id: String,
    pub threat_claim_predicate_id: String,
    pub threat_claim_value_id: String,
    pub threat_claim_cited_belief_id: String,
    pub threat_tick: u64,
    pub threat_expiry_tick: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringAgentCognitionCatalogV1 {
    pub schema_version: u16,
    pub catalog_asset_id: String,
    pub population_subject_ordinal: u32,
    pub evaluation_start_tick: u64,
    pub evaluation_period_ticks: u64,
    pub retrieval_limit: u16,
    pub goal_switch_threshold_q16: i32,
    pub emergency_health_threshold: i32,
    pub planner_max_depth: u8,
    pub planner_max_expanded_nodes: u16,
    pub ordinary_goal_id: String,
    pub emergency_goal_id: String,
    pub navigate_action_id: String,
    pub hold_action_id: String,
    pub route_known_fact_id: String,
    pub travel_needed_fact_id: String,
    pub emergency_fact_id: String,
    pub navigate_ready_fact_id: String,
    pub hold_ready_fact_id: String,
    pub seed_beliefs: Vec<AuthoringSemanticBeliefV1>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringSemanticBeliefV1 {
    pub predicate_id: String,
    pub value_id: String,
    pub confidence_q16: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringWorldNavigationCatalogV1 {
    pub schema_version: u16,
    pub catalog_asset_id: String,
    pub topology_revision: u64,
    pub edge_cost: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringWorldPopulationCatalogV1 {
    pub schema_version: u16,
    pub catalog_asset_id: String,
    pub identity_domain: String,
    pub courier_ordinal: u32,
    pub courier_transition_start_tick: u64,
    pub courier_initial_node_id: String,
    pub courier_goal_node_id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringWorldRoutineCatalogV1 {
    pub schema_version: u16,
    pub catalog_asset_id: String,
    pub profile: AuthoringWorldRoutineProfileV1,
    pub routine: AuthoringWorldRoutineDefinitionV1,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringWorldRoutineProfileV1 {
    pub schema_version: u16,
    pub anchor_simulation_tick: u64,
    pub anchor_world_tick: u64,
    pub world_ticks_per_simulation_tick_num: u64,
    pub world_ticks_per_simulation_tick_den: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringWorldRoutineDefinitionV1 {
    pub schema_version: u16,
    pub subject_id: String,
    pub initial_activity: AuthoringWorldRoutineActivityV1,
    pub transition_world_tick: u64,
    pub next_activity: AuthoringWorldRoutineActivityV1,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum AuthoringWorldRoutineActivityV1 {
    Duty,
    Rest,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringWorldRoutineInteractionBindingV1 {
    pub interaction_id: String,
    pub subject_id: String,
    pub required_activity: AuthoringWorldRoutineActivityV1,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringAnimationCatalogReferenceV1 {
    pub relative_path: String,
    pub source_span: AuthoringSourceSpanV1,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringHumanoidCatalogV2 {
    pub format: String,
    pub asset_set_id: String,
    pub creator: String,
    pub source: String,
    pub license_expression: String,
    pub skeleton: AuthoringSkeletonV1,
    pub clips: Vec<AuthoringAnimationClipV1>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringSkeletonV1 {
    pub asset_id: String,
    pub record_revision: u64,
    pub joints: Vec<AuthoringSkeletonJointV1>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringSkeletonJointV1 {
    pub key: String,
    pub parent: Option<String>,
    pub translation_micrometres: [i64; 3],
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringAnimationClipV1 {
    pub asset_id: String,
    pub record_revision: u64,
    pub clip_id: String,
    pub duration_microseconds: u64,
    pub channels: Vec<AuthoringAnimationChannelV1>,
    pub root_motion_intent: Vec<AuthoringAnimationKeyV1>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringAnimationChannelV1 {
    pub joint: String,
    pub property: AuthoringAnimationPropertyV1,
    pub keys: Vec<AuthoringAnimationKeyV1>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum AuthoringAnimationPropertyV1 {
    Translation,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringAnimationKeyV1 {
    pub time_microseconds: u64,
    pub value: [i64; 3],
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringProjectV2 {
    pub project_id: String,
    pub project_revision: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringProvenanceV1 {
    pub provenance_id: String,
    pub license_id: String,
    pub attribution: String,
    pub source_identity: String,
    pub license_manifest_sha256: String,
    pub referenced_sources: Vec<AuthoringSourceReferenceV1>,
    pub source_span: AuthoringSourceSpanV1,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringSourceReferenceV1 {
    pub logical_source: String,
    pub relative_path: String,
    pub sha256: String,
    pub license_expression: String,
    pub notice_path: String,
    pub source_span: AuthoringSourceSpanV1,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringPartitionV1 {
    pub partition_id: String,
    pub coordinate_profile_id: String,
    pub chunks: Vec<AuthoringChunkV1>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringChunkV1 {
    pub chunk_id: String,
    pub region_id: String,
    pub chunk_asset_id: String,
    pub required_asset_ids: Vec<String>,
    pub source_span: AuthoringSourceSpanV1,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringNeutralRecordV1 {
    pub asset_id: String,
    pub record_id: String,
    pub kind: AuthoringNeutralRecordKindV1,
    #[serde(default)]
    pub persistent_references: Vec<String>,
    #[serde(default)]
    pub asset_dependencies: Vec<String>,
    #[serde(default)]
    pub properties: Vec<AuthoringPropertyV1>,
    pub source_span: AuthoringSourceSpanV1,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum AuthoringNeutralRecordKindV1 {
    Scene,
    Collider,
    CharacterDefinition,
    ItemDefinition,
    InventoryDefinition,
    EquipmentDefinition,
    DialogueDefinition,
    QuestDefinition,
    RelationshipDefinition,
    InteractionDefinition,
    AbilityDefinition,
    WorldChunk,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringPropertyV1 {
    pub property_id: String,
    pub value_id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(super) enum AuthoringRenderRecordV1 {
    Mesh {
        asset_id: String,
        record_revision: u64,
        bounds_min: [i64; 3],
        bounds_max: [i64; 3],
        positions_micrometres: Vec<[i64; 3]>,
        uv0_q16: Vec<[i32; 2]>,
        #[serde(default)]
        normals_snorm16: Option<Vec<[i16; 3]>>,
        indices: Vec<u32>,
        double_sided: bool,
        source_span: AuthoringSourceSpanV1,
    },
    TextureRgba8 {
        asset_id: String,
        record_revision: u64,
        extent: [u32; 3],
        color_space: AuthoringTextureColorSpaceV1,
        alpha: AuthoringTextureAlphaV1,
        texels: Vec<u8>,
        source_span: AuthoringSourceSpanV1,
    },
    /// Scene look L5 (plan `look/05`): a PNG file among the project's
    /// referenced sources, decoded to `RGBA8` with an optional mip chain.
    TexturePng {
        asset_id: String,
        record_revision: u64,
        relative_path: String,
        color_space: AuthoringTextureColorSpaceV1,
        alpha: AuthoringTextureAlphaV1,
        mip_levels: AuthoringTextureMipLevelsV1,
        source_span: AuthoringSourceSpanV1,
    },
    /// Scene look L5b (plan `look/05b`): one primitive of a glTF 2.0 file
    /// among the project's referenced sources, the named node's global
    /// transform baked into the vertices.
    MeshGltf {
        asset_id: String,
        record_revision: u64,
        relative_path: String,
        mesh: usize,
        primitive: usize,
        #[serde(default)]
        node: Option<usize>,
        #[serde(default)]
        double_sided: bool,
        source_span: AuthoringSourceSpanV1,
    },
    Material {
        asset_id: String,
        record_revision: u64,
        texture_asset_id: String,
        /// Scene look L5: the glTF-style metallic-roughness map (`G`
        /// roughness, `B` metallic), linear.
        #[serde(default)]
        metallic_roughness_texture_asset_id: Option<String>,
        /// Scene look L5: the tangent-space normal map, linear.
        #[serde(default)]
        normal_texture_asset_id: Option<String>,
        /// Scene look L5: a uniform UV scale shared by the bindings.
        #[serde(default = "default_uv_scale")]
        uv_scale: f64,
        base_color_rgba_u16: [u16; 4],
        metallic_u16: u16,
        roughness_u16: u16,
        emissive_rgb_u16: [u16; 3],
        double_sided: bool,
        source_span: AuthoringSourceSpanV1,
    },
    B0Profile {
        asset_id: String,
        record_revision: u64,
        fallback_material_asset_id: String,
        fallback_texture_asset_id: String,
        source_span: AuthoringSourceSpanV1,
    },
    BaseSkinningProfile {
        asset_id: String,
        record_revision: u64,
        mesh_asset_id: String,
        skeleton_asset_id: String,
        body_schema_asset_id: String,
        mesh_origin_in_skeleton_micrometres: [i64; 3],
        max_instances_per_frame: u32,
        render_joints: Vec<AuthoringRenderJointV1>,
        vertex_joint_ranges: Vec<AuthoringVertexJointRangeV1>,
        pose_correctives: Vec<AuthoringPoseCorrectiveV1>,
        source_span: AuthoringSourceSpanV1,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringRenderJointV1 {
    pub render_joint_id: String,
    pub parent_render_joint_id: Option<String>,
    pub animation_joint_id: String,
    pub body_semantic_id: String,
    pub bind_translation_micrometres: [i64; 3],
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringVertexJointRangeV1 {
    pub render_joint_id: String,
    pub first_vertex: u32,
    pub vertex_count: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringPoseCorrectiveV1 {
    pub corrective_id: String,
    pub driver_render_joint_id: String,
    pub driver_axis: AuthoringPoseCorrectiveDriverAxisV1,
    pub activation_start_delta_micrometres: i64,
    pub activation_full_delta_micrometres: i64,
    pub lod_class: AuthoringPoseCorrectiveLodClassV1,
    pub vertex_delta_ranges: Vec<AuthoringPoseCorrectiveVertexDeltaRangeV1>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum AuthoringPoseCorrectiveDriverAxisV1 {
    X,
    Y,
    Z,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum AuthoringPoseCorrectiveLodClassV1 {
    Essential,
    Detail,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringPoseCorrectiveVertexDeltaRangeV1 {
    pub first_vertex: u32,
    pub vertex_count: u32,
    pub delta_micrometres: [i64; 3],
}

impl AuthoringRenderRecordV1 {
    pub(super) fn source_span(&self) -> &AuthoringSourceSpanV1 {
        match self {
            Self::Mesh { source_span, .. }
            | Self::TextureRgba8 { source_span, .. }
            | Self::TexturePng { source_span, .. }
            | Self::MeshGltf { source_span, .. }
            | Self::Material { source_span, .. }
            | Self::B0Profile { source_span, .. }
            | Self::BaseSkinningProfile { source_span, .. } => source_span,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum AuthoringTextureColorSpaceV1 {
    Srgb,
    Linear,
    Data,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum AuthoringTextureAlphaV1 {
    Opaque,
    Straight,
}

/// Scene look L5: whether a PNG texture carries its full mip chain.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum AuthoringTextureMipLevelsV1 {
    None,
    Full,
}

fn default_uv_scale() -> f64 {
    1.0
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringTextCatalogV1 {
    pub asset_id: String,
    pub revision: u32,
    pub locale: String,
    pub fallback_locale: Option<String>,
    pub entries: Vec<AuthoringTextEntryV1>,
    pub source_span: AuthoringSourceSpanV1,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringTextEntryV1 {
    pub text_id: String,
    pub template: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "synth", rename_all = "kebab-case", deny_unknown_fields)]
pub(super) enum AuthoringAudioRecordV1 {
    NoiseBurst {
        asset_id: String,
        record_revision: u64,
        sample_rate_hz: u32,
        frames: u32,
        amplitude: i32,
        seed: u32,
        source_span: AuthoringSourceSpanV1,
    },
    TwoTone {
        asset_id: String,
        record_revision: u64,
        sample_rate_hz: u32,
        frames: u32,
        amplitude: i32,
        first_period: u32,
        second_period: u32,
        source_span: AuthoringSourceSpanV1,
    },
    Thud {
        asset_id: String,
        record_revision: u64,
        sample_rate_hz: u32,
        frames: u32,
        amplitude: i32,
        period: u32,
        source_span: AuthoringSourceSpanV1,
    },
    /// Plan `continuum-water/34`: flat noise with short fades at both ends
    /// and a loop region over the whole clip (a flowing-water bed).
    NoiseLoop {
        asset_id: String,
        record_revision: u64,
        sample_rate_hz: u32,
        frames: u32,
        amplitude: i32,
        seed: u32,
        source_span: AuthoringSourceSpanV1,
    },
}

impl AuthoringAudioRecordV1 {
    pub(super) fn source_span(&self) -> &AuthoringSourceSpanV1 {
        match self {
            Self::NoiseBurst { source_span, .. }
            | Self::TwoTone { source_span, .. }
            | Self::Thud { source_span, .. }
            | Self::NoiseLoop { source_span, .. } => source_span,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum AuthoringPresentationTargetV1 {
    None,
    Interactive,
    DisplaylessOffscreen,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringSourceSpanV1 {
    pub relative_path: String,
    pub line: u32,
    pub column: u32,
}
