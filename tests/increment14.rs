use std::fs;
use std::path::PathBuf;

use agent_rig::{
    increment14_scene, increment14_scene_json, load_mesh, parse_scene, run_increment14,
    step_physics, Shape, DEFAULT_DT, INCREMENT14_STEPS,
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

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn len(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}

fn angle_deg(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = (dot(a, b) / (len(a) * len(b) + 1e-12)).clamp(-1.0, 1.0);
    d.acos().to_degrees()
}

#[test]
fn gltf_file_has_normal_texture() {
    let path = pillar_gltf_path();
    assert!(path.is_file(), "meshes/pillar.gltf must be checked in");
    let txt = fs::read_to_string(&path).unwrap();
    let v: serde_json::Value = serde_json::from_str(&txt).expect("pillar.gltf is JSON");
    let ntex = &v["materials"][0]["normalTexture"];
    assert!(ntex.is_object(), "materials[0] must have normalTexture");
    let idx = ntex["index"].as_u64().expect("normalTexture.index");
    let tex = &v["textures"][idx as usize];
    let source = tex["source"].as_u64().expect("texture.source");
    let img = &v["images"][source as usize];
    let uri = img["uri"].as_str().expect("image.uri");
    assert!(
        uri.ends_with(".png") || uri.starts_with("data:"),
        "normal map should be a PNG or data URI, got {uri}"
    );
    if !uri.starts_with("data:") {
        let resolved = path.parent().unwrap().join(uri);
        assert!(
            resolved.is_file(),
            "normal map {uri} should exist at {}",
            resolved.display()
        );
    }
    // Keep the increment-13 MR texture.
    assert!(
        v["materials"][0]["pbrMetallicRoughness"]
            .get("metallicRoughnessTexture")
            .is_some(),
        "must keep metallicRoughnessTexture"
    );
}

#[test]
fn loaded_mesh_uses_sampled_normals_not_geometric() {
    let mesh = load_mesh(&pillar_gltf_path()).expect("glTF should load");
    let gm = mesh
        .gltf_material
        .as_ref()
        .expect("loaded mesh must carry glTF material");
    assert!(
        gm.has_normal_texture(),
        "loaded mesh must attach normalTexture"
    );

    let scene = increment14_scene();
    let body = gltf_body(&scene);
    assert!(
        body.material.albedo_map.is_none(),
        "scene-JSON must not carry a normal / albedo map; the glTF file is the look"
    );
    // Scene-JSON metallic/roughness stay the dull fallback (not a normal map).
    assert!(
        body.material.metallic.abs() < 1e-4 && (body.material.roughness - 0.85).abs() < 1e-3,
        "scene JSON pillar fallback should stay dull, got m={} r={}",
        body.material.metallic,
        body.material.roughness
    );

    let n0 = gm
        .sample_tangent_space_normal(0.12, 0.12)
        .expect("sample ts 0");
    let n1 = gm
        .sample_tangent_space_normal(0.37, 0.37)
        .expect("sample ts 1");
    let n_flat = [0.0f32, 0.0, 1.0];
    assert!(
        angle_deg(n0, n1) > 8.0,
        "two UV samples of the normal map must differ, got {n0:?} vs {n1:?} ({} deg)",
        angle_deg(n0, n1)
    );
    assert!(
        angle_deg(n0, n_flat) > 4.0 || angle_deg(n1, n_flat) > 4.0,
        "sampled tangent normals must not both be +Z (geometric), got {n0:?} {n1:?}"
    );

    // World-space shaded N on a side triangle must leave the geometric face normal.
    let mut found = false;
    for tri in 0..mesh.triangle_count() {
        let geom = mesh.geometric_normal(tri);
        // Skip caps (mostly +Y / -Y).
        if geom[1].abs() > 0.85 {
            continue;
        }
        let s0 = mesh.shaded_normal(tri, 0.20, 0.20).expect("shaded 0");
        let s1 = mesh.shaded_normal(tri, 0.55, 0.25).expect("shaded 1");
        let a0 = angle_deg(s0, geom);
        let a1 = angle_deg(s1, geom);
        let apart = angle_deg(s0, s1);
        if (a0 > 6.0 || a1 > 6.0) && apart > 4.0 {
            found = true;
            break;
        }
    }
    assert!(
        found,
        "expected a side triangle whose sampled normals leave the geometric N"
    );
}

#[test]
fn increment14_keeps_courtyard() {
    let scene = parse_scene(increment14_scene_json()).expect("increment14 JSON should parse");
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
        "keep the increment-13 courtyard (bowl + rock + ball + pillar, directional + point)"
    );
}

#[test]
fn contact_involving_gltf_body_after_step() {
    let scene = increment14_scene();
    let gltf_id = gltf_body(&scene).id.clone();
    let dump = step_physics(&scene, INCREMENT14_STEPS, DEFAULT_DT).expect("physics");
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
fn increment14_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment14.sh");
    assert!(script.is_file(), "scripts/increment14.sh must exist");

    let out = PathBuf::from("target/test-increment14-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment14(&out, INCREMENT14_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment14");
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
        !scene_txt.contains("normalTexture") && !scene_txt.contains("normal_map"),
        "scene-JSON must not be the normal-map look"
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
