use std::fs;
use std::path::PathBuf;
use std::process::Command;

use agent_rig::{
    increment6_scene, increment6_scene_json, load_obj, parse_scene, render_scene, run_increment6,
    simulate_trajectory, step_physics, Shape, DEFAULT_DT, INCREMENT6_STEPS,
};

fn mesh_paths(scene: &agent_rig::Scene) -> Vec<String> {
    scene
        .bodies
        .iter()
        .filter_map(|b| match &b.shape {
            Shape::Mesh { path, .. } => Some(path.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn two_distinct_mesh_files_load() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let rock_path = manifest.join("meshes/rock.obj");
    let wedge_path = manifest.join("meshes/wedge.obj");
    assert!(rock_path.is_file(), "meshes/rock.obj must be checked in");
    assert!(wedge_path.is_file(), "meshes/wedge.obj must be checked in");
    assert_ne!(
        rock_path, wedge_path,
        "the two mesh files must be different paths"
    );

    let rock = load_obj(&rock_path).expect("rock OBJ should load");
    let wedge = load_obj(&wedge_path).expect("wedge OBJ should load");
    assert!(rock.vertex_count() > 0 && rock.triangle_count() > 0);
    assert!(wedge.vertex_count() > 0 && wedge.triangle_count() > 0);
    assert!(
        rock.vertex_count() != wedge.vertex_count()
            || rock.triangle_count() != wedge.triangle_count(),
        "meshes should differ in vertex or triangle count (rock {}v/{}t vs wedge {}v/{}t)",
        rock.vertex_count(),
        rock.triangle_count(),
        wedge.vertex_count(),
        wedge.triangle_count()
    );
}

#[test]
fn increment6_scene_has_two_distinct_mesh_paths() {
    let scene = parse_scene(increment6_scene_json()).expect("increment6 JSON should parse");
    let paths = mesh_paths(&scene);
    assert_eq!(paths.len(), 2, "scene must contain two mesh bodies, got {paths:?}");
    assert_ne!(paths[0], paths[1], "mesh bodies must have different path values");
    assert!(
        paths.iter().any(|p| p.contains("rock")),
        "one mesh should be the existing rock, got {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.contains("wedge") || p.contains("crate")),
        "second mesh should be a distinct file, got {paths:?}"
    );

    let has_prim = scene
        .bodies
        .iter()
        .any(|b| matches!(b.shape, Shape::Box { .. } | Shape::Sphere { .. }));
    assert!(has_prim, "demo must include primitives plus the two meshes");

    for b in &scene.bodies {
        if let Shape::Mesh { collider, .. } = &b.shape {
            let kind = format!("{collider:?}").to_lowercase();
            assert!(
                kind.contains("convex") || kind.contains("trimesh"),
                "mesh {} collider={collider:?}",
                b.id
            );
        }
    }
}

#[test]
fn mesh_contacts_and_collider_types_after_step() {
    let scene = increment6_scene();
    let mesh_ids: Vec<String> = scene
        .bodies
        .iter()
        .filter(|b| matches!(b.shape, Shape::Mesh { .. }))
        .map(|b| b.id.clone())
        .collect();
    assert_eq!(mesh_ids.len(), 2);

    let dump = step_physics(&scene, INCREMENT6_STEPS, DEFAULT_DT).expect("physics");
    let hit = dump
        .contacts
        .iter()
        .any(|c| mesh_ids.iter().any(|id| c.body_a == *id || c.body_b == *id));
    assert!(
        hit,
        "expected a contact involving a mesh body, contacts={:?}",
        dump.contacts
    );

    for id in &mesh_ids {
        let body = dump
            .bodies
            .iter()
            .find(|b| b.id == *id)
            .unwrap_or_else(|| panic!("dump missing mesh body {id}"));
        assert!(
            body.collider == "convex_hull" || body.collider == "trimesh",
            "mesh body {id} collider={}",
            body.collider
        );
    }
}

#[test]
fn increment6_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment6.sh");
    assert!(script.is_file(), "scripts/increment6.sh must exist");

    let out = PathBuf::from("target/test-increment6-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment6(&out, INCREMENT6_STEPS, DEFAULT_DT, 160, 90).expect("run_increment6");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    let scene = parse_scene(&scene_txt).expect("written scene.json parses");
    let paths_in_scene = mesh_paths(&scene);
    assert_eq!(paths_in_scene.len(), 2);
    assert_ne!(paths_in_scene[0], paths_in_scene[1]);

    let phys_txt = fs::read_to_string(&paths.physics).unwrap();
    let v: serde_json::Value = serde_json::from_str(&phys_txt).unwrap();
    assert!(v["bodies"].as_array().unwrap().len() >= 4);
    assert!(v["contacts"].is_array());
    let mesh_states: Vec<_> = v["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|b| {
            let id = b["id"].as_str().unwrap_or("");
            id == "rock" || id == "wedge"
        })
        .collect();
    assert_eq!(mesh_states.len(), 2, "dump should record both mesh bodies");
    for st in &mesh_states {
        let col = st["collider"].as_str().unwrap_or("");
        assert!(
            col == "convex_hull" || col == "trimesh",
            "collider={col} for {}",
            st["id"]
        );
    }
    let png_len = fs::metadata(&paths.frame).unwrap().len();
    assert!(png_len > 256, "png too small: {png_len}");
}

#[test]
fn sim_and_render_load_both_meshes() {
    let scene = increment6_scene();
    let traj = simulate_trajectory(&scene, 3, 10, DEFAULT_DT).expect("sim with two meshes");
    assert_eq!(traj.frames.len(), 3);
    assert!(traj.frames[0].bodies.iter().any(|b| b.id == "rock"));
    assert!(traj.frames[0].bodies.iter().any(|b| b.id == "wedge"));

    let img = render_scene(&scene, 80, 45);
    assert_eq!(img.width(), 80);
    assert_eq!(img.height(), 45);

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = manifest.join("target/test-increment6-cli");
    let _ = fs::remove_dir_all(&out);
    fs::create_dir_all(&out).unwrap();
    let scene_path = out.join("scene.json");
    fs::write(&scene_path, increment6_scene_json()).unwrap();

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
        "agent-rig render with two meshes exited {status}"
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
        "agent-rig sim with two meshes exited {status}"
    );
    assert!(sim_out.join("trajectory.json").is_file());
    assert!(sim_out.join("frame.png").is_file());
}
