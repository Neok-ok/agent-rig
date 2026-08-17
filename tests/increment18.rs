use std::fs;
use std::path::PathBuf;

use agent_rig::{
    area_light_visibility, increment18_scene, increment18_scene_json, parse_scene, run_increment18,
    step_physics, Light, Shape, DEFAULT_DT, INCREMENT18_STEPS,
};

fn gltf_bodies(scene: &agent_rig::Scene) -> Vec<&agent_rig::Body> {
    scene
        .bodies
        .iter()
        .filter(|b| match &b.shape {
            Shape::Mesh { path, .. } => path.ends_with(".gltf") || path.ends_with(".glb"),
            _ => false,
        })
        .collect()
}

fn body_by_id<'a>(scene: &'a agent_rig::Scene, id: &str) -> &'a agent_rig::Body {
    scene
        .bodies
        .iter()
        .find(|b| b.id == id)
        .unwrap_or_else(|| panic!("missing body {id}"))
}

fn area_light(scene: &agent_rig::Scene) -> ([f32; 3], [f32; 2], [f32; 3], f32, [f32; 3]) {
    for light in &scene.lights {
        if let Light::Area {
            position,
            size,
            color,
            intensity,
            normal,
        } = light
        {
            return (*position, *size, *color, *intensity, *normal);
        }
    }
    panic!("scene missing area light");
}

#[test]
fn scene_has_area_light_and_courtyard() {
    let scene = parse_scene(increment18_scene_json()).expect("increment18 JSON should parse");
    let (pos, size, color, intensity, _n) = area_light(&scene);
    assert!(
        pos.iter().any(|c| c.abs() > 1e-4),
        "area light needs a position, got {pos:?}"
    );
    assert!(
        size[0] > 0.5 && size[1] > 0.4,
        "softness must come from an authored size, got {size:?}"
    );
    assert!(
        color[0] > 0.2 && color[1] > 0.2 && color[2] > 0.05,
        "area light color {color:?}"
    );
    assert!(intensity > 1.0, "area light intensity {intensity}");

    let has_dir = scene
        .lights
        .iter()
        .any(|l| matches!(l, Light::Directional { .. }));
    assert!(has_dir, "keep the directional");

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
    let has_pillar = body_by_id(&scene, "pillar").id == "pillar";
    let has_pane = matches!(
        &body_by_id(&scene, "pane").shape,
        Shape::Mesh { path, .. } if path.contains("pane")
    );
    assert!(
        has_bowl && has_rock && has_ball && has_pillar && has_pane,
        "keep the increment-17 courtyard including the pane"
    );
    let gltfs = gltf_bodies(&scene);
    assert!(
        gltfs.len() >= 2,
        "expect pillar + pane glTF bodies, got {}",
        gltfs.len()
    );
    for b in &scene.bodies {
        let v = b.linear_velocity;
        let speed = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        assert!(
            speed < 1e-6,
            "increment 18 is a still; body {} should have zero velocity, got {v:?}",
            b.id
        );
    }
}

#[test]
fn contact_involving_gltf_body_after_step() {
    let scene = increment18_scene();
    let dump = step_physics(&scene, INCREMENT18_STEPS, DEFAULT_DT).expect("physics");
    let hit = dump
        .contacts
        .iter()
        .any(|c| c.body_a == "pillar" || c.body_b == "pillar");
    assert!(
        hit,
        "expected a contact involving the glTF pillar, contacts={:?}",
        dump.contacts
    );
    let body = dump
        .bodies
        .iter()
        .find(|b| b.id == "pillar")
        .expect("dump missing pillar");
    assert!(
        body.collider == "convex_hull" || body.collider == "trimesh",
        "glTF body collider should be convex_hull or trimesh, got {}",
        body.collider
    );
}

