use std::fs;
use std::path::PathBuf;
use std::process::Command;

use agent_rig::{
    increment7_scene, increment7_scene_json, load_obj, parse_scene, render_scene, run_increment7,
    simulate_trajectory, step_physics, MeshCollider, Shape, DEFAULT_DT, INCREMENT7_STEPS,
};

fn env_body(scene: &agent_rig::Scene) -> &agent_rig::Body {
    scene
        .bodies
        .iter()
        .find(|b| b.id == "ground")
        .expect("scene must have a ground / environment body")
}

#[test]
fn environment_mesh_loads_not_a_box() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bowl_path = manifest.join("meshes/bowl.obj");
    let rock_path = manifest.join("meshes/rock.obj");
    let wedge_path = manifest.join("meshes/wedge.obj");
    assert!(bowl_path.is_file(), "meshes/bowl.obj must be checked in");
    assert_ne!(bowl_path, rock_path);
    assert_ne!(bowl_path, wedge_path);

    let bowl = load_obj(&bowl_path).expect("bowl OBJ should load");
    assert!(bowl.vertex_count() > 8, "environment mesh should not be a box, verts={}", bowl.vertex_count());
    assert!(
        bowl.triangle_count() > 12,
        "environment mesh should not be a box, tris={}",
        bowl.triangle_count()
    );

    let scene = parse_scene(increment7_scene_json()).expect("increment7 JSON should parse");
    let ground = env_body(&scene);
    match &ground.shape {
        Shape::Mesh { path, collider } => {
            assert!(
                path.contains("bowl") || path.ends_with(".obj") || path.ends_with(".gltf"),
                "environment path={path}"
            );
            assert_eq!(*collider, MeshCollider::Trimesh, "ground collider should be trimesh");
            let resolved = manifest.join(path);
            assert!(resolved.is_file(), "environment mesh path {path} must exist");
        }
        Shape::Box { .. } => panic!("environment / ground must be a triangle mesh, not a box"),
        other => panic!("environment / ground must be a mesh, got {other:?}"),
    }
    assert!(ground.mass <= 0.0, "environment mesh should be static");

    let has_rock = scene.bodies.iter().any(|b| {
        matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("rock"))
    });
    let has_ball = scene.bodies.iter().any(|b| matches!(b.shape, Shape::Sphere { .. }));
    assert!(has_rock, "demo should reuse the rock mesh");
    assert!(has_ball, "demo should include a metal ball primitive");

    // No leftover box-plane floor.
    let box_floor = scene.bodies.iter().any(|b| {
        b.id == "ground" && matches!(b.shape, Shape::Box { .. })
    });
    assert!(!box_floor, "must not keep a box plane as the floor");
}

#[test]
fn contact_against_environment_mesh_after_step() {
    let scene = increment7_scene();
    let dump = step_physics(&scene, INCREMENT7_STEPS, DEFAULT_DT).expect("physics");
    let hit = dump
        .contacts
        .iter()
        .any(|c| c.body_a == "ground" || c.body_b == "ground");
    assert!(
        hit,
        "expected a contact against the environment mesh, contacts={:?}",
        dump.contacts
    );

    let ground = dump
        .bodies
        .iter()
        .find(|b| b.id == "ground")
        .expect("dump missing ground");
    assert_eq!(
        ground.collider, "trimesh",
        "environment mesh collider should be trimesh, got {}",
        ground.collider
    );
}

#[test]
fn increment7_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment7.sh");
    assert!(script.is_file(), "scripts/increment7.sh must exist");

    let out = PathBuf::from("target/test-increment7-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment7(&out, INCREMENT7_STEPS, DEFAULT_DT, 160, 90).expect("run_increment7");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    let scene = parse_scene(&scene_txt).expect("written scene.json parses");
    let ground = env_body(&scene);
    assert!(
        matches!(ground.shape, Shape::Mesh { .. }),
        "written scene ground must be a mesh"
    );
    assert!(
        !matches!(ground.shape, Shape::Box { .. }),
        "written scene must not use a box floor"
    );

    let phys_txt = fs::read_to_string(&paths.physics).unwrap();
    let v: serde_json::Value = serde_json::from_str(&phys_txt).unwrap();
    assert!(v["bodies"].as_array().unwrap().len() >= 3);
    assert!(v["contacts"].is_array());
    let ground_state = v["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == "ground")
        .expect("dump should record the environment mesh");
    assert_eq!(
        ground_state["collider"].as_str().unwrap_or(""),
        "trimesh",
        "dump must record trimesh for the environment mesh"
    );
    let contacts = v["contacts"].as_array().unwrap();
    let env_hit = contacts.iter().any(|c| {
        c["body_a"] == "ground" || c["body_b"] == "ground"
    });
    assert!(env_hit, "dump contacts should include the environment mesh");
    let png_len = fs::metadata(&paths.frame).unwrap().len();
    assert!(png_len > 256, "png too small: {png_len}");
}

#[test]
fn sim_and_render_load_environment_mesh() {
    let scene = increment7_scene();
    let traj = simulate_trajectory(&scene, 3, 10, DEFAULT_DT).expect("sim with environment mesh");
    assert_eq!(traj.frames.len(), 3);
    assert!(traj.frames[0].bodies.iter().any(|b| b.id == "ground"));
    assert!(traj.frames[0].bodies.iter().any(|b| b.id == "rock"));
    assert!(traj.frames[0].bodies.iter().any(|b| b.id == "ball"));

    let img = render_scene(&scene, 80, 45);
    assert_eq!(img.width(), 80);
    assert_eq!(img.height(), 45);

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = manifest.join("target/test-increment7-cli");
    let _ = fs::remove_dir_all(&out);
    fs::create_dir_all(&out).unwrap();
    let scene_path = out.join("scene.json");
    fs::write(&scene_path, increment7_scene_json()).unwrap();

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
        "agent-rig render with environment mesh exited {status}"
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
        "agent-rig sim with environment mesh exited {status}"
    );
    assert!(sim_out.join("trajectory.json").is_file());
    assert!(sim_out.join("frame.png").is_file());
}
