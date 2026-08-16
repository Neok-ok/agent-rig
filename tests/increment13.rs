use std::fs;
use std::path::PathBuf;

use agent_rig::{
    increment13_scene, increment13_scene_json, load_mesh, parse_scene, run_increment13,
    step_physics, Shape, DEFAULT_DT, INCREMENT13_STEPS,
};

fn gltf_body(scene: &agent_rig::Scene) -> &agent_rig::Body {
    scene
        .bodies
        .iter()
        .find(|b| match &b.shape {
            Shape::Mesh { path, .. } => path.ends_with(".gltf") || path.ends_with(".glb"),
            _ => false,
        })
        .expect("scene must contain a glTF/GLB mesh body")
}

fn pillar_gltf_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("meshes/pillar.gltf")
}

#[test]
fn gltf_file_has_metallic_roughness_texture() {
    let path = pillar_gltf_path();
    assert!(path.is_file(), "meshes/pillar.gltf must be checked in");
    let txt = fs::read_to_string(&path).unwrap();
    let v: serde_json::Value = serde_json::from_str(&txt).expect("pillar.gltf is JSON");
    let pbr = &v["materials"][0]["pbrMetallicRoughness"];
    assert!(
        pbr.get("metallicRoughnessTexture").is_some(),
        "pbrMetallicRoughness must have metallicRoughnessTexture"
    );
    let idx = pbr["metallicRoughnessTexture"]["index"]
        .as_u64()
        .expect("metallicRoughnessTexture.index");
    let tex = &v["textures"][idx as usize];
    let source = tex["source"].as_u64().expect("texture.source");
    let img = &v["images"][source as usize];
    let uri = img["uri"].as_str().expect("image.uri");
    assert!(
        uri.ends_with(".png") || uri.starts_with("data:"),
        "MR texture should be a PNG or data URI, got {uri}"
    );
    if !uri.starts_with("data:") {
        let resolved = path.parent().unwrap().join(uri);
        assert!(
            resolved.is_file(),
            "MR texture {uri} should exist at {}",
            resolved.display()
        );
    }
}

#[test]
fn loaded_mesh_uses_sampled_mr_not_scene_json() {
    let mesh = load_mesh(&pillar_gltf_path()).expect("glTF should load");
    let gm = mesh
        .gltf_material
        .as_ref()
        .expect("loaded mesh must carry glTF pbrMetallicRoughness");
    assert!(
        gm.has_metallic_roughness_texture(),
        "loaded mesh must attach metallicRoughnessTexture"
    );

    let scene = increment13_scene();
    let body = gltf_body(&scene);
    let json_m = body.material.metallic;
    let json_r = body.material.roughness;
    assert!(
        json_m.abs() < 1e-4 && (json_r - 0.85).abs() < 1e-3,
        "scene JSON pillar fallback should be dull (metallic 0, roughness 0.85), got m={json_m} r={json_r}"
    );

    // Horizontal bands: V=0.12 (band 0) smooth metal, V=0.37 (band 1) rough dielectric.
    let (m0, r0) = gm
        .sample_metallic_roughness(0.5, 0.12)
        .expect("sample metal band");
    let (m1, r1) = gm
        .sample_metallic_roughness(0.5, 0.37)
        .expect("sample dielectric band");
    assert!(
        (m0 - m1).abs() > 0.4 && (r0 - r1).abs() > 0.4,
        "two UV samples must differ (metal {m0}/{r0} vs dielectric {m1}/{r1})"
    );
    assert!(
        (m0 - json_m).abs() > 0.4 || (r0 - json_r).abs() > 0.4,
        "sampled MR must not be the scene-JSON constants {json_m}/{json_r} (got {m0}/{r0})"
    );
    assert!(
        (m1 - json_m).abs() > 0.02 || (r1 - json_r).abs() > 0.02,
        "second sample must not collapse to JSON constants (got {m1}/{r1})"
    );
    // Smooth-metal band: high metallic, low roughness.
    assert!(
        m0 > 0.7 && r0 < 0.3,
        "V=0.12 should be smooth metal, got metallic={m0} roughness={r0}"
    );
    // Rough-dielectric band: low metallic, high roughness.
    assert!(
        m1 < 0.2 && r1 > 0.7,
        "V=0.37 should be rough dielectric, got metallic={m1} roughness={r1}"
    );
}

#[test]
fn increment13_keeps_courtyard() {
    let scene = parse_scene(increment13_scene_json()).expect("increment13 JSON should parse");
    let has_bowl = scene
        .bodies
        .iter()
        .any(|b| matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("bowl")));
    let has_rock = scene
        .bodies
        .iter()
        .any(|b| matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("rock")));
    let has_ball = scene
        .bodies
        .iter()
        .any(|b| matches!(b.shape, Shape::Sphere { .. }));
    let has_pillar = gltf_body(&scene).id == "pillar";
    let has_dir = scene
        .lights
        .iter()
        .any(|l| matches!(l, agent_rig::Light::Directional { .. }));
    let has_point = scene
        .lights
        .iter()
        .any(|l| matches!(l, agent_rig::Light::Point { .. }));
    assert!(
        has_bowl && has_rock && has_ball && has_pillar && has_dir && has_point,
        "keep the increment-12 courtyard (bowl + rock + ball + pillar, directional + point)"
    );
}

#[test]
fn contact_involving_gltf_body_after_step() {
    let scene = increment13_scene();
    let gltf_id = gltf_body(&scene).id.clone();
    let dump = step_physics(&scene, INCREMENT13_STEPS, DEFAULT_DT).expect("physics");
    let hit = dump
        .contacts
        .iter()
        .any(|c| c.body_a == gltf_id || c.body_b == gltf_id);
    assert!(
        hit,
        "expected a contact involving the glTF body {gltf_id}, contacts={:?}",
        dump.contacts
    );
    let body = dump
        .bodies
        .iter()
        .find(|b| b.id == gltf_id)
        .expect("dump missing glTF body");
    assert!(
        body.collider == "convex_hull" || body.collider == "trimesh",
        "glTF body collider should be convex_hull or trimesh, got {}",
        body.collider
    );
}

#[test]
fn increment13_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment13.sh");
    assert!(script.is_file(), "scripts/increment13.sh must exist");

    let out = PathBuf::from("target/test-increment13-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment13(&out, INCREMENT13_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment13");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    let scene = parse_scene(&scene_txt).expect("written scene.json parses");
    let body = gltf_body(&scene);
    assert!(
        matches!(&body.shape, Shape::Mesh { path, .. } if path.ends_with(".gltf") || path.ends_with(".glb"))
    );

    let phys_txt = fs::read_to_string(&paths.physics).unwrap();
    let v: serde_json::Value = serde_json::from_str(&phys_txt).unwrap();
    assert!(v["bodies"].as_array().unwrap().len() >= 4);
    assert!(v["contacts"].is_array());
    let gltf_state = v["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == body.id)
        .expect("dump should record the glTF body");
    let col = gltf_state["collider"].as_str().unwrap_or("");
    assert!(
        col == "convex_hull" || col == "trimesh",
        "dump must record collider type for the glTF body, got {col}"
    );
    let contacts = v["contacts"].as_array().unwrap();
    let hit = contacts
        .iter()
        .any(|c| c["body_a"] == body.id || c["body_b"] == body.id);
    assert!(hit, "dump contacts should include the glTF body");
    let png_len = fs::metadata(&paths.frame).unwrap().len();
    assert!(png_len > 256, "png too small: {png_len}");
}
