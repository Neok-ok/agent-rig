use std::fs;
use std::path::PathBuf;

use agent_rig::{
    catalog_ids, increment18_scene_json, increment19_scene_json, increment20_scene_json,
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
    increment57_scene_json, increment58_scene, increment58_scene_json,
    increment59_scene, increment59_scene_json, parse_scene, run_increment59,
    scene_catalog, step_physics, DEFAULT_DT, INCREMENT58_STEPS, INCREMENT59_STEPS,
};

fn prior_scene_jsons() -> [(&'static str, &'static str); 41] {
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
        ("58", increment58_scene_json()),
    ]
}

#[test]
fn increment59_does_not_mutate_prior() {
    for (name, json) in prior_scene_jsons() {
        let parsed = parse_scene(json).unwrap_or_else(|e| panic!("increment {name} parse: {e}"));
        assert!(
            parsed.id.is_empty(),
            "increment {name} scene-level id must be empty, got {:?}",
            parsed.id
        );
    }
    assert!(
        increment58_scene().id.is_empty(),
        "increment58_scene must stay id-free"
    );
    assert_eq!(increment59_scene().id, "lane");
    assert!(
        !increment58_scene_json().contains("\"id\": \"lane\""),
        "increment58 scene JSON must not contain top-level id lane"
    );
}

#[test]
fn increment59_catalog() {
    let ids = catalog_ids();
    assert!(ids.len() >= 2, "catalog must list courtyard and lane");
    assert_eq!(ids[0], "courtyard");
    assert_eq!(ids[1], "lane");

    let catalog = scene_catalog();
    assert!(catalog.len() >= 2);
    assert_eq!(catalog[0].0, "courtyard");
    assert_eq!(catalog[1].0, "lane");

    let courtyard = &catalog[0].1;
    assert!(
        courtyard.id.is_empty(),
        "courtyard catalog entry must not rewrite increment53 with an id"
    );
    let until = courtyard
        .play_until
        .as_ref()
        .expect("courtyard must author play_until");
    assert_eq!(until.kind, "picked_up");
    assert_eq!(until.body, "token");
    assert!(
        courtyard.bodies.iter().any(|b| b.id == "bar"),
        "courtyard is increment53 and must still have bar"
    );
    assert!(
        (courtyard.camera.position[0] - 1.85).abs() < 1e-4
            && (courtyard.camera.position[1] - 1.35).abs() < 1e-4
            && (courtyard.camera.position[2] - 3.15).abs() < 1e-4,
        "courtyard camera should stay increment53, got {:?}",
        courtyard.camera.position
    );
    assert!(courtyard.bodies.iter().all(|b| b.id != "npc"));

    let lane = &catalog[1].1;
    assert_eq!(lane.id, "lane");
    assert!(lane.bodies.iter().any(|b| b.id == "npc"), "lane must have npc");
    assert!(lane.bodies.iter().any(|b| b.id == "walker"));
    assert!(lane.bodies.iter().any(|b| b.id == "token"));
}

#[test]
fn increment58_dump_has_no_scene() {
    let dump = step_physics(&increment58_scene(), INCREMENT58_STEPS, DEFAULT_DT)
        .expect("increment58 physics");
    assert!(
        dump.scene.is_empty(),
        "increment58 dump.scene must be empty, got {:?}",
        dump.scene
    );
    let json = serde_json::to_string(&dump).expect("serialize increment58 dump");
    let v: serde_json::Value = serde_json::from_str(&json).expect("parse increment58 dump");
    assert!(
        v.get("scene").is_none(),
        "increment58 dump JSON must omit scene key"
    );
    assert!(
        dump.bodies.iter().any(|b| b.id == "npc"),
        "increment58 dump still has npc"
    );
}

#[test]
fn increment59_physics_lane() {
    let dump = step_physics(&increment59_scene(), INCREMENT59_STEPS, DEFAULT_DT)
        .expect("increment59 physics");
    assert!(
        (85..=115).contains(&dump.steps),
        "dump.steps should be 85..=115, got {}",
        dump.steps
    );
    assert_eq!(dump.scene, "lane");
    let stopped = dump.stopped.as_ref().expect("dump.stopped");
    assert_eq!(stopped.kind, "entered");
    assert_eq!(stopped.body, "exit");
    assert!(dump.bodies.iter().any(|b| b.id == "npc"), "dump must have npc");
    assert!(dump.bodies.iter().any(|b| b.id == "walker"));
    assert!(dump.bodies.iter().any(|b| b.id == "token"));
    let npc = dump.bodies.iter().find(|b| b.id == "npc").expect("dump npc");
    let walker = dump.bodies.iter().find(|b| b.id == "walker").expect("dump walker");
    let token = dump.bodies.iter().find(|b| b.id == "token").expect("dump token");
    assert!(
        npc.position[0] < 1.00,
        "npc.x should walk -x below 1.00, got {}",
        npc.position[0]
    );
    assert!(
        walker.position[0] > 0.65,
        "walker.x should be past the token (>0.65), got {}",
        walker.position[0]
    );
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
}

#[test]
fn increment59_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment59.sh");
    assert!(script.is_file(), "scripts/increment59.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment59-threejs.sh");
    assert!(three.is_file(), "scripts/increment59-threejs.sh must exist");
    let out = PathBuf::from("target/test-increment59-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment59(&out, INCREMENT59_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment59");
    assert!(paths.scene.is_file() && paths.physics.is_file() && paths.frame.is_file());
    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    assert!(
        scene_txt.contains("\"id\": \"lane\""),
        "written scene must have id lane"
    );
    let v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&paths.physics).unwrap()).unwrap();
    assert_eq!(v["scene"], "lane");
    let scenes_path = out.join("scenes.json");
    assert!(scenes_path.is_file(), "run_increment59 must write scenes.json");
    let scenes: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&scenes_path).unwrap()).unwrap();
    assert_eq!(
        scenes,
        serde_json::json!([{ "id": "courtyard" }, { "id": "lane" }])
    );
    assert!(fs::metadata(&paths.frame).unwrap().len() > 256);
    let _ = increment59_scene_json();
}
