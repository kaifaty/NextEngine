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
const FLUID_SPLAT_VERTEX_SHADER_BYTES: &[u8] = include_bytes!("../shaders/fluid_splat.vert.spv");
const FLUID_SPLAT_FRAGMENT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/fluid_splat.frag.spv");
const FLUID_SCREEN_VERTEX_SHADER_BYTES: &[u8] = include_bytes!("../shaders/fluid_screen.vert.spv");
const FLUID_FILTER_FRAGMENT_SHADER_BYTES: &[u8] =
    include_bytes!("../shaders/fluid_filter.frag.spv");
const FLUID_COMPOSITE_FRAGMENT_SHADER_BYTES: &[u8] =
    include_bytes!("../shaders/fluid_composite.frag.spv");
const FLUID_THICKNESS_FRAGMENT_SHADER_BYTES: &[u8] =
    include_bytes!("../shaders/fluid_thickness.frag.spv");
const FLUID_SPRAY_VERTEX_SHADER_BYTES: &[u8] = include_bytes!("../shaders/fluid_spray.vert.spv");
const FLUID_SPRAY_FRAGMENT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/fluid_spray.frag.spv");
const WATER_SURFACE_VERTEX_SHADER_BYTES: &[u8] =
    include_bytes!("../shaders/water_surface.vert.spv");
const WATER_SURFACE_FRAGMENT_SHADER_BYTES: &[u8] =
    include_bytes!("../shaders/water_surface.frag.spv");
const WATER_SCENE_FRAGMENT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/water_scene.frag.spv");
const REFLECTION_FRAGMENT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/b0_reflect.frag.spv");

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

/// ADR-102 presentation-only particle surface suite: sphere splat with two
/// colour outputs, fullscreen vertex, separable smoothing and composite.
pub(super) struct FluidShaderModules {
    pub(super) splat_vertex: Vec<u32>,
    pub(super) splat_fragment: Vec<u32>,
    pub(super) screen_vertex: Vec<u32>,
    pub(super) filter_fragment: Vec<u32>,
    pub(super) thickness_fragment: Vec<u32>,
    pub(super) composite_fragment: Vec<u32>,
    pub(super) spray_vertex: Vec<u32>,
    pub(super) spray_fragment: Vec<u32>,
}

pub(super) fn fluid_shader_modules() -> Result<FluidShaderModules, &'static str> {
    if !B0_SHADER_MANIFEST.contains("\"fluid_suite\": \"fluid_surface\"") {
        return Err("embedded fluid shader manifest is invalid");
    }
    Ok(FluidShaderModules {
        splat_vertex: decode_spirv(FLUID_SPLAT_VERTEX_SHADER_BYTES)?,
        splat_fragment: decode_spirv(FLUID_SPLAT_FRAGMENT_SHADER_BYTES)?,
        screen_vertex: decode_spirv(FLUID_SCREEN_VERTEX_SHADER_BYTES)?,
        filter_fragment: decode_spirv(FLUID_FILTER_FRAGMENT_SHADER_BYTES)?,
        thickness_fragment: decode_spirv(FLUID_THICKNESS_FRAGMENT_SHADER_BYTES)?,
        composite_fragment: decode_spirv(FLUID_COMPOSITE_FRAGMENT_SHADER_BYTES)?,
        spray_vertex: decode_spirv(FLUID_SPRAY_VERTEX_SHADER_BYTES)?,
        spray_fragment: decode_spirv(FLUID_SPRAY_FRAGMENT_SHADER_BYTES)?,
    })
}

/// Water look L1 (plan `continuum-water/12`): the material suite of
/// `WaterSurface` dynamic rings on the B0 interface.
pub(super) fn water_surface_shader_modules() -> Result<B0ShaderModules, &'static str> {
    if !B0_SHADER_MANIFEST.contains("\"water_suite\": \"water_surface\"") {
        return Err("embedded water surface shader manifest is invalid");
    }
    Ok(B0ShaderModules {
        vertex: decode_spirv(WATER_SURFACE_VERTEX_SHADER_BYTES)?,
        fragment: decode_spirv(WATER_SURFACE_FRAGMENT_SHADER_BYTES)?,
    })
}

