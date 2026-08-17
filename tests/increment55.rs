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
    increment51_scene_json, increment52_scene_json, increment53_scene_json,
    increment54_scene, increment54_scene_json, increment55_scene,
    increment55_scene_json, parse_scene, run_increment55, step_physics, Shape,
    DEFAULT_DT, INCREMENT54_STEPS, INCREMENT55_STEPS,
};

const COURTYARD_IDS: &[&str] = &[
    "gate", "bob", "cork", "lantern", "charm", "pane", "bench", "crate", "drawer",
    "lid", "platform", "rider", "rock", "pillar", "bar",
];

fn prior_scene_jsons() -> [(&'static str, &'static str); 37] {
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
        ("54", increment54_scene_json()),
    ]
}

#[test]
fn increment55_does_not_mutate_prior_scene_json() {
    for (name, json) in prior_scene_jsons() {
        assert!(
            !json.contains(r#""id": "exit""#),
            "increment {name} must not contain trigger id exit"
        );
    }
    assert!(
        !increment54_scene_json().contains("\"entered\""),
        "increment54 JSON must not contain entered"
    );
    let live54 = increment54_scene();
    let until54 = live54.play_until.as_ref().expect("increment54 play_until");
    assert_eq!(until54.kind, "picked_up");
    assert_eq!(until54.body, "token");
    assert!(live54.triggers.iter().any(|t| t.id == "token_zone"));
    assert!(live54.triggers.iter().all(|t| t.id != "exit"));

    let live55 = increment55_scene();
    assert!(live55.triggers.iter().any(|t| t.id == "exit"));
    let until55 = live55.play_until.as_ref().expect("increment55 play_until");
    assert_eq!(until55.kind, "entered");
    assert_eq!(until55.body, "exit");
}

#[test]
fn increment55_scene_adds_exit() {
    let parsed = parse_scene(increment55_scene_json()).expect("increment55 JSON should parse");
    assert_eq!(parsed.camera.position, [0.20, 1.15, 2.20]);
    assert_eq!(parsed.camera.look_at, [0.20, 0.28, 0.00]);
    assert!((parsed.camera.fov_y_deg - 40.0).abs() < 1e-5);
    let follow = parsed.camera.follow.as_ref().expect("lane camera must follow walker");
    assert_eq!(follow.body, "walker");
    assert_eq!(follow.offset, [-1.00, 0.80, 1.60]);
    let until = parsed.play_until.as_ref().expect("lane must author play_until");
    assert_eq!(until.kind, "entered");
    assert_eq!(until.body, "exit");
    let until_keys: serde_json::Value = serde_json::from_str(increment55_scene_json()).unwrap();
    let pu = until_keys["play_until"].as_object().expect("play_until object");
    assert_eq!(pu.len(), 2, "PlayUntil stays {{kind, body}}, got {pu:?}");
    assert!(pu.contains_key("kind") && pu.contains_key("body"));
    for name in ["53", "54"] {
        let json = prior_scene_jsons()
            .into_iter()
            .find(|(n, _)| *n == name)
            .unwrap()
            .1;
        let v: serde_json::Value = serde_json::from_str(json).unwrap();
        let obj = v["play_until"].as_object().expect("prior play_until");
        assert_eq!(obj.len(), 2, "increment{name} play_until extra keys: {obj:?}");
        assert_eq!(obj["kind"], "picked_up");
        assert_eq!(obj["body"], "token");
    }
    assert_eq!(parsed.pickups.len(), 1);
    assert_eq!(parsed.pickups[0].body, "token");
    assert_eq!(parsed.pickups[0].trigger, "token_zone");
    assert_eq!(parsed.pickups[0].by, "walker");
    assert!(parsed.bodies.iter().any(|b| b.id == "token"), "token still in bodies");

    let zone = parsed
        .triggers
        .iter()
        .find(|t| t.id == "token_zone")
        .expect("token_zone");
    match &zone.shape {
        Shape::Box { size } => assert_eq!(*size, [0.40, 0.40, 0.40]),
        _ => panic!("token_zone must be a box"),
    }
    let exit = parsed
        .triggers
        .iter()
        .find(|t| t.id == "exit")
        .expect("exit");
    match &exit.shape {
        Shape::Box { size } => assert_eq!(*size, [0.40, 0.40, 0.40]),
        _ => panic!("exit must be a box"),
    }
    assert_eq!(exit.position, [1.00, 0.20, 0.00]);

    let live54 = increment54_scene();
    let until54 = live54.play_until.as_ref().expect("increment54 play_until");
    assert_eq!(until54.kind, "picked_up");
    assert_eq!(until54.body, "token");
    assert!(live54.triggers.iter().all(|t| t.id != "exit"));
    for body in &parsed.bodies {
        assert!(!COURTYARD_IDS.contains(&body.id.as_str()), "courtyard id leaked: {}", body.id);
    }
}

#[test]
fn increment54_dump_still_pickup_stop() {
    let scene = increment54_scene();
    assert!(scene.triggers.iter().all(|t| t.id != "exit"));
    let dump = step_physics(&scene, INCREMENT54_STEPS, DEFAULT_DT).expect("increment54 physics");
    assert_eq!(dump.steps, 67, "increment54 dump.steps still 67, got {}", dump.steps);
    let stopped = dump.stopped.as_ref().expect("increment54 dump stopped");
    assert_eq!(stopped.kind, "picked_up");
    assert_eq!(stopped.body, "token");
    assert_eq!(stopped.at_step, 66);
    let walker = dump.bodies.iter().find(|b| b.id == "walker").expect("walker");
    assert!(
        (walker.position[0] - 0.415).abs() < 0.08,
        "increment54 walker.x ≈ 0.415, got {}",
        walker.position[0]
    );
    let v = serde_json::to_value(&dump).expect("serialize increment54 dump");
    assert!(v.get("stopped").is_some(), "serialized increment54 dump has stopped");
}

#[test]
fn increment55_physics_entered() {
    let dump = step_physics(&increment55_scene(), INCREMENT55_STEPS, DEFAULT_DT)
        .expect("increment55 physics");
    assert!(
        (85..=115).contains(&dump.steps),
        "dump.steps should be 85..=115, got {}",
        dump.steps
    );
    let stopped = dump.stopped.as_ref().expect("dump.stopped");
    assert_eq!(stopped.kind, "entered");
    assert_eq!(stopped.body, "exit");
    assert!(dump.bodies.iter().all(|b| b.id != "token"), "token should be picked up");
    assert!(dump.bodies.iter().any(|b| b.id == "walker"));
    assert!(dump.bodies.iter().any(|b| b.id == "ground"));
    assert!(dump.bodies.iter().any(|b| b.id == "block"));
    assert!(dump.picked_up.iter().any(|p| p.id == "token" && p.by == "walker"));
    assert!(
        dump.overlaps
            .iter()
            .any(|o| o.trigger == "exit" && o.body == "walker"),
        "overlaps must include exit/walker, got {:?}",
        dump.overlaps
    );
    assert!(dump.spawned.is_empty());
    assert!(dump.despawned.is_empty());
    let walker = dump.bodies.iter().find(|b| b.id == "walker").expect("dump walker");
    let ctrl = dump.controllers.iter().find(|c| c.id == "walker").expect("walker controller");
    assert!(ctrl.grounded);
    assert!(
        walker.position[0] > 0.65,
        "walker.x should be past the token (>0.65), got {}",
        walker.position[0]
    );
    assert!(
        walker.position[1] >= 0.14 && walker.position[1] <= 0.28,
        "walker.y on floor, got {}",
        walker.position[1]
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
}

#[test]
fn increment55_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment55.sh");
    assert!(script.is_file(), "scripts/increment55.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment55-threejs.sh");
    assert!(three.is_file(), "scripts/increment55-threejs.sh must exist");
    let out = PathBuf::from("target/test-increment55-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment55(&out, INCREMENT55_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment55");
    assert!(paths.scene.is_file() && paths.physics.is_file() && paths.frame.is_file());
    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    assert!(scene_txt.contains("\"entered\""));
    assert!(scene_txt.contains("\"exit\""));
    assert!(scene_txt.contains("\"play_until\""));
    let v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&paths.physics).unwrap()).unwrap();
    assert_eq!(v["stopped"]["kind"], "entered");
    assert!(v.get("camera").is_some(), "written dump must have camera");
    let bodies = v["bodies"].as_array().unwrap();
    assert!(bodies.iter().all(|b| b["id"] != "token"));
    assert!(bodies.iter().any(|b| b["id"] == "walker"));
    assert!(fs::metadata(&paths.frame).unwrap().len() > 256);
}
