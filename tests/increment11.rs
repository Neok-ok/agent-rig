use std::fs;
use std::path::PathBuf;
use std::process::Command;

use agent_rig::{
    increment11_scene, increment11_scene_json, parse_scene, point_light_occluded, render_scene,
    run_increment11, simulate_trajectory, step_physics, Light, Shape, DEFAULT_DT, INCREMENT11_STEPS,
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

fn point_light_pos(scene: &agent_rig::Scene) -> [f32; 3] {
    for light in &scene.lights {
        if let Light::Point { position, .. } = light {
            return *position;
        }
    }
    panic!("scene missing point light");
}

#[test]
fn scene_has_point_and_directional_and_courtyard() {
    let scene = parse_scene(increment11_scene_json()).expect("increment11 JSON should parse");
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
    assert!(has_dir, "increment 11 keeps the directional");
    assert!(has_point, "increment 11 keeps the point light");

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
        "keep the increment-10 bowl + rock + ball + copper pillar"
    );
}

#[test]
fn contact_involving_gltf_body_after_step() {
    let scene = increment11_scene();
    let gltf_id = gltf_body(&scene).id.clone();
    let dump = step_physics(&scene, INCREMENT11_STEPS, DEFAULT_DT).expect("physics");
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
fn increment11_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment11.sh");
    assert!(script.is_file(), "scripts/increment11.sh must exist");

    let out = PathBuf::from("target/test-increment11-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment11(&out, INCREMENT11_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment11");
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
fn point_light_shadow_ray_occluded_behind_box() {
    let json = r#"{
      "camera": { "position": [0, 2, 6], "look_at": [0, 0.3, 0], "fov_y_deg": 40 },
      "lights": [
        { "type": "directional", "direction": [0, -1, 0], "color": [1, 1, 1], "intensity": 0.2 },
        { "type": "point", "position": [2.0, 1.2, 0.0], "color": [1.0, 0.8, 0.5], "intensity": 12.0 }
      ],
      "bodies": [
        {
          "id": "ground",
          "shape": { "type": "box", "size": [10, 0.2, 10] },
          "position": [0, -0.1, 0],
          "mass": 0,
          "material": { "albedo": [0.5, 0.5, 0.5], "roughness": 0.8, "metallic": 0.0 }
        },
        {
          "id": "wall",
          "shape": { "type": "box", "size": [0.3, 1.6, 2.0] },
          "position": [0.0, 0.8, 0.0],
          "mass": 0,
          "material": { "albedo": [0.6, 0.6, 0.6], "roughness": 0.7, "metallic": 0.0 }
        }
      ]
    }"#;
    let scene = parse_scene(json).expect("occluder scene");
    let lamp = [2.0, 1.2, 0.0];
    let n = [0.0, 1.0, 0.0];
    assert!(
        point_light_occluded(&scene, [-1.2, 0.02, 0.0], n, lamp),
        "floor behind the wall should be shadowed from the point light"
    );
    assert!(
        !point_light_occluded(&scene, [1.2, 0.02, 0.0], n, lamp),
        "floor on the lamp side of the wall should not be occluded"
    );
}

#[test]
fn courtyard_floor_behind_pillar_is_shadowed() {
    let scene = increment11_scene();
    let lamp = point_light_pos(&scene);
    let pillar = scene
        .bodies
        .iter()
        .find(|b| b.id == "pillar")
        .expect("pillar");
    // Project lamp -> pillar mid onto the bowl floor (y ~ 0.02).
    let mid = [
        pillar.position[0],
        pillar.position[1] + 0.40,
        pillar.position[2],
    ];
    let d = [mid[0] - lamp[0], mid[1] - lamp[1], mid[2] - lamp[2]];
    let t = if d[1].abs() < 1e-4 {
        2.0
    } else {
        (0.02 - lamp[1]) / d[1]
    };
    let far = [lamp[0] + d[0] * t, 0.02, lamp[2] + d[2] * t];
    let n = [0.0, 1.0, 0.0];
    assert!(
        point_light_occluded(&scene, far, n, lamp),
        "floor point {far:?} behind the pillar from lamp {lamp:?} should be occluded"
    );
    // Lamp-side floor, toward the camera from the lamp.
    let near = [lamp[0] - 0.15, 0.02, lamp[2] + 0.25];
    assert!(
        !point_light_occluded(&scene, near, n, lamp),
        "floor on the lamp side {near:?} should see the lamp"
    );
}

#[test]
fn png_has_darker_umbra_than_nearby_lit_floor() {
    let scene = increment11_scene();
    let img = render_scene(&scene, 200, 112);
    let w = img.width() as f32;
    let h = img.height() as f32;
    // Screen-space: pillar sits camera-right; its point-light umbra falls on the
    // bowl floor just to the +X / -Z side of the pillar (away from the lamp).
    let mut umbra_acc = 0.0f32;
    let mut umbra_n = 0usize;
    let mut lit_acc = 0.0f32;
    let mut lit_n = 0usize;
    for (x, y, p) in img.enumerate_pixels() {
        let u = (x as f32 + 0.5) / w;
        let v = (y as f32 + 0.5) / h;
        let r = p[0] as f32 / 255.0;
        let g = p[1] as f32 / 255.0;
        let b = p[2] as f32 / 255.0;
        let l = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        if l < 0.03 {
            continue;
        }
        if v < 0.40 {
            continue;
        }
        // Floor umbra to the camera-right of the pillar (away from the lamp).
        if (0.61..=0.70).contains(&u) && (0.62..=0.72).contains(&v) {
            umbra_acc += l;
            umbra_n += 1;
        }
        // Nearby lit floor under the lamp (camera-left of the pillar).
        if (0.42..=0.50).contains(&u) && (0.62..=0.72).contains(&v) {
            lit_acc += l;
            lit_n += 1;
        }
    }
    assert!(umbra_n > 10, "expected umbra floor pixels, got {umbra_n}");
    assert!(lit_n > 10, "expected lit floor pixels, got {lit_n}");
    let umbra = umbra_acc / umbra_n as f32;
    let lit = lit_acc / lit_n as f32;
    assert!(
        umbra + 0.03 < lit,
        "point-light umbra should be darker than nearby lit floor (umbra={umbra:.3} lit={lit:.3})"
    );
}

#[test]
fn sim_and_render_load_point_light() {
    let scene = increment11_scene();
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
    let out = manifest.join("target/test-increment11-cli");
    let _ = fs::remove_dir_all(&out);
    fs::create_dir_all(&out).unwrap();
    let scene_path = out.join("scene.json");
    fs::write(&scene_path, increment11_scene_json()).unwrap();

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
