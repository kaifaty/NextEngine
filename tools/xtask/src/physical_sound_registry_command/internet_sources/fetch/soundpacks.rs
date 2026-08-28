use serde::Serialize;

const SCHEMA: &str = "nextengine.experimental-soundpacks-glass-recordings-identity.v1";
const SOURCE_URL: &str = "https://soundpacks.com/free-sound-packs/glass-recordings/";
const ARCHIVE_URL: &str =
    "https://www.mediafire.com/file/nxuj8iiakqecpnu/Glass_Recordings_by_kaffekrus.rar/file";

#[derive(Serialize)]
struct PackIdentity {
    schema: &'static str,
    source_url: &'static str,
    creator: &'static str,
    title: &'static str,
    published_at: &'static str,
    modified_at: &'static str,
    description: [&'static str; 3],
    sample_count: u16,
    ambient_sound_count: u16,
    glass_hit_count: u16,
    format: &'static str,
    archive_url: &'static str,
    archive_display_size: &'static str,
}

pub(super) fn normalize_glass_recordings_identity(
    url: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, String> {
    if url != SOURCE_URL {
        return Err(
            "SoundPacks Glass Recordings normalization requires its canonical URL".to_owned(),
        );
    }
    let page = std::str::from_utf8(bytes)
        .map_err(|error| format!("SoundPacks page must be UTF-8: {error}"))?;
    let compacted = page.split_ascii_whitespace().collect::<Vec<_>>().join(" ");
    for marker in [
        "<title>Glass Recordings - SoundPacks.com</title>",
        "<link rel=\"canonical\" href=\"https://soundpacks.com/free-sound-packs/glass-recordings/\" />",
        "<meta property=\"article:published_time\" content=\"2024-01-12T04:52:04+00:00\" />",
        "<meta property=\"article:modified_time\" content=\"2024-10-30T05:51:24+00:00\" />",
        "<meta name=\"author\" content=\"kaffekrus\" />",
        "Made by <a href=\"https://soundpacks.com/entity/kaffekrus/\" rel=\"author\">kaffekrus</a>",
        "Glass Recordings is kaffekrus&#8217; second free sample pack, which focuses on field recordings and organic sounds.",
        "recordings derived from his apartment windows, mirrors, drinking glasses, vases, and other stuff around the house.",
        "The sample pack includes 11 ambient sounds and 28 glass hits",
        "href=\"//www.mediafire.com/?nxuj8iiakqecpnu\"",
        "33.53MB",
        "href=\"https://soundpacks.com/format/wav/\" rel=\"tag\">WAV</a>",
    ] {
        if !page.contains(marker) {
            return Err(format!(
                "SoundPacks Glass Recordings page is missing frozen identity marker {marker:?}"
            ));
        }
    }
    if !compacted.contains("39 Samples") {
        return Err(
            "SoundPacks Glass Recordings page is missing its frozen 39-sample identity".to_owned(),
        );
    }
    let identity = PackIdentity {
        schema: SCHEMA,
        source_url: SOURCE_URL,
        creator: "kaffekrus",
        title: "Glass Recordings",
        published_at: "2024-01-12T04:52:04Z",
        modified_at: "2024-10-30T05:51:24Z",
        description: [
            "field recordings and organic sounds",
            "apartment windows, mirrors, drinking glasses, vases, and other household objects",
            "11 ambient sounds and 28 glass hits",
        ],
        sample_count: 39,
        ambient_sound_count: 11,
        glass_hit_count: 28,
        format: "WAV",
        archive_url: ARCHIVE_URL,
        archive_display_size: "33.53MB",
    };
    let mut output = serde_json::to_vec_pretty(&identity).map_err(|error| error.to_string())?;
    output.push(b'\n');
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unrelated_recommendations_do_not_change_identity() {
        let first = fixture("suggestion-a");
        let second = fixture("suggestion-b");
        assert_eq!(
            normalize_glass_recordings_identity(SOURCE_URL, first.as_bytes())
                .expect("first identity"),
            normalize_glass_recordings_identity(SOURCE_URL, second.as_bytes())
                .expect("second identity")
        );
    }

    #[test]
    fn material_or_archive_drift_fails_closed() {
        let raw = fixture("suggestion").replace("28 glass hits", "28 metal hits");
        assert!(normalize_glass_recordings_identity(SOURCE_URL, raw.as_bytes()).is_err());
        let raw = fixture("suggestion").replace("nxuj8iiakqecpnu", "other");
        assert!(normalize_glass_recordings_identity(SOURCE_URL, raw.as_bytes()).is_err());
    }

    fn fixture(recommendation: &str) -> String {
        format!(
            r#"<html><head><title>Glass Recordings - SoundPacks.com</title>
<link rel="canonical" href="https://soundpacks.com/free-sound-packs/glass-recordings/" />
<meta property="article:published_time" content="2024-01-12T04:52:04+00:00" />
<meta property="article:modified_time" content="2024-10-30T05:51:24+00:00" />
<meta name="author" content="kaffekrus" /></head><body>
Made by <a href="https://soundpacks.com/entity/kaffekrus/" rel="author">kaffekrus</a>
Glass Recordings is kaffekrus&#8217; second free sample pack, which focuses on field recordings and organic sounds.
recordings derived from his apartment windows, mirrors, drinking glasses, vases, and other stuff around the house.
The sample pack includes 11 ambient sounds and 28 glass hits
<a href="//www.mediafire.com/?nxuj8iiakqecpnu">download</a>
33.53MB
39
							Samples
<a href="https://soundpacks.com/format/wav/" rel="tag">WAV</a>{recommendation}</body></html>"#
        )
    }
}