/// Water look L2 + L3 (plan `continuum-water/13`): the water pass suite
/// (the B0 vertex program with the scene-sampling water fragment).
pub(super) fn water_scene_shader_modules() -> Result<B0ShaderModules, &'static str> {
    if !B0_SHADER_MANIFEST.contains("\"water_scene_suite\": \"water_scene\"") {
        return Err("embedded water scene shader manifest is invalid");
    }
    Ok(B0ShaderModules {
        vertex: decode_spirv(WATER_SURFACE_VERTEX_SHADER_BYTES)?,
        fragment: decode_spirv(WATER_SCENE_FRAGMENT_SHADER_BYTES)?,
    })
}

/// Water look L5 (plan `continuum-water/15`): the mirrored reflection pass
/// suite (the B0 vertex program with the plane-clipped B0 fragment).
pub(super) fn reflection_shader_modules() -> Result<B0ShaderModules, &'static str> {
    if !B0_SHADER_MANIFEST.contains("\"reflection_suite\": \"b0_reflect\"") {
        return Err("embedded reflection shader manifest is invalid");
    }
    Ok(B0ShaderModules {
        vertex: decode_spirv(B0_VERTEX_SHADER_BYTES)?,
        fragment: decode_spirv(REFLECTION_FRAGMENT_SHADER_BYTES)?,
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
        assert_eq!(
            hex(sha256(FLUID_SPLAT_VERTEX_SHADER_BYTES)),
            "45ab330e163d707bf1ec7b4be4443d4e66bb2a3d37a9240dcba1a0e20fd9f1e1"
        );
        assert_eq!(
            hex(sha256(FLUID_SPLAT_FRAGMENT_SHADER_BYTES)),
            "9e88b4a6d3173f454fe91d5c23f1d1829d014ab7fc7548fe8f274d72dc02b895"
        );
        assert_eq!(
            hex(sha256(FLUID_SCREEN_VERTEX_SHADER_BYTES)),
            "e01d54a40327a65a5c888c1b49b001eefce41cf9a8a9c19b20f3001dc7977446"
        );
        assert_eq!(
            hex(sha256(FLUID_FILTER_FRAGMENT_SHADER_BYTES)),
            "d518618b9082efce226e4ed3a5792f5618073e2dc09e05bdac48c56ac50c0aac"
        );
        assert_eq!(
            hex(sha256(FLUID_THICKNESS_FRAGMENT_SHADER_BYTES)),
            "c200a690a50548331edf9b36f686417d524ea8bce1116bd4e098bfd600d61a62"
        );
        assert_eq!(
            hex(sha256(FLUID_COMPOSITE_FRAGMENT_SHADER_BYTES)),
            "b12a24602f85c57882a5edb13b8bbe278ce5d1008636a0015ddb0ddb6c3d0752"
        );
        assert_eq!(
            hex(sha256(FLUID_SPRAY_VERTEX_SHADER_BYTES)),
            "e4094f109bb6e0b4bdc5824a1ee1237061c7bd82cb84898e7679c521124a6c9b"
        );
        assert_eq!(
            hex(sha256(FLUID_SPRAY_FRAGMENT_SHADER_BYTES)),
            "58468bada0f2b887ac22621119433ef90ea38dee1d3c149a3b23bb0e495fcaf0"
        );
        assert_eq!(
            hex(sha256(WATER_SURFACE_VERTEX_SHADER_BYTES)),
            "c0fb4c8395fcab8c7233f433eed0aaf93b3dcdb09dd16818c51179694673650f"
        );
        assert_eq!(
            hex(sha256(WATER_SURFACE_FRAGMENT_SHADER_BYTES)),
            "902a4f3664ae70034646c051a65fd3f82bc3752fd0e145f3326e13bd1bc6773f"
        );
        let water = water_surface_shader_modules().expect("checked-in water modules decode");
        assert_eq!(water.fragment[0], SPIRV_MAGIC);
        assert_eq!(
            hex(sha256(WATER_SCENE_FRAGMENT_SHADER_BYTES)),
            "de91969c2865e1eb7c35658a9a7c9fc3c0dc64e89745c48964ec23dfbb1aabcf"
        );
        let water_scene =
            water_scene_shader_modules().expect("checked-in water scene modules decode");
        assert_eq!(water_scene.fragment[0], SPIRV_MAGIC);
        let fluid = fluid_shader_modules().expect("checked-in fluid modules decode");
        assert_eq!(fluid.splat_vertex[0], SPIRV_MAGIC);
        assert_eq!(fluid.composite_fragment[0], SPIRV_MAGIC);
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
