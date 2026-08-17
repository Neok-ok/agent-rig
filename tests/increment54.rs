use std::fs;
use std::path::PathBuf;

use agent_rig::{
    increment18_scene_json, increment19_scene_json, increment20_scene_json,
    increment21_scene_json, increment22_scene_json, increment23_scene_json,
    increment24_scene_json, increment25_scene_json, increment26_scene_json,
    increment27_scene_json, increment28_scene_json, increment29_scene_json,
    increment30_scene_json, increment31_scene_json, increment32_scene_json,
    increment33_scene_json, increment34_scene_json, increment35_scene_json,
    increment36_scene_json, increment37_scene_json, increment38_scene_json,
    increment39_scene_json, increment40_scene_json, increment41_scene_json,
    increment42_scene_json, increment43_scene_json, increment44_scene_json,
    increment45_scene_json, increment46_scene_json, increment47_scene_json,
    increment48_scene_json, increment49_scene_json, increment50_scene_json,
    increment51_scene_json, increment52_scene_json, increment53_scene,
    increment53_scene_json, increment54_scene, increment54_scene_json,
    parse_scene, run_increment54, step_physics, Shape, DEFAULT_DT,
    INCREMENT53_STEPS, INCREMENT54_STEPS,
};

fn body_by_id<'a>(scene: &'a agent_rig::Scene, id: &str) -> &'a agent_rig::Body {
    scene
        .bodies
        .iter()
        .find(|b| b.id == id)
        .unwrap_or_else(|| panic!("missing body {id}"))
}

const COURTYARD_IDS: &[&str] = &[
    "gate", "bob", "cork", "lantern", "charm", "pane", "bench", "crate", "drawer",
    "lid", "platform", "rider", "rock", "pillar", "bar",
];

fn prior_scene_jsons() -> [(&'static str, &'static str); 36] {
    [
        ("18", increment18_scene_json()),
        ("19", increment19_scene_json()),
        ("20", increment20_scene_json()),
        ("21", increment21_scene_json()),
        ("22", increment22_scene_json()),
        ("23", increment23_scene_json()),
        ("24", increment24_scene_json()),
        ("25", increment25_scene_json()),
        ("26", increment26_scene_json()),
        ("27", increment27_scene_json()),
        ("28", increment28_scene_json()),
        ("29", increment29_scene_json()),
        ("30", increment30_scene_json()),
        ("31", increment31_scene_json()),
        ("32", increment32_scene_json()),
        ("33", increment33_scene_json()),
        ("34", increment34_scene_json()),
        ("35", increment35_scene_json()),
        ("36", increment36_scene_json()),
        ("37", increment37_scene_json()),
        ("38", increment38_scene_json()),
        ("39", increment39_scene_json()),
        ("40", increment40_scene_json()),
        ("41", increment41_scene_json()),
        ("42", increment42_scene_json()),
        ("43", increment43_scene_json()),
        ("44", increment44_scene_json()),
        ("45", increment45_scene_json()),
        ("46", increment46_scene_json()),
        ("47", increment47_scene_json()),
        ("48", increment48_scene_json()),
        ("49", increment49_scene_json()),
        ("50", increment50_scene_json()),
        ("51", increment51_scene_json()),
        ("52", increment52_scene_json()),
        ("53", increment53_scene_json()),
    ]
}

