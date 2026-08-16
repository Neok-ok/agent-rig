use std::fs;
use std::path::PathBuf;
use std::process::Command;

use agent_rig::{
    increment2_scene, increment2_scene_json, parse_scene, run_increment2, step_physics, Shape,
    DEFAULT_DT, FRAME_HEIGHT, FRAME_WIDTH, INCREMENT2_STEPS,
};

#[test]
fn increment2_scene_parses_and_has_required_bodies() {
    let scene = parse_scene(increment2_scene_json()).expect("increment2 JSON should parse");
    let non_ground: Vec<_> = scene.bodies.iter().filter(|b| b.id != "ground").collect();
    assert!(
        non_ground.len() >= 3,
        "need ≥3 bodies besides ground, got {}",
        non_ground.len()
    );

    let has_sphere = scene.bodies.iter().any(|b| matches!(b.shape, Shape::Sphere { .. }));
    let has_box = scene.bodies.iter().any(|b| matches!(b.shape, Shape::Box { .. }));
    assert!(has_sphere, "scene must have a sphere");
    assert!(has_box, "scene must have a box");

    let has_metal = scene.bodies.iter().any(|b| b.material.metallic >= 0.8);
    let has_rough_dielectric = scene
        .bodies
        .iter()
        .any(|b| b.material.metallic == 0.0 && b.material.roughness >= 0.7);
    assert!(has_metal, "need metal (metallic≥0.8)");
    assert!(has_rough_dielectric, "need rough dielectric (metallic 0, roughness≥0.7)");
}

#[test]
fn increment2_bodies_move_and_have_contacts() {
    let scene = increment2_scene();
    let dump = step_physics(&scene, INCREMENT2_STEPS, DEFAULT_DT).expect("physics");

    let mut moved = 0;
    for body in &scene.bodies {
        if body.mass <= 0.0 {
            continue;
        }
        let state = dump.bodies.iter().find(|b| b.id == body.id).expect("body in dump");
        let dx = state.position[0] - body.position[0];
        let dy = state.position[1] - body.position[1];
        let dz = state.position[2] - body.position[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        if dist > 0.05 {
            moved += 1;
        }
        // velocities field must be present (may be near-zero after settling)
        assert_eq!(state.linear_velocity.len(), 3);
        assert_eq!(state.angular_velocity.len(), 3);
    }
    assert!(moved >= 2, "at least two dynamic bodies should have moved, got {moved}");
    assert!(!dump.contacts.is_empty(), "dump should have contacts");

    let body_body = dump
        .contacts
        .iter()
        .any(|c| c.body_a != "ground" && c.body_b != "ground");
    assert!(
        body_body,
        "expected a non-ground/non-ground contact, contacts={:?}",
        dump.contacts
    );
}

#[test]
fn cli_step_and_render_write_files() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = manifest.join("target/test-increment2-cli");
    let _ = fs::remove_dir_all(&out);
    fs::create_dir_all(&out).unwrap();

    let scene_path = out.join("scene.json");
    fs::write(&scene_path, increment2_scene_json()).unwrap();

    let phys_path = out.join("physics.json");
    let status = Command::new(env!("CARGO_BIN_EXE_agent-rig"))
        .args([
            "step",
            scene_path.to_str().unwrap(),
            "--out",
            phys_path.to_str().unwrap(),
            "--steps",
            "120",
        ])
        .status()
        .expect("step");
    assert!(status.success(), "agent-rig step exited {status}");
    assert!(phys_path.is_file(), "step should write physics.json");
    let phys_txt = fs::read_to_string(&phys_path).unwrap();
    let v: serde_json::Value = serde_json::from_str(&phys_txt).unwrap();
    assert!(v["bodies"].as_array().unwrap().len() >= 4);
    assert!(v["contacts"].is_array());
    assert!(v["bodies"][0]["linear_velocity"].is_array());

    let frame_path = out.join("frame.png");
    let status = Command::new(env!("CARGO_BIN_EXE_agent-rig"))
        .args([
            "render",
            scene_path.to_str().unwrap(),
            "--out",
            frame_path.to_str().unwrap(),
            "--physics",
            phys_path.to_str().unwrap(),
            "--width",
            "160",
            "--height",
            "90",
        ])
        .status()
        .expect("render");
    assert!(status.success(), "agent-rig render exited {status}");
    assert!(frame_path.is_file(), "render should write frame.png");
    assert!(fs::metadata(&frame_path).unwrap().len() > 256);
}

#[test]
fn increment2_writes_three_artifacts() {
    let out = PathBuf::from("target/test-increment2-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment2(&out, INCREMENT2_STEPS, DEFAULT_DT, FRAME_WIDTH, FRAME_HEIGHT)
        .expect("run_increment2");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    parse_scene(&scene_txt).expect("written scene.json parses");
    let phys_txt = fs::read_to_string(&paths.physics).unwrap();
    let v: serde_json::Value = serde_json::from_str(&phys_txt).unwrap();
    assert!(v["bodies"].as_array().unwrap().len() >= 4);
    assert!(v["contacts"].is_array());
    let png_len = fs::metadata(&paths.frame).unwrap().len();
    assert!(png_len > 1024);
}
