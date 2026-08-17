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
    increment54_scene_json, increment55_scene_json, increment56_scene_json,
    increment57_scene, increment57_scene_json, increment58_scene, increment58_scene_json,
    parse_scene, run_increment58, step_physics, DEFAULT_DT, INCREMENT57_STEPS,
    INCREMENT58_STEPS,
};

fn prior_scene_jsons() -> [(&'static str, &'static str); 40] {
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
        ("57", increment57_scene_json()),
    ]
}

#[test]
fn increment58_does_not_mutate_prior() {
    for (name, json) in prior_scene_jsons() {
        assert!(
            !json.contains("\"npc\""),
            "increment {name} JSON must not contain \"npc\""
        );
        assert!(
            !json.contains("\"id\": \"npc\""),
            "increment {name} JSON must not contain npc body id"
        );
    }
    let live57 = increment57_scene();
    assert!(
        live57.bodies.iter().all(|b| b.id != "npc"),
        "increment57_scene must stay npc-free"
    );
    assert!(live57.bodies.iter().any(|b| b.id == "walker"));
    assert_eq!(live57.drops.len(), 1);
    assert_eq!(live57.drops[0].body, "token");
    assert_eq!(live57.drops[0].by, "walker");

    let live58 = increment58_scene();
    assert!(live58.bodies.iter().any(|b| b.id == "npc"), "increment58 must have npc");
    assert!(live58.bodies.iter().any(|b| b.id == "walker"));
    assert!(live58.bodies.iter().any(|b| b.id == "token"));
    assert!(live58.triggers.iter().any(|t| t.id == "exit"));
    assert_eq!(live58.drops.len(), 1);
    assert_eq!(live58.drops[0].body, "token");
    assert_eq!(live58.drops[0].by, "walker");
}

#[test]
fn increment58_scene_adds_npc() {
    let parsed = parse_scene(increment58_scene_json()).expect("increment58 JSON should parse");
    let npc = parsed
        .bodies
        .iter()
        .find(|b| b.id == "npc")
        .expect("increment58 must author npc");
    match npc.shape {
        agent_rig::Shape::Box { size } => {
            assert!((size[0] - 0.18).abs() < 1e-5);
            assert!((size[1] - 0.36).abs() < 1e-5);
            assert!((size[2] - 0.18).abs() < 1e-5);
        }
        _ => panic!("npc must be a box"),
    }
    assert!((npc.position[0] - 1.35).abs() < 1e-5);
    assert!((npc.position[1] - 0.20).abs() < 1e-5);
    assert!((npc.position[2] - 0.00).abs() < 1e-5);
    let ctrl = npc.controller.as_ref().expect("npc controller");
    assert!((ctrl.desired_velocity[0] + 0.40).abs() < 1e-5);
    assert!((ctrl.desired_velocity[1] - 0.00).abs() < 1e-5);
    assert!((ctrl.desired_velocity[2] - 0.00).abs() < 1e-5);
    assert_eq!(npc.collision_groups.membership, 2);
    assert_eq!(npc.collision_groups.filter, 1);
    assert!((npc.material.albedo[0] - 0.90).abs() < 1e-5);
    assert!((npc.material.albedo[1] - 0.55).abs() < 1e-5);
    assert!((npc.material.albedo[2] - 0.18).abs() < 1e-5);
    assert!((npc.material.roughness - 0.45).abs() < 0.06);
    assert!((npc.material.metallic - 0.00).abs() < 1e-5);
    let follow = parsed.camera.follow.as_ref().expect("lane camera must follow walker");
    assert_eq!(follow.body, "walker");
    assert_eq!(follow.offset, [-1.00, 0.80, 1.60]);
    let until = parsed.play_until.as_ref().expect("lane must author play_until");
    assert_eq!(until.kind, "entered");
    assert_eq!(until.body, "exit");
    assert!(
        !increment57_scene_json().contains("\"npc\""),
        "increment57 JSON must not contain npc"
    );
}