#[test]
fn increment18_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment18.sh");
    assert!(script.is_file(), "scripts/increment18.sh must exist");

    let out = PathBuf::from("target/test-increment18-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment18(&out, INCREMENT18_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment18");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    let scene = parse_scene(&scene_txt).expect("written scene.json parses");
    let (_pos, size, _color, intensity, _n) = area_light(&scene);
    assert!(
        size[0] > 0.5 && size[1] > 0.4,
        "written scene must author area size, got {size:?}"
    );
    assert!(intensity > 0.0);
    assert!(
        scene
            .bodies
            .iter()
            .any(|b| matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("pane")))
    );
    assert!(
        scene
            .bodies
            .iter()
            .any(|b| matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("pillar")))
    );

    let phys_txt = fs::read_to_string(&paths.physics).unwrap();
    let v: serde_json::Value = serde_json::from_str(&phys_txt).unwrap();
    assert!(v["bodies"].as_array().unwrap().len() >= 5);
    assert!(v["contacts"].is_array());
    let pillar_state = v["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == "pillar")
        .expect("dump should record the pillar");
    let col = pillar_state["collider"].as_str().unwrap_or("");
    assert!(
        col == "convex_hull" || col == "trimesh",
        "dump must record collider type for the glTF pillar, got {col}"
    );
    let contacts = v["contacts"].as_array().unwrap();
    let hit = contacts
        .iter()
        .any(|c| c["body_a"] == "pillar" || c["body_b"] == "pillar");
    assert!(hit, "dump contacts should include the glTF pillar");
    let png_len = fs::metadata(&paths.frame).unwrap().len();
    assert!(png_len > 256, "png too small: {png_len}");
}

#[test]
fn softness_comes_from_authored_size() {
    // Sphere sitting on a plane; overhead area light. A floor point just outside
    // the contact is a penumbra for a large panel and binary for a tiny one.
    let json = r#"{
      "camera": { "position": [0, 2, 6], "look_at": [0, 0.3, 0], "fov_y_deg": 40 },
      "lights": [
        { "type": "directional", "direction": [0, -1, 0], "color": [1, 1, 1], "intensity": 0.15 },
        { "type": "area", "position": [0.0, 1.6, 0.0], "size": [1.2, 0.8], "color": [1.0, 0.8, 0.5], "intensity": 20.0, "normal": [0.0, -1.0, 0.0] }
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
          "id": "ball",
          "shape": { "type": "sphere", "radius": 0.4 },
          "position": [0.0, 0.4, 0.0],
          "mass": 0,
          "material": { "albedo": [0.8, 0.2, 0.2], "roughness": 0.4, "metallic": 0.0 }
        }
      ]
    }"#;
    let scene = parse_scene(json).expect("penumbra scene");
    let n = [0.0, 1.0, 0.0];
    let lamp = [0.0, 1.6, 0.0];
    let large = [1.2, 0.8];
    let tiny = [0.02, 0.02];
    let down = [0.0, -1.0, 0.0];
    // Deep under the ball: umbra for both sizes.
    let under = [0.0, 0.02, 0.0];
    let vis_under_large = area_light_visibility(&scene, under, n, lamp, large, down);
    assert!(
        vis_under_large < 0.15,
        "under the ball should be umbra, vis={vis_under_large}"
    );
    // Just outside the sphere (r=0.4) on the floor: large panel has a penumbra.
    let edge = [0.48, 0.02, 0.0];
    let vis_large = area_light_visibility(&scene, edge, n, lamp, large, down);
    let vis_tiny = area_light_visibility(&scene, edge, n, lamp, tiny, down);
    assert!(
        vis_large > 0.05 && vis_large < 0.95,
        "authored 1.2x0.8 size should make a penumbra at {edge:?}, vis={vis_large}"
    );
    assert!(
        vis_tiny < 0.05 || vis_tiny > 0.95,
        "tiny size should be nearly a hard umbra/lit, vis={vis_tiny}"
    );
    // Far from the ball: fully lit.
    let far = [1.8, 0.02, 0.0];
    let vis_far = area_light_visibility(&scene, far, n, lamp, large, down);
    assert!(vis_far > 0.85, "far floor should see the panel, vis={vis_far}");
}
