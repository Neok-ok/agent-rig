use std::fs;
use std::path::PathBuf;
use std::process::Command;

use agent_rig::{
    increment9_scene, increment9_scene_json, load_mesh, parse_scene, render_scene, run_increment9,
    simulate_trajectory, step_physics, Shape, DEFAULT_DT, INCREMENT9_STEPS,
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

fn pillar_gltf_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("meshes/pillar.gltf")
}

#[test]
fn gltf_file_has_pbr_metallic_roughness() {
    let path = pillar_gltf_path();
    assert!(path.is_file(), "meshes/pillar.gltf must be checked in");
    let txt = fs::read_to_string(&path).unwrap();
    let v: serde_json::Value = serde_json::from_str(&txt).expect("pillar.gltf is JSON");
    let mats = v["materials"]
        .as_array()
        .expect("glTF must have materials[]");
    assert!(!mats.is_empty(), "glTF must declare at least one material");
    let pbr = &mats[0]["pbrMetallicRoughness"];
    assert!(pbr.is_object(), "material must have pbrMetallicRoughness");
    let has_factor = pbr["baseColorFactor"].as_array().map(|a| a.len() >= 3) == Some(true);
    let has_tex = pbr.get("baseColorTexture").is_some();
    assert!(
        has_factor || has_tex,
        "pbrMetallicRoughness needs baseColorFactor and/or baseColorTexture"
    );
    let prim_mat = v["meshes"][0]["primitives"][0]["material"].as_u64();
    assert!(prim_mat.is_some(), "primitive must reference materials[i]");
}

#[test]
fn loaded_mesh_uses_gltf_factor_not_scene_json() {
    let mesh = load_mesh(&pillar_gltf_path()).expect("glTF should load");
    let gm = mesh
        .gltf_material
        .as_ref()
        .expect("loaded mesh must carry glTF pbrMetallicRoughness");
    let factor = gm.base_color_rgb();
    assert!(
        (factor[0] - 0.85).abs() < 1e-4
            && (factor[1] - 0.45).abs() < 1e-4
            && (factor[2] - 0.18).abs() < 1e-4,
        "loaded factor should be the copper in the file, got {factor:?}"
    );
    assert!(
        (gm.metallic_factor - 1.0).abs() < 1e-4,
        "metallicFactor {}",
        gm.metallic_factor
    );
    assert!(
        (gm.roughness_factor - 1.0).abs() < 1e-4,
        "roughnessFactor {}",
        gm.roughness_factor
    );

    let scene = increment9_scene();
    let body = gltf_body(&scene);
    let json_albedo = body.material.albedo;
    assert!(
        (json_albedo[0] - json_albedo[1]).abs() < 0.05
            && (json_albedo[1] - json_albedo[2]).abs() < 0.08,
        "scene JSON pillar fallback should be dull gray, got {json_albedo:?}"
    );
    let d0 = (json_albedo[0] - factor[0]).abs();
    let d1 = (json_albedo[1] - factor[1]).abs();
    let d2 = (json_albedo[2] - factor[2]).abs();
    assert!(
        d0 + d1 + d2 > 0.4,
        "JSON fallback must differ from glTF factor (json={json_albedo:?} gltf={factor:?})"
    );

    let resolved = scene
        .resolved_body_material(body)
        .expect("resolve glTF material");
    assert!(
        (resolved.albedo[0] - factor[0]).abs() < 1e-4
            && (resolved.albedo[1] - factor[1]).abs() < 1e-4
            && (resolved.albedo[2] - factor[2]).abs() < 1e-4,
        "resolved material must use the glTF factor, not JSON albedo {:?}",
        resolved.albedo
    );
    assert!(
        (resolved.metallic - 1.0).abs() < 1e-4,
        "resolved metallic should come from the file"
    );
}

#[test]
fn increment9_keeps_courtyard_and_gltf_body() {
    let scene = parse_scene(increment9_scene_json()).expect("increment9 JSON should parse");
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
        }
        other => panic!("expected mesh, got {other:?}"),
    }
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
    assert!(has_bowl && has_rock && has_ball, "keep the increment-8 set");
}

