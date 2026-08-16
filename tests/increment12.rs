use std::fs;
use std::path::{Path, PathBuf};

use agent_rig::{
    increment12_scene, increment12_scene_json, orbit_camera_position, orbit_radius_and_height,
    parse_scene, run_increment12, step_physics, Light, Shape, DEFAULT_DT, INCREMENT12_ORBIT_FRAMES,
    INCREMENT12_STEPS,
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

fn is_png(path: &Path) -> bool {
    let bytes = fs::read(path).unwrap_or_default();
    bytes.len() > 256 && bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A])
}

fn mean_abs_diff(a: &[u8], b: &[u8]) -> f32 {
    let img_a = image::load_from_memory(a).expect("png a").to_rgb8();
    let img_b = image::load_from_memory(b).expect("png b").to_rgb8();
    assert_eq!(img_a.dimensions(), img_b.dimensions());
    let mut acc = 0.0f32;
    let n = (img_a.width() * img_a.height()) as f32;
    for (pa, pb) in img_a.pixels().zip(img_b.pixels()) {
        acc += (pa[0] as f32 - pb[0] as f32).abs()
            + (pa[1] as f32 - pb[1] as f32).abs()
            + (pa[2] as f32 - pb[2] as f32).abs();
    }
    acc / (n * 3.0 * 255.0)
}

#[test]
fn scene_is_increment11_courtyard() {
    let scene = parse_scene(increment12_scene_json()).expect("increment12 JSON should parse");
    let mut has_dir = false;
    let mut has_point = false;
    for light in &scene.lights {
        match light {
            Light::Directional { .. } => has_dir = true,
            Light::Point { position, .. } => {
                has_point = true;
                assert!(
                    (position[0] - 0.55).abs() < 1e-4
                        && (position[1] - 0.82).abs() < 1e-4
                        && (position[2] - 1.10).abs() < 1e-4,
                    "keep the increment-11 point light at [0.55, 0.82, 1.10], got {position:?}"
                );
            }
        }
    }
    assert!(has_dir && has_point, "keep directional + point light");

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
    assert!(
        has_bowl && has_rock && has_ball && has_pillar,
        "keep the increment-11 bowl + rock + ball + copper pillar"
    );
}

#[test]
fn orbit_cameras_are_unique() {
    let scene = increment12_scene();
    let (radius, height) = orbit_radius_and_height(&scene.camera);
    assert!(radius > 1.0, "orbit radius should match increment-11 camera, got {radius}");
    assert!(
        (height - scene.camera.position[1]).abs() < 1e-5,
        "orbit height should match increment-11 camera y"
    );
    let mut positions: Vec<[f32; 3]> = Vec::new();
    for i in 0..INCREMENT12_ORBIT_FRAMES {
        let p = orbit_camera_position(scene.camera.look_at, radius, height, i);
        assert!(
            (p[1] - height).abs() < 1e-5,
            "camera {i} height should stay {height}, got {}",
            p[1]
        );
        for (j, other) in positions.iter().enumerate() {
            let d = (p[0] - other[0]).hypot(p[2] - other[2]);
            assert!(
                d > 0.5,
                "cameras {j} and {i} too close: {other:?} vs {p:?} dist={d}"
            );
        }
        positions.push(p);
    }
    assert_eq!(positions.len(), 8);
}

#[test]
fn contact_involving_gltf_body_after_step() {
    let scene = increment12_scene();
    let gltf_id = gltf_body(&scene).id.clone();
    let dump = step_physics(&scene, INCREMENT12_STEPS, DEFAULT_DT).expect("physics");
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
fn increment12_writes_scene_dump_and_orbit_pngs() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment12.sh");
    assert!(script.is_file(), "scripts/increment12.sh must exist");

    let out = PathBuf::from("target/test-increment12-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment12(&out, INCREMENT12_STEPS, DEFAULT_DT, 80, 45)
        .expect("run_increment12");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert_eq!(
        paths.frames.len(),
        INCREMENT12_ORBIT_FRAMES as usize,
        "expected 8 orbit frames"
    );

    for i in 0..8u32 {
        let path = out.join(format!("frame_{i:02}.png"));
        assert!(path.is_file(), "missing {}", path.display());
        assert!(is_png(&path), "{} is not a real PNG", path.display());
    }

    let f0 = fs::read(out.join("frame_00.png")).unwrap();
    let f2 = fs::read(out.join("frame_02.png")).unwrap();
    assert_ne!(f0, f2, "orbit frames must not be byte-identical copies");
    let diff = mean_abs_diff(&f0, &f2);
    assert!(
        diff > 0.02,
        "cameras 0 and 2 (90°) should differ, mean abs diff={diff:.4}"
    );

    let scene = increment12_scene();
    let body = gltf_body(&scene);
    let phys_txt = fs::read_to_string(&paths.physics).unwrap();
    let v: serde_json::Value = serde_json::from_str(&phys_txt).unwrap();
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
}
