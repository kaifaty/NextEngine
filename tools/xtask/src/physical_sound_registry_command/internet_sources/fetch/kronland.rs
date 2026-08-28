use serde::Serialize;

const SCHEMA: &str = "nextengine.experimental-kronland-material-page-identity.v1";
const SOURCE_URL: &str = "https://kronland.fr/publications/controlling-the-perceived-material-in-an-impact-sound-synthesizer/";
const AUDIO_PREFIX: &str = "https://kronland.fr/wp-content/uploads/2007/09/";
const RECORDED_SCOPE: &str = "Impact sounds from everyday life objects made of Wood, Metal and Glass materials were recorded";
const AUTHORS: &str = "Aramaki M., Besson M., Kronland-Martinet R., Ystad S.";

#[derive(Serialize)]
struct PageIdentity {
    schema: &'static str,
    source_url: &'static str,
    title: &'static str,
    authors: &'static str,
    publication_date: &'static str,
    journal: &'static str,
    recorded_scope: &'static str,
    tracks: Vec<TrackIdentity>,
}

#[derive(Serialize)]
struct TrackIdentity {
    material_label: &'static str,
    track_number: u8,
    original_file_name: String,
    original_url: String,
    synthesized_file_name: String,
    tuned_file_name: String,
}

const TRACKS: [(&str, u8, &str); 15] = [
    ("Wood", 1, "b1"),
    ("Wood", 2, "b2"),
    ("Wood", 3, "b3"),
    ("Wood", 4, "b4"),
    ("Wood", 5, "b5"),
    ("Metal", 1, "m1"),
    ("Metal", 2, "m5"),
    ("Metal", 3, "m8"),
    ("Metal", 4, "m9"),
    ("Metal", 5, "m10"),
    ("Glass", 1, "v1"),
    ("Glass", 2, "v2"),
    ("Glass", 3, "v4"),
    ("Glass", 4, "v5"),
    ("Glass", 5, "v6"),
];

pub(super) fn normalize_material_page_identity(url: &str, bytes: &[u8]) -> Result<Vec<u8>, String> {
    if url != SOURCE_URL {
        return Err("Kronland material-page normalization requires its canonical URL".to_owned());
    }
    let page = std::str::from_utf8(bytes)
        .map_err(|error| format!("Kronland material page must be UTF-8: {error}"))?;
    for marker in [
        "Controlling the perceived material in an impact sound synthesizer",
        "The Language of Sounds",
        AUTHORS,
        "<strong>Publication Date:</strong> September 2011",
        "<strong>Journal:</strong> IEEE Transactions on Audio, Speech, and Language Processing",
        RECORDED_SCOPE,
    ] {
        if !page.contains(marker) {
            return Err(format!(
                "Kronland material page is missing frozen identity marker {marker:?}"
            ));
        }
    }

    let mut tracks = Vec::with_capacity(TRACKS.len());
    for (material_label, track_number, recording_id) in TRACKS {
        let original_file_name = format!("AST_{recording_id}_expe.wav");
        let synthesized_file_name = format!("AST_{recording_id}_synth.wav");
        let tuned_file_name = format!("AST_{recording_id}_transp.wav");
        let original_url = format!("{AUDIO_PREFIX}{original_file_name}");
        for marker in [
            format!("<a href='{original_url}'>{material_label} {track_number} original</a>"),
            format!(
                "<a href='{AUDIO_PREFIX}{synthesized_file_name}'>{material_label} {track_number} synthesized</a>"
            ),
            format!(
                "<a href='{AUDIO_PREFIX}{tuned_file_name}'>{material_label} {track_number} tuned</a>"
            ),
        ] {
            if !page.contains(&marker) {
                return Err(format!(
                    "Kronland material page is missing frozen track marker {marker:?}"
                ));
            }
        }
        tracks.push(TrackIdentity {
            material_label,
            track_number,
            original_file_name,
            original_url,
            synthesized_file_name,
            tuned_file_name,
        });
    }

    let identity = PageIdentity {
        schema: SCHEMA,
        source_url: SOURCE_URL,
        title: "Controlling the perceived material in an impact sound synthesizer",
        authors: AUTHORS,
        publication_date: "September 2011",
        journal: "IEEE Transactions on Audio, Speech, and Language Processing",
        recorded_scope: RECORDED_SCOPE,
        tracks,
    };
    let mut output = serde_json::to_vec_pretty(&identity).map_err(|error| error.to_string())?;
    output.push(b'\n');
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_wordpress_state_does_not_change_identity() {
        let first = fixture("nonce-a");
        let second = fixture("nonce-b");
        assert_eq!(
            normalize_material_page_identity(SOURCE_URL, first.as_bytes()).expect("first identity"),
            normalize_material_page_identity(SOURCE_URL, second.as_bytes())
                .expect("second identity")
        );
    }

    #[test]
    fn missing_original_or_material_scope_fails_closed() {
        let raw = fixture("nonce").replace("Wood 1 original", "Wood 1 synthesized");
        assert!(normalize_material_page_identity(SOURCE_URL, raw.as_bytes()).is_err());
        let raw = fixture("nonce").replace("Wood, Metal and Glass", "Wood and Metal");
        assert!(normalize_material_page_identity(SOURCE_URL, raw.as_bytes()).is_err());
    }

    fn fixture(nonce: &str) -> String {
        let mut page = format!(
            "Controlling the perceived material in an impact sound synthesizer\n\
             The Language of Sounds\n{AUTHORS}\n\
             <strong>Publication Date:</strong> September 2011\n\
             <strong>Journal:</strong> IEEE Transactions on Audio, Speech, and Language Processing\n\
             {RECORDED_SCOPE}\n{nonce}\n"
        );
        for (material_label, track_number, recording_id) in TRACKS {
            for (suffix, label) in [
                ("expe", "original"),
                ("synth", "synthesized"),
                ("transp", "tuned"),
            ] {
                page.push_str(&format!(
                    "<a href='{AUDIO_PREFIX}AST_{recording_id}_{suffix}.wav'>{material_label} {track_number} {label}</a>\n"
                ));
            }
        }
        page
    }
}
