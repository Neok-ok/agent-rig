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
    increment53_scene_json, increment54_scene_json, increment55_scene_json,
    increment56_scene_json, increment57_scene_json, increment58_scene_json,
    increment59_scene, increment59_scene_json, parse_scene, run_increment60,
    scene_by_id, step_catalog_scene, step_physics, DEFAULT_DT, INCREMENT53_STEPS,
    INCREMENT59_STEPS, INCREMENT60_STEPS,
};

fn prior_scene_jsons() -> [(&'static str, &'static str); 42] {
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
    ]
}

fn top_level_id(json: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    v.get("id").and_then(|x| x.as_str()).map(|s| s.to_string())
}

#[test]
fn increment60_does_not_mutate_prior() {
    assert!(
        increment53_scene().id.is_empty(),
        "increment53_scene must stay id-free"
    );
    assert_eq!(increment59_scene().id, "lane");
    for (name, json) in prior_scene_jsons() {
        let parsed = parse_scene(json).unwrap_or_else(|e| panic!("increment {name} parse: {e}"));
        if name == "59" {
            assert_eq!(parsed.id, "lane", "increment59 must stay lane");
            assert_eq!(top_level_id(json).as_deref(), Some("lane"));
        } else {
            assert!(
                parsed.id.is_empty(),
                "increment {name} scene-level id must be empty, got {:?}",
                parsed.id
            );
        }
    }
    let inc53 = increment53_scene_json();
    let top = top_level_id(inc53);
    assert!(
        top.as_deref() != Some("lane") && top.as_deref() != Some("courtyard"),
        "increment53 JSON must not have top-level id lane/courtyard, got {top:?}"
    );
}

#[test]
fn increment60_catalog_run() {
    let courtyard = scene_by_id("courtyard").expect("courtyard in catalog");
    assert!(
        courtyard.bodies.iter().any(|b| b.id == "bar")
            || courtyard
                .play_until
                .as_ref()
                .map(|u| u.kind == "picked_up")
                .unwrap_or(false),
        "courtyard must have bar or play_until picked_up"
    );
    let lane = scene_by_id("lane").expect("lane in catalog");
    assert!(lane.bodies.iter().any(|b| b.id == "npc"), "lane must have npc");
    assert!(scene_by_id("nope").is_none(), "unknown id must be None");
}

#[test]
fn increment53_dump_still_no_scene() {
    let dump = step_physics(&increment53_scene(), INCREMENT53_STEPS, DEFAULT_DT)
        .expect("increment53 physics");
    assert!(
        dump.scene.is_empty(),
        "increment53 dump.scene must be empty, got {:?}",
        dump.scene
    );
    let json = serde_json::to_string(&dump).expect("serialize increment53 dump");
    let v: serde_json::Value = serde_json::from_str(&json).expect("parse increment53 dump");
    assert!(
        v.get("scene").is_none(),
        "increment53 dump JSON must omit scene key"
    );
    assert!(
        (30..=31).contains(&dump.steps),
        "increment53 dump.steps 30..=31, got {}",
        dump.steps
    );
    assert!(dump.bodies.iter().any(|b| b.id == "bar"), "increment53 has bar");
}

#[test]
fn increment59_dump_still_lane() {
    let dump = step_physics(&increment59_scene(), INCREMENT59_STEPS, DEFAULT_DT)
        .expect("increment59 physics");
    assert_eq!(dump.scene, "lane");
    assert!(
        (85..=115).contains(&dump.steps),
        "increment59 dump.steps ~100, got {}",
        dump.steps
    );
    assert!(dump.bodies.iter().any(|b| b.id == "npc"), "increment59 has npc");
    assert!(
        dump.bodies.iter().all(|b| b.id != "bar"),
        "increment59 must not have bar"
    );
}

#[test]
fn increment60_physics_courtyard() {
    let dump = step_catalog_scene("courtyard", INCREMENT60_STEPS, DEFAULT_DT)
        .expect("step_catalog_scene courtyard");
    assert_eq!(dump.scene, "courtyard");
    assert!(
        (30..=31).contains(&dump.steps),
        "dump.steps 30..=31, got {}",
        dump.steps
    );
    let stopped = dump.stopped.as_ref().expect("dump.stopped");
    assert_eq!(stopped.kind, "picked_up");
    assert_eq!(stopped.body, "token");
    assert!(
        (29..=31).contains(&stopped.at_step),
        "stopped.at_step ~30, got {}",
        stopped.at_step
    );
    assert!(dump.bodies.iter().any(|b| b.id == "bar"), "HAS bar");
    assert!(dump.bodies.iter().any(|b| b.id == "walker"), "HAS walker");
    assert!(dump.bodies.iter().all(|b| b.id != "npc"), "NO npc");
    assert!(
        dump.bodies.iter().all(|b| b.id != "token"),
        "courtyard token is picked, no token-on-ground"
    );
    assert!(
        dump.spawned.iter().any(|s| s.id == "token" && s.at_step == 30),
        "spawned token@30"
    );
    let walker = dump
        .bodies
        .iter()
        .find(|b| b.id == "walker")
        .expect("dump walker");
    let cam = dump.camera.as_ref().expect("dump.camera");
    let expect_pos = [
        walker.position[0] + 1.20,
        walker.position[1] + 0.90,
        walker.position[2] + 1.50,
    ];
    for i in 0..3 {
        assert!(
            (cam.position[i] - expect_pos[i]).abs() < 0.08,
            "courtyard follow-cam position[{i}] got {} want {}",
            cam.position[i],
            expect_pos[i]
        );
    }
}

#[test]
fn increment60_writes_and_unknown_id() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment60.sh");
    assert!(script.is_file(), "scripts/increment60.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment60-threejs.sh");
    assert!(three.is_file(), "scripts/increment60-threejs.sh must exist");
    assert!(scene_by_id("unknown").is_none(), "unknown id is None");
    assert!(
        step_catalog_scene("unknown", 1, DEFAULT_DT).is_err(),
        "unknown catalog id is a hard error"
    );
    let out = PathBuf::from("target/test-increment60-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment60(&out, INCREMENT60_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment60");
    assert!(paths.scene.is_file() && paths.physics.is_file() && paths.frame.is_file());
    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    assert!(scene_txt.contains("\"bar\""), "scene.json is courtyard (has bar)");
    assert!(
        !scene_txt.contains("\"npc\""),
        "scene.json is courtyard (no npc)"
    );
    let v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&paths.physics).unwrap()).unwrap();
    assert_eq!(v["scene"], "courtyard");
    assert!(fs::metadata(&paths.frame).unwrap().len() > 256);
}
