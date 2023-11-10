//! This crate contains all gameplay code.

use std::f32::consts::PI;

use arete_public::*;
use c_str_macro::c_str;
use game_module_macro::*;
use nalgebra_glm as glm;
use noise::{NoiseFn, Perlin};

// With ECS, a "Component" is the structure that holds game object (entity) data.
//
// "Entities" are composed exclusively of Components. Any data you wish to assign to
// an entity will need to exist inside one or more components.
//
// A struct must implement `Component` to be used as such. As a convenience, you can
// simply `#[derive(Component)]` to automatically implement `Component` on a struct.

#[derive(Component)]
pub struct Velocity {
    val: Vec3,
}

#[derive(Component)]
pub struct PlayerTank {
    /// The current direction the player tank is facing
    angle: f32,
}

#[derive(Component)]
pub struct AiTank {
    /// This id seeds the noise function used for movement
    id: u32,
}

// A `Resource` is a struct that will have one global instance. It is not assignable to entities.
//
// Resources must implement Default.

#[derive(Resource, Default)]
pub struct Noise {
    generator: Perlin,
}

// With ECS, Components (and Resources) specify your data, and Systems specify your logic.
//
// Systems may take any number of Resources, and/or any number of Queries (described later).
//
// Systems work via dependency injection: simply specify the desired inputs as function parameters,
// and the engine will provide the proper inputs.
//
// In this example, we want to be able to spawn things, so we specify the Engine resource as an input.
//
// To specify a function as a sytem that runs only once at startup, tag it with `#[system_once]`.

#[system_once]
fn spawn_tanks(engine: &Engine) {
    // load the tank static mesh

    let mesh = &DynamicStaticMesh {
        asset_id: engine.load_asset(c_str!("tank.glb")),
    };

    // spawn player tank

    let color = &Color {
        val: Vec3::new(1.0, 0.0, 0.0),
    };

    let point_light = &PointLight {
        position: Vec3::default(),
        intensity: color.val * 5.0,
    };

    engine.spawn(bundle!(
        color,
        mesh,
        point_light,
        &PlayerTank { angle: 0.0 },
        &Transform::default(),
    ));

    // spawn AI tanks

    for id in 1..20 {
        let color = &Color {
            val: tank_color(id),
        };

        let point_light = &PointLight {
            position: Vec3::default(),
            intensity: color.val * 5.0,
        };

        engine.spawn(bundle!(
            color,
            mesh,
            point_light,
            &AiTank { id },
            &Transform::default()
        ));
    }
}

/// Helper function. Generates an RGB color based on the tank id.
fn tank_color(tank_id: u32) -> Vec3 {
    let hue = (tank_id % 20) as f32 * 18.0;
    let x = 1.0 - ((hue / 60.0) % 2.0 - 1.0).abs();

    if hue < 60.0 {
        Vec3::new(1.0, x, 0.0)
    } else if hue < 120.0 {
        Vec3::new(x, 1.0, 0.0)
    } else if hue < 180.0 {
        Vec3::new(0.0, 1.0, x)
    } else if hue < 240.0 {
        Vec3::new(0.0, x, 1.0)
    } else if hue < 300.0 {
        Vec3::new(x, 0.0, 1.0)
    } else {
        Vec3::new(1.0, 0.0, x)
    }
}

// We use a separate startup system to spawn the floor.
//
// Tank spawning and floor spawning could be done in the same startup system,
// but separating them results in cleaner and more understandable code.

#[system_once]
fn spawn_floor(engine: &Engine) {
    let transform = &Transform {
        position: Vec3::new(0.0, -0.5, 0.0),
        scale: Vec3::new(200.0, 1.0, 200.0),
        ..Default::default()
    };

    let color = &Color {
        val: Vec3::new(0.8, 0.8, 0.8),
    };

    let mesh = &DynamicStaticMesh {
        asset_id: engine.load_asset(c_str!("cube.glb")),
    };

    engine.spawn(bundle!(transform, color, mesh));
}

// In order to set the ambient lighting, we specify GlobalLighting as a resource input

#[system_once]
fn set_up_lighting(engine: &Engine, lighting: &mut GlobalLighting) {
    // spawn the camera

    engine.spawn(bundle!(
        &Camera::default(),
        &camera_transform(&Transform::default())
    ));

    // set the ambient lighting intensity

    lighting.ambient_intensity = Vec3::new(0.05, 0.05, 0.05);

    // spawn a sunlight, which will cast shadows

    let sun = &DirectionalLight {
        direction: Vec3::new(0.717, -0.717, 0.0),
        intensity: Vec3::new(0.6, 0.6, 0.6),
    };

    engine.spawn(bundle!(sun));
}

// Here, we create a system to update each AI tank.
//
// Systems are able to access entity data via "Queries". Queries greedily match all
// entities containing *at least* the components of the query. In this function, the
// set of (AiTank, Transform, Color) components matches the AI tank entities.
//
// To specify a function as a system that runs once per frame, tag it with `#[system]`.

#[system]
fn ai_tank_update(
    mut query: Query<(&AiTank, &mut Transform, &Color)>,
    noise: &Noise,
    frame_constants: &FrameConstants,
    engine: &Engine,
) {
    query.par_for_each(|(tank, transform, color)| {
        // Update the tank transform based on a perlin noise function.

        let seed = transform.position / 10.0;
        let noise = noise
            .generator
            .get([seed.x as f64, tank.id as f64, seed.z as f64]) as f32;
        let angle = (0.5 + noise) * 4.0 * PI;

        let tank_direction = Vec3::new(angle.sin(), 0.0, angle.cos());

        transform.position += tank_direction * frame_constants.delta_time * 5.0;
        transform.rotation = glm::quat_angle_axis(angle, &glm::Vec3::y()).into();

        // Shoot one cannonball per frame.

        spawn_cannonball(engine, color, transform);
    });
}

