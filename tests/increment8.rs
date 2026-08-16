use std::fs;
use std::path::PathBuf;
use std::process::Command;

use agent_rig::{
    increment8_scene, increment8_scene_json, load_mesh, parse_scene, render_scene, run_increment8,
    simulate_trajectory, step_physics, Shape, DEFAULT_DT, INCREMENT8_STEPS,
};

fn gltf_body(scene: &agent_rig::Scene) -> &agent_rig::Body {
    scene
        .bodies
        .iter()
        .find(|b| match &b.shape {
            Shape::Mesh { path, .. } => {
                path.ends_with(".gltf") || path.ends_with(".glb")
            }
            _ => false,
        })
        .expect("scene must contain a glTF/GLB mesh body")
}

#[test]
fn gltf_mesh_loads_with_vertices_and_triangles() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("meshes/pillar.gltf");
    assert!(path.is_file(), "meshes/pillar.gltf must be checked in");
    let mesh = load_mesh(&path).expect("glTF should load");
    assert!(
        mesh.vertex_count() > 0,
        "glTF vertex count should be > 0, got {}",
        mesh.vertex_count()
    );
    assert!(
        mesh.triangle_count() > 0,
        "glTF triangle count should be > 0, got {}",
        mesh.triangle_count()
    );
}

#[test]
fn increment8_scene_has_gltf_mesh_body() {
    let scene = parse_scene(increment8_scene_json()).expect("increment8 JSON should parse");
    let body = gltf_body(&scene);
    match &body.shape {
        Shape::Mesh { path, collider } => {
            assert!(
                path.ends_with(".gltf") || path.ends_with(".glb"),
                "mesh path should be glTF/GLB, got {path}"
            );
            let kind = format!("{collider:?}").to_lowercase();
            assert!(
                kind.contains("convex") || kind.contains("trimesh"),
                "glTF body collider={collider:?}"
            );
            let resolved = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path);
            assert!(resolved.is_file(), "glTF path {path} must exist");
        }
        other => panic!("expected mesh, got {other:?}"),
    }
    let has_bowl = scene.bodies.iter().any(|b| {
        matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("bowl"))
    });
    let has_rock = scene.bodies.iter().any(|b| {
        matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("rock"))
    });
    let has_ball = scene
        .bodies
        .iter()
        .any(|b| matches!(b.shape, Shape::Sphere { .. }));
    assert!(has_bowl, "increment 8 keeps the bowl environment");
    assert!(has_rock, "increment 8 keeps the rock");
    assert!(has_ball, "increment 8 keeps the ball");
}

#[test]
fn contact_involving_gltf_body_after_step() {
    let scene = increment8_scene();
    let gltf_id = gltf_body(&scene).id.clone();
    let dump = step_physics(&scene, INCREMENT8_STEPS, DEFAULT_DT).expect("physics");
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
fn increment8_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment8.sh");
    assert!(script.is_file(), "scripts/increment8.sh must exist");

    let out = PathBuf::from("target/test-increment8-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment8(&out, INCREMENT8_STEPS, DEFAULT_DT, 160, 90).expect("run_increment8");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    let scene = parse_scene(&scene_txt).expect("written scene.json parses");
    let body = gltf_body(&scene);
    assert!(
        matches!(&body.shape, Shape::Mesh { path, .. } if path.ends_with(".gltf") || path.ends_with(".glb"))
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
    let hit = contacts.iter().any(|c| {
        c["body_a"] == body.id || c["body_b"] == body.id
    });
    assert!(hit, "dump contacts should include the glTF body");
    let png_len = fs::metadata(&paths.frame).unwrap().len();
    assert!(png_len > 256, "png too small: {png_len}");
}

#[test]
fn sim_and_render_load_gltf_path() {
    let scene = increment8_scene();
    let gltf_id = gltf_body(&scene).id.clone();
    let traj = simulate_trajectory(&scene, 3, 10, DEFAULT_DT).expect("sim with glTF mesh");
    assert_eq!(traj.frames.len(), 3);
    assert!(traj.frames[0].bodies.iter().any(|b| b.id == gltf_id));
    assert!(traj.frames[0].bodies.iter().any(|b| b.id == "ground"));
    assert!(traj.frames[0].bodies.iter().any(|b| b.id == "rock"));
    assert!(traj.frames[0].bodies.iter().any(|b| b.id == "ball"));

    let img = render_scene(&scene, 80, 45);
    assert_eq!(img.width(), 80);
    assert_eq!(img.height(), 45);

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = manifest.join("target/test-increment8-cli");
    let _ = fs::remove_dir_all(&out);
    fs::create_dir_all(&out).unwrap();
    let scene_path = out.join("scene.json");
    fs::write(&scene_path, increment8_scene_json()).unwrap();

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
        "agent-rig render with glTF mesh exited {status}"
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
        "agent-rig sim with glTF mesh exited {status}"
    );
    assert!(sim_out.join("trajectory.json").is_file());
    assert!(sim_out.join("frame.png").is_file());
}
