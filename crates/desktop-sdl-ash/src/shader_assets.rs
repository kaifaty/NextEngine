const SPIRV_MAGIC: u32 = 0x0723_0203;

const B0_VERTEX_SHADER_BYTES: &[u8] = include_bytes!("../shaders/b0_textured.vert.spv");
const B0_FRAGMENT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/b0_textured.frag.spv");
const B0_NO_SHADOW_FRAGMENT_SHADER_BYTES: &[u8] =
    include_bytes!("../shaders/b0_textured_no_shadow.frag.spv");
const SHADOW_VERTEX_SHADER_BYTES: &[u8] = include_bytes!("../shaders/shadow_depth.vert.spv");
const UI_VERTEX_SHADER_BYTES: &[u8] = include_bytes!("../shaders/ui_overlay.vert.spv");
const UI_FRAGMENT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/ui_overlay.frag.spv");
const SKY_VERTEX_SHADER_BYTES: &[u8] = include_bytes!("../shaders/sky_analytic.vert.spv");
const SKY_FRAGMENT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/sky_analytic.frag.spv");
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
const WATER_UNDER_FRAGMENT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/water_under.frag.spv");
const WATER_WET_FRAGMENT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/water_wet.frag.spv");
const REFLECTION_FRAGMENT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/b0_reflect.frag.spv");
const GBUFFER_VERTEX_SHADER_BYTES: &[u8] = include_bytes!("../shaders/gbuffer.vert.spv");
const GBUFFER_FRAGMENT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/gbuffer.frag.spv");
const TONEMAP_FRAGMENT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/tonemap.frag.spv");
const AO_FRAGMENT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/ao.frag.spv");
const AO_BLUR_FRAGMENT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/ao_blur.frag.spv");
const TAA_FRAGMENT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/taa.frag.spv");
const BLOOM_DOWN_FRAGMENT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/bloom_down.frag.spv");
const BLOOM_UP_FRAGMENT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/bloom_up.frag.spv");
const POST_FRAGMENT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/post.frag.spv");
const FOG_FRAGMENT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/fog.frag.spv");

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

/// Scene look L1 (plan `look/01`): the tone-map suite (the fullscreen
/// vertex program of the fluid suite with the exposure and ACES fragment).
pub(super) fn tonemap_shader_modules() -> Result<B0ShaderModules, &'static str> {
    if !B0_SHADER_MANIFEST.contains("\"tonemap_suite\": \"tonemap\"") {
        return Err("embedded tonemap shader manifest is invalid");
    }
    Ok(B0ShaderModules {
        vertex: decode_spirv(FLUID_SCREEN_VERTEX_SHADER_BYTES)?,
        fragment: decode_spirv(TONEMAP_FRAGMENT_SHADER_BYTES)?,
    })
}

/// Scene look L3 (plan `look/03`): the occlusion suite (the fullscreen
/// vertex program of the fluid suite with the GTAO fragment).
pub(super) fn ambient_occlusion_shader_modules() -> Result<B0ShaderModules, &'static str> {
    if !B0_SHADER_MANIFEST.contains("\"ao_suite\": \"ambient_occlusion\"") {
        return Err("embedded occlusion shader manifest is invalid");
    }
    Ok(B0ShaderModules {
        vertex: decode_spirv(FLUID_SCREEN_VERTEX_SHADER_BYTES)?,
        fragment: decode_spirv(AO_FRAGMENT_SHADER_BYTES)?,
    })
}

/// Scene look L3: the depth-aware blur of the occlusion suite.
pub(super) fn ambient_occlusion_blur_shader_modules() -> Result<B0ShaderModules, &'static str> {
    if !B0_SHADER_MANIFEST.contains("\"ao_suite\": \"ambient_occlusion\"") {
        return Err("embedded occlusion shader manifest is invalid");
    }
    Ok(B0ShaderModules {
        vertex: decode_spirv(FLUID_SCREEN_VERTEX_SHADER_BYTES)?,
        fragment: decode_spirv(AO_BLUR_FRAGMENT_SHADER_BYTES)?,
    })
}

