use std::fs;
use std::path::PathBuf;
use std::process::Command;

use agent_rig::{
    increment10_scene, increment10_scene_json, parse_scene, render_scene, run_increment10,
    simulate_trajectory, step_physics, Light, Shape, DEFAULT_DT, INCREMENT10_STEPS,
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


#[test]
fn scene_has_point_and_directional_lights() {
    let scene = parse_scene(increment10_scene_json()).expect("increment10 JSON should parse");
    let mut has_dir = false;
    let mut has_point = false;
    for light in &scene.lights {
        match light {
            Light::Directional {
                direction,
                color,
                intensity,
            } => {
                has_dir = true;
                assert!(
                    direction.iter().any(|c| c.abs() > 1e-4),
                    "directional needs a direction"
                );
                assert!(color.iter().any(|c| *c > 0.0), "directional color");
                assert!(*intensity > 0.0, "directional intensity");
            }
            Light::Point {
                position,
                color,
                intensity,
            } => {
                has_point = true;
                assert!(
                    position.iter().any(|c| c.abs() > 1e-4),
                    "point light needs a position, got {position:?}"
                );
                assert!(
                    color[0] > 0.2 && color[1] > 0.2 && color[2] > 0.05,
                    "point light color {color:?}"
                );
                assert!(*intensity > 1.0, "point light intensity {intensity}");
            }
            Light::Area { .. } => {}
        }
    }
    assert!(has_dir, "increment 10 keeps the directional");
    assert!(has_point, "increment 10 adds a point light");

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
        "keep the increment-9 bowl + rock + ball + copper pillar"
    );
}

#[test]
fn contact_involving_gltf_body_after_step() {
    let scene = increment10_scene();
    let gltf_id = gltf_body(&scene).id.clone();
    let dump = step_physics(&scene, INCREMENT10_STEPS, DEFAULT_DT).expect("physics");
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
fn increment10_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment10.sh");
    assert!(script.is_file(), "scripts/increment10.sh must exist");

    let out = PathBuf::from("target/test-increment10-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment10(&out, INCREMENT10_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment10");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    let scene = parse_scene(&scene_txt).expect("written scene.json parses");
    let mut has_point = false;
    let mut has_dir = false;
    for light in &scene.lights {
        match light {
            Light::Point {
                position,
                color,
                intensity,
            } => {
                has_point = true;
                assert!(position.iter().any(|c| c.abs() > 1e-4));
                assert!(color.iter().any(|c| *c > 0.0));
                assert!(*intensity > 0.0);
            }
            Light::Directional { .. } => has_dir = true,
            Light::Area { .. } => {}
        }
    }
    assert!(has_point && has_dir, "written scene must keep both lights");

    let body = gltf_body(&scene);
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

#[test]
fn png_shows_local_point_light_falloff() {
    let scene = increment10_scene();
    let img = render_scene(&scene, 200, 112);
    let w = img.width() as f32;
    let h = img.height() as f32;
    let mut near_acc = 0.0f32;
    let mut near_n = 0usize;
    let mut far_acc = 0.0f32;
    let mut far_n = 0usize;
    for (x, y, p) in img.enumerate_pixels() {
        let u = (x as f32 + 0.5) / w;
        let v = (y as f32 + 0.5) / h;
        let r = p[0] as f32 / 255.0;
        let g = p[1] as f32 / 255.0;
        let b = p[2] as f32 / 255.0;
        let l = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        if l < 0.04 {
            continue;
        }
        // Skip sky (top of frame). Keep blown-out local highlight pixels.
        if v < 0.32 {
            continue;
        }
        // Floor facing the lamp (courtyard center-right). Avoid the pillar umbra.
        if (0.40..=0.54).contains(&u) && (0.60..=0.78).contains(&v) {
            near_acc += l;
            near_n += 1;
        }
        // Far left bowl rim, well away from the lamp.
        if (0.02..=0.16).contains(&u) && (0.58..=0.82).contains(&v) {
            far_acc += l;
            far_n += 1;
        }
    }
    assert!(near_n > 20, "expected near-lamp surface pixels, got {near_n}");
    assert!(far_n > 20, "expected far-rim surface pixels, got {far_n}");
    let near = near_acc / near_n as f32;
    let far = far_acc / far_n as f32;
    assert!(
        near > far + 0.06,
        "local light should read brighter near the lamp than the far rim (near={near:.3} far={far:.3})"
    );
}

#[test]
fn sim_and_render_load_point_light() {
    let scene = increment10_scene();
    let has_point = scene
        .lights
        .iter()
        .any(|l| matches!(l, Light::Point { .. }));
    assert!(has_point, "shared scene load must carry the point light");

    let traj = simulate_trajectory(&scene, 3, 10, DEFAULT_DT).expect("sim with point light");
    assert_eq!(traj.frames.len(), 3);
    assert!(traj.frames[0].bodies.iter().any(|b| b.id == "pillar"));
    assert!(traj.frames[0].bodies.iter().any(|b| b.id == "ground"));

    let img = render_scene(&scene, 80, 45);
    assert_eq!(img.width(), 80);
    assert_eq!(img.height(), 45);

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = manifest.join("target/test-increment10-cli");
    let _ = fs::remove_dir_all(&out);
    fs::create_dir_all(&out).unwrap();
    let scene_path = out.join("scene.json");
    fs::write(&scene_path, increment10_scene_json()).unwrap();

    let frame_path = out.join("frame.png");
    let status = Command::new(env!("CARGO_BIN_EXE_agent-rig"))
        .args([
            "render",
            scene_path.to_str().unwrap(),
            "--out",
            frame_path.to_str().unwrap(),
            "--width",
            "80",
            "--height",
            "45",
        ])
        .current_dir(&manifest)
        .status()
        .expect("render");
    assert!(
        status.success(),
        "agent-rig render with point light exited {status}"
    );
    assert!(frame_path.is_file());

    let sim_out = out.join("sim");
    let status = Command::new(env!("CARGO_BIN_EXE_agent-rig"))
        .args([
            "sim",
            scene_path.to_str().unwrap(),
            "--out",
            sim_out.to_str().unwrap(),
            "--frames",
            "2",
            "--stride",
            "5",
            "--width",
            "64",
            "--height",
            "36",
        ])
        .current_dir(&manifest)
        .status()
        .expect("sim");
    assert!(
        status.success(),
        "agent-rig sim with point light exited {status}"
    );
    assert!(sim_out.join("trajectory.json").is_file());
    assert!(sim_out.join("frame.png").is_file());
}
