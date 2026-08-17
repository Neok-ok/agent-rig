use std::fs;
use std::path::PathBuf;

use agent_rig::{
    increment16_scene, increment16_scene_json, load_mesh, parse_scene, run_increment16,
    step_physics, Shape, DEFAULT_DT, INCREMENT16_STEPS,
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

fn len3(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

#[test]
fn gltf_file_has_emissive_texture() {
    let path = pillar_gltf_path();
    assert!(path.is_file(), "meshes/pillar.gltf must be checked in");
    let txt = fs::read_to_string(&path).unwrap();
    let v: serde_json::Value = serde_json::from_str(&txt).expect("pillar.gltf is JSON");
    let mat = &v["materials"][0];
    let etex = &mat["emissiveTexture"];
    assert!(etex.is_object(), "materials[0] must have emissiveTexture");
    let idx = etex["index"].as_u64().expect("emissiveTexture.index");
    let tex = &v["textures"][idx as usize];
    let source = tex["source"].as_u64().expect("texture.source");
    let img = &v["images"][source as usize];
    let uri = img["uri"].as_str().expect("image.uri");
    assert!(
        uri.ends_with(".png") || uri.starts_with("data:"),
        "emissive map should be a PNG or data URI, got {uri}"
    );
    if !uri.starts_with("data:") {
        let resolved = path.parent().unwrap().join(uri);
        assert!(
            resolved.is_file(),
            "emissive map {uri} should exist at {}",
            resolved.display()
        );
    }
    let factor = mat["emissiveFactor"]
        .as_array()
        .expect("materials[0] must have emissiveFactor");
    assert_eq!(factor.len(), 3);
    let fr = factor[0].as_f64().unwrap();
    let fg = factor[1].as_f64().unwrap();
    let fb = factor[2].as_f64().unwrap();
    assert!(
        fr + fg + fb > 0.5,
        "emissiveFactor should be a visible glow, got [{fr}, {fg}, {fb}]"
    );
    // Keep increment-13/14 maps.
    assert!(
        mat["pbrMetallicRoughness"]
            .get("metallicRoughnessTexture")
            .is_some(),
        "must keep metallicRoughnessTexture"
    );
    assert!(mat.get("normalTexture").is_some(), "must keep normalTexture");
}

#[test]
fn loaded_mesh_sampled_emissive_is_not_fallback() {
    let mesh = load_mesh(&pillar_gltf_path()).expect("glTF should load");
    let gm = mesh
        .gltf_material
        .as_ref()
        .expect("loaded mesh must carry glTF material");
    assert!(
        gm.has_emissive_texture(),
        "loaded mesh must attach emissiveTexture"
    );
    let factor = gm.emissive_factor;
    assert!(
        len3(factor) > 0.5,
        "emissiveFactor should be a real glow, got {factor:?}"
    );

    // V=0.12 glow band (cyan texel * factor); V=0.37 black (texture, not factor-only).
    let e_glow = gm.sample_emissive(0.5, 0.12).expect("sample glow");
    let e_dark = gm.sample_emissive(0.5, 0.37).expect("sample dark");
    assert!(
        len3(e_glow) > 0.2,
        "glow UV must be non-zero sampled emissive, got {e_glow:?}"
    );
    assert!(
        len3(e_dark) < 0.05,
        "black UV must be near zero (texture used), got {e_dark:?}"
    );
    // No-texture fallback returns the factor at every UV. Dark ≈ 0 proves we sampled the map.
    assert!(
        (e_dark[0] - factor[0]).abs() > 0.1
            || (e_dark[1] - factor[1]).abs() > 0.1
            || (e_dark[2] - factor[2]).abs() > 0.1,
        "sampled dark emissive must not be the no-texture fallback {factor:?}, got {e_dark:?}"
    );
    // Cyan texture zeros R, so glow R is not the factor R either.
    assert!(
        (e_glow[0] - factor[0]).abs() > 0.05 || (e_glow[1] - e_dark[1]).abs() > 0.2,
        "glow sample should differ from factor-only / dark, glow={e_glow:?} factor={factor:?}"
    );
}

#[test]
fn increment16_keeps_courtyard() {
    let scene = parse_scene(increment16_scene_json()).expect("increment16 JSON should parse");
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
        "keep the increment-14 courtyard (bowl + rock + ball + pillar, directional + point)"
    );
    // Single still: no shove.
    for b in &scene.bodies {
        let v = b.linear_velocity;
        let speed = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        assert!(
            speed < 1e-6,
            "increment 16 is a still; body {} should have zero velocity, got {v:?}",
            b.id
        );
    }
}

#[test]
fn contact_involving_gltf_body_after_step() {
    let scene = increment16_scene();
    let gltf_id = gltf_body(&scene).id.clone();
    let dump = step_physics(&scene, INCREMENT16_STEPS, DEFAULT_DT).expect("physics");
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
fn increment16_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment16.sh");
    assert!(script.is_file(), "scripts/increment16.sh must exist");

    let out = PathBuf::from("target/test-increment16-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment16(&out, INCREMENT16_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment16");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    let scene = parse_scene(&scene_txt).expect("written scene.json parses");
    let body = gltf_body(&scene);
    assert!(
        matches!(&body.shape, Shape::Mesh { path, .. } if path.ends_with(".gltf") || path.ends_with(".glb"))
    );
    assert!(
        !scene_txt.contains("emissiveTexture") && !scene_txt.contains("emissive_map"),
        "scene-JSON must not be the emissive look"
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
