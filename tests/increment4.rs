use std::fs;
use std::path::PathBuf;
use std::process::Command;

use agent_rig::{
    increment4_scene, increment4_scene_json, load_obj, parse_scene, render_scene, run_increment4,
    simulate_trajectory, step_physics, MeshCollider, Shape, DEFAULT_DT, INCREMENT4_STEPS,
};

#[test]
fn mesh_file_loads_as_triangle_mesh() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("meshes/rock.obj");
    assert!(path.is_file(), "meshes/rock.obj must be checked in");
    let mesh = load_obj(&path).expect("OBJ should load");
    assert!(mesh.vertex_count() > 0, "need vertices");
    assert!(mesh.triangle_count() > 0, "need triangles");
    assert!(
        mesh.vertex_count() > 8,
        "mesh should not be a box (8 verts), got {}",
        mesh.vertex_count()
    );
    assert!(
        mesh.triangle_count() > 12,
        "mesh should not be a box (12 tris), got {}",
        mesh.triangle_count()
    );

    // Not a sphere: radii from centroid must vary.
    let n = mesh.vertex_count() as f32;
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for v in &mesh.vertices {
        cx += v[0];
        cy += v[1];
        cz += v[2];
    }
    cx /= n;
    cy /= n;
    cz /= n;
    let mut rmin = f32::MAX;
    let mut rmax = 0.0f32;
    for v in &mesh.vertices {
        let dx = v[0] - cx;
        let dy = v[1] - cy;
        let dz = v[2] - cz;
        let r = (dx * dx + dy * dy + dz * dz).sqrt();
        rmin = rmin.min(r);
        rmax = rmax.max(r);
    }
    assert!(
        rmax - rmin > 0.15,
        "radii too uniform for a non-sphere mesh: min={rmin} max={rmax}"
    );

    let scene = parse_scene(increment4_scene_json()).expect("increment4 JSON should parse");
    let mesh_body = scene
        .bodies
        .iter()
        .find(|b| matches!(b.shape, Shape::Mesh { .. }))
        .expect("scene must contain a mesh body");
    match &mesh_body.shape {
        Shape::Mesh { path, collider } => {
            assert!(path.ends_with(".obj"), "path={path}");
            assert_eq!(*collider, MeshCollider::ConvexHull);
        }
        _ => unreachable!(),
    }
    let has_prim = scene
        .bodies
        .iter()
        .any(|b| matches!(b.shape, Shape::Box { .. } | Shape::Sphere { .. }));
    assert!(has_prim, "demo must include primitives plus the mesh");
}

#[test]
fn mesh_body_has_contact_after_step() {
    let scene = increment4_scene();
    let mesh_ids: Vec<String> = scene
        .bodies
        .iter()
        .filter(|b| matches!(b.shape, Shape::Mesh { .. }))
        .map(|b| b.id.clone())
        .collect();
    assert!(!mesh_ids.is_empty());

    let dump = step_physics(&scene, INCREMENT4_STEPS, DEFAULT_DT).expect("physics");
    let hit = dump.contacts.iter().any(|c| {
        mesh_ids.iter().any(|id| c.body_a == *id || c.body_b == *id)
    });
    assert!(
        hit,
        "expected a contact involving the mesh body, contacts={:?}",
        dump.contacts
    );

    for b in &dump.bodies {
        if mesh_ids.iter().any(|id| *id == b.id) {
            assert!(
                b.collider == "convex_hull" || b.collider == "trimesh",
                "mesh body {} collider={}",
                b.id,
                b.collider
            );
        }
        if b.id == "ground" {
            assert_eq!(b.collider, "cuboid");
        }
        if b.id == "ball" {
            assert_eq!(b.collider, "ball");
        }
    }
}

#[test]
fn increment4_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment4.sh");
    assert!(script.is_file(), "scripts/increment4.sh must exist");

    let out = PathBuf::from("target/test-increment4-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment4(&out, INCREMENT4_STEPS, DEFAULT_DT, 160, 90).expect("run_increment4");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    let scene = parse_scene(&scene_txt).expect("written scene.json parses");
    assert!(scene.bodies.iter().any(|b| matches!(b.shape, Shape::Mesh { .. })));

    let phys_txt = fs::read_to_string(&paths.physics).unwrap();
    let v: serde_json::Value = serde_json::from_str(&phys_txt).unwrap();
    assert!(v["bodies"].as_array().unwrap().len() >= 3);
    assert!(v["contacts"].is_array());
    let mesh_state = v["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == "rock")
        .expect("rock in dump");
    let col = mesh_state["collider"].as_str().unwrap_or("");
    assert!(col == "convex_hull" || col == "trimesh", "collider={col}");
    let png_len = fs::metadata(&paths.frame).unwrap().len();
    assert!(png_len > 256, "png too small: {png_len}");
}

#[test]
fn sim_and_render_accept_mesh_bodies() {
    let scene = increment4_scene();
    let traj = simulate_trajectory(&scene, 3, 10, DEFAULT_DT).expect("sim trajectory with mesh");
    assert_eq!(traj.frames.len(), 3);
    assert!(traj.frames[0].bodies.iter().any(|b| b.id == "rock"));

    let img = render_scene(&scene, 80, 45);
    assert_eq!(img.width(), 80);
    assert_eq!(img.height(), 45);

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = manifest.join("target/test-increment4-cli");
    let _ = fs::remove_dir_all(&out);
    fs::create_dir_all(&out).unwrap();
    let scene_path = out.join("scene.json");
    fs::write(&scene_path, increment4_scene_json()).unwrap();

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
    assert!(status.success(), "agent-rig render with mesh exited {status}");
    assert!(frame_path.is_file());
}
