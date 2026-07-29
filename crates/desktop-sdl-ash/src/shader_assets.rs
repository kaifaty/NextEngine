const SPIRV_MAGIC: u32 = 0x0723_0203;

const B0_VERTEX_SHADER_BYTES: &[u8] = include_bytes!("../shaders/b0_textured.vert.spv");
const B0_FRAGMENT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/b0_textured.frag.spv");

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
            "9957f29a421307bd2e93eee6594438291f06904457cb2098076ca7106daa345f"
        );
        assert_eq!(
            hex(sha256(B0_FRAGMENT_SHADER_BYTES)),
            "21ca7f029466a2c23f7d0dff4da99dfca01ece2a7512645c980e70ed8b6f84ee"
        );
        assert!(B0_SHADER_MANIFEST.contains("\"schema_version\": 1"));
        assert!(B0_SHADER_MANIFEST.contains(
            "\"interface_contract_sha256\": \
             \"8091123413e9b20e6410d2f841d9db55113aa9069c4fe8cd491d36ecb2dd80c0\""
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
