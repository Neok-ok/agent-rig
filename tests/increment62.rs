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
    increment57_scene_json, increment58_scene_json, increment59_scene_json,
    increment60_scene_json, increment61_scene, increment61_scene_json,
    increment62_scene, increment62_scene_json, parse_scene, run_increment62,
    step_catalog_scene_with_carry, step_physics, DEFAULT_DT, INCREMENT61_STEPS,
    INCREMENT62_STEPS,
};

fn prior_scene_jsons() -> [(&'static str, &'static str); 44] {
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
        ("59", increment59_scene_json()),
        ("60", increment60_scene_json()),
        ("61", increment61_scene_json()),
    ]
}

fn drops_len(json: &str) -> usize {
    let v: serde_json::Value = serde_json::from_str(json).expect("parse scene json");
    v.get("drops")
        .and_then(|d| d.as_array())
        .map(|a| a.len())
        .unwrap_or(0)
}

#[test]
fn increment62_does_not_mutate_prior() {
    assert!(
        !increment61_scene().drops.is_empty(),
        "increment61_scene must still have drops"
    );
    let tr = increment61_scene().transition.expect("increment61 transition");
    assert_eq!(tr.to, "courtyard");
    for (name, json) in prior_scene_jsons() {
        let parsed = parse_scene(json).unwrap_or_else(|e| panic!("increment {name} parse: {e}"));
        if name == "61" {
            assert!(!parsed.drops.is_empty(), "increment61 JSON must still have drops");
            assert!(drops_len(json) > 0, "increment61 json still has drops");
        }
    }
    let inc62 = increment62_scene_json();
    assert_eq!(drops_len(inc62), 0, "increment62 json has no drops (or empty)");
}

#[test]
fn increment62_scene_is_lane_no_drop() {
    let scene = increment62_scene();
    assert_eq!(scene.id, "lane");
    let tr = scene.transition.as_ref().expect("transition");
    assert_eq!(tr.to, "courtyard");
    assert!(scene.drops.is_empty(), "drops must be empty");
    assert!(scene.bodies.iter().any(|b| b.id == "npc"), "has npc");
    let until = scene.play_until.as_ref().expect("play_until");
    assert_eq!(until.kind, "entered");
    assert_eq!(until.body, "exit");
    assert!(scene.pickups.iter().any(|p| p.hold), "has hold pickup");
}

#[test]
fn increment61_dump_still_drops() {
    let dump = step_physics(&increment61_scene(), INCREMENT61_STEPS, DEFAULT_DT)
        .expect("increment61 physics");
    assert!(!dump.dropped.is_empty(), "increment61 dump has dropped");
    let walker = dump
        .bodies
        .iter()
        .find(|b| b.id == "walker")
        .expect("walker");
    let token = dump
        .bodies
        .iter()
        .find(|b| b.id == "token")
        .expect("token");
    let drop = [
        walker.position[0] + 0.22,
        walker.position[1] - 0.06,
        walker.position[2] + 0.00,
    ];
    for i in 0..3 {
        assert!(
            (token.position[i] - drop[i]).abs() < 0.04,
            "increment61 token at drop_offset[{i}] got {} want {}",
            token.position[i],
            drop[i]
        );
    }
    assert!(
        token.position[1] < 0.20,
        "increment61 token.y on ground, got {}",
        token.position[1]
    );
    let hold = [
        walker.position[0] + 0.16,
        walker.position[1] + 0.22,
        walker.position[2] + 0.00,
    ];
    let still_held = (0..3).all(|i| (token.position[i] - hold[i]).abs() < 0.04);
    assert!(!still_held, "increment61 token must not stay at hold_offset");
    let art = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("artifacts/increment61/next-physics.json");
    if art.is_file() {
        let next: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&art).unwrap()).unwrap();
        let bodies = next["bodies"].as_array().expect("next bodies");
        assert!(
            bodies.iter().all(|b| b["id"] != "token"),
            "increment61 next-physics must have NO token"
        );
    }
}