/// Scene look L4 (plan `look/04`): the temporal resolve suite.
pub(super) fn temporal_aa_shader_modules() -> Result<B0ShaderModules, &'static str> {
    if !B0_SHADER_MANIFEST.contains("\"taa_suite\": \"temporal_aa\"") {
        return Err("embedded temporal resolve shader manifest is invalid");
    }
    Ok(B0ShaderModules {
        vertex: decode_spirv(FLUID_SCREEN_VERTEX_SHADER_BYTES)?,
        fragment: decode_spirv(TAA_FRAGMENT_SHADER_BYTES)?,
    })
}

/// Scene look L7 (plan `look/07`): the post chain's bloom downsample.
pub(super) fn bloom_down_shader_modules() -> Result<B0ShaderModules, &'static str> {
    if !B0_SHADER_MANIFEST.contains("\"post_suite\": \"post_chain\"") {
        return Err("embedded post chain shader manifest is invalid");
    }
    Ok(B0ShaderModules {
        vertex: decode_spirv(FLUID_SCREEN_VERTEX_SHADER_BYTES)?,
        fragment: decode_spirv(BLOOM_DOWN_FRAGMENT_SHADER_BYTES)?,
    })
}

/// Scene look L7: the post chain's bloom upsample.
pub(super) fn bloom_up_shader_modules() -> Result<B0ShaderModules, &'static str> {
    if !B0_SHADER_MANIFEST.contains("\"post_suite\": \"post_chain\"") {
        return Err("embedded post chain shader manifest is invalid");
    }
    Ok(B0ShaderModules {
        vertex: decode_spirv(FLUID_SCREEN_VERTEX_SHADER_BYTES)?,
        fragment: decode_spirv(BLOOM_UP_FRAGMENT_SHADER_BYTES)?,
    })
}

/// Scene look L7 (revision 3): the post chain's half-resolution volume.
pub(super) fn fog_shader_modules() -> Result<B0ShaderModules, &'static str> {
    if !B0_SHADER_MANIFEST.contains("\"post_suite\": \"post_chain\"") {
        return Err("embedded post chain shader manifest is invalid");
    }
    Ok(B0ShaderModules {
        vertex: decode_spirv(FLUID_SCREEN_VERTEX_SHADER_BYTES)?,
        fragment: decode_spirv(FOG_FRAGMENT_SHADER_BYTES)?,
    })
}

/// Scene look L7: the post chain's composite (fog, bloom, tone, grade).
pub(super) fn post_shader_modules() -> Result<B0ShaderModules, &'static str> {
    if !B0_SHADER_MANIFEST.contains("\"post_suite\": \"post_chain\"") {
        return Err("embedded post chain shader manifest is invalid");
    }
    Ok(B0ShaderModules {
        vertex: decode_spirv(FLUID_SCREEN_VERTEX_SHADER_BYTES)?,
        fragment: decode_spirv(POST_FRAGMENT_SHADER_BYTES)?,
    })
}

pub(super) fn sky_shader_modules() -> Result<B0ShaderModules, &'static str> {
    if !B0_SHADER_MANIFEST.contains("\"sky_suite\": \"sky_analytic\"") {
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

/// Plan `continuum-water/33`: the underwater suite (the fullscreen vertex
/// program of the particle pass with the water-between fragment).
pub(super) fn water_under_shader_modules() -> Result<B0ShaderModules, &'static str> {
    if !B0_SHADER_MANIFEST.contains("\"water_under_suite\": \"water_under\"") {
        return Err("embedded underwater shader manifest is invalid");
    }
    Ok(B0ShaderModules {
        vertex: decode_spirv(FLUID_SCREEN_VERTEX_SHADER_BYTES)?,
        fragment: decode_spirv(WATER_UNDER_FRAGMENT_SHADER_BYTES)?,
    })
}