#[test]
fn increment57_dump_still_solo() {
    let dump = step_physics(&increment57_scene(), INCREMENT57_STEPS, DEFAULT_DT)
        .expect("increment57 physics");
    assert!(
        dump.bodies.iter().all(|b| b.id != "npc"),
        "increment57 dump must stay npc-free"
    );
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
    assert!(!dump.dropped.is_empty(), "increment57 dump must keep dropped");
}

#[test]
fn increment58_physics_npc() {
    let dump = step_physics(&increment58_scene(), INCREMENT58_STEPS, DEFAULT_DT)
        .expect("increment58 physics");
    assert!(
        (85..=115).contains(&dump.steps),
        "dump.steps should be 85..=115, got {}",
        dump.steps
    );
    let stopped = dump.stopped.as_ref().expect("dump.stopped");
    assert_eq!(stopped.kind, "entered");
    assert_eq!(stopped.body, "exit");
    assert!(dump.bodies.iter().any(|b| b.id == "npc"), "dump must have npc");
    assert!(dump.bodies.iter().any(|b| b.id == "walker"));
    assert!(dump.bodies.iter().any(|b| b.id == "token"));
    assert!(dump.bodies.iter().any(|b| b.id == "ground"));
    assert!(dump.bodies.iter().any(|b| b.id == "block"));
    let npc = dump.bodies.iter().find(|b| b.id == "npc").expect("dump npc");
    let walker = dump.bodies.iter().find(|b| b.id == "walker").expect("dump walker");
    let token = dump.bodies.iter().find(|b| b.id == "token").expect("dump token");
    assert!(
        npc.position[0] < 1.00,
        "npc.x should walk -x below 1.00, got {}",
        npc.position[0]
    );
    assert!(
        npc.position[0] < 1.35,
        "npc.x should be less than start 1.35, got {}",
        npc.position[0]
    );
    assert!(
        npc.position[1] >= 0.14 && npc.position[1] <= 0.28,
        "npc.y on floor, got {}",
        npc.position[1]
    );
    let npc_ctrl = dump.controllers.iter().find(|c| c.id == "npc").expect("npc controller");
    assert!(npc_ctrl.grounded, "npc must be grounded");
    assert!(
        walker.position[0] > 0.65,
        "walker.x should be past the token (>0.65), got {}",
        walker.position[0]
    );
    let walker_ctrl = dump.controllers.iter().find(|c| c.id == "walker").expect("walker controller");
    assert!(walker_ctrl.grounded);
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
    let picked = dump
        .picked_up
        .iter()
        .find(|p| p.id == "token" && p.by == "walker")
        .expect("picked_up token by walker");
    assert_eq!(picked.at_step, 66);
    assert_eq!(dump.held.len(), 1);
    assert_eq!(dump.held[0].id, "token");
    assert_eq!(dump.held[0].by, "walker");
    assert_eq!(dump.held[0].at_step, 66);
    assert_eq!(dump.dropped.len(), 1);
    assert_eq!(dump.dropped[0].id, "token");
    assert_eq!(dump.dropped[0].by, "walker");
    assert_eq!(dump.dropped[0].at_step, 99);
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
    assert!(
        dump.picked_up.iter().all(|p| p.by != "npc"),
        "npc must not fire pickups"
    );
    assert!(
        dump.held.iter().all(|h| h.by != "npc"),
        "npc must not hold"
    );
    assert!(
        dump.dropped.iter().all(|d| d.by != "npc"),
        "npc must not fire drops"
    );
}

#[test]
fn increment58_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment58.sh");
    assert!(script.is_file(), "scripts/increment58.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment58-threejs.sh");
    assert!(three.is_file(), "scripts/increment58-threejs.sh must exist");
    let out = PathBuf::from("target/test-increment58-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment58(&out, INCREMENT58_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment58");
    assert!(paths.scene.is_file() && paths.physics.is_file() && paths.frame.is_file());
    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    assert!(scene_txt.contains("\"npc\""), "written scene must have npc");
    let v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&paths.physics).unwrap()).unwrap();
    let bodies = v["bodies"].as_array().unwrap();
    assert!(bodies.iter().any(|b| b["id"] == "npc"), "written dump must have npc");
    assert!(fs::metadata(&paths.frame).unwrap().len() > 256);
}