// Update the player tank.

#[system]
fn player_tank_update(
    mut query: Query<(&mut PlayerTank, &mut Transform, &Color)>,
    input: &InputState,
    frame_constants: &FrameConstants,
    engine: &Engine,
) {
    query.par_for_each(|(tank, transform, color)| {
        // Check turn input.

        // keyboard input

        if input.key_d.pressed {
            tank.angle -= frame_constants.delta_time * 2.0;
        }

        if input.key_a.pressed {
            tank.angle += frame_constants.delta_time * 2.0;
        }

        // touch input

        if let Some(touch) = input.touches().next() {
            // touch.position.x is in range [0, 1]. (0.5 - touch.position.x) * 2.0 gives us a value in
            // range [-1, 1], and the extra .2 gives us a margin with max input on the sides of the screen.
            let input_val = ((0.5 - touch.position.x) * 2.2).clamp(-1.0, 1.0);
            tank.angle += frame_constants.delta_time * input_val * 2.0;
        }

        // Calculate direction from angle and orient tank.

        transform.rotation = glm::quat_angle_axis(tank.angle, &glm::Vec3::y()).into();

        // Check forward/back (W/S) input

        let tank_direction = Vec3::new(tank.angle.sin(), 0.0, tank.angle.cos());

        if input.key_w.pressed {
            transform.position += tank_direction * frame_constants.delta_time * 5.0;
        }

        if input.key_s.pressed {
            transform.position -= tank_direction * frame_constants.delta_time * 5.0;
        }

        // Spawn one cannonball per frame.

        if input.key_space.pressed || input.touches_len > 0 {
            spawn_cannonball(engine, color, transform);
        }
    });
}

/// A helper function used by `ai_tank_update` and `player_tank_update`.
/// This function is NOT tagged with `#[system]`, so it is not included in frame processing.
fn spawn_cannonball(engine: &Engine, color: &Color, tank_transform: &Transform) {
    // Shoot from the tip of the cannon, which is (0.0, 1.235, 0.324) in local coordinates
    let position_offset_glm = glm::quat_rotate_vec(
        &tank_transform.rotation,
        &glm::Vec4::new(0.0, 1.235, 0.324, 0.0),
    )
    .xyz();

    let transform = &Transform {
        position: tank_transform.position + position_offset_glm.into(),
        rotation: tank_transform.rotation,
        scale: Vec3::new(0.2, 0.2, 0.2),
    };

    let velocity_glm =
        glm::quat_rotate_vec(&transform.rotation, &glm::Vec4::new(0.0, 0.717, 0.8, 0.0)) * 20.0;

    let velocity = &Velocity {
        val: velocity_glm.xyz().into(),
    };

    let mesh = &DynamicStaticMesh {
        asset_id: engine.load_asset(c_str!("sphere.glb")),
    };

    engine.spawn(bundle!(transform, color, mesh, velocity));
}

#[system]
fn cannonball_update(
    mut query: Query<(&mut Transform, &mut Velocity, &EntityId)>,
    frame_constants: &FrameConstants,
    engine: &Engine,
) {
    query.par_for_each(|(transform, velocity, entity_id)| {
        // Move cannonball by the current velocity.

        transform.position += velocity.val * frame_constants.delta_time;

        // Bounce if position drops below floor.

        if transform.position.y < 0.1 {
            transform.position.y += 0.1 - transform.position.y;

            let damping = Vec3::new(0.8, -0.8, 0.8);
            velocity.val *= damping;
        }

        // Acceleration due to gravity.

        velocity.val.y -= 9.82 * frame_constants.delta_time;

        // Despawn if velocity drops low enough.

        if velocity.val.norm_squared() < 0.1 {
            engine.despawn(*entity_id);
        }
    });
}

#[system]
fn point_light_update(mut query: Query<(&Transform, &mut PointLight)>) {
    query.par_for_each(|(tank_transform, light)| {
        // Position a tank's light directly above it.
        light.position = tank_transform.position + Vec3::new(0.0, 2.0, 0.0);
    });
}

#[system]
fn camera_update(
    mut query_camera: Query<(&Camera, &mut Transform)>,
    query_player_tank: Query<(&PlayerTank, &Transform)>,
) {
    let Some(tank_transform) = query_player_tank.get_first::<Transform>() else {
        return;
    };

    query_camera.par_for_each(|(_, transform)| {
        *transform = camera_transform(tank_transform);
    });
}

fn camera_transform(tank_transform: &Transform) -> Transform {
    // Position the camera above and behind the player tank.

    let camera_local_position =
        glm::quat_rotate_vec3(&tank_transform.rotation, &glm::Vec3::new(0.0, 5.0, -10.0));

    let position = tank_transform.position + camera_local_position.into();
    let direction = tank_transform.position + Vec3::y() - position;

    // glm::quat_look_at seems bugged, need to invert the quaternion
    let rotation = glm::quat_look_at(&direction.into(), &glm::Vec3::y())
        .try_inverse()
        .unwrap()
        .into();

    Transform {
        position,
        rotation,
        ..Default::default()
    }
}

// This includes auto-generated C FFI code (saves you from writing it manually).
include!(concat!(env!("OUT_DIR"), "/ffi.rs"));
