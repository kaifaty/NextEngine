use std::collections::BTreeSet;

use crate::ids::{ContentHash, SchemaId};
use crate::manifest_jcs::{JcsValue, encode_canonical_jcs};
use crate::project::domain_hash;

use super::{PresentationContractError, number, object, string};

pub const UI_SEMANTIC_SNAPSHOT_SCHEMA_VERSION: u32 = 1;
pub const SEMANTIC_UI_PRESENTATION_RECORD_SCHEMA_VERSION: u32 = 1;
pub const PRESENTATION_MAX_SEMANTIC_UI_RECORDS: usize = 4_096;
pub const PRESENTATION_DEFAULT_SEMANTIC_UI_RECORDS_PER_BATCH: usize = 64;
pub const UI_MAX_PANELS_PER_SURFACE: usize = 256;
pub const UI_MAX_ELEMENTS_PER_PANEL: usize = 512;
pub const UI_MAX_TEXT_ARGUMENTS: usize = 16;
pub const UI_MAX_AFFORDANCES_PER_ELEMENT: usize = 8;
pub const UI_MAX_FOCUS_ORDER: usize = 1_024;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum UiElementRoleV1 {
    Label = 0,
    Value = 1,
    Meter = 2,
    Button = 3,
    List = 4,
    ListItem = 5,
    Panel = 6,
    Hint = 7,
    /// Voice-absent subtitle fallback line (SPEC-08); visibility is a local
    /// `PresentationOnly` preference and never gameplay state.
    Subtitle = 8,
}