/// Plan `continuum-water/35`: the wet band suite (the fullscreen vertex
/// program with the wet band fragment).
pub(super) fn water_wet_shader_modules() -> Result<B0ShaderModules, &'static str> {
    if !B0_SHADER_MANIFEST.contains("\"water_wet_suite\": \"water_wet\"") {
        return Err("embedded wet band shader manifest is invalid");
    }
    Ok(B0ShaderModules {
        vertex: decode_spirv(FLUID_SCREEN_VERTEX_SHADER_BYTES)?,
        fragment: decode_spirv(WATER_WET_FRAGMENT_SHADER_BYTES)?,
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

/// Water look L8 (plan `continuum-water/18`): the G-buffer suite (thin
/// G-buffer, motion vectors and linear depth from the plan's draws).
pub(super) fn gbuffer_shader_modules() -> Result<B0ShaderModules, &'static str> {
    if !B0_SHADER_MANIFEST.contains("\"gbuffer_suite\": \"gbuffer\"") {
        return Err("embedded G-buffer shader manifest is invalid");
    }
    Ok(B0ShaderModules {
        vertex: decode_spirv(GBUFFER_VERTEX_SHADER_BYTES)?,
        fragment: decode_spirv(GBUFFER_FRAGMENT_SHADER_BYTES)?,
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
            "48a69bf6b528e2dda1413116bd12970b493346fac9a0dd65f612a7a214e2612c"
        );
        assert_eq!(
            hex(sha256(B0_FRAGMENT_SHADER_BYTES)),
            "2633ba6061234bd0c811a3ba365aece32c7d236cfd81886cb86d5d91e696a9e0"
        );
        assert_eq!(
            hex(sha256(B0_NO_SHADOW_FRAGMENT_SHADER_BYTES)),
            "e650e9822c81b874d915aaabe9f062d6b89713152c17dd3f48530eb374481c62"
        );
        assert_eq!(
            hex(sha256(SHADOW_VERTEX_SHADER_BYTES)),
            "80d064f1276c6588b3c0abf2f353c31682626dbe1bb48e41d645fc2a2b94a8e4"
        );
        assert_eq!(
            hex(sha256(UI_VERTEX_SHADER_BYTES)),
            "0ed38332dc6999eef5f6929ad944ebc59fc550f3b7393954d0f149b6b04e8ab1"
        );
        assert_eq!(
            hex(sha256(UI_FRAGMENT_SHADER_BYTES)),
            "aced1675e3a4771e1e17b1d9e260b75f39b6c07ebf737bf4c24856e0bf4fe645"
        );
        assert_eq!(
            hex(sha256(SKY_VERTEX_SHADER_BYTES)),
            "e679fcdb1a4f0ff935c38cb65d9e7ff0a0176c5b369f1652e6337835f33ad2bf"
        );
        assert_eq!(
            hex(sha256(SKY_FRAGMENT_SHADER_BYTES)),
            "fe9a354695b2145ad5b0b6ad2cea9cec82908643fd872178dc69d4a37c01c067"
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
            "1982e04d8be3e9c8202c0dafd0b8f17b9cf90f42bcc6fddcbc762ce946d6ee50"
        );
        let water = water_surface_shader_modules().expect("checked-in water modules decode");
        assert_eq!(water.fragment[0], SPIRV_MAGIC);
        assert_eq!(
            hex(sha256(WATER_SCENE_FRAGMENT_SHADER_BYTES)),
            "81dca08d13fcb71c4bff3246b30af0bada60bcb90d5210359f00f733b469413e"
        );
        let water_scene =
            water_scene_shader_modules().expect("checked-in water scene modules decode");
        assert_eq!(water_scene.fragment[0], SPIRV_MAGIC);
        assert_eq!(
            hex(sha256(WATER_UNDER_FRAGMENT_SHADER_BYTES)),
            "55eee47de2f313e46af77a8efde8070ac712139f6f345dd0fa1e6ebd5bd05202"
        );
        let water_under =
            water_under_shader_modules().expect("checked-in underwater modules decode");
        assert_eq!(water_under.fragment[0], SPIRV_MAGIC);
        assert_eq!(
            hex(sha256(WATER_WET_FRAGMENT_SHADER_BYTES)),
            "3f15ed6a6781ca6405be8d42fd723587b01326e5cbabf38a2e8f274224443ba9"
        );
        let water_wet = water_wet_shader_modules().expect("checked-in wet band modules decode");
        assert_eq!(water_wet.fragment[0], SPIRV_MAGIC);
        assert_eq!(
            hex(sha256(GBUFFER_VERTEX_SHADER_BYTES)),
            "265146836530325496863b164ad1e13b787d1959ebac77b52021ac44eab2f9b8"
        );
        assert_eq!(
            hex(sha256(GBUFFER_FRAGMENT_SHADER_BYTES)),
            "d5a8cb0e1eb16943697492d4c9b12eaedfc515561fef3b0f64fe5650c6dc2f83"
        );
        assert_eq!(
            hex(sha256(TONEMAP_FRAGMENT_SHADER_BYTES)),
            "92adca13b3013e18015c36bf4cf7dc9b8040528ffaebfc4444674117398128a7"
        );
        assert_eq!(
            hex(sha256(AO_FRAGMENT_SHADER_BYTES)),
            "15b9856b7f42e3f299235a249e9810fc0ea4eaa938aeb60a6ca9abfc5d710a5e"
        );
        assert_eq!(
            hex(sha256(AO_BLUR_FRAGMENT_SHADER_BYTES)),
            "1c103c73fc335cbce62a1203fa35bee94d1df11b7fb5eb30300e04a00f1f930a"
        );
        assert_eq!(
            hex(sha256(TAA_FRAGMENT_SHADER_BYTES)),
            "a4d50eb484f8d78e851fcf8058be7869b46a6784a19f13fa29f23e3624a2279d"
        );
        let temporal = temporal_aa_shader_modules().expect("checked-in temporal modules decode");
        assert_eq!(temporal.fragment[0], SPIRV_MAGIC);
        assert_eq!(
            hex(sha256(BLOOM_DOWN_FRAGMENT_SHADER_BYTES)),
            "425726979231916b09c096e3ede058a5e88eb75611028df1a993be7a9645f715"
        );
        assert_eq!(
            hex(sha256(BLOOM_UP_FRAGMENT_SHADER_BYTES)),
            "c6af12139d53c76da5dcde1fe01069d272014072b52709944e51aa2a02c635fe"
        );
        assert_eq!(
            hex(sha256(POST_FRAGMENT_SHADER_BYTES)),
            "d86ffbbeff611b80eedda498af41db626b103cb23736b6655391498e5ca715a5"
        );
        let bloom_down = bloom_down_shader_modules().expect("checked-in bloom modules decode");
        assert_eq!(bloom_down.fragment[0], SPIRV_MAGIC);
        let bloom_up = bloom_up_shader_modules().expect("checked-in bloom modules decode");
        assert_eq!(bloom_up.fragment[0], SPIRV_MAGIC);
        let post = post_shader_modules().expect("checked-in post chain modules decode");
        assert_eq!(post.fragment[0], SPIRV_MAGIC);
        assert_eq!(
            hex(sha256(FOG_FRAGMENT_SHADER_BYTES)),
            "2ec014b0ee0ec186f1416dfb5b07c0e429613f6c09f27df35c65d59e7bc4e291"
        );
        let fog = fog_shader_modules().expect("checked-in fog modules decode");
        assert_eq!(fog.fragment[0], SPIRV_MAGIC);
        let occlusion =
            ambient_occlusion_shader_modules().expect("checked-in occlusion modules decode");
        assert_eq!(occlusion.fragment[0], SPIRV_MAGIC);
        let blur = ambient_occlusion_blur_shader_modules().expect("checked-in blur modules decode");
        assert_eq!(blur.fragment[0], SPIRV_MAGIC);
        let tonemap = tonemap_shader_modules().expect("checked-in tonemap modules decode");
        assert_eq!(tonemap.fragment[0], SPIRV_MAGIC);
        let gbuffer = gbuffer_shader_modules().expect("checked-in G-buffer modules decode");
        assert_eq!(gbuffer.vertex[0], SPIRV_MAGIC);
        assert_eq!(gbuffer.fragment[0], SPIRV_MAGIC);
        let fluid = fluid_shader_modules().expect("checked-in fluid modules decode");
        assert_eq!(fluid.splat_vertex[0], SPIRV_MAGIC);
        assert_eq!(fluid.composite_fragment[0], SPIRV_MAGIC);
        assert!(B0_SHADER_MANIFEST.contains("\"schema_version\": 1"));
        assert!(B0_SHADER_MANIFEST.contains(
            "\"interface_contract_sha256\": \
             \"0c1cece9abc2942f24dc8321255cf93de23b57048121fe200e12da72e228eacd\""
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