#[test]
fn increment62_physics_lane_held() {
    let dump = step_physics(&increment62_scene(), INCREMENT62_STEPS, DEFAULT_DT)
        .expect("increment62 physics");
    assert_eq!(dump.scene, "lane");
    assert!(
        (85..=115).contains(&dump.steps),
        "dump.steps 85..=115, got {}",
        dump.steps
    );
    let stopped = dump.stopped.as_ref().expect("dump.stopped");
    assert_eq!(stopped.kind, "entered");
    assert_eq!(stopped.body, "exit");
    assert!(
        (90..=110).contains(&stopped.at_step),
        "stopped entered/exit ~99, got {}",
        stopped.at_step
    );
    let tr = dump.transition.as_ref().expect("dump.transition");
    assert_eq!(tr.to, "courtyard");
    assert_eq!(tr.at_step, stopped.at_step);
    assert!(dump.bodies.iter().any(|b| b.id == "token"), "HAS token");
    assert!(dump.bodies.iter().any(|b| b.id == "npc"), "HAS npc");
    assert!(dump.bodies.iter().any(|b| b.id == "walker"), "HAS walker");
    let walker = dump
        .bodies
        .iter()
        .find(|b| b.id == "walker")
        .expect("dump walker");
    let token = dump
        .bodies
        .iter()
        .find(|b| b.id == "token")
        .expect("dump token");
    let expect = [
        walker.position[0] + 0.16,
        walker.position[1] + 0.22,
        walker.position[2] + 0.00,
    ];
    for i in 0..3 {
        assert!(
            (token.position[i] - expect[i]).abs() < 0.04,
            "token at hold_offset[{i}] got {} want {}",
            token.position[i],
            expect[i]
        );
    }
    let held = dump
        .held
        .iter()
        .find(|h| h.id == "token")
        .expect("dump.held token");
    assert_eq!(held.by, "walker");
    assert_eq!(held.at_step, 66);
    assert!(dump.dropped.is_empty(), "dump.dropped omitted / empty");
    let v = serde_json::to_value(&dump).expect("serialize dump");
    assert!(v.get("dropped").is_none(), "serialized dump must omit dropped");
}

#[test]
fn increment62_carry_into_courtyard() {
    let lane = step_physics(&increment62_scene(), INCREMENT62_STEPS, DEFAULT_DT)
        .expect("increment62 lane");
    let next = step_catalog_scene_with_carry("courtyard", INCREMENT62_STEPS, DEFAULT_DT, Some(&lane))
        .expect("carry into courtyard");
    assert_eq!(next.scene, "courtyard");
    assert!(
        (1..=31).contains(&next.steps),
        "next dump.steps 1..=31, got {}",
        next.steps
    );
    let token_count = next.bodies.iter().filter(|b| b.id == "token").count();
    assert_eq!(token_count, 1, "only one token (courtyard spawn skipped)");
    assert!(next.bodies.iter().any(|b| b.id == "bar"), "HAS bar");
    assert!(next.bodies.iter().any(|b| b.id == "token"), "HAS token");
    assert!(next.bodies.iter().all(|b| b.id != "npc"), "NO npc");
    let walker = next
        .bodies
        .iter()
        .find(|b| b.id == "walker")
        .expect("courtyard walker");
    let token = next
        .bodies
        .iter()
        .find(|b| b.id == "token")
        .expect("courtyard token");
    let expect = [
        walker.position[0] + 0.16,
        walker.position[1] + 0.22,
        walker.position[2] + 0.00,
    ];
    for i in 0..3 {
        assert!(
            (token.position[i] - expect[i]).abs() < 0.08,
            "carried token[{i}] got {} want {}",
            token.position[i],
            expect[i]
        );
    }
    assert!(
        next.held.iter().any(|h| h.id == "token"),
        "dump.held includes token"
    );
}

#[test]
fn increment62_writes() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment62.sh");
    assert!(script.is_file(), "scripts/increment62.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment62-threejs.sh");
    assert!(three.is_file(), "scripts/increment62-threejs.sh must exist");
    let out = PathBuf::from("target/test-increment62-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment62(&out, INCREMENT62_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment62");
    assert!(paths.scene.is_file() && paths.physics.is_file() && paths.frame.is_file());
    assert!(fs::metadata(&paths.frame).unwrap().len() > 256);
    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    assert_eq!(drops_len(&scene_txt), 0, "scene.json is lane without drops");
    assert!(scene_txt.contains("\"id\": \"lane\"") || scene_txt.contains("\"id\":\"lane\""));
    let v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&paths.physics).unwrap()).unwrap();
    assert_eq!(v["scene"], "lane");
    assert!(v.get("held").is_some(), "physics.json has held");
    assert!(v.get("dropped").is_none(), "physics.json has no dropped");
    let next_path = out.join("next-physics.json");
    assert!(next_path.is_file(), "next-physics.json must exist");
    let next: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&next_path).unwrap()).unwrap();
    assert_eq!(next["scene"], "courtyard");
    let bodies = next["bodies"].as_array().expect("next bodies");
    assert!(bodies.iter().any(|b| b["id"] == "token"), "next HAS token");
    assert!(bodies.iter().any(|b| b["id"] == "bar"), "next HAS bar");
    let next_frame = out.join("next-frame.png");
    assert!(next_frame.is_file(), "next-frame.png must exist");
    assert!(fs::metadata(&next_frame).unwrap().len() > 256);
}
