#include <arete/arete.hpp>
#include <glm/gtx/quaternion.hpp>
#include "PerlinNoise.hpp"
#include <algorithm>
#include <cmath>
#include <numbers>

// With ECS, a "Component" is the structure that holds game object (entity) data.
//
// "Entities" are composed exclusively of Components. Any data you wish to assign to
// an entity will need to exist inside one or more components.
//
// COMPONENT is a macro which declares a struct that may be used as a component.
// You may add as many members to the struct as needed.

COMPONENT(Velocity) {
    glm::vec3 value;
};

COMPONENT(PlayerTank) {
    /// The current direction the player tank is facing
    float angle;
};

COMPONENT(AiTank) {
    /// This id seeds the noise function used for movement
    int id;
};

// A `Resource` is a struct which will have only a single global instance. It is not assignable to entities.
//
// Resources must be default-constructable.

RESOURCE(Noise) {
    siv::PerlinNoise perlin;
};


// Helper function declarations (definitions at bottom of file).

Color tank_color(int tank_id);
void spawn_cannonball(const Engine& engine, const Color& color, const Transform& transform);
Transform camera_transform(const Transform& tank_transform);


// With ECS, Components (and Resources) specify your data, and Systems specify your logic.
//
// Systems may take any number of Resources, and/or any number of Queries (described later).
//
// Systems work via dependency injection: simply specify the desired inputs as function parameters,
// and the engine will provide the proper inputs.
//
// In this example, we want to be able to spawn things, so we specify the Engine resource as an input.
//
// To specify a function as a sytem that runs only once at startup, use the SYSTEM_ONCE macro.

SYSTEM_ONCE(spawn_tanks, const Engine& engine) {
    // load the tank static mesh

    DynamicStaticMesh mesh = { engine.load_asset("tank.glb") };

    // spawn player tank

    Color color = tank_color(0);

    PointLight point_light = {
        .intensity = color.value * 5.f,
    };

    // create a new entity consisting of the provided components
    engine.spawn(color, mesh, point_light, Transform{}, PlayerTank{});

    // spawn AI tanks

    for (int id = 1; id < 20; id++) {
        Color color = tank_color(id);

        PointLight point_light = {
            .intensity = color.value * 5.f,
        };

        engine.spawn(color, mesh, point_light, Transform{}, AiTank{ id });
    }
}

// We use a separate startup system to spawn the floor.
//
// Tank spawning and floor spawning could be done in the same startup system,
// but separating them results in cleaner and more understandable code.

SYSTEM_ONCE(spawn_floor, const Engine& engine) {
    Transform floor_transform{};
    floor_transform.position.y = -0.5f;
    floor_transform.scale = { 200.f, 1.f, 200.f };

    DynamicStaticMesh floor_mesh{ engine.load_asset("cube.glb") };

    Color floor_color{ glm::vec3(0.8f) };

    engine.spawn(floor_transform, floor_color, floor_mesh);
}

// In order to set the ambient lighting, we specify GlobalLighting as a resource input

SYSTEM_ONCE(set_up_lighting, const Engine& engine, GlobalLighting& lighting) {
    // spawn the camera

    engine.spawn(Camera{}, camera_transform({}));

    // set the ambient lighting intensity

    lighting.ambient_intensity = glm::vec3(0.05f);

    // spawn a sunlight, which will cast shadows

    DirectionalLight sun = {
        .direction = glm::vec3(0.717, -0.717, 0.0),
        .intensity = glm::vec3(0.6f),
    };

    engine.spawn(sun);
}

// Here, we create a system to update each AI tank.
//
// Systems are able to access entity data via "Queries". Queries greedily match all
// entities containing *at least* the components of the query. In this function, the
// set of (AiTank, Transform, Color) components matches the AI tank entities.
//
// To specify a function as a system that runs once per frame, use the `SYSTEM` macro.

SYSTEM(
    ai_tank_update,
    Query<const AiTank&, Transform&, const Color&> query,
    const Noise& noise,
    const FrameConstants& frame_constants,
    const Engine& engine
) {
    query.par_for_each([&](const AiTank& tank, Transform& transform, const Color& color) {
        // Update the tank transform based on a perlin noise function.

        const glm::vec3 seed = transform.position / 10.f;
        const float noise_val = noise.perlin.noise3D(seed.x, tank.id, seed.z);
        const float angle = (0.5f + noise_val) * 4.f * std::numbers::pi;

        const glm::vec3 tank_direction(sinf(angle), 0.0, cosf(angle));

        transform.position += tank_direction * frame_constants.delta_time * 5.f;
        transform.rotation = glm::angleAxis(angle, glm::vec3(0.f, 1.f, 0.f));

        // Shoot one cannonball ber frame.

        spawn_cannonball(engine, color, transform);
    });
}

// Update the player tank.

