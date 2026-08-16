use std::fs;
use std::path::PathBuf;
use std::process::Command;

use agent_rig::{
    apply_physics_to_scene, demo_scene, demo_scene_json, parse_scene, render_scene, run_increment1,
    step_physics, DEFAULT_DT, DEFAULT_STEPS, FRAME_HEIGHT, FRAME_WIDTH,
};

#[test]
fn parse_demo_scene_json() {
    let scene = parse_scene(demo_scene_json()).expect("demo JSON should parse");
    assert_eq!(scene.bodies.len(), 2);
    assert_eq!(scene.bodies[0].id, "ground");
    assert_eq!(scene.bodies[1].id, "ball");
    assert!((scene.bodies[1].position[1] - 2.2).abs() < 1e-5);
    assert_eq!(scene.lights.len(), 1);
}

#[test]
fn ball_falls_and_contacts_ground() {
    let scene = demo_scene();
    let start_y = scene
        .bodies
        .iter()
        .find(|b| b.id == "ball")
        .unwrap()
        .position[1];
    let dump = step_physics(&scene, DEFAULT_STEPS, DEFAULT_DT).expect("physics");
    let ball = dump.bodies.iter().find(|b| b.id == "ball").expect("ball");
    assert!(
        ball.position[1] < start_y - 0.5,
        "ball y should drop, start={start_y} end={}",
        ball.position[1]
    );
    let rest_y = 0.4; // sphere radius sitting on ground top at y=0
    let near_rest = (ball.position[1] - rest_y).abs() < 0.15;
    let has_contact = dump.contacts.iter().any(|c| {
        (c.body_a == "ball" && c.body_b == "ground") || (c.body_a == "ground" && c.body_b == "ball")
    });
    assert!(
        has_contact || near_rest,
        "expected ball-ground contact or rest height, y={}, contacts={}",
        ball.position[1],
        dump.contacts.len()
    );
}

#[test]
fn render_png_is_real_lit_frame() {
    let scene = demo_scene();
    let dump = step_physics(&scene, DEFAULT_STEPS, DEFAULT_DT).expect("physics");
    let mut framed = scene;
    apply_physics_to_scene(&mut framed, &dump);
    let img = render_scene(&framed, FRAME_WIDTH, FRAME_HEIGHT);
    assert_eq!(img.width(), FRAME_WIDTH);
    assert_eq!(img.height(), FRAME_HEIGHT);

    let dir = PathBuf::from("target/test-render");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("frame.png");
    img.save(&path).unwrap();
    let bytes = fs::metadata(&path).unwrap().len();
    assert!(bytes > 1024, "png too small: {bytes}");

    let loaded = image::open(&path).unwrap();
    assert_eq!(loaded.width(), FRAME_WIDTH);
    assert_eq!(loaded.height(), FRAME_HEIGHT);
    let rgba = loaded.to_rgb8();
    let first = rgba.get_pixel(0, 0);
    let all_same = rgba.pixels().all(|p| p == first);
    assert!(!all_same, "frame is a solid color");
}

#[test]
fn increment1_writes_three_artifacts() {
    let out = PathBuf::from("target/test-increment1-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment1(&out, DEFAULT_STEPS, DEFAULT_DT, FRAME_WIDTH, FRAME_HEIGHT)
        .expect("run_increment1");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    parse_scene(&scene_txt).expect("written scene.json parses");
    let phys_txt = fs::read_to_string(&paths.physics).unwrap();
    let v: serde_json::Value = serde_json::from_str(&phys_txt).unwrap();
    assert!(v["bodies"].as_array().unwrap().len() >= 2);
    assert!(v["contacts"].is_array());
    let png_len = fs::metadata(&paths.frame).unwrap().len();
    assert!(png_len > 1024);
}

#[test]
fn increment1_script_or_cli_writes_artifacts() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let script = manifest.join("scripts/increment1.sh");
    assert!(script.is_file(), "scripts/increment1.sh must exist");

    let out = manifest.join("target/test-cli-artifacts");
    let _ = fs::remove_dir_all(&out);
    let status = Command::new(env!("CARGO_BIN_EXE_agent-rig"))
        .args([
            "--demo",
            "--out",
            out.to_str().unwrap(),
            "--steps",
            "90",
        ])
        .status()
        .expect("run agent-rig");
    assert!(status.success(), "agent-rig --demo exited {}", status);
    assert!(out.join("scene.json").is_file());
    assert!(out.join("physics.json").is_file());
    assert!(out.join("frame.png").is_file());
}

#[test]
fn threejs_baseline_png_is_real_image() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let script = manifest.join("scripts/threejs-baseline.sh");
    assert!(script.is_file(), "scripts/threejs-baseline.sh must exist");
    assert!(
        manifest.join("compare/threejs-baseline.html").is_file(),
        "compare/threejs-baseline.html must exist"
    );
    assert!(
        manifest.join("compare/three.module.min.js").is_file(),
        "vendored three.module.min.js must exist"
    );

    let status = Command::new(&script)
        .current_dir(&manifest)
        .status()
        .expect("run threejs-baseline.sh");
    assert!(status.success(), "threejs-baseline.sh exited {}", status);

    let path = manifest.join("artifacts/threejs-frame.png");
    assert!(path.is_file(), "missing artifacts/threejs-frame.png");
    let bytes = fs::metadata(&path).unwrap().len();
    assert!(bytes > 1024, "threejs png too small: {bytes}");

    let loaded = image::open(&path).expect("threejs-frame.png should open");
    assert!(loaded.width() >= 200, "width {}", loaded.width());
    assert!(loaded.height() >= 100, "height {}", loaded.height());
    let rgb = loaded.to_rgb8();
    let first = rgb.get_pixel(0, 0);
    let all_same = rgb.pixels().all(|p| p == first);
    assert!(!all_same, "threejs-frame.png is a solid color");

    let ours = manifest.join("artifacts/frame.png");
    if ours.is_file() {
        let a = fs::read(&path).unwrap();
        let b = fs::read(&ours).unwrap();
        assert_ne!(a, b, "threejs and our frames should not be byte-identical");
    }
}
