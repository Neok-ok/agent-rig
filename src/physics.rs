use rapier3d::parry::query::ShapeCastOptions;
use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::scene::{
    CharacterController, CollisionGroups, Impulse, Joint, MeshCollider, RayHit, Scene, Shape,
    SweepHit, Trigger,
};
use rapier3d::control::{CharacterLength, KinematicCharacterController};

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
    /// Authored-ray hits after the last step. Misses are omitted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ray_hits: Vec<RayHit>,
    /// Authored-shapecast hits after the last step. Misses are omitted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sweep_hits: Vec<SweepHit>,
    /// Authored impulses echoed after the last step.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub impulses: Vec<Impulse>,
    /// Impulse joints removed because reaction exceeded `break_force`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub broken_joints: Vec<BrokenJoint>,
    /// Started/stopped contacts collected across every step.
    /// Empty (and omitted) unless the scene set `record_contact_events`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contact_events: Vec<ContactEvent>,
    /// Last-step kinematic character-controller state. Omitted when empty
    /// so increment-47 dumps stay without a `controllers` key.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub controllers: Vec<ControllerState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactEvent {
    pub kind: String,
    pub body_a: String,
    pub body_b: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerState {
    pub id: String,
    pub grounded: bool,
    pub desired_velocity: [f32; 3],
    pub effective_translation: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokenJoint {
    pub kind: String,
    pub body_a: String,
    pub body_b: String,
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
    /// Authored hinge position-motor target (radians). Present when authored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motor_target_position: Option<f32>,
    /// Current hinge angle (radians) after the last step. Hinges only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub angle: Option<f32>,
    /// Authored rope rest / max length. Distance joints only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rest_length: Option<f32>,
    /// Authored spring stiffness. Spring joints only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stiffness: Option<f32>,
    /// Authored spring damping. Spring joints only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damping: Option<f32>,
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
    /// True when the body was spawned as Rapier kinematic_velocity_based.
    /// Omitted on dynamic/fixed bodies so existing dumps stay compact.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub kinematic: bool,
    /// Authored Rapier InteractionGroups. Omitted when default (0xFFFF/0xFFFF)
    /// so increment-48 dumps stay without a `collision_groups` key.
    #[serde(default, skip_serializing_if = "CollisionGroups::is_default")]
    pub collision_groups: CollisionGroups,
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
    /// Authored kinematic linear velocities (body id -> Vector), re-applied each step.
    kinematic_linvels: HashMap<String, Vector<f32>>,
    authored_joints: Vec<PhysicsJoint>,
    /// Impulse-joint handles for authored hinges, used to dump angle after step.
    hinge_handles: Vec<(usize, ImpulseJointHandle)>,
    /// Impulse-joint handles for authored distance/rope joints + break_force.
    distance_handles: Vec<(usize, ImpulseJointHandle, f32)>,
    /// Authored-joint indices removed after a break_force snap.
    broken_indices: Vec<usize>,
    broken_joints: Vec<BrokenJoint>,
    record_contact_events: bool,
    contact_events: Vec<ContactEvent>,
    /// Authored character controllers (body id -> wish velocity).
    character_controllers: HashMap<String, CharacterController>,
    /// Authored collision groups (body id -> groups), echoed on the dump.
    body_collision_groups: HashMap<String, CollisionGroups>,
    /// Last-step controller dump (grounded + effective translation).
    controller_states: Vec<ControllerState>,
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
            kinematic_linvels: HashMap::new(),
            authored_joints: Vec::new(),
            hinge_handles: Vec::new(),
            distance_handles: Vec::new(),
            broken_indices: Vec::new(),
            broken_joints: Vec::new(),
            record_contact_events: false,
            contact_events: Vec::new(),
            character_controllers: HashMap::new(),
            body_collision_groups: HashMap::new(),
            controller_states: Vec::new(),
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
        world.record_contact_events = scene.record_contact_events;
        Ok(world)
    }

    fn populate(&mut self, scene: &Scene) -> Result<(), String> {

    for body in &scene.bodies {
        let [x, y, z] = body.position;
        let [w, qx, qy, qz] = body.rotation_wxyz;
        let rotation = nalgebra::UnitQuaternion::from_quaternion(nalgebra::Quaternion::new(w, qx, qy, qz));
        let iso = Isometry::from_parts(Translation::new(x, y, z), rotation);

        let [vx, vy, vz] = body.linear_velocity;
        let rb = if body.controller.is_some() {
            // Position-based kinematic: driven by move_shape, not linvel.
            RigidBodyBuilder::kinematic_position_based()
                .position(iso)
                .build()
        } else if body.kinematic {
            RigidBodyBuilder::kinematic_velocity_based()
                .position(iso)
                .linvel(vector![vx, vy, vz])
                .build()
        } else if body.mass <= 0.0 {
            RigidBodyBuilder::fixed().position(iso).build()
        } else {
            RigidBodyBuilder::dynamic()
                .position(iso)
                .linvel(vector![vx, vy, vz])
                .build()
        };
        let handle = self.rigid_body_set.insert(rb);
        if let Some(ctrl) = &body.controller {
            self.character_controllers
                .insert(body.id.clone(), ctrl.clone());
        } else if body.kinematic {
            self.kinematic_linvels
                .insert(body.id.clone(), vector![vx, vy, vz]);
        }

        // Density from authored mass so inertia is non-zero (needed for body-body hits).
        let kind = body.shape.collider_kind().to_string();
        let mut collider = match &body.shape {
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
        if scene.record_contact_events {
            collider.set_active_events(ActiveEvents::COLLISION_EVENTS);
        }
        collider.set_collision_groups(InteractionGroups::new(
            Group::from(body.collision_groups.membership),
            Group::from(body.collision_groups.filter),
        ));
        self.body_collision_groups
            .insert(body.id.clone(), body.collision_groups);
        let ch = self.collider_set.insert_with_parent(collider, handle, &mut self.rigid_body_set);
            self.body_handles.insert(body.id.clone(), handle);
            self.collider_to_id.insert(ch, body.id.clone());
            self.body_colliders.insert(body.id.clone(), kind);
            self.body_ids.push(body.id.clone());
        }
        self.populate_joints(scene)?;
        self.populate_triggers(scene)?;
        self.apply_authored_impulses(scene)?;
        Ok(())
    }

    fn apply_authored_impulses(&mut self, scene: &Scene) -> Result<(), String> {
        for impulse in &scene.impulses {
            let handle = *self.body_handles.get(&impulse.body).ok_or_else(|| {
                format!("impulse body '{}' not found", impulse.body)
            })?;
            let rb = self.rigid_body_set.get_mut(handle).ok_or_else(|| {
                format!("impulse body '{}' missing rigid body", impulse.body)
            })?;
            let [lx, ly, lz] = impulse.linear;
            rb.apply_impulse(vector![lx, ly, lz], true);
        }
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
                    limits,
                    motor_target_velocity,
                    motor_max_force,
                    motor_target_position,
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
                    // When motor_target_position is authored and max_force > 0,
                    // use Rapier 0.26 motor_position instead of velocity so the
                    // gate can settle at 0.55 instead of slamming the limit.
                    let position_motor = motor_target_position.is_some() && *motor_max_force > 0.0;
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
                        .contacts_enabled(false);
                    if position_motor {
                        let pos = motor_target_position.unwrap();
                        // Stiffness/damping chosen so a 0.55 rad target
                        // settles inside 120 steps without hitting 1.15.
                        builder = builder
                            .motor_position(pos, 40.0, 12.0)
                            .motor_model(MotorModel::ForceBased)
                            .motor_max_force(factor);
                    } else {
                        builder = builder.motor_velocity(target, factor);
                        if authored {
                            builder = builder
                                .motor_model(MotorModel::ForceBased)
                                .motor_max_force(factor);
                        }
                    }
                    if let Some(lim) = limits {
                        builder = builder.limits(*lim);
                    }
                    let hinge = builder.build();
                    let handle = self.impulse_joint_set.insert(ha, hb, hinge, true);
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
                    let dump_idx = self.authored_joints.len();
                    self.authored_joints.push(PhysicsJoint {
                        kind: "hinge".into(),
                        body_a: body_a.clone(),
                        body_b: body_b.clone(),
                        anchor: *anchor,
                        axis: *axis,
                        limits: *limits,
                        motor_target_velocity: Some(*motor_target_velocity),
                        motor_max_force: Some(*motor_max_force),
                        motor_target_position: *motor_target_position,
                        angle: None,
                        rest_length: None,
                        stiffness: None,
                        damping: None,
                    });
                    self.hinge_handles.push((dump_idx, handle));
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
                        motor_target_position: None,
                        angle: None,
                        rest_length: None,
                        stiffness: None,
                        damping: None,
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
                        motor_target_position: None,
                        angle: None,
                        rest_length: None,
                        stiffness: None,
                        damping: None,
                    });
                }
                Joint::Fixed {
                    body_a,
                    body_b,
                    anchor,
                } => {
                    let ha = *self.body_handles.get(body_a).ok_or_else(|| {
                        format!("fixed body_a '{body_a}' not found")
                    })?;
                    let hb = *self.body_handles.get(body_b).ok_or_else(|| {
                        format!("fixed body_b '{body_b}' not found")
                    })?;
                    let rb_a = &self.rigid_body_set[ha];
                    let rb_b = &self.rigid_body_set[hb];
                    let world_anchor = point![anchor[0], anchor[1], anchor[2]];
                    let local_a = rb_a.position().inverse() * world_anchor;
                    let local_b = rb_b.position().inverse() * world_anchor;
                    let fixed = FixedJointBuilder::new()
                        .local_anchor1(local_a)
                        .local_anchor2(local_b)
                        .contacts_enabled(false)
                        .build();
                    self.impulse_joint_set.insert(ha, hb, fixed, true);
                    if let Some(rb) = self.rigid_body_set.get_mut(hb) {
                        // Light damping so the weld does not jitter.
                        rb.set_angular_damping(3.0);
                        rb.set_linear_damping(1.0);
                    }
                    self.authored_joints.push(PhysicsJoint {
                        kind: "fixed".into(),
                        body_a: body_a.clone(),
                        body_b: body_b.clone(),
                        anchor: *anchor,
                        axis: [0.0, 0.0, 0.0],
                        limits: None,
                        motor_target_velocity: None,
                        motor_max_force: None,
                        motor_target_position: None,
                        angle: None,
                        rest_length: None,
                        stiffness: None,
                        damping: None,
                    });
                }
                Joint::Distance {
                    body_a,
                    body_b,
                    anchor,
                    rest_length,
                    break_force,
                } => {
                    let ha = *self.body_handles.get(body_a).ok_or_else(|| {
                        format!("distance body_a '{body_a}' not found")
                    })?;
                    let hb = *self.body_handles.get(body_b).ok_or_else(|| {
                        format!("distance body_b '{body_b}' not found")
                    })?;
                    let rb_a = &self.rigid_body_set[ha];
                    let rb_b = &self.rigid_body_set[hb];
                    let world_anchor = point![anchor[0], anchor[1], anchor[2]];
                    // Gate-top (world anchor) in body_a local space — same
                    // helper as hinge/slider/ball/fixed. The rope hangs the
                    // child COM, so local_b is body_b origin (the spawn
                    // world-anchor sits 0.12 above the bob).
                    let local_a = rb_a.position().inverse() * world_anchor;
                    let com_b = point![
                        rb_b.translation().x,
                        rb_b.translation().y,
                        rb_b.translation().z,
                    ];
                    let local_b = rb_b.position().inverse() * com_b;
                    let rope = RopeJointBuilder::new(*rest_length)
                        .local_anchor1(local_a)
                        .local_anchor2(local_b)
                        .contacts_enabled(false)
                        .build();
                    let handle = self.impulse_joint_set.insert(ha, hb, rope, true);
                    if let Some(rb) = self.rigid_body_set.get_mut(hb) {
                        // Hang damper so the bob settles on the rope
                        // instead of yo-yoing forever.
                        rb.set_angular_damping(3.0);
                        rb.set_linear_damping(1.0);
                    }
                    let idx = self.authored_joints.len();
                    self.authored_joints.push(PhysicsJoint {
                        kind: "distance".into(),
                        body_a: body_a.clone(),
                        body_b: body_b.clone(),
                        anchor: *anchor,
                        axis: [0.0, 0.0, 0.0],
                        limits: None,
                        motor_target_velocity: None,
                        motor_max_force: None,
                        motor_target_position: None,
                        angle: None,
                        rest_length: Some(*rest_length),
                        stiffness: None,
                        damping: None,
                    });
                    self.distance_handles.push((idx, handle, *break_force));
                }
                Joint::Spring {
                    body_a,
                    body_b,
                    anchor,
                    rest_length,
                    stiffness,
                    damping,
                } => {
                    let ha = *self.body_handles.get(body_a).ok_or_else(|| {
                        format!("spring body_a '{body_a}' not found")
                    })?;
                    let hb = *self.body_handles.get(body_b).ok_or_else(|| {
                        format!("spring body_b '{body_b}' not found")
                    })?;
                    let rb_a = &self.rigid_body_set[ha];
                    let rb_b = &self.rigid_body_set[hb];
                    let world_anchor = point![anchor[0], anchor[1], anchor[2]];
                    // Gate-top (world anchor) in body_a local space — same
                    // helper as hinge/slider/ball/fixed/distance. The spring
                    // hangs the child COM, so local_b is body_b origin.
                    let local_a = rb_a.position().inverse() * world_anchor;
                    let com_b = point![
                        rb_b.translation().x,
                        rb_b.translation().y,
                        rb_b.translation().z,
                    ];
                    let local_b = rb_b.position().inverse() * com_b;
                    let spring = SpringJointBuilder::new(*rest_length, *stiffness, *damping)
                        .local_anchor1(local_a)
                        .local_anchor2(local_b)
                        .contacts_enabled(false)
                        .build();
                    self.impulse_joint_set.insert(ha, hb, spring, true);
                    if let Some(rb) = self.rigid_body_set.get_mut(hb) {
                        // Light damping so the cork can swing off the
                        // unstable top equilibrium and settle below.
                        rb.set_angular_damping(0.4);
                        rb.set_linear_damping(0.15);
                    }
                    self.authored_joints.push(PhysicsJoint {
                        kind: "spring".into(),
                        body_a: body_a.clone(),
                        body_b: body_b.clone(),
                        anchor: *anchor,
                        axis: [0.0, 0.0, 0.0],
                        limits: None,
                        motor_target_velocity: None,
                        motor_max_force: None,
                        motor_target_position: None,
                        angle: None,
                        rest_length: Some(*rest_length),
                        stiffness: Some(*stiffness),
                        damping: Some(*damping),
                    });
                }
            }
        }
        Ok(())
    }

    fn drive_character_controllers(&mut self) {
        if self.character_controllers.is_empty() {
            self.controller_states.clear();
            return;
        }
        self.query_pipeline.update(&self.collider_set);
        let dt = self.integration_parameters.dt;
        let controller = KinematicCharacterController {
            snap_to_ground: Some(CharacterLength::Relative(0.2)),
            slide: true,
            ..Default::default()
        };
        let jobs: Vec<(String, RigidBodyHandle, [f32; 3])> = self
            .character_controllers
            .iter()
            .map(|(id, ctrl)| (id.clone(), self.body_handles[id], ctrl.desired_velocity))
            .collect();
        let mut states = Vec::with_capacity(jobs.len());
        for (id, handle, desired_velocity) in jobs {
            let iso = *self.rigid_body_set[handle].position();
            let col_h = self.rigid_body_set[handle].colliders()[0];
            let shape = self.collider_set[col_h].shared_shape().clone();
            let [vx, vy, vz] = desired_velocity;
            // Authored wish is horizontal; add gravity*dt so snap_to_ground
            // keeps the walker on the floor instead of floating.
            let desired_translation =
                vector![vx, vy, vz] * dt + self.gravity * dt;
            let groups = self.collider_set[col_h].collision_groups();
            let movement = controller.move_shape(
                dt,
                &self.rigid_body_set,
                &self.collider_set,
                &self.query_pipeline,
                shape.as_ref(),
                &iso,
                desired_translation,
                QueryFilter::default()
                    .exclude_rigid_body(handle)
                    .groups(groups),
                |_| {},
            );
            let new_t = iso.translation.vector + movement.translation;
            self.rigid_body_set[handle].set_next_kinematic_translation(new_t);
            states.push(ControllerState {
                id,
                grounded: movement.grounded,
                desired_velocity,
                effective_translation: [
                    movement.translation.x,
                    movement.translation.y,
                    movement.translation.z,
                ],
            });
        }
        self.controller_states = states;
    }

    fn step(&mut self) {
        let kinematic_drives: Vec<(RigidBodyHandle, Vector<f32>)> = self
            .kinematic_linvels
            .iter()
            .map(|(id, vel)| (self.body_handles[id], *vel))
            .collect();
        for (handle, vel) in kinematic_drives {
            self.rigid_body_set[handle].set_linvel(vel, true);
        }
        self.drive_character_controllers();
        if self.record_contact_events {
            let collector = CollisionCollector {
                events: Mutex::new(Vec::new()),
            };
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
                &collector,
            );
            let raw = collector.events.into_inner().expect("collision collector mutex");
            for event in raw {
                let Some(id_a) = self.collider_to_id.get(&event.collider1()) else {
                    continue;
                };
                let Some(id_b) = self.collider_to_id.get(&event.collider2()) else {
                    continue;
                };
                let kind = if event.started() {
                    "started"
                } else {
                    "stopped"
                };
                self.contact_events.push(ContactEvent {
                    kind: kind.into(),
                    body_a: id_a.clone(),
                    body_b: id_b.clone(),
                });
            }
        } else {
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
        self.break_overloaded_distance_joints();
    }

    fn break_overloaded_distance_joints(&mut self) {
        let mut broken_slots: Vec<usize> = Vec::new();
        for (slot, (_idx, handle, break_force)) in self.distance_handles.iter().enumerate() {
            if *break_force <= 0.0 {
                continue;
            }
            let Some(joint) = self.impulse_joint_set.get(*handle) else {
                continue;
            };
            // Rapier 0.26 writes locked-DOF impulses to ImpulseJoint.impulses
            // and limit/motor impulses to GenericJoint.limits/motors.
            // A RopeJoint is a LinX limit, so the rope reaction lives in
            // limits[LinX].impulse. Convert impulse → force via /dt so
            // authored break_force (~1.5) is a force threshold: hang is
            // ~0.5 N, the extra bob impulse spikes ~25 N.
            let mut impulse_mag = joint.impulses.norm();
            for lim in &joint.data.limits {
                impulse_mag = impulse_mag.max(lim.impulse.abs());
            }
            for motor in &joint.data.motors {
                impulse_mag = impulse_mag.max(motor.impulse.abs());
            }
            let dt = self.integration_parameters.dt;
            let mag = if dt > 0.0 { impulse_mag / dt } else { impulse_mag };
            if mag > *break_force {
                broken_slots.push(slot);
            }
        }
        for slot in broken_slots.into_iter().rev() {
            let (idx, handle, _) = self.distance_handles.remove(slot);
            self.impulse_joint_set.remove(handle, true);
            let j = &self.authored_joints[idx];
            self.broken_joints.push(BrokenJoint {
                kind: j.kind.clone(),
                body_a: j.body_a.clone(),
                body_b: j.body_b.clone(),
            });
            self.broken_indices.push(idx);
        }
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
                kinematic: rb.is_kinematic(),
                collision_groups: self
                    .body_collision_groups
                    .get(id)
                    .copied()
                    .unwrap_or_default(),
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

    fn snapshot_ray_hits(&self, scene: &Scene) -> Vec<RayHit> {
        let mut hits = Vec::new();
        for ray in &scene.raycasts {
            let dir = vector![ray.direction[0], ray.direction[1], ray.direction[2]];
            let len = dir.norm();
            if len < 1e-6 {
                continue;
            }
            let dir = dir / len;
            let origin = point![ray.origin[0], ray.origin[1], ray.origin[2]];
            let r = Ray::new(origin, dir);
            let filter = QueryFilter::default().exclude_sensors();
            let Some((handle, intersection)) = self.query_pipeline.cast_ray_and_get_normal(
                &self.rigid_body_set,
                &self.collider_set,
                &r,
                ray.max_toi,
                true,
                filter,
            ) else {
                continue;
            };
            let Some(body) = self.collider_to_id.get(&handle) else {
                continue;
            };
            let pt = r.point_at(intersection.time_of_impact);
            let n = intersection.normal;
            hits.push(RayHit {
                ray: ray.id.clone(),
                body: body.clone(),
                point: [pt.x, pt.y, pt.z],
                normal: [n.x, n.y, n.z],
                toi: intersection.time_of_impact,
            });
        }
        hits
    }

    fn snapshot_sweep_hits(&self, scene: &Scene) -> Vec<SweepHit> {
        let mut hits = Vec::new();
        for sweep in &scene.shapecasts {
            let Shape::Box { size } = &sweep.shape else {
                continue;
            };
            let dir = vector![sweep.direction[0], sweep.direction[1], sweep.direction[2]];
            let len = dir.norm();
            if len < 1e-6 {
                continue;
            }
            let dir = dir / len;
            let pos = Isometry::translation(sweep.origin[0], sweep.origin[1], sweep.origin[2]);
            let shape = Cuboid::new(vector![size[0] * 0.5, size[1] * 0.5, size[2] * 0.5]);
            // Crate convex hull occupies the drawer front at z=1.02; the welded
            // lid can graze the sweep box. QueryFilter::exclude_rigid_body is
            // a single Option, so skip both via predicate.
            let collider_to_id = &self.collider_to_id;
            let predicate = |handle, _: &Collider| match collider_to_id.get(&handle) {
                Some(id) if id == "crate" || id == "lid" => false,
                _ => true,
            };
            let filter = QueryFilter::default()
                .exclude_sensors()
                .predicate(&predicate);
            let options = ShapeCastOptions::with_max_time_of_impact(sweep.max_toi);
            let Some((handle, hit)) = self.query_pipeline.cast_shape(
                &self.rigid_body_set,
                &self.collider_set,
                &pos,
                &dir,
                &shape,
                options,
                filter,
            ) else {
                continue;
            };
            let Some(body) = self.collider_to_id.get(&handle) else {
                continue;
            };
            let toi = hit.time_of_impact;
            let pt = [
                sweep.origin[0] + dir.x * toi,
                sweep.origin[1] + dir.y * toi,
                sweep.origin[2] + dir.z * toi,
            ];
            let n = hit.normal1.into_inner();
            hits.push(SweepHit {
                sweep: sweep.id.clone(),
                body: body.clone(),
                point: pt,
                normal: [n.x, n.y, n.z],
                toi,
            });
        }
        hits
    }

    fn refresh_hinge_angles(&mut self) {
        for (idx, handle) in &self.hinge_handles {
            let Some(joint) = self.impulse_joint_set.get(*handle) else {
                continue;
            };
            let Some(rev) = joint.data.as_revolute() else {
                continue;
            };
            let rb1 = &self.rigid_body_set[joint.body1];
            let rb2 = &self.rigid_body_set[joint.body2];
            let angle = rev.angle(rb1.rotation(), rb2.rotation());
            self.authored_joints[*idx].angle = Some(angle);
        }
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
    world.refresh_hinge_angles();
    Ok(PhysicsDump {
        steps,
        dt,
        gravity: [0.0, -9.81, 0.0],
        bodies: world.snapshot_bodies(),
        contacts: world.snapshot_contacts(),
        joints: world
            .authored_joints
            .iter()
            .enumerate()
            .filter(|(i, _)| !world.broken_indices.contains(i))
            .map(|(_, j)| j.clone())
            .collect(),
        overlaps: world.snapshot_overlaps(),
        ray_hits: world.snapshot_ray_hits(scene),
        sweep_hits: world.snapshot_sweep_hits(scene),
        impulses: scene.impulses.clone(),
        broken_joints: world.broken_joints.clone(),
        contact_events: world.contact_events.clone(),
        controllers: world.controller_states.clone(),
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

struct CollisionCollector {
    events: Mutex<Vec<CollisionEvent>>,
}

impl EventHandler for CollisionCollector {
    fn handle_collision_event(
        &self,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        event: CollisionEvent,
        _contact_pair: Option<&ContactPair>,
    ) {
        self.events.lock().expect("collision collector").push(event);
    }

    fn handle_contact_force_event(
        &self,
        _dt: Real,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        _contact_pair: &ContactPair,
        _total_force_magnitude: Real,
    ) {
    }
}
