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
    increment54_scene_json, increment55_scene_json, increment56_scene,
    increment56_scene_json, increment57_scene, increment57_scene_json,
    parse_scene, run_increment57, step_physics, DEFAULT_DT, INCREMENT56_STEPS,
    INCREMENT57_STEPS,
};

fn prior_scene_jsons() -> [(&'static str, &'static str); 39] {
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
        ("56", increment56_scene_json()),
    ]
}

#[test]
fn increment57_does_not_mutate_prior() {
    for (name, json) in prior_scene_jsons() {
        assert!(
            !json.contains("\"drops\""),
            "increment {name} JSON must not contain \"drops\""
        );
        assert!(
            !json.contains("drop_offset"),
            "increment {name} JSON must not contain drop_offset"
        );
    }
    let live56 = increment56_scene();
    assert!(live56.drops.is_empty(), "increment56_scene must stay drop-free");
    assert!(live56.pickups[0].hold, "increment56 pickup must stay hold=true");
    assert_eq!(live56.pickups[0].hold_offset, [0.16, 0.22, 0.00]);
    let until56 = live56.play_until.as_ref().expect("increment56 play_until");
    assert_eq!(until56.kind, "entered");
    assert_eq!(until56.body, "exit");

    let live57 = increment57_scene();
    assert_eq!(live57.drops.len(), 1);
    assert_eq!(live57.drops[0].body, "token");
    assert_eq!(live57.drops[0].trigger, "exit");
    assert_eq!(live57.drops[0].by, "walker");
    assert_eq!(live57.drops[0].drop_offset, [0.22, -0.06, 0.00]);
}

#[test]
fn increment57_scene_adds_drop() {
    let parsed = parse_scene(increment57_scene_json()).expect("increment57 JSON should parse");
    assert_eq!(parsed.drops.len(), 1);
    assert_eq!(parsed.drops[0].body, "token");
    assert_eq!(parsed.drops[0].trigger, "exit");
    assert_eq!(parsed.drops[0].by, "walker");
    assert_eq!(parsed.drops[0].drop_offset, [0.22, -0.06, 0.00]);
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
        !increment56_scene_json().contains("\"drops\""),
        "increment56 JSON must not contain drops"
    );
}

#[test]
fn increment56_dump_still_holds() {
    let dump = step_physics(&increment56_scene(), INCREMENT56_STEPS, DEFAULT_DT)
        .expect("increment56 physics");
    assert!(
        dump.steps == 100 || (85..=115).contains(&dump.steps),
        "increment56 dump.steps still ~100, got {}",
        dump.steps
    );
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
    assert!(!dump.held.is_empty(), "increment56 dump must keep held");
    assert!(dump.dropped.is_empty(), "increment56 dump dropped must be empty");
    let v = serde_json::to_value(&dump).expect("serialize increment56 dump");
    assert!(v.get("held").is_some(), "serialized increment56 dump must have held");
    assert!(v.get("dropped").is_none(), "serialized increment56 dump must omit dropped");
}

#[test]
fn increment57_physics_drop() {
    let dump = step_physics(&increment57_scene(), INCREMENT57_STEPS, DEFAULT_DT)
        .expect("increment57 physics");
    assert!(
        (85..=115).contains(&dump.steps),
        "dump.steps should be 85..=115, got {}",
        dump.steps
    );
    let stopped = dump.stopped.as_ref().expect("dump.stopped");
    assert_eq!(stopped.kind, "entered");
    assert_eq!(stopped.body, "exit");
    assert!(dump.bodies.iter().any(|b| b.id == "token"), "token should remain");
    assert!(dump.bodies.iter().any(|b| b.id == "walker"));
    assert!(dump.bodies.iter().any(|b| b.id == "ground"));
    assert!(dump.bodies.iter().any(|b| b.id == "block"));
    let walker = dump.bodies.iter().find(|b| b.id == "walker").expect("dump walker");
    let token = dump.bodies.iter().find(|b| b.id == "token").expect("dump token");
    let expect = [
        walker.position[0] + 0.22,
        walker.position[1] - 0.06,
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
    assert!(
        token.position[1] < 0.20,
        "token.y should be on the ground (<0.20), got {}",
        token.position[1]
    );
    let hold = [
        walker.position[0] + 0.16,
        walker.position[1] + 0.22,
        walker.position[2] + 0.00,
    ];
    let still_held = (0..3).all(|i| (token.position[i] - hold[i]).abs() < 0.04);
    assert!(!still_held, "token must not stay at hold_offset");
    let picked = dump
        .picked_up
        .iter()
        .find(|p| p.id == "token" && p.by == "walker")
        .expect("picked_up token by walker");
    assert!(
        (50..=80).contains(&picked.at_step),
        "picked_up at_step ~66, got {}",
        picked.at_step
    );
    assert_eq!(dump.held.len(), 1);
    assert_eq!(dump.held[0].id, "token");
    assert_eq!(dump.held[0].by, "walker");
    assert_eq!(dump.held[0].at_step, picked.at_step);
    assert_eq!(dump.dropped.len(), 1);
    assert_eq!(dump.dropped[0].id, "token");
    assert_eq!(dump.dropped[0].by, "walker");
    assert_eq!(dump.dropped[0].at_step, stopped.at_step);
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
fn increment57_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment57.sh");
    assert!(script.is_file(), "scripts/increment57.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment57-threejs.sh");
    assert!(three.is_file(), "scripts/increment57-threejs.sh must exist");
    let out = PathBuf::from("target/test-increment57-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment57(&out, INCREMENT57_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment57");
    assert!(paths.scene.is_file() && paths.physics.is_file() && paths.frame.is_file());
    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    assert!(scene_txt.contains("\"drops\""));
    let v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&paths.physics).unwrap()).unwrap();
    assert!(v.get("dropped").is_some(), "written dump must have dropped");
    let bodies = v["bodies"].as_array().unwrap();
    assert!(bodies.iter().any(|b| b["id"] == "token"));
    assert!(bodies.iter().any(|b| b["id"] == "walker"));
    assert!(fs::metadata(&paths.frame).unwrap().len() > 256);
}
