use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::scene::{Joint, MeshCollider, Scene, Shape, Trigger};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsDump {
    pub steps: u32,
    pub dt: f32,
    pub gravity: [f32; 3],
    pub bodies: Vec<PhysicsBodyState>,
    pub contacts: Vec<PhysicsContact>,
    #[serde(default)]
    pub joints: Vec<PhysicsJoint>,
    /// Sensor overlaps after the last step. Empty when the scene has no triggers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overlaps: Vec<PhysicsOverlap>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsOverlap {
    pub trigger: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsJoint {
    pub kind: String,
    pub body_a: String,
    pub body_b: String,
    pub anchor: [f32; 3],
    pub axis: [f32; 3],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limits: Option<[f32; 2]>,
    /// Authored hinge motor target (rad/s). Present on hinges even if 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motor_target_velocity: Option<f32>,
    /// Authored hinge motor max force / factor. Present on hinges even if 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motor_max_force: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsBodyState {
    pub id: String,
    pub position: [f32; 3],
    pub rotation_wxyz: [f32; 4],
    pub linear_velocity: [f32; 3],
    pub angular_velocity: [f32; 3],
    #[serde(default)]
    pub collider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsContact {
    pub body_a: String,
    pub body_b: String,
    pub point: [f32; 3],
    pub normal: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trajectory {
    pub dt: f32,
    pub frame_stride: u32,
    pub frames: Vec<TrajectoryFrame>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryFrame {
    pub step: u32,
    pub bodies: Vec<PhysicsBodyState>,
    pub contacts: Vec<PhysicsContact>,
}

struct PhysicsWorld {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    body_handles: HashMap<String, RigidBodyHandle>,
    collider_to_id: HashMap<ColliderHandle, String>,
    trigger_collider_to_id: HashMap<ColliderHandle, String>,
    body_colliders: HashMap<String, String>,
    body_ids: Vec<String>,
    authored_joints: Vec<PhysicsJoint>,
    gravity: Vector<f32>,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
}

impl PhysicsWorld {
    fn from_scene(scene: &Scene, dt: f32) -> Result<Self, String> {
        let mut world = Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            body_handles: HashMap::new(),
            collider_to_id: HashMap::new(),
            trigger_collider_to_id: HashMap::new(),
            body_colliders: HashMap::new(),
            body_ids: Vec::new(),
            authored_joints: Vec::new(),
            gravity: vector![0.0, -9.81, 0.0],
            integration_parameters: {
                let mut p = IntegrationParameters::default();
                p.dt = dt;
                p
            },
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
        };
        world.populate(scene)?;
        Ok(world)
    }

    fn populate(&mut self, scene: &Scene) -> Result<(), String> {

    for body in &scene.bodies {
        let [x, y, z] = body.position;
        let [w, qx, qy, qz] = body.rotation_wxyz;
        let rotation = nalgebra::UnitQuaternion::from_quaternion(nalgebra::Quaternion::new(w, qx, qy, qz));
        let iso = Isometry::from_parts(Translation::new(x, y, z), rotation);

        let rb = if body.mass <= 0.0 {
            RigidBodyBuilder::fixed().position(iso).build()
        } else {
            let [vx, vy, vz] = body.linear_velocity;
            RigidBodyBuilder::dynamic()
                .position(iso)
                .linvel(vector![vx, vy, vz])
                .build()
        };
        let handle = self.rigid_body_set.insert(rb);

        // Density from authored mass so inertia is non-zero (needed for body-body hits).
        let kind = body.shape.collider_kind().to_string();
        let collider = match &body.shape {
            Shape::Box { size } => {
                let mut b = ColliderBuilder::cuboid(size[0] * 0.5, size[1] * 0.5, size[2] * 0.5)
                    .friction(0.35)
                    .restitution(0.05);
                if body.mass > 0.0 {
                    let vol = (size[0] * size[1] * size[2]).max(1e-8);
                    b = b.density(body.mass / vol);
                }
                b.build()
            }
            Shape::Sphere { radius } => {
                let mut b = ColliderBuilder::ball(*radius).friction(0.25).restitution(0.05);
                if body.mass > 0.0 {
                    let vol = (4.0 / 3.0 * std::f32::consts::PI * radius * radius * radius).max(1e-8);
                    b = b.density(body.mass / vol);
                }
                b.build()
            }
            Shape::Mesh { path, collider } => {
                let mesh = scene.load_body_mesh(path)?;
                let points: Vec<Point<f32>> = mesh
                    .vertices
                    .iter()
                    .map(|v| Point::new(v[0], v[1], v[2]))
                    .collect();
                let mut b = match collider {
                    MeshCollider::ConvexHull => ColliderBuilder::convex_hull(&points)
                        .ok_or_else(|| format!("convex hull failed for mesh {path}"))?,
                    MeshCollider::Trimesh => ColliderBuilder::trimesh(points, mesh.indices.clone())
                        .map_err(|e| format!("trimesh collider for {path}: {e}"))?,
                };
                b = b.friction(0.45).restitution(0.05);
                if body.mass > 0.0 {
                    b = b.density(body.mass / mesh.volume());
                }
                b.build()
            }
        };
        let ch = self.collider_set.insert_with_parent(collider, handle, &mut self.rigid_body_set);
            self.body_handles.insert(body.id.clone(), handle);
            self.collider_to_id.insert(ch, body.id.clone());
            self.body_colliders.insert(body.id.clone(), kind);
            self.body_ids.push(body.id.clone());
        }
        self.populate_joints(scene)?;
        self.populate_triggers(scene)?;
        Ok(())
    }

    fn populate_triggers(&mut self, scene: &Scene) -> Result<(), String> {
        for trigger in &scene.triggers {
            let Trigger { id, shape, position } = trigger;
            let [x, y, z] = *position;
            let iso = Isometry::translation(x, y, z);
            let rb = RigidBodyBuilder::fixed().position(iso).build();
            let handle = self.rigid_body_set.insert(rb);
            let collider = match shape {
                Shape::Box { size } => ColliderBuilder::cuboid(
                    size[0] * 0.5,
                    size[1] * 0.5,
                    size[2] * 0.5,
                )
                .sensor(true)
                .build(),
                _ => {
                    return Err(format!(
                        "trigger '{id}' only supports box shape, got {shape:?}"
                    ))
                }
            };
            let ch = self
                .collider_set
                .insert_with_parent(collider, handle, &mut self.rigid_body_set);
            self.trigger_collider_to_id.insert(ch, id.clone());
        }
        Ok(())
    }

    fn populate_joints(&mut self, scene: &Scene) -> Result<(), String> {
        for joint in &scene.joints {
            match joint {
                Joint::Hinge {
                    body_a,
                    body_b,
                    anchor,
                    axis,
                    motor_target_velocity,
                    motor_max_force,
                } => {
                    let ha = *self.body_handles.get(body_a).ok_or_else(|| {
                        format!("hinge body_a '{body_a}' not found")
                    })?;
                    let hb = *self.body_handles.get(body_b).ok_or_else(|| {
                        format!("hinge body_b '{body_b}' not found")
                    })?;
                    let rb_a = &self.rigid_body_set[ha];
                    let rb_b = &self.rigid_body_set[hb];
                    let world_anchor = point![anchor[0], anchor[1], anchor[2]];
                    let local_a = rb_a.position().inverse() * world_anchor;
                    let local_b = rb_b.position().inverse() * world_anchor;
                    let world_axis = vector![axis[0], axis[1], axis[2]];
                    let local_axis_a = rb_a.rotation().inverse() * world_axis;
                    let unit = UnitVector::try_new(local_axis_a, 1e-6).ok_or_else(|| {
                        format!("hinge axis too small: {axis:?}")
                    })?;
                    // Authored motor. Both 0 (increment 28/29) keeps the hang
                    // damper-to-zero: motor_velocity(0, 8). Nonzero target
                    // drives the lantern around the hinge axis with a
                    // ForceBased motor so motor_max_force (~8) can beat gravity.
                    let authored = *motor_target_velocity != 0.0 || *motor_max_force != 0.0;
                    let target = *motor_target_velocity;
                    let factor = if authored {
                        if *motor_max_force == 0.0 { 8.0 } else { *motor_max_force }
                    } else {
                        8.0
                    };
                    let mut builder = RevoluteJointBuilder::new(unit)
                        .local_anchor1(local_a)
                        .local_anchor2(local_b)
                        .contacts_enabled(false)
                        .motor_velocity(target, factor);
                    if authored {
                        builder = builder
                            .motor_model(MotorModel::ForceBased)
                            .motor_max_force(factor);
                    }
                    let hinge = builder.build();
                    self.impulse_joint_set.insert(ha, hb, hinge, true);
                    if let Some(rb) = self.rigid_body_set.get_mut(hb) {
                        // Heavy damping is for the hang settle. A driven motor
                        // only needs light damping so it can keep swinging.
                        if authored {
                            rb.set_angular_damping(0.4);
                            rb.set_linear_damping(0.2);
                        } else {
                            rb.set_angular_damping(3.0);
                            rb.set_linear_damping(1.0);
                        }
                    }
                    self.authored_joints.push(PhysicsJoint {
                        kind: "hinge".into(),
                        body_a: body_a.clone(),
                        body_b: body_b.clone(),
                        anchor: *anchor,
                        axis: *axis,
                        limits: None,
                        motor_target_velocity: Some(*motor_target_velocity),
                        motor_max_force: Some(*motor_max_force),
                    });
                }
                Joint::Slider {
                    body_a,
                    body_b,
                    axis,
                    limits,
                    anchor,
                    motor_target_velocity,
                    motor_max_force,
                } => {
                    let ha = *self.body_handles.get(body_a).ok_or_else(|| {
                        format!("slider body_a '{body_a}' not found")
                    })?;
                    let hb = *self.body_handles.get(body_b).ok_or_else(|| {
                        format!("slider body_b '{body_b}' not found")
                    })?;
                    let rb_a = &self.rigid_body_set[ha];
                    let rb_b = &self.rigid_body_set[hb];
                    let world_anchor = match anchor {
                        Some(a) => point![a[0], a[1], a[2]],
                        None => {
                            let t = rb_b.translation();
                            point![t.x, t.y, t.z]
                        }
                    };
                    let dump_anchor = [world_anchor.x, world_anchor.y, world_anchor.z];
                    let local_a = rb_a.position().inverse() * world_anchor;
                    let local_b = rb_b.position().inverse() * world_anchor;
                    let world_axis = vector![axis[0], axis[1], axis[2]];
                    let local_axis_a = rb_a.rotation().inverse() * world_axis;
                    let local_axis_b = rb_b.rotation().inverse() * world_axis;
                    let unit_a = UnitVector::try_new(local_axis_a, 1e-6).ok_or_else(|| {
                        format!("slider axis too small: {axis:?}")
                    })?;
                    let unit_b = UnitVector::try_new(local_axis_b, 1e-6).ok_or_else(|| {
                        format!("slider axis too small: {axis:?}")
                    })?;
                    // Authored motor. Both 0 (increment 29–33) keeps the
                    // open-from-velocity slider. Nonzero target drives the
                    // drawer along the axis with a ForceBased motor so
                    // motor_max_force (~6) can close against residual motion.
                    let authored = *motor_target_velocity != 0.0 || *motor_max_force != 0.0;
                    let mut builder = PrismaticJointBuilder::new(unit_a)
                        .local_axis1(unit_a)
                        .local_axis2(unit_b)
                        .local_anchor1(local_a)
                        .local_anchor2(local_b)
                        .limits(*limits)
                        .contacts_enabled(false);
                    if authored {
                        let target = *motor_target_velocity;
                        let factor = if *motor_max_force == 0.0 { 6.0 } else { *motor_max_force };
                        builder = builder
                            .motor_velocity(target, factor)
                            .motor_model(MotorModel::ForceBased)
                            .motor_max_force(factor);
                    }
                    let slider = builder.build();
                    self.impulse_joint_set.insert(ha, hb, slider, true);
                    if let Some(rb) = self.rigid_body_set.get_mut(hb) {
                        // Heavy damping is for the free-slide settle. A driven
                        // motor only needs light damping so it can close.
                        if authored {
                            rb.set_linear_damping(0.2);
                            rb.set_angular_damping(0.4);
                        } else {
                            rb.set_linear_damping(1.0);
                            rb.set_angular_damping(3.0);
                        }
                    }
                    self.authored_joints.push(PhysicsJoint {
                        kind: "slider".into(),
                        body_a: body_a.clone(),
                        body_b: body_b.clone(),
                        anchor: dump_anchor,
                        axis: *axis,
                        limits: Some(*limits),
                        motor_target_velocity: Some(*motor_target_velocity),
                        motor_max_force: Some(*motor_max_force),
                    });
                }
                Joint::Ball {
                    body_a,
                    body_b,
                    anchor,
                } => {
                    let ha = *self.body_handles.get(body_a).ok_or_else(|| {
                        format!("ball body_a '{body_a}' not found")
                    })?;
                    let hb = *self.body_handles.get(body_b).ok_or_else(|| {
                        format!("ball body_b '{body_b}' not found")
                    })?;
                    let rb_a = &self.rigid_body_set[ha];
                    let rb_b = &self.rigid_body_set[hb];
                    let world_anchor = point![anchor[0], anchor[1], anchor[2]];
                    let local_a = rb_a.position().inverse() * world_anchor;
                    let local_b = rb_b.position().inverse() * world_anchor;
                    let ball = SphericalJointBuilder::new()
                        .local_anchor1(local_a)
                        .local_anchor2(local_b)
                        .contacts_enabled(false)
                        .build();
                    self.impulse_joint_set.insert(ha, hb, ball, true);
                    if let Some(rb) = self.rigid_body_set.get_mut(hb) {
                        // Hang damper so the charm settles below the socket
                        // instead of spinning as a T-pose.
                        rb.set_angular_damping(3.0);
                        rb.set_linear_damping(1.0);
                    }
                    self.authored_joints.push(PhysicsJoint {
                        kind: "ball".into(),
                        body_a: body_a.clone(),
                        body_b: body_b.clone(),
                        anchor: *anchor,
                        axis: [0.0, 0.0, 0.0],
                        limits: None,
                        motor_target_velocity: None,
                        motor_max_force: None,
                    });
                }
            }
        }
        Ok(())
    }

    fn step(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &(),
        );
    }

    fn snapshot_bodies(&self) -> Vec<PhysicsBodyState> {
        let mut bodies = Vec::new();
        for id in &self.body_ids {
            let handle = self.body_handles[id];
            let rb = &self.rigid_body_set[handle];
            let t = rb.translation();
            let r = rb.rotation();
            let lv = rb.linvel();
            let av = rb.angvel();
            bodies.push(PhysicsBodyState {
                id: id.clone(),
                position: [t.x, t.y, t.z],
                rotation_wxyz: [r.w, r.i, r.j, r.k],
                linear_velocity: [lv.x, lv.y, lv.z],
                angular_velocity: [av.x, av.y, av.z],
                collider: self.body_colliders.get(id).cloned().unwrap_or_default(),
            });
        }
        bodies
    }

    fn snapshot_contacts(&self) -> Vec<PhysicsContact> {
        let mut contacts = Vec::new();
        for pair in self.narrow_phase.contact_pairs() {
            if !pair.has_any_active_contact {
                continue;
            }
            let Some(id_a) = self.collider_to_id.get(&pair.collider1) else {
                continue;
            };
            let Some(id_b) = self.collider_to_id.get(&pair.collider2) else {
                continue;
            };
            let c1 = &self.collider_set[pair.collider1];

            let mut pushed = false;
            for manifold in &pair.manifolds {
                for sc in &manifold.data.solver_contacts {
                    let p = sc.point;
                    let n = manifold.data.normal;
                    contacts.push(PhysicsContact {
                        body_a: id_a.clone(),
                        body_b: id_b.clone(),
                        point: [p.x, p.y, p.z],
                        normal: [n.x, n.y, n.z],
                    });
                    pushed = true;
                }
                if !pushed {
                    for contact in &manifold.points {
                        let p = c1.position() * contact.local_p1;
                        let n = c1.position() * manifold.local_n1;
                        contacts.push(PhysicsContact {
                            body_a: id_a.clone(),
                            body_b: id_b.clone(),
                            point: [p.x, p.y, p.z],
                            normal: [n.x, n.y, n.z],
                        });
                        pushed = true;
                    }
                }
            }
        }
        contacts
    }

    fn snapshot_overlaps(&self) -> Vec<PhysicsOverlap> {
        let mut overlaps = Vec::new();
        for (h1, h2, intersecting) in self.narrow_phase.intersection_pairs() {
            if !intersecting {
                continue;
            }
            let (trigger, body) = if let Some(trig) = self.trigger_collider_to_id.get(&h1) {
                let Some(body) = self.collider_to_id.get(&h2) else {
                    continue;
                };
                (trig, body)
            } else if let Some(trig) = self.trigger_collider_to_id.get(&h2) {
                let Some(body) = self.collider_to_id.get(&h1) else {
                    continue;
                };
                (trig, body)
            } else {
                continue;
            };
            overlaps.push(PhysicsOverlap {
                trigger: trigger.clone(),
                body: body.clone(),
            });
        }
        overlaps.sort_by(|a, b| {
            (&a.trigger, &a.body).cmp(&(&b.trigger, &b.body))
        });
        overlaps.dedup_by(|a, b| a.trigger == b.trigger && a.body == b.body);
        overlaps
    }

    fn snapshot_frame(&self, step: u32) -> TrajectoryFrame {
        TrajectoryFrame {
            step,
            bodies: self.snapshot_bodies(),
            contacts: self.snapshot_contacts(),
        }
    }
}

pub fn step_physics(scene: &Scene, steps: u32, dt: f32) -> Result<PhysicsDump, String> {
    let mut world = PhysicsWorld::from_scene(scene, dt)?;
    for _ in 0..steps {
        world.step();
    }
    Ok(PhysicsDump {
        steps,
        dt,
        gravity: [0.0, -9.81, 0.0],
        bodies: world.snapshot_bodies(),
        contacts: world.snapshot_contacts(),
        joints: world.authored_joints.clone(),
        overlaps: world.snapshot_overlaps(),
    })
}

/// Step physics and record a snapshot every `frame_stride` steps, including step 0.
/// `frame_count` is the number of snapshots (and later, PNGs), not the physics step count.
pub fn simulate_trajectory(
    scene: &Scene,
    frame_count: u32,
    frame_stride: u32,
    dt: f32,
) -> Result<Trajectory, String> {
    if frame_count == 0 {
        return Err("frame_count must be ≥ 1".into());
    }
    if frame_stride == 0 {
        return Err("frame_stride must be ≥ 1".into());
    }
    let mut world = PhysicsWorld::from_scene(scene, dt)?;
    let mut frames = Vec::with_capacity(frame_count as usize);
    frames.push(world.snapshot_frame(0));
    for i in 1..frame_count {
        for _ in 0..frame_stride {
            world.step();
        }
        frames.push(world.snapshot_frame(i * frame_stride));
    }
    Ok(Trajectory {
        dt,
        frame_stride,
        frames,
    })
}