fn has_lane_ground_size(json: &str) -> bool {
    json.contains(r#""size": [3.0, 0.04, 1.6]"#)
        || json.contains(r#""size": [3.00, 0.04, 1.60]"#)
        || json.contains("[3.0, 0.04, 1.6]")
        || json.contains("[3.00, 0.04, 1.60]")
}

fn has_lane_rest_camera(json: &str) -> bool {
    json.contains("[0.20, 1.15, 2.20]") || json.contains("[0.2, 1.15, 2.2]")
}

#[test]
fn increment54_does_not_mutate_prior_scene_json() {
    for (name, json) in prior_scene_jsons() {
        assert!(
            !has_lane_rest_camera(json),
            "increment {name} must not contain lane rest camera [0.20, 1.15, 2.20]"
        );
        assert!(
            !has_lane_ground_size(json),
            "increment {name} must not contain lane ground size [3.00, 0.04, 1.60]"
        );
        if name != "53" {
            assert!(
                !json.contains("\"play_until\""),
                "increment {name} must not contain play_until"
            );
        }
    }
    let live53 = increment53_scene();
    assert!(live53.play_until.is_some(), "increment53_scene still has play_until");
    assert!(live53.bodies.iter().any(|b| b.id == "bar"), "increment53 still has bar");
    assert!(live53.bodies.iter().any(|b| b.id == "gate"), "increment53 still has gate");
}

#[test]
fn increment54_scene_is_a_lane() {
    let parsed = parse_scene(increment54_scene_json()).expect("increment54 JSON should parse");
    assert_eq!(parsed.camera.position, [0.20, 1.15, 2.20]);
    assert_eq!(parsed.camera.look_at, [0.20, 0.28, 0.00]);
    assert!((parsed.camera.fov_y_deg - 40.0).abs() < 1e-5);
    let follow = parsed.camera.follow.as_ref().expect("lane camera must follow walker");
    assert_eq!(follow.body, "walker");
    assert_eq!(follow.offset, [-1.00, 0.80, 1.60]);
    let until = parsed.play_until.as_ref().expect("lane must author play_until");
    assert_eq!(until.kind, "picked_up");
    assert_eq!(until.body, "token");
    assert_eq!(parsed.pickups.len(), 1);
    assert_eq!(parsed.pickups[0].body, "token");
    assert_eq!(parsed.pickups[0].trigger, "token_zone");
    assert_eq!(parsed.pickups[0].by, "walker");
    assert!(parsed.bodies.iter().any(|b| b.id == "token"), "token must be in bodies from t=0");
    assert!(parsed.spawns.is_empty(), "lane has no spawns");
    assert!(parsed.despawns.is_empty(), "lane has no despawns");
    assert!(parsed.joints.is_empty());
    assert!(parsed.impulses.is_empty());
    assert!(parsed.raycasts.is_empty());
    assert!(parsed.shapecasts.is_empty());
    assert!(!parsed.record_contact_events);

    let allowed = ["ground", "walker", "token", "block"];
    assert!(
        parsed.bodies.len() >= 3 && parsed.bodies.len() <= 4,
        "bodies should be ground/walker/token plus at most one prop, got {}",
        parsed.bodies.len()
    );
    for body in &parsed.bodies {
        assert!(allowed.contains(&body.id.as_str()), "unexpected lane body {}", body.id);
        assert!(!COURTYARD_IDS.contains(&body.id.as_str()), "courtyard id leaked: {}", body.id);
    }

    let ground = body_by_id(&parsed, "ground");
    match ground.shape {
        Shape::Box { size } => assert_eq!(size, [3.00, 0.04, 1.60]),
        _ => panic!("ground must be a box, got {:?}", ground.shape),
    }
    assert_eq!(ground.position, [0.00, -0.02, 0.00]);
    assert!(ground.mass.abs() < 1e-5);
    assert_eq!(ground.material.albedo, [0.42, 0.40, 0.36]);
    assert_eq!(ground.collision_groups.membership, 1);
    assert_eq!(ground.collision_groups.filter, 0xFFFF);

    let walker = body_by_id(&parsed, "walker");
    match walker.shape {
        Shape::Box { size } => assert_eq!(size, [0.18, 0.36, 0.18]),
        _ => panic!("walker must be a box, got {:?}", walker.shape),
    }
    assert_eq!(walker.position, [-0.20, 0.20, 0.00]);
    assert!(walker.mass.abs() < 1e-5);
    let v = walker
        .controller
        .as_ref()
        .expect("walker must author a controller")
        .desired_velocity;
    assert_eq!(v, [0.55, 0.0, 0.0]);
    assert_eq!(walker.collision_groups.membership, 2);
    assert_eq!(walker.collision_groups.filter, 1);
    let alb = walker.material.albedo;
    assert!(
        alb[0] > 0.7 && alb[1] < 0.4 && alb[2] > 0.3,
        "walker albedo should be warm coral/magenta, got {alb:?}"
    );

    let token = body_by_id(&parsed, "token");
    match token.shape {
        Shape::Sphere { radius } => {
            assert!((radius - 0.10).abs() < 1e-5, "token radius {radius}")
        }
        _ => panic!("token must be a sphere, got {:?}", token.shape),
    }
    assert_eq!(token.position, [0.70, 0.12, 0.00]);
    assert!(token.mass.abs() < 1e-5);
    assert_eq!(token.material.albedo, [0.95, 0.78, 0.22]);
    assert!((token.material.metallic - 0.85).abs() < 1e-5);
    assert!((token.material.roughness - 0.35).abs() < 1e-5);
    assert_eq!(token.collision_groups.membership, 4);
    assert_eq!(token.collision_groups.filter, 0xFFFF);

    let zone = parsed
        .triggers
        .iter()
        .find(|t| t.id == "token_zone")
        .expect("token_zone");
    match &zone.shape {
        Shape::Box { size } => assert_eq!(*size, [0.40, 0.40, 0.40]),
        _ => panic!("token_zone must be a box"),
    }
    assert_eq!(zone.position, [0.70, 0.12, 0.00]);

    let live53 = increment53_scene();
    assert!(live53.bodies.iter().any(|b| b.id == "bar"));
    assert!(live53.play_until.is_some());
    assert_eq!(live53.camera.position, [1.85, 1.35, 3.15]);
}

#[test]
fn increment53_dump_still_courtyard() {
    let dump = step_physics(&increment53_scene(), INCREMENT53_STEPS, DEFAULT_DT)
        .expect("increment53 physics");
    assert!(
        (30..=31).contains(&dump.steps),
        "increment53 dump.steps 30..=31, got {}",
        dump.steps
    );
    assert!(dump.bodies.iter().any(|b| b.id == "bar"), "increment53 dump still has bar");
    assert!(dump.bodies.iter().all(|b| b.id != "token"), "increment53 dump has no token");
    let stopped = dump.stopped.as_ref().expect("increment53 dump stopped");
    assert_eq!(stopped.kind, "picked_up");
    assert_eq!(stopped.body, "token");
}

#[test]
fn increment54_physics_lane() {
    let dump = step_physics(&increment54_scene(), INCREMENT54_STEPS, DEFAULT_DT)
        .expect("increment54 physics");
    assert!(
        (30..=110).contains(&dump.steps),
        "dump.steps should be 30..=110, got {}",
        dump.steps
    );
    let stopped = dump.stopped.as_ref().expect("dump.stopped");
    assert_eq!(stopped.kind, "picked_up");
    assert_eq!(stopped.body, "token");
    assert_eq!(stopped.at_step, dump.picked_up[0].at_step);
    assert!(dump.bodies.iter().all(|b| b.id != "token"), "token should be picked up");
    assert!(dump.bodies.iter().any(|b| b.id == "walker"));
    assert!(dump.bodies.iter().any(|b| b.id == "ground"));
    assert!(dump.picked_up.iter().any(|p| p.id == "token" && p.by == "walker"));
    assert!(dump.spawned.is_empty());
    assert!(dump.despawned.is_empty());
    let walker = dump.bodies.iter().find(|b| b.id == "walker").expect("dump walker");
    let ctrl = dump.controllers.iter().find(|c| c.id == "walker").expect("walker controller");
    assert!(ctrl.grounded);
    assert!(
        walker.position[1] >= 0.14 && walker.position[1] <= 0.28,
        "walker.y on floor, got {}",
        walker.position[1]
    );
    assert!(
        walker.position[0] > -0.20,
        "walker should have walked +x from -0.20, got {}",
        walker.position[0]
    );
    let cam = dump.camera.as_ref().expect("dump.camera");
    let expect_pos = [
        walker.position[0] - 1.00,
        walker.position[1] + 0.80,
        walker.position[2] + 1.60,
    ];
    let expect_look = [
        walker.position[0],
        walker.position[1] + 0.15,
        walker.position[2],
    ];
    for i in 0..3 {
        assert!(
            (cam.position[i] - expect_pos[i]).abs() < 0.08,
            "dump.camera.position[{i}] got {} want {}",
            cam.position[i],
            expect_pos[i]
        );
        assert!(
            (cam.look_at[i] - expect_look[i]).abs() < 0.08,
            "dump.camera.look_at[{i}] got {} want {}",
            cam.look_at[i],
            expect_look[i]
        );
    }
    assert!(dump.bodies.iter().all(|b| b.id != "bar"), "lane dump must not have bar");
}

#[test]
fn increment54_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment54.sh");
    assert!(script.is_file(), "scripts/increment54.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment54-threejs.sh");
    assert!(three.is_file(), "scripts/increment54-threejs.sh must exist");
    let out = PathBuf::from("target/test-increment54-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment54(&out, INCREMENT54_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment54");
    assert!(paths.scene.is_file() && paths.physics.is_file() && paths.frame.is_file());
    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    assert!(scene_txt.contains("\"play_until\""));
    assert!(scene_txt.contains("\"follow\""));
    assert!(!scene_txt.contains("\"bar\""));
    let v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&paths.physics).unwrap()).unwrap();
    assert!(v.get("stopped").is_some(), "written dump must have stopped");
    assert!(v.get("camera").is_some(), "written dump must have camera");
    let bodies = v["bodies"].as_array().unwrap();
    assert!(bodies.iter().all(|b| b["id"] != "token"));
    assert!(bodies.iter().any(|b| b["id"] == "walker"));
    assert!(fs::metadata(&paths.frame).unwrap().len() > 256);
}
