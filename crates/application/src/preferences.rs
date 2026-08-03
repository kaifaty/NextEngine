//! Local `PlayerPreferenceProfileV1` store (SPEC-18 §Accessibility).
//!
//! The store keeps the versioned `PresentationOnly` profile inside the user
//! state root. It never participates in save/domain state: a missing file
//! yields bounded defaults, and any decode/validation failure follows
//! `PLAYER_PREFERENCE_INVALID` — the unreadable file is quarantined aside and
//! bounded defaults are returned; gameplay and saves are untouched. Writes go
//! through a same-directory temporary file plus rename so a crash cannot
//! leave a torn profile behind.

use std::fs;
use std::path::{Path, PathBuf};

use next_contracts::canonical::CanonicalDecodeLimits;
use next_contracts::preferences::{PLAYER_PREFERENCE_INVALID_CODE, PlayerPreferenceProfileV1};

use crate::ApplicationError;

pub const PLAYER_PREFERENCE_FILE_NAME: &str = "player-preferences.v1.bin";
pub const PLAYER_PREFERENCE_QUARANTINE_SUFFIX: &str = "quarantine";

/// What happened while loading the local profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerPreferenceLoadOutcomeV1 {
    /// A valid stored profile was loaded.
    Loaded,
    /// No stored profile exists; bounded defaults are in use.
    Absent,
    /// The stored profile failed validation and was quarantined; bounded
    /// defaults are in use. The diagnostic code is stable
    /// (`PLAYER_PREFERENCE_INVALID`).
    Quarantined { diagnostic_code: &'static str },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerPreferenceLoadV1 {
    pub profile: PlayerPreferenceProfileV1,
    pub outcome: PlayerPreferenceLoadOutcomeV1,
}

/// File-backed local preference store rooted at the application user state
/// root. The store performs no caching; every load re-reads the file.
#[derive(Clone, Debug)]
pub struct PlayerPreferenceStoreV1 {
    path: PathBuf,
}

impl PlayerPreferenceStoreV1 {
    #[must_use]
    pub fn new(state_root: &Path) -> Self {
        Self {
            path: state_root.join(PLAYER_PREFERENCE_FILE_NAME),
        }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Loads the stored profile. IO failures other than "not found" are hard
    /// errors (storage is unavailable); content failures quarantine the file
    /// and fall back to bounded defaults.
    pub fn load(&self) -> Result<PlayerPreferenceLoadV1, ApplicationError> {
        let bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(PlayerPreferenceLoadV1 {
                    profile: PlayerPreferenceProfileV1::bounded_defaults(),
                    outcome: PlayerPreferenceLoadOutcomeV1::Absent,
                });
            }
            Err(error) => return Err(ApplicationError::Io(error)),
        };
        match PlayerPreferenceProfileV1::from_canonical_bytes(
            &bytes,
            CanonicalDecodeLimits::default(),
        ) {
            Ok(profile) => Ok(PlayerPreferenceLoadV1 {
                profile,
                outcome: PlayerPreferenceLoadOutcomeV1::Loaded,
            }),
            Err(_) => {
                self.quarantine()?;
                Ok(PlayerPreferenceLoadV1 {
                    profile: PlayerPreferenceProfileV1::bounded_defaults(),
                    outcome: PlayerPreferenceLoadOutcomeV1::Quarantined {
                        diagnostic_code: PLAYER_PREFERENCE_INVALID_CODE,
                    },
                })
            }
        }
    }

    /// Validates and stores the profile, replacing any previous one. The
    /// write is same-directory temporary file + rename; a quarantine file
    /// from an earlier invalid profile is left untouched for diagnostics.
    pub fn save(&self, profile: &PlayerPreferenceProfileV1) -> Result<(), ApplicationError> {
        let bytes = profile.canonical_bytes()?;
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let temporary = self.path.with_extension("tmp");
        fs::write(&temporary, &bytes)?;
        fs::rename(&temporary, &self.path)?;
        Ok(())
    }

    /// Moves the unreadable profile aside. A previous quarantine file is
    /// replaced; only the latest invalid profile is retained (bounded
    /// diagnostic footprint).
    fn quarantine(&self) -> Result<(), ApplicationError> {
        let quarantine = quarantine_path(&self.path);
        if quarantine.exists() {
            fs::remove_file(&quarantine)?;
        }
        fs::rename(&self.path, &quarantine)?;
        Ok(())
    }
}

#[must_use]
pub fn quarantine_path(profile_path: &Path) -> PathBuf {
    let mut name = profile_path.as_os_str().to_owned();
    name.push(".");
    name.push(PLAYER_PREFERENCE_QUARANTINE_SUFFIX);
    PathBuf::from(name)
}