#[test]
fn contact_involving_gltf_body_after_step() {
    let scene = increment9_scene();
    let gltf_id = gltf_body(&scene).id.clone();
    let dump = step_physics(&scene, INCREMENT9_STEPS, DEFAULT_DT).expect("physics");
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

fn color_dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dr = a[0] - b[0];
    let dg = a[1] - b[1];
    let db = a[2] - b[2];
    (dr * dr + dg * dg + db * db).sqrt()
}

#[test]
fn increment9_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment9.sh");
    assert!(script.is_file(), "scripts/increment9.sh must exist");

    let out = PathBuf::from("target/test-increment9-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment9(&out, INCREMENT9_STEPS, DEFAULT_DT, 200, 112).expect("run_increment9");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    let scene = parse_scene(&scene_txt).expect("written scene.json parses");
    let body = gltf_body(&scene);
    assert!(
        matches!(&body.shape, Shape::Mesh { path, .. } if path.ends_with(".gltf") || path.ends_with(".glb"))
    );
    let json_albedo = body.material.albedo;

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

    // Isolated pillar, tight crop: pixels should track the glTF copper, not the JSON gray.
    let mut unit = increment9_scene();
    unit.bodies.retain(|b| b.id == "pillar");
    unit.camera.position = [1.10, 0.52, 1.85];
    unit.camera.look_at = [1.10, 0.50, 0.70];
    unit.camera.fov_y_deg = 28.0;
    let img = render_scene(&unit, 160, 120);
    let mut acc = [0.0f32; 3];
    let mut n = 0usize;
    let mut warm = 0usize;
    for p in img.pixels() {
        let r = p[0] as f32 / 255.0;
        let g = p[1] as f32 / 255.0;
        let b = p[2] as f32 / 255.0;
        let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        if luma < 0.05 || luma > 0.92 {
            continue;
        }
        if b > r + 0.06 && b > g {
            continue;
        }
        n += 1;
        if r > g + 0.03 && r > b + 0.06 {
            acc[0] += r;
            acc[1] += g;
            acc[2] += b;
            warm += 1;
        }
    }
    assert!(n > 80, "expected pillar pixels, got {n}");
    assert!(
        warm > 40,
        "pillar should read warm copper from the glTF factor (warm={warm} n={n})"
    );
    let mean = [acc[0] / warm as f32, acc[1] / warm as f32, acc[2] / warm as f32];
    // IBL on the chrome MR bands lifts the mean toward the sky, so compare tint
    // (copper R>G>B) rather than Euclidean distance to the factor swatch.
    assert!(
        mean[0] > mean[1] + 0.03 && mean[0] > mean[2] + 0.04,
        "warm pillar pixels should stay copper-tinted from the glTF factor, not JSON gray {json_albedo:?} (mean={mean:?})"
    );
    let _ = color_dist(mean, json_albedo);
}

#[test]
fn sim_and_render_load_gltf_materials() {
    let scene = increment9_scene();
    let gltf_id = gltf_body(&scene).id.clone();
    let resolved = scene
        .resolved_body_material(gltf_body(&scene))
        .expect("sim path resolves glTF material");
    assert!(
        (resolved.albedo[0] - 0.85).abs() < 1e-4,
        "sim/render loader must see the glTF factor"
    );

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
    let out = manifest.join("target/test-increment9-cli");
    let _ = fs::remove_dir_all(&out);
    fs::create_dir_all(&out).unwrap();
    let scene_path = out.join("scene.json");
    fs::write(&scene_path, increment9_scene_json()).unwrap();

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
        "agent-rig render with glTF materials exited {status}"
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
        "agent-rig sim with glTF materials exited {status}"
    );
    assert!(sim_out.join("trajectory.json").is_file());
    assert!(sim_out.join("frame.png").is_file());
}
