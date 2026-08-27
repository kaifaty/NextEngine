use std::collections::BTreeSet;

use serde::Serialize;

const SCHEMA: &str = "nextengine.experimental-freesound-pack-identity.v1";
const MAX_SOUNDS: usize = 64;

#[derive(Serialize)]
struct PackIdentity<'a> {
    schema: &'static str,
    source_url: &'a str,
    author: &'a str,
    author_id: String,
    pack_id: &'a str,
    title: String,
    description: String,
    sounds: Vec<SoundIdentity>,
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Serialize)]
struct SoundIdentity {
    sound_id: String,
    title: String,
    duration_seconds: String,
    sample_rate_hz: String,
    lq_mp3_url: String,
    license_label: String,
}

pub(super) fn normalize_pack_identity(url: &str, bytes: &[u8]) -> Result<Vec<u8>, String> {
    let (author, pack_id) = parse_url(url)?;
    let page = std::str::from_utf8(bytes)
        .map_err(|error| format!("Freesound pack page must be UTF-8: {error}"))?;
    let title_prefix = "<title>Freesound - ";
    let title_suffix = format!(" by {author}</title>");
    let title_start = page
        .find(title_prefix)
        .ok_or_else(|| "Freesound pack page has no canonical title".to_owned())?
        + title_prefix.len();
    let title_end = page[title_start..]
        .find(&title_suffix)
        .map(|offset| title_start + offset)
        .ok_or_else(|| "Freesound pack title does not match its author".to_owned())?;
    let title = bounded_plain_text(&page[title_start..title_end], "Freesound pack title")?;

    let description_start_marker = "<p>This pack contains ";
    let description_start = page
        .find(description_start_marker)
        .ok_or_else(|| "Freesound pack page has no object description".to_owned())?;
    let description_end = page[description_start..]
        .find("</p>")
        .map(|offset| description_start + offset)
        .ok_or_else(|| "Freesound pack description is incomplete".to_owned())?;
    let description = bounded_plain_text(
        &page[description_start + 3..description_end],
        "Freesound pack description",
    )?;

    let blocks = page
        .split("class=\"bw-player\"")
        .skip(1)
        .collect::<Vec<_>>();
    if blocks.len() < 2 || blocks.len() > MAX_SOUNDS {
        return Err(format!(
            "Freesound pack must expose 2..={MAX_SOUNDS} sound cards"
        ));
    }
    let mut author_id = None;
    let mut sound_ids = BTreeSet::new();
    let mut sounds = Vec::with_capacity(blocks.len());
    for block in blocks {
        let sound_id = attribute(block, "data-sound-id")?;
        let username = attribute(block, "data-username")?;
        let user_id = attribute(block, "data-user-id")?;
        let sound_title = attribute(block, "data-title")?;
        let duration_seconds = attribute(block, "data-duration")?;
        let sample_rate_hz = attribute(block, "data-samplerate")?;
        let lq_mp3_url = attribute(block, "data-mp3")?;
        let license_label = attribute_after(block, "title=\"License: ")?;
        if username != author
            || !decimal(sound_id)
            || !decimal(user_id)
            || !decimal_float(duration_seconds)
            || !decimal_float(sample_rate_hz)
        {
            return Err("Freesound pack card identity is not canonical".to_owned());
        }
        let preview_bucket = sound_id
            .parse::<u64>()
            .map_err(|error| format!("parse Freesound sound id: {error}"))?
            / 1_000;
        let expected_preview = format!(
            "https://cdn.freesound.org/previews/{preview_bucket}/{sound_id}_{user_id}-lq.mp3"
        );
        if lq_mp3_url != expected_preview {
            return Err(format!(
                "Freesound sound {sound_id} has a non-canonical LQ preview URL"
            ));
        }
        if !sound_ids.insert(sound_id.to_owned()) {
            return Err(format!("Freesound pack repeats sound id {sound_id}"));
        }
        match &author_id {
            Some(expected) if expected != user_id => {
                return Err("Freesound pack cards disagree on author id".to_owned());
            }
            None => author_id = Some(user_id.to_owned()),
            Some(_) => {}
        }
        sounds.push(SoundIdentity {
            sound_id: sound_id.to_owned(),
            title: bounded_plain_text(sound_title, "Freesound sound title")?,
            duration_seconds: duration_seconds.to_owned(),
            sample_rate_hz: sample_rate_hz.to_owned(),
            lq_mp3_url: lq_mp3_url.to_owned(),
            license_label: bounded_plain_text(license_label, "Freesound license label")?,
        });
    }
    sounds.sort();
    let identity = PackIdentity {
        schema: SCHEMA,
        source_url: url,
        author,
        author_id: author_id.ok_or_else(|| "Freesound pack has no author id".to_owned())?,
        pack_id,
        title,
        description,
        sounds,
    };
    let mut output = serde_json::to_vec_pretty(&identity).map_err(|error| error.to_string())?;
    output.push(b'\n');
    Ok(output)
}

