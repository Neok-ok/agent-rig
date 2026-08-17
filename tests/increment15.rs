use std::fs;
use std::path::{Path, PathBuf};

use agent_rig::{
    increment15_scene, increment15_scene_json, parse_scene, run_increment15, simulate_trajectory,
    Light, Shape, DEFAULT_DT, INCREMENT15_FRAMES, INCREMENT15_STRIDE,
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

fn moving_body(scene: &agent_rig::Scene) -> &agent_rig::Body {
    scene
        .bodies
        .iter()
        .find(|b| {
            let v = b.linear_velocity;
            v[0] * v[0] + v[1] * v[1] + v[2] * v[2] > 1e-8
        })
        .expect("scene must have a body with non-zero initial linear velocity")
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
fn ball_or_rock_has_nonzero_initial_velocity() {
    let scene = parse_scene(increment15_scene_json()).expect("increment15 JSON should parse");
    let body = moving_body(&scene);
    assert!(
        body.id == "ball" || body.id == "rock",
        "shoved body should be the ball or rock, got {}",
        body.id
    );
    let v = body.linear_velocity;
    let speed = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    assert!(speed > 0.5, "initial linear velocity should be a real shove, got {v:?} speed={speed}");
}

#[test]
fn scene_is_increment14_courtyard_same_camera() {
    let scene = parse_scene(increment15_scene_json()).expect("increment15 JSON should parse");
    assert_eq!(scene.camera.position, [3.6, 2.35, 5.2]);
    assert_eq!(scene.camera.look_at, [0.1, 0.38, 0.0]);

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
            Light::Area { .. } => {}
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
        "keep the increment-14 bowl + rock + ball + pillar"
    );
}

#[test]
fn frame0_and_frame7_body_positions_differ() {
    let scene = increment15_scene();
    let id = moving_body(&scene).id.clone();
    let traj = simulate_trajectory(&scene, INCREMENT15_FRAMES, INCREMENT15_STRIDE, DEFAULT_DT)
        .expect("trajectory");
    assert_eq!(traj.frames.len(), INCREMENT15_FRAMES as usize);
    let p0 = traj.frames[0]
        .bodies
        .iter()
        .find(|b| b.id == id)
        .expect("frame 0 body")
        .position;
    let p7 = traj.frames[7]
        .bodies
        .iter()
        .find(|b| b.id == id)
        .expect("frame 7 body")
        .position;
    let dx = p7[0] - p0[0];
    let dy = p7[1] - p0[1];
    let dz = p7[2] - p0[2];
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    assert!(
        dist > 0.15,
        "body {id} should move between frame 0 and 7, p0={p0:?} p7={p7:?} dist={dist}"
    );
    let r7 = (p7[0] * p7[0] + p7[2] * p7[2]).sqrt();
    assert!(
        r7 < 3.5 && p7[1] < 2.0,
        "body {id} should stay in the bowl, p7={p7:?} xz_r={r7}"
    );
}

#[test]
fn dump_has_pillar_collider_and_contact() {
    let scene = increment15_scene();
    let gltf_id = gltf_body(&scene).id.clone();
    let traj = simulate_trajectory(&scene, INCREMENT15_FRAMES, INCREMENT15_STRIDE, DEFAULT_DT)
        .expect("trajectory");
    let last = traj.frames.last().expect("last frame");
    let body = last
        .bodies
        .iter()
        .find(|b| b.id == gltf_id)
        .expect("dump missing glTF body");
    assert!(
        body.collider == "convex_hull" || body.collider == "trimesh",
        "glTF body collider should be convex_hull or trimesh, got {}",
        body.collider
    );
    let hit = last
        .contacts
        .iter()
        .any(|c| c.body_a == gltf_id || c.body_b == gltf_id)
        || traj.frames.iter().any(|f| {
            f.contacts
                .iter()
                .any(|c| c.body_a == gltf_id || c.body_b == gltf_id)
        });
    assert!(
        hit,
        "expected a contact involving the glTF body {gltf_id} after the sim"
    );
}

#[test]
fn increment15_writes_scene_dump_and_eight_pngs() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment15.sh");
    assert!(script.is_file(), "scripts/increment15.sh must exist");

    let out = PathBuf::from("target/test-increment15-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment15(
        &out,
        INCREMENT15_FRAMES,
        INCREMENT15_STRIDE,
        DEFAULT_DT,
        80,
        45,
    )
    .expect("run_increment15");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert_eq!(
        paths.frames.len(),
        INCREMENT15_FRAMES as usize,
        "expected 8 physics frames"
    );

    for i in 0..8u32 {
        let path = out.join(format!("frame_{i:02}.png"));
        assert!(path.is_file(), "missing {}", path.display());
        assert!(is_png(&path), "{} is not a real PNG", path.display());
    }

    let f0 = fs::read(out.join("frame_00.png")).unwrap();
    let f7 = fs::read(out.join("frame_07.png")).unwrap();
    assert_ne!(f0, f7, "frame_00 and frame_07 must not be byte-identical copies");
    let diff = mean_abs_diff(&f0, &f7);
    assert!(
        diff > 0.002,
        "frame_00 vs frame_07 should differ, mean abs diff={diff:.4}"
    );

    let scene = increment15_scene();
    let body = gltf_body(&scene);
    let mover = moving_body(&scene);
    let phys_txt = fs::read_to_string(&paths.physics).unwrap();
    let v: serde_json::Value = serde_json::from_str(&phys_txt).unwrap();
    let frames = v["frames"].as_array().expect("physics.json should be a trajectory with frames");
    assert_eq!(frames.len(), 8, "physics dump should have 8 frames");

    let p0 = &frames[0]["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == mover.id)
        .expect("frame 0 moving body")["position"];
    let p7 = &frames[7]["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == mover.id)
        .expect("frame 7 moving body")["position"];
    assert_ne!(p0, p7, "dump frame 0 vs 7 body position must differ");

    let gltf_state = v["bodies"]
        .as_array()
        .expect("dump bodies")
        .iter()
        .find(|b| b["id"] == body.id)
        .expect("dump should record the glTF body");
    let col = gltf_state["collider"].as_str().unwrap_or("");
    assert!(
        col == "convex_hull" || col == "trimesh",
        "dump must record collider type for the glTF body, got {col}"
    );
    let contacts = v["contacts"].as_array().unwrap();
    let top_hit = contacts
        .iter()
        .any(|c| c["body_a"] == body.id || c["body_b"] == body.id);
    let frame_hit = frames.iter().any(|f| {
        f["contacts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["body_a"] == body.id || c["body_b"] == body.id)
    });
    assert!(
        top_hit || frame_hit,
        "dump contacts should include the glTF body"
    );
}