impl UiElementRoleV1 {
    const fn token(self) -> &'static str {
        match self {
            Self::Label => "Label",
            Self::Value => "Value",
            Self::Meter => "Meter",
            Self::Button => "Button",
            Self::List => "List",
            Self::ListItem => "ListItem",
            Self::Panel => "Panel",
            Self::Hint => "Hint",
            Self::Subtitle => "Subtitle",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum UiStyleRoleV1 {
    Default = 0,
    Muted = 1,
    Accent = 2,
    Warning = 3,
    Danger = 4,
}

impl UiStyleRoleV1 {
    const fn token(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Muted => "Muted",
            Self::Accent => "Accent",
            Self::Warning => "Warning",
            Self::Danger => "Danger",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum UiAccessibilityRoleV1 {
    Standard = 0,
    Heading = 1,
    Status = 2,
    Alert = 3,
}

impl UiAccessibilityRoleV1 {
    const fn token(self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::Heading => "Heading",
            Self::Status => "Status",
            Self::Alert => "Alert",
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum UiTextArgumentV1 {
    SignedInteger(i64),
    TextId(SchemaId),
}

impl UiTextArgumentV1 {
    fn value(&self) -> JcsValue {
        match self {
            Self::SignedInteger(value) => object([
                ("kind", string("SignedInteger")),
                ("value", string(format!("{:016x}", *value as u64))),
            ]),
            Self::TextId(text_id) => object([
                ("kind", string("TextId")),
                ("value", string(text_id.as_str())),
            ]),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiTextRefV1 {
    pub text_id: SchemaId,
    pub arguments: Vec<UiTextArgumentV1>,
}

impl UiTextRefV1 {
    pub fn new(
        text_id: SchemaId,
        arguments: Vec<UiTextArgumentV1>,
    ) -> Result<Self, PresentationContractError> {
        let value = Self { text_id, arguments };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), PresentationContractError> {
        if self.arguments.len() > UI_MAX_TEXT_ARGUMENTS {
            return Err(PresentationContractError::InvalidUiText);
        }
        Ok(())
    }

    fn value(&self) -> JcsValue {
        object([
            (
                "arguments",
                JcsValue::Array(self.arguments.iter().map(UiTextArgumentV1::value).collect()),
            ),
            ("text_id", string(self.text_id.as_str())),
        ])
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiElementValueV1 {
    None,
    Scalar { current: i64, maximum: i64 },
}

impl UiElementValueV1 {
    pub fn validate(self) -> Result<(), PresentationContractError> {
        if let Self::Scalar { current, maximum } = self
            && (maximum < 0 || current < 0 || current > maximum)
        {
            return Err(PresentationContractError::InvalidUiValue);
        }
        Ok(())
    }

    fn value(self) -> JcsValue {
        match self {
            Self::None => object([("kind", string("None"))]),
            Self::Scalar { current, maximum } => object([
                ("current", string(format!("{:016x}", current as u64))),
                ("kind", string("Scalar")),
                ("maximum", string(format!("{:016x}", maximum as u64))),
            ]),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct UiActionAffordanceV1 {
    pub action_id: SchemaId,
    pub enabled: bool,
}

impl UiActionAffordanceV1 {
    fn value(&self) -> JcsValue {
        object([
            ("action_id", string(self.action_id.as_str())),
            (
                "enabled",
                string(if self.enabled { "true" } else { "false" }),
            ),
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiSemanticElementV1 {
    pub element_id: SchemaId,
    pub role: UiElementRoleV1,
    pub style_role: UiStyleRoleV1,
    pub accessibility_role: UiAccessibilityRoleV1,
    pub enabled: bool,
    pub visible: bool,
    pub selected: bool,
    pub text_or_none: Option<UiTextRefV1>,
    pub value: UiElementValueV1,
    pub affordances: Vec<UiActionAffordanceV1>,
    pub canonical_hash: ContentHash,
}

impl UiSemanticElementV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the semantic element record keeps every canonical presentation field explicit"
    )]
    pub fn new(
        element_id: SchemaId,
        role: UiElementRoleV1,
        style_role: UiStyleRoleV1,
        accessibility_role: UiAccessibilityRoleV1,
        enabled: bool,
        visible: bool,
        selected: bool,
        text_or_none: Option<UiTextRefV1>,
        value: UiElementValueV1,
        mut affordances: Vec<UiActionAffordanceV1>,
    ) -> Result<Self, PresentationContractError> {
        affordances.sort_by(|left, right| left.action_id.cmp(&right.action_id));
        let mut record = Self {
            element_id,
            role,
            style_role,
            accessibility_role,
            enabled,
            visible,
            selected,
            text_or_none,
            value,
            affordances,
            canonical_hash: ContentHash::default(),
        };
        record.validate_body()?;
        record.canonical_hash = record.computed_hash();
        Ok(record)
    }

    pub fn validate(&self) -> Result<(), PresentationContractError> {
        self.validate_body()?;
        if self.computed_hash() != self.canonical_hash {
            return Err(PresentationContractError::HashMismatch);
        }
        Ok(())
    }

    fn validate_body(&self) -> Result<(), PresentationContractError> {
        if let Some(text) = &self.text_or_none {
            text.validate()?;
        }
        self.value.validate()?;
        validate_affordances(&self.affordances)?;
        Ok(())
    }

    fn computed_hash(&self) -> ContentHash {
        domain_hash(
            "nextengine.ui-semantic-element.v1",
            &encode_canonical_jcs(&self.body_value()),
        )
    }

    fn body_value(&self) -> JcsValue {
        object([
            (
                "accessibility_role",
                string(self.accessibility_role.token()),
            ),
            (
                "affordances",
                JcsValue::Array(
                    self.affordances
                        .iter()
                        .map(UiActionAffordanceV1::value)
                        .collect(),
                ),
            ),
            ("element_id", string(self.element_id.as_str())),
            (
                "enabled",
                string(if self.enabled { "true" } else { "false" }),
            ),
            ("role", string(self.role.token())),
            (
                "selected",
                string(if self.selected { "true" } else { "false" }),
            ),
            ("style_role", string(self.style_role.token())),
            (
                "text_or_none",
                self.text_or_none
                    .as_ref()
                    .map_or_else(|| string("none"), UiTextRefV1::value),
            ),
            ("value", self.value.value()),
            (
                "visible",
                string(if self.visible { "true" } else { "false" }),
            ),
        ])
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct UiElementFocusKeyV1 {
    pub semantic_path_id: SchemaId,
    pub element_id: SchemaId,
}

impl UiElementFocusKeyV1 {
    fn value(&self) -> JcsValue {
        object([
            ("element_id", string(self.element_id.as_str())),
            ("semantic_path_id", string(self.semantic_path_id.as_str())),
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiSemanticPanelV1 {
    pub semantic_path_id: SchemaId,
    pub elements: Vec<UiSemanticElementV1>,
    pub canonical_hash: ContentHash,
}

impl UiSemanticPanelV1 {
    pub fn new(
        semantic_path_id: SchemaId,
        mut elements: Vec<UiSemanticElementV1>,
    ) -> Result<Self, PresentationContractError> {
        if elements.len() > UI_MAX_ELEMENTS_PER_PANEL {
            return Err(PresentationContractError::LimitExceeded);
        }
        for element in &elements {
            element.validate()?;
        }
        elements.sort_by(|left, right| left.element_id.cmp(&right.element_id));
        ensure_element_ids_unique(&elements)?;
        let mut value = Self {
            semantic_path_id,
            elements,
            canonical_hash: ContentHash::default(),
        };
        value.canonical_hash = value.computed_hash();
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), PresentationContractError> {
        if self.elements.len() > UI_MAX_ELEMENTS_PER_PANEL {
            return Err(PresentationContractError::LimitExceeded);
        }
        for element in &self.elements {
            element.validate()?;
        }
        if self
            .elements
            .windows(2)
            .any(|pair| pair[0].element_id >= pair[1].element_id)
        {
            return Err(PresentationContractError::NonCanonicalOrder);
        }
        if self.computed_hash() != self.canonical_hash {
            return Err(PresentationContractError::HashMismatch);
        }
        Ok(())
    }

    fn computed_hash(&self) -> ContentHash {
        domain_hash(
            "nextengine.ui-semantic-panel.v1",
            &encode_canonical_jcs(&object([
                (
                    "ordered_element_hashes",
                    JcsValue::Array(
                        self.elements
                            .iter()
                            .map(|element| string(element.canonical_hash.to_hex()))
                            .collect(),
                    ),
                ),
                ("semantic_path_id", string(self.semantic_path_id.as_str())),
            ])),
        )
    }

    fn contains_element(&self, element_id: &SchemaId) -> bool {
        self.elements
            .iter()
            .any(|element| &element.element_id == element_id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiSemanticSnapshotV1 {
    pub schema_version: u32,
    pub surface_id: SchemaId,
    pub panels: Vec<UiSemanticPanelV1>,
    pub focus_order: Vec<UiElementFocusKeyV1>,
    pub focused_element_or_none: Option<UiElementFocusKeyV1>,
    pub canonical_hash: ContentHash,
}

impl UiSemanticSnapshotV1 {
    pub fn new(
        surface_id: SchemaId,
        mut panels: Vec<UiSemanticPanelV1>,
        focus_order: Vec<UiElementFocusKeyV1>,
        focused_element_or_none: Option<UiElementFocusKeyV1>,
    ) -> Result<Self, PresentationContractError> {
        if panels.len() > UI_MAX_PANELS_PER_SURFACE {
            return Err(PresentationContractError::LimitExceeded);
        }
        for panel in &panels {
            panel.validate()?;
        }
        panels.sort_by(|left, right| left.semantic_path_id.cmp(&right.semantic_path_id));
        let mut value = Self {
            schema_version: UI_SEMANTIC_SNAPSHOT_SCHEMA_VERSION,
            surface_id,
            panels,
            focus_order,
            focused_element_or_none,
            canonical_hash: ContentHash::default(),
        };
        value.validate_body()?;
        value.canonical_hash = value.computed_hash();
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), PresentationContractError> {
        if self.schema_version != UI_SEMANTIC_SNAPSHOT_SCHEMA_VERSION {
            return Err(PresentationContractError::UiSchemaIncompatible);
        }
        for panel in &self.panels {
            panel.validate()?;
        }
        if self
            .panels
            .windows(2)
            .any(|pair| pair[0].semantic_path_id >= pair[1].semantic_path_id)
        {
            return Err(PresentationContractError::NonCanonicalOrder);
        }
        self.validate_body()?;
        if self.computed_hash() != self.canonical_hash {
            return Err(PresentationContractError::HashMismatch);
        }
        Ok(())
    }

    fn validate_body(&self) -> Result<(), PresentationContractError> {
        if self.panels.len() > UI_MAX_PANELS_PER_SURFACE {
            return Err(PresentationContractError::LimitExceeded);
        }
        ensure_panel_ids_unique(&self.panels)?;
        if self.focus_order.len() > UI_MAX_FOCUS_ORDER {
            return Err(PresentationContractError::LimitExceeded);
        }
        let mut seen_focus = BTreeSet::new();
        for key in &self.focus_order {
            if !seen_focus.insert(key) {
                return Err(PresentationContractError::InvalidUiFocusGraph);
            }
            self.resolve_focus_key(key)?;
        }
        if let Some(focused) = &self.focused_element_or_none
            && !seen_focus.contains(focused)
        {
            return Err(PresentationContractError::InvalidUiFocusGraph);
        }
        Ok(())
    }

    fn resolve_focus_key(
        &self,
        key: &UiElementFocusKeyV1,
    ) -> Result<(), PresentationContractError> {
        let panel = self
            .panels
            .iter()
            .find(|panel| panel.semantic_path_id == key.semantic_path_id)
            .ok_or(PresentationContractError::InvalidUiFocusGraph)?;
        if !panel.contains_element(&key.element_id) {
            return Err(PresentationContractError::InvalidUiFocusGraph);
        }
        Ok(())
    }

    fn computed_hash(&self) -> ContentHash {
        domain_hash(
            "nextengine.ui-semantic-snapshot.v1",
            &encode_canonical_jcs(&object([
                (
                    "focus_order",
                    JcsValue::Array(
                        self.focus_order
                            .iter()
                            .map(UiElementFocusKeyV1::value)
                            .collect(),
                    ),
                ),
                (
                    "focused_element_or_none",
                    self.focused_element_or_none
                        .as_ref()
                        .map_or_else(|| string("none"), UiElementFocusKeyV1::value),
                ),
                (
                    "ordered_panel_hashes",
                    JcsValue::Array(
                        self.panels
                            .iter()
                            .map(|panel| string(panel.canonical_hash.to_hex()))
                            .collect(),
                    ),
                ),
                ("schema_version", number(self.schema_version)),
                ("surface_id", string(self.surface_id.as_str())),
            ])),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticUiPresentationRecordV1 {
    pub schema_version: u32,
    pub snapshot_epoch: ContentHash,
    pub surface_id: SchemaId,
    pub semantic_path_id: SchemaId,
    pub source_snapshot_hash: ContentHash,
    pub element: UiSemanticElementV1,
    pub canonical_hash: ContentHash,
}

impl SemanticUiPresentationRecordV1 {
    pub fn new(
        snapshot_epoch: ContentHash,
        surface_id: SchemaId,
        semantic_path_id: SchemaId,
        source_snapshot_hash: ContentHash,
        element: UiSemanticElementV1,
    ) -> Result<Self, PresentationContractError> {
        let mut value = Self {
            schema_version: SEMANTIC_UI_PRESENTATION_RECORD_SCHEMA_VERSION,
            snapshot_epoch,
            surface_id,
            semantic_path_id,
            source_snapshot_hash,
            element,
            canonical_hash: ContentHash::default(),
        };
        value.validate_body()?;
        value.canonical_hash = value.computed_hash();
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), PresentationContractError> {
        if self.schema_version != SEMANTIC_UI_PRESENTATION_RECORD_SCHEMA_VERSION {
            return Err(PresentationContractError::UiSchemaIncompatible);
        }
        self.validate_body()?;
        if self.computed_hash() != self.canonical_hash {
            return Err(PresentationContractError::HashMismatch);
        }
        Ok(())
    }

    fn validate_body(&self) -> Result<(), PresentationContractError> {
        self.element.validate()?;
        if self.source_snapshot_hash == ContentHash::default() {
            return Err(PresentationContractError::InvalidUiIdentifier);
        }
        Ok(())
    }

    fn computed_hash(&self) -> ContentHash {
        domain_hash(
            "nextengine.semantic-ui-presentation-record.v1",
            &encode_canonical_jcs(&self.body_value()),
        )
    }

    fn body_value(&self) -> JcsValue {
        object([
            ("element", self.element.body_value()),
            ("schema_version", number(self.schema_version)),
            ("semantic_path_id", string(self.semantic_path_id.as_str())),
            ("snapshot_epoch", string(self.snapshot_epoch.to_hex())),
            (
                "source_snapshot_hash",
                string(self.source_snapshot_hash.to_hex()),
            ),
            ("surface_id", string(self.surface_id.as_str())),
        ])
    }

    fn sort_key(&self) -> (&SchemaId, &SchemaId, &SchemaId) {
        (
            &self.surface_id,
            &self.semantic_path_id,
            &self.element.element_id,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticUiPresentationBatchV1 {
    pub batch_index: u32,
    pub first_global_ordinal: u32,
    pub records: Vec<SemanticUiPresentationRecordV1>,
    pub records_root: ContentHash,
}

impl SemanticUiPresentationBatchV1 {
    fn new(
        batch_index: u32,
        first_global_ordinal: u32,
        records: Vec<SemanticUiPresentationRecordV1>,
    ) -> Result<Self, PresentationContractError> {
        if records.is_empty() {
            return Err(PresentationContractError::EmptyBatch);
        }
        let mut value = Self {
            batch_index,
            first_global_ordinal,
            records,
            records_root: ContentHash::default(),
        };
        value.records_root = value.computed_root();
        Ok(value)
    }

    fn validate(&self) -> Result<(), PresentationContractError> {
        if self.records.is_empty() {
            return Err(PresentationContractError::EmptyBatch);
        }
        for record in &self.records {
            record.validate()?;
        }
        if self.computed_root() != self.records_root {
            return Err(PresentationContractError::HashMismatch);
        }
        Ok(())
    }

    fn computed_root(&self) -> ContentHash {
        domain_hash(
            "nextengine.presentation-batch.v1",
            &encode_canonical_jcs(&object([
                ("batch_index", number(self.batch_index)),
                ("batch_kind", string("SemanticUi")),
                ("first_global_ordinal", number(self.first_global_ordinal)),
                (
                    "record_count",
                    number(u32::try_from(self.records.len()).unwrap_or(u32::MAX)),
                ),
                (
                    "ordered_record_hashes",
                    JcsValue::Array(
                        self.records
                            .iter()
                            .map(|record| string(record.canonical_hash.to_hex()))
                            .collect(),
                    ),
                ),
            ])),
        )
    }
}

pub fn build_semantic_ui_batches(
    snapshot_epoch: ContentHash,
    mut records: Vec<SemanticUiPresentationRecordV1>,
    max_records_per_batch: usize,
) -> Result<Vec<SemanticUiPresentationBatchV1>, PresentationContractError> {
    if max_records_per_batch == 0 {
        return Err(PresentationContractError::InvalidBatchProfile);
    }
    if records.len() > PRESENTATION_MAX_SEMANTIC_UI_RECORDS {
        return Err(PresentationContractError::LimitExceeded);
    }
    for record in &records {
        record.validate()?;
        if record.snapshot_epoch != snapshot_epoch {
            return Err(PresentationContractError::SnapshotEpochMismatch);
        }
    }
    records.sort_by(|left, right| left.sort_key().cmp(&right.sort_key()));
    ensure_record_keys_unique(records.iter())?;
    records
        .chunks(max_records_per_batch)
        .enumerate()
        .map(|(batch_index, records)| {
            let first = batch_index
                .checked_mul(max_records_per_batch)
                .and_then(|value| u32::try_from(value).ok())
                .ok_or(PresentationContractError::LimitExceeded)?;
            SemanticUiPresentationBatchV1::new(
                u32::try_from(batch_index).map_err(|_| PresentationContractError::LimitExceeded)?,
                first,
                records.to_vec(),
            )
        })
        .collect()
}

pub fn validate_semantic_ui_batches(
    snapshot_epoch: ContentHash,
    batches: &[SemanticUiPresentationBatchV1],
) -> Result<(), PresentationContractError> {
    let mut expected_batch_index = 0_u32;
    let mut expected_ordinal = 0_u32;
    let mut records = Vec::new();
    for batch in batches {
        if batch.batch_index != expected_batch_index
            || batch.first_global_ordinal != expected_ordinal
        {
            return Err(PresentationContractError::InvalidBatchBoundary);
        }
        batch.validate()?;
        if batch
            .records
            .iter()
            .any(|record| record.snapshot_epoch != snapshot_epoch)
        {
            return Err(PresentationContractError::SnapshotEpochMismatch);
        }
        expected_batch_index = expected_batch_index
            .checked_add(1)
            .ok_or(PresentationContractError::LimitExceeded)?;
        expected_ordinal = expected_ordinal
            .checked_add(
                u32::try_from(batch.records.len())
                    .map_err(|_| PresentationContractError::LimitExceeded)?,
            )
            .ok_or(PresentationContractError::LimitExceeded)?;
        records.extend(batch.records.iter());
    }
    if records.len() > PRESENTATION_MAX_SEMANTIC_UI_RECORDS
        || records
            .windows(2)
            .any(|pair| pair[0].sort_key() >= pair[1].sort_key())
    {
        return Err(PresentationContractError::NonCanonicalOrder);
    }
    ensure_record_keys_unique(records)
}

pub(super) fn semantic_ui_batches_value(batches: &[SemanticUiPresentationBatchV1]) -> JcsValue {
    JcsValue::Array(
        batches
            .iter()
            .map(|batch| {
                object([
                    ("batch_index", number(batch.batch_index)),
                    ("first_global_ordinal", number(batch.first_global_ordinal)),
                    ("records_root", string(batch.records_root.to_hex())),
                ])
            })
            .collect(),
    )
}

fn validate_affordances(
    affordances: &[UiActionAffordanceV1],
) -> Result<(), PresentationContractError> {
    if affordances.len() > UI_MAX_AFFORDANCES_PER_ELEMENT {
        return Err(PresentationContractError::InvalidUiAffordance);
    }
    let mut seen = BTreeSet::new();
    for affordance in affordances {
        if !seen.insert(&affordance.action_id) {
            return Err(PresentationContractError::InvalidUiAffordance);
        }
    }
    if affordances
        .windows(2)
        .any(|pair| pair[0].action_id > pair[1].action_id)
    {
        return Err(PresentationContractError::NonCanonicalOrder);
    }
    Ok(())
}

fn ensure_element_ids_unique(
    elements: &[UiSemanticElementV1],
) -> Result<(), PresentationContractError> {
    let mut ids = BTreeSet::new();
    for element in elements {
        if !ids.insert(&element.element_id) {
            return Err(PresentationContractError::DuplicateUiElementKey);
        }
    }
    Ok(())
}

fn ensure_panel_ids_unique(panels: &[UiSemanticPanelV1]) -> Result<(), PresentationContractError> {
    let mut ids = BTreeSet::new();
    for panel in panels {
        if !ids.insert(&panel.semantic_path_id) {
            return Err(PresentationContractError::DuplicateUiPanelId);
        }
    }
    Ok(())
}

fn ensure_record_keys_unique<'a>(
    records: impl IntoIterator<Item = &'a SemanticUiPresentationRecordV1>,
) -> Result<(), PresentationContractError> {
    let mut keys = BTreeSet::new();
    for record in records {
        if !keys.insert(record.sort_key()) {
            return Err(PresentationContractError::DuplicateUiElementKey);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