/// Maps a profile to the desktop adapter UI options: the requested locale (or
/// the project default when unset) and the rasterizer text scale derived from
/// the bounded milli value (`500..=2000` → `1..=4`).
#[must_use]
pub fn preference_ui_options(profile: &PlayerPreferenceProfileV1) -> (String, u32) {
    let locale = profile
        .ui_locale_or_none
        .as_ref()
        .map_or_else(|| "en".to_owned(), |tag| tag.as_str().to_owned());
    (
        locale,
        next_contracts::preferences::text_scale_from_milli(profile.text_scale_milli),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        PLAYER_PREFERENCE_FILE_NAME, PlayerPreferenceLoadOutcomeV1, PlayerPreferenceStoreV1,
        preference_ui_options, quarantine_path,
    };
    use next_contracts::localization::TextLocaleTagV1;
    use next_contracts::preferences::{PlayerPreferenceProfileV1, text_scale_from_milli};
    use std::sync::atomic::{AtomicU64, Ordering};

    fn unique_state_root() -> std::path::PathBuf {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "nextengine-preference-store-test-{}-{id}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("state root");
        root
    }

    fn sample_profile() -> PlayerPreferenceProfileV1 {
        PlayerPreferenceProfileV1::new(
            2,
            1_500,
            Some(TextLocaleTagV1::new("qps-ploc").expect("locale")),
        )
        .expect("profile")
    }

    #[test]
    fn absent_store_yields_bounded_defaults() {
        let root = unique_state_root();
        let store = PlayerPreferenceStoreV1::new(&root);
        let load = store.load().expect("load");
        assert_eq!(load.outcome, PlayerPreferenceLoadOutcomeV1::Absent);
        assert_eq!(load.profile, PlayerPreferenceProfileV1::bounded_defaults());
    }

    #[test]
    fn save_then_load_round_trips() {
        let root = unique_state_root();
        let store = PlayerPreferenceStoreV1::new(&root);
        let profile = sample_profile();
        store.save(&profile).expect("save");
        let load = store.load().expect("load");
        assert_eq!(load.outcome, PlayerPreferenceLoadOutcomeV1::Loaded);
        assert_eq!(load.profile, profile);
    }

    #[test]
    fn corrupt_profile_is_quarantined_and_defaults_used() {
        let root = unique_state_root();
        let store = PlayerPreferenceStoreV1::new(&root);
        std::fs::write(store.path(), b"not-a-preference-profile").expect("write corrupt");
        let load = store.load().expect("load");
        assert_eq!(
            load.outcome,
            PlayerPreferenceLoadOutcomeV1::Quarantined {
                diagnostic_code: "PLAYER_PREFERENCE_INVALID"
            }
        );
        assert_eq!(load.profile, PlayerPreferenceProfileV1::bounded_defaults());
        assert!(!store.path().exists());
        assert!(quarantine_path(store.path()).exists());
        // The next load behaves like an absent store.
        let reload = store.load().expect("reload");
        assert_eq!(reload.outcome, PlayerPreferenceLoadOutcomeV1::Absent);
    }

    #[test]
    fn tampered_hash_is_quarantined() {
        let root = unique_state_root();
        let store = PlayerPreferenceStoreV1::new(&root);
        let mut bytes = sample_profile().canonical_bytes().expect("encode");
        let last = bytes.len() - 1;
        bytes[last] ^= 0xff;
        std::fs::write(store.path(), &bytes).expect("write tampered");
        let load = store.load().expect("load");
        assert!(matches!(
            load.outcome,
            PlayerPreferenceLoadOutcomeV1::Quarantined { .. }
        ));
    }

    #[test]
    fn save_replaces_previous_profile_and_keeps_quarantine() {
        let root = unique_state_root();
        let store = PlayerPreferenceStoreV1::new(&root);
        std::fs::write(store.path(), b"junk").expect("write corrupt");
        let _ = store.load().expect("quarantine load");
        let profile = sample_profile();
        store.save(&profile).expect("save");
        assert!(quarantine_path(store.path()).exists());
        assert_eq!(store.load().expect("load").profile, profile);
        assert_eq!(
            store.path().file_name().expect("name"),
            PLAYER_PREFERENCE_FILE_NAME
        );
    }

    #[test]
    fn ui_options_map_locale_and_scale() {
        let (locale, scale) = preference_ui_options(&sample_profile());
        assert_eq!(locale, "qps-ploc");
        assert_eq!(scale, 3);
        let (locale, scale) = preference_ui_options(&PlayerPreferenceProfileV1::bounded_defaults());
        assert_eq!(locale, "en");
        assert_eq!(scale, 2);
        assert_eq!(text_scale_from_milli(0), 1);
        assert_eq!(text_scale_from_milli(499), 1); // defensive, contract rejects below 500
        assert_eq!(text_scale_from_milli(500), 1);
        assert_eq!(text_scale_from_milli(2_000), 4);
        assert_eq!(text_scale_from_milli(u32::MAX), 4);
    }
}
