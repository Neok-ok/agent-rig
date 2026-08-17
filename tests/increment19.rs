use std::fs;
use std::path::PathBuf;

use agent_rig::{
    increment19_scene, increment19_scene_json, load_mesh, parse_scene, run_increment19,
    step_physics, Light, Shape, DEFAULT_DT, INCREMENT19_STEPS,
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

fn pillar_gltf_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("meshes/pillar.gltf")
}

fn area_light(scene: &agent_rig::Scene) -> ([f32; 3], [f32; 2], [f32; 3], f32, [f32; 3]) {
    for light in &scene.lights {
        if let Light::Area {
            position,
            size,
            color,
            intensity,
            normal,
        } = light
        {
            return (*position, *size, *color, *intensity, *normal);
        }
    }
    panic!("scene missing area light");
}

#[test]
fn gltf_file_has_occlusion_texture() {
    let path = pillar_gltf_path();
    assert!(path.is_file(), "meshes/pillar.gltf must be checked in");
    let txt = fs::read_to_string(&path).unwrap();
    let v: serde_json::Value = serde_json::from_str(&txt).expect("pillar.gltf is JSON");
    let mat = &v["materials"][0];
    let otex = &mat["occlusionTexture"];
    assert!(otex.is_object(), "materials[0] must have occlusionTexture");
    let idx = otex["index"].as_u64().expect("occlusionTexture.index");
    let tex = &v["textures"][idx as usize];
    let source = tex["source"].as_u64().expect("texture.source");
    let img = &v["images"][source as usize];
    let uri = img["uri"].as_str().expect("image.uri");
    assert!(
        uri.ends_with(".png") || uri.starts_with("data:"),
        "AO map should be a PNG or data URI, got {uri}"
    );
    if !uri.starts_with("data:") {
        let resolved = path.parent().unwrap().join(uri);
        assert!(
            resolved.is_file(),
            "AO map {uri} should exist at {}",
            resolved.display()
        );
    }
    // Keep increment-13/14/16 maps.
    assert!(
        mat["pbrMetallicRoughness"]
            .get("metallicRoughnessTexture")
            .is_some(),
        "must keep metallicRoughnessTexture"
    );
    assert!(mat.get("normalTexture").is_some(), "must keep normalTexture");
    assert!(
        mat.get("emissiveTexture").is_some(),
        "must keep emissiveTexture"
    );
}

#[test]
fn loaded_mesh_sampled_ao_is_not_one() {
    let mesh = load_mesh(&pillar_gltf_path()).expect("glTF should load");
    let gm = mesh
        .gltf_material
        .as_ref()
        .expect("loaded mesh must carry glTF material");
    assert!(
        gm.has_occlusion_texture(),
        "loaded mesh must attach occlusionTexture"
    );

    // Grooves / base contact are dark; facing flats stay open.
    // Hex unwrap: seams at k/6, mid-face flute at (k+0.5)/6, flats near 0.12 + k/6.
    let samples = [
        gm.sample_ao(0.0, 0.50).expect("sample face-edge groove"),
        gm.sample_ao(1.0 / 6.0, 0.50).expect("sample next seam"),
        gm.sample_ao(0.12, 0.06).expect("sample flat U at base"),
        gm.sample_ao(0.12, 0.50).expect("sample facing flat"),
        gm.sample_ao(0.12 + 1.0 / 6.0, 0.60).expect("sample another flat"),
        gm.sample_ao(0.5 / 6.0, 0.50).expect("sample mid-face flute"),
    ];
    assert!(
        samples.iter().any(|&a| (a - 1.0).abs() > 0.05),
        "sampled AO must not be 1.0 everywhere, got {samples:?}"
    );
    let min_ao = samples.iter().cloned().fold(1.0f32, f32::min);
    let max_ao = samples.iter().cloned().fold(0.0f32, f32::max);
    assert!(
        min_ao < 0.35,
        "crevice / contact AO should be obviously dark, min={min_ao} samples={samples:?}"
    );
    assert!(
        max_ao > 0.7,
        "facing flats should stay open, max={max_ao} samples={samples:?}"
    );
}

#[test]
fn increment19_keeps_courtyard() {
    let scene = parse_scene(increment19_scene_json()).expect("increment19 JSON should parse");
    let (_pos, size, _color, intensity, _n) = area_light(&scene);
    assert!(
        size[0] > 0.5 && size[1] > 0.4,
        "keep the authored area light size, got {size:?}"
    );
    assert!(intensity > 1.0, "area light intensity {intensity}");

    let has_dir = scene
        .lights
        .iter()
        .any(|l| matches!(l, Light::Directional { .. }));
    assert!(has_dir, "keep the directional");

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
    assert!(
        has_bowl && has_rock && has_ball && has_pillar && has_pane,
        "keep the increment-18 courtyard including the pane"
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
            "increment 19 is a still; body {} should have zero velocity, got {v:?}",
            b.id
        );
    }
}

#[test]
fn contact_involving_gltf_body_after_step() {
    let scene = increment19_scene();
    let dump = step_physics(&scene, INCREMENT19_STEPS, DEFAULT_DT).expect("physics");
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
fn increment19_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment19.sh");
    assert!(script.is_file(), "scripts/increment19.sh must exist");

    let out = PathBuf::from("target/test-increment19-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment19(&out, INCREMENT19_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment19");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    let scene = parse_scene(&scene_txt).expect("written scene.json parses");
    let (_pos, size, _color, intensity, _n) = area_light(&scene);
    assert!(
        size[0] > 0.5 && size[1] > 0.4,
        "written scene must author area size, got {size:?}"
    );
    assert!(intensity > 0.0);
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
