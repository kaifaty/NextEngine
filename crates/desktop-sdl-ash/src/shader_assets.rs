const SPIRV_MAGIC: u32 = 0x0723_0203;

const B0_VERTEX_SHADER_BYTES: &[u8] = include_bytes!("../shaders/b0_textured.vert.spv");
const B0_FRAGMENT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/b0_textured.frag.spv");
const B0_NO_SHADOW_FRAGMENT_SHADER_BYTES: &[u8] =
    include_bytes!("../shaders/b0_textured_no_shadow.frag.spv");
const SHADOW_VERTEX_SHADER_BYTES: &[u8] = include_bytes!("../shaders/shadow_depth.vert.spv");
const UI_VERTEX_SHADER_BYTES: &[u8] = include_bytes!("../shaders/ui_overlay.vert.spv");
const UI_FRAGMENT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/ui_overlay.frag.spv");
const SKY_VERTEX_SHADER_BYTES: &[u8] = include_bytes!("../shaders/sky_gradient.vert.spv");
const SKY_FRAGMENT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/sky_gradient.frag.spv");

pub(super) const B0_SHADER_MANIFEST: &str = include_str!("../shaders/manifest.json");

pub(super) struct B0ShaderModules {
    pub(super) vertex: Vec<u32>,
    pub(super) fragment: Vec<u32>,
}

pub(super) fn b0_shader_modules() -> Result<B0ShaderModules, &'static str> {
    let expected_interface = format!(
        "\"interface_contract_sha256\": \"{}\"",
        next_contracts::render_content::b0_shader_interface_manifest_sha256().to_hex()
    );
    if !B0_SHADER_MANIFEST.contains("\"schema_version\": 1")
        || !B0_SHADER_MANIFEST.contains("\"suite\": \"b0_textured\"")
        || !B0_SHADER_MANIFEST.contains(&expected_interface)
    {
        return Err("embedded shader manifest is invalid");
    }
    Ok(B0ShaderModules {
        vertex: decode_spirv(B0_VERTEX_SHADER_BYTES)?,
        fragment: decode_spirv(B0_FRAGMENT_SHADER_BYTES)?,
    })
}

pub(super) fn b0_no_shadow_shader_modules() -> Result<B0ShaderModules, &'static str> {
    if !B0_SHADER_MANIFEST.contains("\"fallback_suite\": \"b0_textured_no_shadow\"") {
        return Err("embedded no-shadow shader manifest is invalid");
    }
    Ok(B0ShaderModules {
        vertex: decode_spirv(B0_VERTEX_SHADER_BYTES)?,
        fragment: decode_spirv(B0_NO_SHADOW_FRAGMENT_SHADER_BYTES)?,
    })
}

pub(super) fn shadow_vertex_shader_module() -> Result<Vec<u32>, &'static str> {
    if !B0_SHADER_MANIFEST.contains("\"shadow_suite\": \"shadow_depth\"") {
        return Err("embedded shadow shader manifest is invalid");
    }
    decode_spirv(SHADOW_VERTEX_SHADER_BYTES)
}

pub(super) fn ui_shader_modules() -> Result<B0ShaderModules, &'static str> {
    if !B0_SHADER_MANIFEST.contains("\"ui_suite\": \"ui_overlay\"") {
        return Err("embedded UI shader manifest is invalid");
    }
    Ok(B0ShaderModules {
        vertex: decode_spirv(UI_VERTEX_SHADER_BYTES)?,
        fragment: decode_spirv(UI_FRAGMENT_SHADER_BYTES)?,
    })
}

pub(super) fn sky_shader_modules() -> Result<B0ShaderModules, &'static str> {
    if !B0_SHADER_MANIFEST.contains("\"sky_suite\": \"sky_gradient\"") {
        return Err("embedded sky shader manifest is invalid");
    }
    Ok(B0ShaderModules {
        vertex: decode_spirv(SKY_VERTEX_SHADER_BYTES)?,
        fragment: decode_spirv(SKY_FRAGMENT_SHADER_BYTES)?,
    })
}

fn decode_spirv(bytes: &[u8]) -> Result<Vec<u32>, &'static str> {
    if bytes.len() < 20 || !bytes.len().is_multiple_of(4) {
        return Err("embedded SPIR-V module has an invalid byte length");
    }
    let words = bytes
        .chunks_exact(4)
        .map(|word| u32::from_le_bytes([word[0], word[1], word[2], word[3]]))
        .collect::<Vec<_>>();
    if words.first().copied() != Some(SPIRV_MAGIC) {
        return Err("embedded SPIR-V module has an invalid magic word");
    }
    Ok(words)
}

#[cfg(test)]
mod tests {
    use next_contracts::canonical::sha256;

    use super::*;

    #[test]
    fn checked_in_modules_match_the_offline_manifest() {
        let modules = b0_shader_modules().expect("checked-in modules decode");
        assert_eq!(modules.vertex[0], SPIRV_MAGIC);
        assert_eq!(modules.fragment[0], SPIRV_MAGIC);
        assert_eq!(
            hex(sha256(B0_VERTEX_SHADER_BYTES)),
            "6a08d7aa1c81e41a537a28cf703562d233114682a40d11c40b66ee30301dad7d"
        );
        assert_eq!(
            hex(sha256(B0_FRAGMENT_SHADER_BYTES)),
            "84765392e08061e1bed7fd1d81983c374aaa187622a1a20968672ff03f354d6a"
        );
        assert_eq!(
            hex(sha256(B0_NO_SHADOW_FRAGMENT_SHADER_BYTES)),
            "2cbe4fbc0e049a2238064adca0be8143a7f5ec6ab45bd4006dddfdd56fc4a97a"
        );
        assert_eq!(
            hex(sha256(SHADOW_VERTEX_SHADER_BYTES)),
            "1566638f72144e2931f5246a41b1f9fa927e84ce4f67f859227db042e78c3b65"
        );
        assert_eq!(
            hex(sha256(UI_VERTEX_SHADER_BYTES)),
            "9d28209b9d413a8bf91ee5e09817330df8bd19df79c1075a68c94f2bf405e19a"
        );
        assert_eq!(
            hex(sha256(UI_FRAGMENT_SHADER_BYTES)),
            "c03cd67fa05eeb1fea77246aa406ad68117583e3f207c9960687eb64d2531cda"
        );
        assert_eq!(
            hex(sha256(SKY_VERTEX_SHADER_BYTES)),
            "5624b656b86d1ba74615dc79f9d477cb2a30cd83f1e108013e9cefc24897261d"
        );
        assert_eq!(
            hex(sha256(SKY_FRAGMENT_SHADER_BYTES)),
            "b0b6682b742f4486ce03b46802e71f34003ac2bd026f6fba55b27c6170f4dd00"
        );
        assert!(B0_SHADER_MANIFEST.contains("\"schema_version\": 1"));
        assert!(B0_SHADER_MANIFEST.contains(
            "\"interface_contract_sha256\": \
             \"204ed27a6ed7535d094a8ad9d4dd6cc0ad8794f6664c6bf402154006f400c10c\""
        ));
        assert!(!B0_SHADER_MANIFEST.contains("\"runtime_compilation\""));
    }

    fn hex(bytes: [u8; 32]) -> String {
        const DIGITS: &[u8; 16] = b"0123456789abcdef";
        let mut value = String::with_capacity(64);
        for byte in bytes {
            value.push(char::from(DIGITS[usize::from(byte >> 4)]));
            value.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
        }
        value
    }
}