SYSTEM(
    player_tank_update,
    Query<PlayerTank&, Transform&, const Color&> query,
    const InputState& input,
    const FrameConstants& frame_constants,
    const Engine& engine
) {
    query.par_for_each([&](PlayerTank& tank, Transform& transform, const Color& color) {
        // Check turn input.

        // keyboard input

        if (input.key_d.pressed) {
            tank.angle -= frame_constants.delta_time * 2.f;
        }

        if (input.key_a.pressed) {
            tank.angle += frame_constants.delta_time * 2.f;
        }

        // touch input

        if (input.touches_len > 0) {
            const float touch_position = input.touches[0].position.x;
            // touch_position is in range [0, 1]. (.5 - touch_position) * 2 gives us a value in range
            // [-1, 1], and the extra .2 gives us a margin with max input on the sides of the screen.
            const float input_val = std::clamp(((0.5f - touch_position) * 2.2f), -1.f, 1.f);
            tank.angle += frame_constants.delta_time * input_val * 2.f;
        }

        // Calculate direction from angle and orient tank.

        transform.rotation = glm::angleAxis(tank.angle, glm::vec3(0.f, 1.f, 0.f));

        // Check forward/back (W/S) input

        const glm::vec3 tank_direction(sinf(tank.angle), 0.f, cosf(tank.angle));

        if (input.key_w.pressed) {
            transform.position += tank_direction * frame_constants.delta_time * 5.f;
        }

        if (input.key_s.pressed) {
            transform.position -= tank_direction * frame_constants.delta_time * 5.f;
        }

        // Spawn one cannonball per frame.

        if (input.key_space.pressed || input.touches_len > 0) {
            spawn_cannonball(engine, color, transform);
        }
    });
}

SYSTEM(
    cannonball_update,
    Query<Transform&, Velocity&, const EntityId&> query,
    const FrameConstants& frame_constants,
    const Engine& engine
) {
    query.par_for_each([&](Transform& transform, Velocity& velocity, const EntityId& entity_id) {
        // Move cannonball by the current velocity.

        transform.position += velocity.value * frame_constants.delta_time;

        // Bounce if position drops below floor.

        if (transform.position.y < 0.1f) {
            transform.position.y += 0.1f - transform.position.y;

            // Damping.
            velocity.value *= glm::vec3(0.8f, -0.8f, 0.8f);
        }

        // Acceleration due to gravity.

        velocity.value.y -= 9.82f * frame_constants.delta_time;

        // Despawn if velocity drops low enough.

        if (glm::length2(velocity.value) < 0.1f) {
            engine.despawn(entity_id);
        }
    });
}

SYSTEM(point_light_update, Query<const Transform&, PointLight&> query) {
    query.par_for_each([](const Transform& tank_transform, PointLight& light) {
        // Position a tank's light directly above it.
        light.position = tank_transform.position + glm::vec3(0.f, 2.f, 0.f);
    });
}

SYSTEM(camera_update,
    Query<const Camera&, Transform&> query_camera,
    Query<const PlayerTank&, const Transform&> query_player_tank
) {
    const Transform* tank_transform = query_player_tank.get_first<Transform>();

    if (!tank_transform) {
        return;
    }

    query_camera.par_for_each([&](const Camera&, Transform& transform) {
        transform = camera_transform(*tank_transform);
    });
}


// Helper function definitions.

/// Generates an RGB color based on the tank id.
Color tank_color(int tank_id) {
    Color color{};

    float hue = (tank_id % 20) * 18.f;
    float x = 1.f - fabsf(fmodf(hue / 60.f, 2.f) - 1.f);

    if (hue < 60.f) {
        color.value.r = 1.f;
        color.value.g = x;
    } else if (hue < 120.f) {
        color.value.r = x;
        color.value.g = 1.f;
    } else if (hue < 180.f) {
        color.value.g = 1.f;
        color.value.b = x;
    } else if (hue < 240.f) {
        color.value.g = x;
        color.value.b = 1.f;
    } else if (hue < 300.f) {
        color.value.r = x;
        color.value.b = 1.f;
    } else {
        color.value.r = 1.f;
        color.value.b = x;
    }

    return color;
}

void spawn_cannonball(const Engine& engine, const Color& color, const Transform& transform) {
    // Shoot from the tip of the cannon, which is (0.0, 1.235, 0.324) in local coordinates
    const glm::vec3 spawn_offset = glm::rotate(transform.rotation, glm::vec3(0.0, 1.235, 0.324));

    Transform spawn_transform{
        .position = transform.position + spawn_offset,
        .rotation = transform.rotation,
        .scale = glm::vec3(0.2),
    };

    Velocity velocity { glm::rotate(transform.rotation, glm::vec3(0.0, 0.717, 0.8)) * 20.f };

    DynamicStaticMesh mesh { engine.load_asset("sphere.glb") };

    engine.spawn(spawn_transform, color, mesh, velocity);
}

Transform camera_transform(const Transform& tank_transform) {
    // Position the camera above and behind the player tank.

    const glm::vec3 camera_local_position = glm::rotate(tank_transform.rotation, glm::vec3(0.f, 5.f, -10.f));

    const glm::vec3 position = tank_transform.position + camera_local_position;
    const glm::vec3 direction = glm::normalize(tank_transform.position + glm::vec3(0, 1, 0) - position);
    const glm::quat rotation = glm::quatLookAt(direction, glm::vec3(0, 1, 0));

    return {
        .position = position,
        .rotation = rotation,
    };
}
