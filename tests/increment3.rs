use std::fs;
use std::path::PathBuf;
use std::process::Command;

use agent_rig::{
    increment3_scene, increment3_scene_json, parse_scene, run_increment3, simulate_trajectory,
    DEFAULT_DT, INCREMENT3_FRAMES, INCREMENT3_STRIDE, Shape,
};

fn is_identity_rotation(r: [f32; 4]) -> bool {
    (r[0] - 1.0).abs() < 0.02 && r[1].abs() < 0.02 && r[2].abs() < 0.02 && r[3].abs() < 0.02
}

#[test]
fn increment3_scene_has_ramp_or_stack() {
    let scene = parse_scene(increment3_scene_json()).expect("increment3 JSON should parse");
    let ramp_or_stack = scene.bodies.iter().any(|b| {
        matches!(b.shape, Shape::Box { .. })
            && b.mass <= 0.0
            && b.id != "ground"
            && !is_identity_rotation(b.rotation_wxyz)
    }) || scene.bodies.iter().any(|b| b.id == "ramp")
        || scene
            .bodies
            .iter()
            .filter(|b| matches!(b.shape, Shape::Box { .. }) && b.mass > 0.0)
            .count()
            >= 2;
    assert!(
        ramp_or_stack,
        "increment3 scene must have a ramp (rotated static box) or a stack"
    );

    let ball = scene.bodies.iter().find(|b| b.id == "ball").expect("ball");
    assert!(ball.material.metallic >= 0.85, "ball metallic {}", ball.material.metallic);
    assert!(ball.material.roughness <= 0.2, "ball roughness {}", ball.material.roughness);
    assert!(matches!(ball.shape, Shape::Sphere { .. }));

    let crate_body = scene.bodies.iter().find(|b| b.id == "crate").expect("crate");
    assert_eq!(crate_body.material.metallic, 0.0);
    assert!(crate_body.material.roughness >= 0.7);
}

#[test]
fn sim_writes_frames_and_trajectory() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = manifest.join("target/test-increment3-sim");
    let _ = fs::remove_dir_all(&out);
    fs::create_dir_all(&out).unwrap();

    let scene_path = out.join("scene.json");
    fs::write(&scene_path, increment3_scene_json()).unwrap();

    let status = Command::new(env!("CARGO_BIN_EXE_agent-rig"))
        .args([
            "sim",
            scene_path.to_str().unwrap(),
            "--frames",
            "8",
            "--stride",
            "20",
            "--out",
            out.to_str().unwrap(),
            "--width",
            "80",
            "--height",
            "45",
        ])
        .status()
        .expect("sim");
    assert!(status.success(), "agent-rig sim exited {status}");

    let frames_dir = out.join("frames");
    let mut pngs: Vec<_> = fs::read_dir(&frames_dir)
        .expect("frames/")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("png"))
        .collect();
    pngs.sort();
    assert!(
        pngs.len() >= 8,
        "sim should write ≥8 PNGs, got {}",
        pngs.len()
    );
    assert!(out.join("frames/frame_00.png").is_file());
    assert!(out.join("frame.png").is_file());

    let traj_path = out.join("trajectory.json");
    assert!(traj_path.is_file(), "sim should write trajectory.json");
    let traj_txt = fs::read_to_string(&traj_path).unwrap();
    let v: serde_json::Value = serde_json::from_str(&traj_txt).unwrap();
    let frames = v["frames"].as_array().expect("frames array");
    assert_eq!(frames.len(), pngs.len(), "one trajectory snapshot per frame PNG");
    for snap in frames {
        assert!(snap["bodies"].as_array().unwrap().len() >= 2);
        let body = &snap["bodies"][0];
        assert!(body["position"].is_array());
        assert!(body["linear_velocity"].is_array());
        assert!(snap["contacts"].is_array());
        assert!(snap["step"].is_number());
    }
}

#[test]
fn trajectory_dynamic_body_moves() {
    let scene = increment3_scene();
    let start = scene
        .bodies
        .iter()
        .find(|b| b.id == "ball")
        .expect("ball")
        .position;
    let traj = simulate_trajectory(&scene, INCREMENT3_FRAMES, INCREMENT3_STRIDE, DEFAULT_DT)
        .expect("trajectory");
    assert!(traj.frames.len() >= 8);
    let last = traj.frames.last().unwrap();
    let ball = last.bodies.iter().find(|b| b.id == "ball").expect("ball");
    let dx = ball.position[0] - start[0];
    let dy = ball.position[1] - start[1];
    let dist = (dx * dx + dy * dy).sqrt();
    assert!(
        dist > 0.8,
        "ball should travel down the ramp, start={start:?} end={:?} dist={dist}",
        ball.position
    );
    assert!(
        dx.abs() > 0.4 || dy.abs() > 0.3,
        "x or y should change by a real amount, dx={dx} dy={dy}"
    );
}

#[test]
fn increment3_writes_scene_trajectory_frame_and_frames_dir() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment3.sh");
    assert!(script.is_file(), "scripts/increment3.sh must exist");

    let out = PathBuf::from("target/test-increment3-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment3(&out, 8, INCREMENT3_STRIDE, DEFAULT_DT, 80, 45)
        .expect("run_increment3");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.trajectory.is_file(), "missing trajectory.json");
    assert!(paths.frame.is_file(), "missing frame.png");
    assert!(paths.frames_dir.is_dir(), "missing frames/");

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    parse_scene(&scene_txt).expect("written scene.json parses");

    let n = fs::read_dir(&paths.frames_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("png"))
        .count();
    assert!(n >= 8, "frames/ should have ≥8 PNGs, got {n}");
    assert!(out.join("frames/frame_00.png").is_file());
    assert!(fs::metadata(&paths.frame).unwrap().len() > 256);
}
