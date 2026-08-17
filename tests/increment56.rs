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
    increment54_scene_json, increment55_scene, increment55_scene_json,
    increment56_scene, increment56_scene_json, parse_scene, run_increment56,
    step_physics, DEFAULT_DT, INCREMENT55_STEPS, INCREMENT56_STEPS,
};

fn prior_scene_jsons() -> [(&'static str, &'static str); 38] {
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
        ("55", increment55_scene_json()),
    ]
}

#[test]
fn increment56_does_not_mutate_prior() {
    for (name, json) in prior_scene_jsons() {
        assert!(
            !json.contains("\"hold\""),
            "increment {name} JSON must not contain \"hold\""
        );
        assert!(
            !json.contains("hold_offset"),
            "increment {name} JSON must not contain hold_offset"
        );
    }
    let live55 = increment55_scene();
    assert!(!live55.pickups[0].hold, "increment55 pickup must stay hold=false");
    assert_eq!(live55.pickups[0].hold_offset, [0.0, 0.0, 0.0]);
    assert!(live55.triggers.iter().any(|t| t.id == "exit"));
    let until55 = live55.play_until.as_ref().expect("increment55 play_until");
    assert_eq!(until55.kind, "entered");
    assert_eq!(until55.body, "exit");

    let live56 = increment56_scene();
    assert!(live56.pickups[0].hold);
    assert_eq!(live56.pickups[0].hold_offset, [0.16, 0.22, 0.00]);
}

#[test]
fn increment56_scene_adds_hold() {
    let parsed = parse_scene(increment56_scene_json()).expect("increment56 JSON should parse");
    assert_eq!(parsed.pickups.len(), 1);
    assert_eq!(parsed.pickups[0].body, "token");
    assert_eq!(parsed.pickups[0].trigger, "token_zone");
    assert_eq!(parsed.pickups[0].by, "walker");
    assert!(parsed.pickups[0].hold);
    assert_eq!(parsed.pickups[0].hold_offset, [0.16, 0.22, 0.00]);
    let until = parsed.play_until.as_ref().expect("lane must author play_until");
    assert_eq!(until.kind, "entered");
    assert_eq!(until.body, "exit");
    let follow = parsed.camera.follow.as_ref().expect("lane camera must follow walker");
    assert_eq!(follow.body, "walker");
    assert_eq!(follow.offset, [-1.00, 0.80, 1.60]);
    assert!(parsed.bodies.iter().any(|b| b.id == "token"), "token still in bodies");
    assert!(parsed.triggers.iter().any(|t| t.id == "exit"));
    assert!(
        !increment55_scene_json().contains("\"hold\""),
        "increment55 JSON must not contain hold"
    );
}

#[test]
fn increment55_dump_still_despawns() {
    let dump = step_physics(&increment55_scene(), INCREMENT55_STEPS, DEFAULT_DT)
        .expect("increment55 physics");
    assert!(
        dump.steps == 100 || (85..=115).contains(&dump.steps),
        "increment55 dump.steps still ~100, got {}",
        dump.steps
    );
    let stopped = dump.stopped.as_ref().expect("increment55 dump stopped");
    assert_eq!(stopped.kind, "entered");
    assert_eq!(stopped.body, "exit");
    assert!(dump.bodies.iter().all(|b| b.id != "token"), "token should be despawned");
    assert!(dump.held.is_empty());
    let v = serde_json::to_value(&dump).expect("serialize increment55 dump");
    assert!(v.get("held").is_none(), "serialized increment55 dump must omit held");
}

#[test]
fn increment56_physics_hold() {
    let dump = step_physics(&increment56_scene(), INCREMENT56_STEPS, DEFAULT_DT)
        .expect("increment56 physics");
    assert!(
        (85..=115).contains(&dump.steps),
        "dump.steps should be 85..=115, got {}",
        dump.steps
    );
    let stopped = dump.stopped.as_ref().expect("dump.stopped");
    assert_eq!(stopped.kind, "entered");
    assert_eq!(stopped.body, "exit");
    assert!(dump.bodies.iter().any(|b| b.id == "token"), "token should be held");
    assert!(dump.bodies.iter().any(|b| b.id == "walker"));
    assert!(dump.bodies.iter().any(|b| b.id == "ground"));
    assert!(dump.bodies.iter().any(|b| b.id == "block"));
    let walker = dump.bodies.iter().find(|b| b.id == "walker").expect("dump walker");
    let token = dump.bodies.iter().find(|b| b.id == "token").expect("dump token");
    let expect = [
        walker.position[0] + 0.16,
        walker.position[1] + 0.22,
        walker.position[2] + 0.00,
    ];
    for i in 0..3 {
        assert!(
            (token.position[i] - expect[i]).abs() < 0.04,
            "token.pos[{i}] got {} want {}",
            token.position[i],
            expect[i]
        );
    }
    let picked = dump
        .picked_up
        .iter()
        .find(|p| p.id == "token" && p.by == "walker")
        .expect("picked_up token by walker");
    assert_eq!(dump.held.len(), 1);
    assert_eq!(dump.held[0].id, "token");
    assert_eq!(dump.held[0].by, "walker");
    assert_eq!(dump.held[0].at_step, picked.at_step);
    assert!(
        dump.overlaps
            .iter()
            .any(|o| o.trigger == "exit" && o.body == "walker"),
        "overlaps must include exit/walker, got {:?}",
        dump.overlaps
    );
    assert!(dump.spawned.is_empty());
    assert!(dump.despawned.is_empty());
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
    for i in 0..3 {
        assert!(
            (cam.position[i] - expect_pos[i]).abs() < 0.08,
            "dump.camera.position[{i}] got {} want {}",
            cam.position[i],
            expect_pos[i]
        );
    }
}

#[test]
fn increment56_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment56.sh");
    assert!(script.is_file(), "scripts/increment56.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment56-threejs.sh");
    assert!(three.is_file(), "scripts/increment56-threejs.sh must exist");
    let out = PathBuf::from("target/test-increment56-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment56(&out, INCREMENT56_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment56");
    assert!(paths.scene.is_file() && paths.physics.is_file() && paths.frame.is_file());
    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    assert!(scene_txt.contains("\"hold\""));
    let v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&paths.physics).unwrap()).unwrap();
    assert!(v.get("held").is_some(), "written dump must have held");
    let bodies = v["bodies"].as_array().unwrap();
    assert!(bodies.iter().any(|b| b["id"] == "token"));
    assert!(bodies.iter().any(|b| b["id"] == "walker"));
    assert!(fs::metadata(&paths.frame).unwrap().len() > 256);
}