fn parse_url(url: &str) -> Result<(&str, &str), String> {
    let remainder = url
        .strip_prefix("https://freesound.org/people/")
        .and_then(|value| value.strip_suffix('/'))
        .ok_or_else(|| "Freesound pack normalization requires the canonical pack URL".to_owned())?;
    let (author, pack_id) = remainder
        .split_once("/packs/")
        .ok_or_else(|| "Freesound pack URL has no author/pack identity".to_owned())?;
    if author.is_empty()
        || author.len() > 64
        || !author
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        || !decimal(pack_id)
    {
        return Err("Freesound pack URL identity is not canonical".to_owned());
    }
    Ok((author, pack_id))
}

fn attribute<'a>(block: &'a str, name: &str) -> Result<&'a str, String> {
    attribute_after(block, &format!("{name}=\""))
}

fn attribute_after<'a>(block: &'a str, marker: &str) -> Result<&'a str, String> {
    let start = block
        .find(marker)
        .map(|offset| offset + marker.len())
        .ok_or_else(|| format!("Freesound card is missing {marker}"))?;
    let end = block[start..]
        .find('"')
        .map(|offset| start + offset)
        .ok_or_else(|| format!("Freesound card has an incomplete {marker}"))?;
    Ok(&block[start..end])
}

fn bounded_plain_text(value: &str, role: &str) -> Result<String, String> {
    if value.is_empty()
        || value.len() > 2_048
        || !value.is_ascii()
        || value.bytes().any(|byte| byte.is_ascii_control())
        || value.contains(['<', '>'])
    {
        return Err(format!("{role} is not bounded plain ASCII text"));
    }
    Ok(value.to_owned())
}

fn decimal(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn decimal_float(value: &str) -> bool {
    let mut point = false;
    !value.is_empty()
        && value.bytes().all(|byte| {
            if byte == b'.' && !point {
                point = true;
                true
            } else {
                byte.is_ascii_digit()
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const URL: &str = "https://freesound.org/people/ascap/packs/14905/";

    #[test]
    fn dynamic_fields_do_not_change_pack_identity() {
        let first = fixture("csrf-a", "79");
        let second = fixture("csrf-b", "999");
        let first = normalize_pack_identity(URL, first.as_bytes()).expect("first identity");
        let second = normalize_pack_identity(URL, second.as_bytes()).expect("second identity");
        assert_eq!(first, second);
        let json: serde_json::Value = serde_json::from_slice(&first).expect("identity JSON");
        assert_eq!(json["pack_id"], "14905");
        assert_eq!(json["sounds"].as_array().expect("sounds").len(), 2);
        assert_eq!(json["sounds"][0]["sound_id"], "242459");
    }

    #[test]
    fn duplicate_sound_identity_fails_closed() {
        let raw = fixture("csrf-a", "79").replace("242459", "242460");
        assert!(normalize_pack_identity(URL, raw.as_bytes()).is_err());
    }

    fn fixture(csrf: &str, downloads: &str) -> String {
        format!(
            r#"<html><head><title>Freesound - medium-pitched glass bowl by ascap</title></head>
<body><input value="{csrf}"><p>This pack contains the sounds of a medium-pitched glass bowl.</p>
<div class="bw-player" data-sound-id="242460" data-username="ascap" data-user-id="4420518"
data-mp3="https://cdn.freesound.org/previews/242/242460_4420518-lq.mp3"
data-title="wood hit medium glass bowl 1.mp3" data-duration="4.0" data-samplerate="44100.0"
data-num-downloads="{downloads}"></div><div title="License: Attribution NonCommercial"></div>
<div class="bw-player" data-sound-id="242459" data-username="ascap" data-user-id="4420518"
data-mp3="https://cdn.freesound.org/previews/242/242459_4420518-lq.mp3"
data-title="wood hit medium glass bowl 2.mp3" data-duration="4.0" data-samplerate="44100.0"></div>
<div title="License: Attribution NonCommercial"></div></body></html>"#
        )
    }
}
