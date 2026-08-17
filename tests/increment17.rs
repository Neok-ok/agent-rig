use std::fs;
use std::path::PathBuf;

use agent_rig::{
    increment17_scene, increment17_scene_json, load_mesh, parse_scene, run_increment17,
    step_physics, GltfAlphaMode, Shape, DEFAULT_DT, INCREMENT17_STEPS,
};

fn gltf_bodies(scene: &agent_rig::Scene) -> Vec<&agent_rig::Body> {
    scene
        .bodies
        .iter()
        .filter(|b| match &b.shape {
            Shape::Mesh { path, .. } => path.ends_with(".gltf") || path.ends_with(".glb"),
            _ => false,
        })
        .collect()
}

fn body_by_id<'a>(scene: &'a agent_rig::Scene, id: &str) -> &'a agent_rig::Body {
    scene
        .bodies
        .iter()
        .find(|b| b.id == id)
        .unwrap_or_else(|| panic!("missing body {id}"))
}

fn pane_gltf_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("meshes/pane.gltf")
}

fn pillar_gltf_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("meshes/pillar.gltf")
}

#[test]
fn gltf_file_has_alpha_mode_blend() {
    let path = pane_gltf_path();
    assert!(path.is_file(), "meshes/pane.gltf must be checked in");
    let txt = fs::read_to_string(&path).unwrap();
    let v: serde_json::Value = serde_json::from_str(&txt).expect("pane.gltf is JSON");
    let mat = &v["materials"][0];
    let mode = mat["alphaMode"].as_str().unwrap_or("OPAQUE");
    assert!(
        mode == "BLEND" || mode == "MASK",
        "materials[0] alphaMode should be BLEND or MASK, got {mode}"
    );
    let factor = mat["pbrMetallicRoughness"]["baseColorFactor"]
        .as_array()
        .expect("baseColorFactor");
    assert!(factor.len() >= 4, "baseColorFactor must include alpha");
    let a = factor[3].as_f64().unwrap();
    assert!(
        a > 0.05 && a < 0.99,
        "baseColorFactor[3] should be see-through, got {a}"
    );
}

#[test]
fn loaded_mesh_sampled_alpha_is_not_one() {
    let mesh = load_mesh(&pane_gltf_path()).expect("pane glTF should load");
    let gm = mesh
        .gltf_material
        .as_ref()
        .expect("loaded pane must carry glTF material");
    assert_eq!(gm.alpha_mode, GltfAlphaMode::Blend);
    let samples = [
        gm.sample_alpha(0.1, 0.1).unwrap(),
        gm.sample_alpha(0.5, 0.5).unwrap(),
        gm.sample_alpha(0.9, 0.2).unwrap(),
    ];
    assert!(
        samples.iter().any(|&a| (a - 1.0).abs() > 0.02),
        "sampled alpha must not be 1.0 everywhere, got {samples:?}"
    );
    for a in samples {
        assert!(
            a > 0.05 && a < 0.95,
            "pane alpha should be obviously see-through (0.25–0.45 target), got {a}"
        );
    }

    // Pillar stays opaque.
    let pillar = load_mesh(&pillar_gltf_path()).expect("pillar");
    let pg = pillar.gltf_material.as_ref().expect("pillar material");
    assert_eq!(pg.alpha_mode, GltfAlphaMode::Opaque);
    assert!((pg.alpha_factor() - 1.0).abs() < 1e-4);
}

#[test]
fn increment17_keeps_courtyard() {
    let scene = parse_scene(increment17_scene_json()).expect("increment17 JSON should parse");
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
    let has_pillar = body_by_id(&scene, "pillar").id == "pillar";
    let has_pane = matches!(
        &body_by_id(&scene, "pane").shape,
        Shape::Mesh { path, .. } if path.contains("pane")
    );
    let has_dir = scene
        .lights
        .iter()
        .any(|l| matches!(l, agent_rig::Light::Directional { .. }));
    let has_point = scene
        .lights
        .iter()
        .any(|l| matches!(l, agent_rig::Light::Point { .. }));
    assert!(
        has_bowl && has_rock && has_ball && has_pillar && has_pane && has_dir && has_point,
        "keep the increment-16 courtyard and add the glass pane"
    );
    let gltfs = gltf_bodies(&scene);
    assert!(
        gltfs.len() >= 2,
        "expect pillar + pane glTF bodies, got {}",
        gltfs.len()
    );
    for b in &scene.bodies {
        let v = b.linear_velocity;
        let speed = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        assert!(
            speed < 1e-6,
            "increment 17 is a still; body {} should have zero velocity, got {v:?}",
            b.id
        );
    }
}

#[test]
fn contact_involving_gltf_body_after_step() {
    let scene = increment17_scene();
    let dump = step_physics(&scene, INCREMENT17_STEPS, DEFAULT_DT).expect("physics");
    let hit = dump
        .contacts
        .iter()
        .any(|c| c.body_a == "pillar" || c.body_b == "pillar");
    assert!(
        hit,
        "expected a contact involving the glTF pillar, contacts={:?}",
        dump.contacts
    );
    let body = dump
        .bodies
        .iter()
        .find(|b| b.id == "pillar")
        .expect("dump missing pillar");
    assert!(
        body.collider == "convex_hull" || body.collider == "trimesh",
        "glTF body collider should be convex_hull or trimesh, got {}",
        body.collider
    );
}

#[test]
fn increment17_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment17.sh");
    assert!(script.is_file(), "scripts/increment17.sh must exist");

    let out = PathBuf::from("target/test-increment17-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment17(&out, INCREMENT17_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment17");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    let scene = parse_scene(&scene_txt).expect("written scene.json parses");
    assert!(
        scene
            .bodies
            .iter()
            .any(|b| matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("pane")))
    );
    assert!(
        scene
            .bodies
            .iter()
            .any(|b| matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("pillar")))
    );

    let phys_txt = fs::read_to_string(&paths.physics).unwrap();
    let v: serde_json::Value = serde_json::from_str(&phys_txt).unwrap();
    assert!(v["bodies"].as_array().unwrap().len() >= 5);
    assert!(v["contacts"].is_array());
    let pillar_state = v["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == "pillar")
        .expect("dump should record the pillar");
    let col = pillar_state["collider"].as_str().unwrap_or("");
    assert!(
        col == "convex_hull" || col == "trimesh",
        "dump must record collider type for the glTF pillar, got {col}"
    );
    let contacts = v["contacts"].as_array().unwrap();
    let hit = contacts
        .iter()
        .any(|c| c["body_a"] == "pillar" || c["body_b"] == "pillar");
    assert!(hit, "dump contacts should include the glTF pillar");
    let png_len = fs::metadata(&paths.frame).unwrap().len();
    assert!(png_len > 256, "png too small: {png_len}");
}
