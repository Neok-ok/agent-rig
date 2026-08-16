use std::fs;
use std::path::PathBuf;
use std::process::Command;

use agent_rig::{
    increment5_scene, increment5_scene_json, parse_scene, render_scene, run_increment5,
    simulate_trajectory, Shape, DEFAULT_DT, INCREMENT5_STEPS,
};

#[test]
fn texture_png_loads_as_pattern() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("textures/rock.png");
    assert!(path.is_file(), "textures/rock.png must be checked in");
    let img = image::open(&path).expect("texture PNG should open").to_rgb8();
    assert!(
        img.width() >= 16 && img.height() >= 16,
        "texture must not be 1x1, got {}x{}",
        img.width(),
        img.height()
    );
    let first = img.get_pixel(0, 0);
    let different = img.pixels().any(|p| p != first);
    assert!(different, "texture is a solid color stand-in");

    // High-contrast: at least two well-separated hues.
    let mut min_r = 255u8;
    let mut max_r = 0u8;
    let mut min_b = 255u8;
    let mut max_b = 0u8;
    for p in img.pixels() {
        min_r = min_r.min(p[0]);
        max_r = max_r.max(p[0]);
        min_b = min_b.min(p[2]);
        max_b = max_b.max(p[2]);
    }
    assert!(
        max_r - min_r > 80 && max_b - min_b > 80,
        "texture contrast too low: r {min_r}..{max_r} b {min_b}..{max_b}"
    );
}

#[test]
fn increment5_mesh_material_has_albedo_map() {
    let scene = parse_scene(increment5_scene_json()).expect("increment5 JSON should parse");
    let mesh = scene
        .bodies
        .iter()
        .find(|b| matches!(b.shape, Shape::Mesh { .. }))
        .expect("scene must contain a mesh body");
    let map = mesh
        .material
        .albedo_map
        .as_ref()
        .expect("mesh material must have albedo_map");
    assert!(
        map.ends_with(".png"),
        "albedo_map should point at a PNG, got {map}"
    );
    assert!(
        map.contains("rock") || map.contains("texture"),
        "albedo_map should name the checked-in texture, got {map}"
    );
    let resolved = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(map);
    assert!(
        resolved.is_file(),
        "albedo_map {map} should exist at {}",
        resolved.display()
    );
    let has_prim = scene
        .bodies
        .iter()
        .any(|b| matches!(b.shape, Shape::Box { .. } | Shape::Sphere { .. }));
    assert!(has_prim, "demo must include primitives plus the textured mesh");
}

fn count_warm_cool(img: &image::RgbImage) -> (usize, usize) {
    let mut warm = 0usize;
    let mut cool = 0usize;
    for p in img.pixels() {
        let r = p[0] as i32;
        let g = p[1] as i32;
        let b = p[2] as i32;
        if r > g && r > b + 20 {
            warm += 1;
        }
        if b > r + 15 && g > r {
            cool += 1;
        }
    }
    (warm, cool)
}

#[test]
fn increment5_writes_scene_dump_and_textured_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment5.sh");
    assert!(script.is_file(), "scripts/increment5.sh must exist");

    let out = PathBuf::from("target/test-increment5-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment5(&out, INCREMENT5_STEPS, DEFAULT_DT, 200, 112).expect("run_increment5");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    let scene = parse_scene(&scene_txt).expect("written scene.json parses");
    let mesh = scene
        .bodies
        .iter()
        .find(|b| matches!(b.shape, Shape::Mesh { .. }))
        .expect("written scene has mesh");
    assert!(mesh.material.albedo_map.is_some());

    let phys_txt = fs::read_to_string(&paths.physics).unwrap();
    let v: serde_json::Value = serde_json::from_str(&phys_txt).unwrap();
    assert!(v["bodies"].as_array().unwrap().len() >= 3);
    assert!(v["contacts"].is_array());
    let png_len = fs::metadata(&paths.frame).unwrap().len();
    assert!(png_len > 256, "png too small: {png_len}");

    let frame = image::open(&paths.frame).unwrap().to_rgb8();
    let (warm, cool) = count_warm_cool(&frame);
    assert!(
        warm > 15 && cool > 15,
        "frame mesh pixels look like a single albedo (warm={warm} cool={cool})"
    );
}

#[test]
fn sim_and_render_load_albedo_map() {
    let scene = increment5_scene();
    let traj = simulate_trajectory(&scene, 2, 10, DEFAULT_DT).expect("sim with textured mesh");
    assert_eq!(traj.frames.len(), 2);

    // Unit render of just the mesh so the checker is unambiguous.
    let mut unit = increment5_scene();
    unit.bodies.retain(|b| b.id == "rock");
    unit.camera.position = [1.8, 1.15, 2.4];
    unit.camera.look_at = [0.15, 0.42, 0.0];
    let img = render_scene(&unit, 120, 90);
    let (warm, cool) = count_warm_cool(&img);
    assert!(
        warm > 30 && cool > 30,
        "unit mesh render is a single albedo (warm={warm} cool={cool})"
    );

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = manifest.join("target/test-increment5-cli");
    let _ = fs::remove_dir_all(&out);
    fs::create_dir_all(&out).unwrap();
    let scene_path = out.join("scene.json");
    fs::write(&scene_path, increment5_scene_json()).unwrap();

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
    assert!(status.success(), "agent-rig render with albedo_map exited {status}");
    assert!(frame_path.is_file());
}
