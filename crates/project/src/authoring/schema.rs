use serde::Deserialize;

pub(super) const AUTHORING_FORMAT_V6: &str = "nextengine.project-authoring.v6";

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProjectAuthoringManifestV6 {
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
pub(super) struct AuthoringHumanoidCatalogV1 {
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
    Material {
        asset_id: String,
        record_revision: u64,
        texture_asset_id: String,
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
}

impl AuthoringRenderRecordV1 {
    pub(super) fn source_span(&self) -> &AuthoringSourceSpanV1 {
        match self {
            Self::Mesh { source_span, .. }
            | Self::TextureRgba8 { source_span, .. }
            | Self::Material { source_span, .. }
            | Self::B0Profile { source_span, .. } => source_span,
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
}

impl AuthoringAudioRecordV1 {
    pub(super) fn source_span(&self) -> &AuthoringSourceSpanV1 {
        match self {
            Self::NoiseBurst { source_span, .. }
            | Self::TwoTone { source_span, .. }
            | Self::Thud { source_span, .. } => source_span,
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
