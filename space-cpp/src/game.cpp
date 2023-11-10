#include <arete/arete.hpp>
#include <glm/gtx/quaternion.hpp>
#include <algorithm>
#include <atomic>
#include <cmath>
#include <numbers>
#include <random>
#include <vector>


//--------- constants

/// The width of the playable space.
const int STAGE_WIDTH = 18;

/// The width of the playable space, from the center to the edge.
const float STAGE_HALF_WIDTH = STAGE_WIDTH / 2.f;

/// The vertical length of the stage. Enemies will spawn here.
const float STAGE_LENGTH = 120.f;

/// The radius in which enemies collide/cause damage to the player.
const float ENEMY_DAMAGE_RADIUS = 0.9f;

/// The radius in which the player collides with an upgrade.
const float UPGRADE_RADIUS = 2.f;

/// The radius in which lasers collide/cause damage to enemies.
const float LASER_DAMAGE_RADIUS = 1.5f;

const glm::vec3 EXPLOSION_COLOR{ 2.f, 0.1f, 0.0f };

/// Explosion effect radius.
const float EXPLOSION_SIZE = 0.5f;

/// Explosion effect duration.
const float EXPLOSION_DURATION = 0.5f;

/// Number of particles to spawn per explosion.
const int EXPLOSION_PARTICLE_COUNT = 60;

/// Laser range. Lasers despawn after this distance.
const float LASER_DISTANCE = 70.f;

/// Number of seconds of inactivity (no enemy spawning) between waves.
const float SECONDS_BETWEEN_WAVES = 5.f;

/// Maximum concurrent allies.
const int MAX_LASER_ALLY_COUNT = 20;


//--------- enums

enum class GameStates {
    Start,
    Running,
    Ended,
};

enum class UpgradeType {
    Health,
    Laser,
    Grenade,
    UberLaser,
};

enum class WeaponType {
    Laser,
    Grenade,
};


//--------- wave descriptions

struct EnemyDescription {
    float speed_min;
    float speed_max;
    float turn_rate;
    /// The maximum angle an enemy may turn, in radians.
    float max_angle{ 1.05f };  // 60 degrees
    int health;
    int damage;
    float spawn_rate;
    float scale;
    const char* asset_path;
};

struct UpgradeDescription {
    UpgradeType type;
    float speed_min;
    float speed_max;
    float spawn_rate;
};

struct WaveDescription {
    float duration;
    std::vector<EnemyDescription> enemies;
    std::vector<UpgradeDescription> upgrades;
};

const std::vector<WaveDescription> WAVE_DESCRIPTIONS{{
    .duration = 30.f,
    .enemies = {{  // Persuit drones
        .speed_min = 10.f,
        .speed_max = 20.f,
        .turn_rate = 1.f,
        .health = 100,
        .damage = 10,
        .spawn_rate = 5.f,
        .scale = 1.f,
        .asset_path = "enemy.glb",
    }, {  // Uber drones
        .speed_min = 120.f,
        .speed_max = 120.f,
        .health = 2000,
        .damage = 10,
        .spawn_rate = .12f,
        .scale = 2.f,
        .asset_path = "enemy.glb",
    }},
    .upgrades = {{
        .type = UpgradeType::Grenade,
        .speed_min = 15.f,
        .speed_max = 15.f,
        .spawn_rate = 1.f / 15.f,
    }, {
        .type = UpgradeType::Laser,
        .speed_min = 15.f,
        .speed_max = 15.f,
        .spawn_rate = 1.f / 8.f,
    }},
}, {   // WAVE 2: Idea is to have fast but weak enemies, in a large number, still easy enough to pass but gets the player engaged
    .duration = 30.f,
    .enemies = {{   // Speed drones
        .speed_min = 20.f,
        .speed_max = 50.f,
        .turn_rate = 0.18f,
        .health = 1,
        .damage = 1,
        .spawn_rate = 20.f,
        .scale = 1.f,
        .asset_path = "enemy.glb",
    }, {  // Uber drones
        .speed_min = 120.f,
        .speed_max = 120.f,
        .health = 2000,
        .damage = 10,
        .spawn_rate = .12f,
        .scale = 2.f,
        .asset_path = "enemy.glb",
    }},
    .upgrades = {{
        .type = UpgradeType::Health,
        .speed_min = 15.f,
        .speed_max = 15.f,
        .spawn_rate = 0.03f,
    }, {
        .type = UpgradeType::Grenade,
        .speed_min = 15.f,
        .speed_max = 15.f,
        .spawn_rate = 1.f / 8.f,
    }, {
        .type = UpgradeType::Laser,
        .speed_min = 15.f,
        .speed_max = 15.f,
        .spawn_rate = 1.f / 8.f,
    }},
}, {   // WAVE 3: slower paste with larger enemies that require more shots to kill. Idea is to have the player get used to the game and the controls
    .duration = 35.f,
    .enemies = {{   // Large drones
        .speed_min = 20.f,
        .speed_max = 20.f,
        .turn_rate = .3f,
        .health = 1000,
        .damage = 50,
        .spawn_rate = 2.f,
        .scale = 3.f,
        .asset_path = "enemy.glb",
    }},
    .upgrades = {{
        .type = UpgradeType::Grenade,
        .speed_min = 15.f,
        .speed_max = 15.f,
        .spawn_rate = 1.f / 8.f,
    }, {
        .type = UpgradeType::Laser,
        .speed_min = 15.f,
        .speed_max = 15.f,
        .spawn_rate = 1.f / 8.f,
    }, {
        .type = UpgradeType::Health,
        .speed_min = 15.f,
        .speed_max = 15.f,
        .spawn_rate = 0.03f,
    }},
}, {   // WAVE 4: start to combine the past 3 waves into one.
    .duration = 35.f,
    .enemies = {{  // Persuit drones
        .speed_min = 15.f,
        .speed_max = 15.f,
        .turn_rate = 1.f,
        .health = 1,
        .damage = 10,
        .spawn_rate = 50.f,
        .scale = 1.f,
        .asset_path = "enemy.glb",
    }, {  // Large drones
        .speed_min = 20.f,
        .speed_max = 20.f,
        .health = 1500,
        .damage = 50,
        .spawn_rate = 1.7f,
        .scale = 3.f,
        .asset_path = "enemy.glb",
    }, {  // Uber drones
        .speed_min = 120.f,
        .speed_max = 120.f,
        .health = 2000,
        .damage = 10,
        .spawn_rate = .12f,
        .scale = 2.f,
        .asset_path = "enemy.glb",
    }},
    .upgrades = {{
        .type = UpgradeType::UberLaser,
        .speed_min = 15.f,
        .speed_max = 15.f,
        .spawn_rate = .01f,
    }, {
        .type = UpgradeType::Grenade,
        .speed_min = 15.f,
        .speed_max = 15.f,
        .spawn_rate = 1.f / 8.f,
    }, {
        .type = UpgradeType::Laser,
        .speed_min = 15.f,
        .speed_max = 15.f,
        .spawn_rate = 1.f / 8.f,
    }, {
        .type = UpgradeType::Health,
        .speed_min = 15.f,
        .speed_max = 15.f,
        .spawn_rate = 0.03f,
    }},
}, {   // WAVE 5: final wave, all the waves before, stronger and faster, no more upgrades aside from uber laser, increased spawnrate
    .duration = 60.f,
    .enemies = {{  // Persuit drones
        .speed_min = 20.f,
        .speed_max = 20.f,
        .turn_rate = 1.f,
        .health = 1,
        .damage = 10,
        .spawn_rate = 200.f,
        .scale = 1.f,
        .asset_path = "enemy.glb",
    }, {   // Speed drones
        .speed_min = 20.f,
        .speed_max = 45.f,
        .health = 1,
        .damage = 10,
        .spawn_rate = 50.f,
        .scale = 1.f,
        .asset_path = "enemy.glb",
    }, {   // Large drones
        .speed_min = 20.f,
        .speed_max = 20.f,
        .turn_rate = .2f,
        .health = 2000,
        .damage = 50,
        .spawn_rate = .8f,
        .scale = 3.f,
        .asset_path = "enemy.glb",
    }, {  // Uber drones
        .speed_min = 120.f,
        .speed_max = 120.f,
        .health = 2000,
        .damage = 10,
        .spawn_rate = .3f,
        .scale = 2.f,
        .asset_path = "enemy.glb",
    }},
    .upgrades = {{
        .type = UpgradeType::UberLaser,
        .speed_min = 15.f,
        .speed_max = 15.f,
        .spawn_rate = .02f,
    }, {
        .type = UpgradeType::Grenade,
        .speed_min = 15.f,
        .speed_max = 15.f,
        .spawn_rate = 1.f / 8.f,
    }, {
        .type = UpgradeType::Laser,
        .speed_min = 15.f,
        .speed_max = 15.f,
        .spawn_rate = 1.f / 8.f,
    }, {
        .type = UpgradeType::Health,
        .speed_min = 15.f,
        .speed_max = 15.f,
        .spawn_rate = 0.03f,
    }},
}};


//--------- an atomic int which follows Arete's memory model (allows atomic access when const)

class AreteAtomicInt {
    // we can't use std::atomic because it is not trivially copyable
    mutable int value;

public:

    AreteAtomicInt() : value{} {}
    AreteAtomicInt(int value) : value{ value } {}

    int& get_mut() {
        return value;
    }

    int load(std::memory_order m = std::memory_order_seq_cst) const {
        return reinterpret_cast<std::atomic<int>*>(&value)->load(m);
    }

    int fetch_add(int op) const {
        auto* atomic_val = reinterpret_cast<std::atomic<int>*>(&value);
        return atomic_val->fetch_add(op, std::memory_order_relaxed);
    }
};


//--------- components

COMPONENT(Player) {
    float tilt_angle;
    float fire_rate{ 2.f };
    int damage{ 100 };
};

COMPONENT(SupportUnit) {
    float angle;
    WeaponType weapon;

    /// A value from 0-1 which determines things like fire rate and damage
    float random_scale;
};

COMPONENT(Enemy) {
    int damage;
    float speed;
    /// Homing turn rate, in radians per second.
    float turn_rate;
    float angle;
    float max_angle;
};

COMPONENT(Upgrade) {
    float speed;
    UpgradeType type;
    /// A value from 0-1 which determines things like fire rate and damage
    float random_scale;
};

COMPONENT(Ally) {
    float fire_timer;
};

COMPONENT(Health) {
    AreteAtomicInt value{};
};

COMPONENT(Laser) {
    int damage;
};

COMPONENT(SpartanLaser) {
    float timer;
    float damage_per_second{ 100000 };
    float accumulated_damage;
};

COMPONENT(UberLaser) {
    int damage{ 1000 };
};

COMPONENT(Grenade) {
    int damage{ 300 };
    float damage_radius{ 8.f };
};

COMPONENT(Explosion) {
    float timer;
};

COMPONENT(Velocity) {
    glm::vec3 value;
};

COMPONENT(HealthBarSegment) {
    int index;
};

COMPONENT(Score) {};

COMPONENT(Star) {};

COMPONENT(DespawnOnGameRestart) {};


//--------- resources

RESOURCE(StarSpawnTimer) {
    float value;
};

RESOURCE(GameState) {
    GameStates state{ GameStates::Start };
    AreteAtomicInt score{};

    WaveDescription wave;
    float wave_timer;
    size_t wave_count;

    float upgrade_timers[5];

    /// Lasers are expensive without spatial partitioning, limit them to 20.
    int laser_ally_count;

    bool spawning_enemies{ false };

    void start() {
        state = GameStates::Running;
        score = 0;

        wave = WAVE_DESCRIPTIONS[0];
        wave_timer = 0.f;
        wave_count = 0;

        for (int i = 0; i < 5; i++) {
            upgrade_timers[i] = 0.f;
        }

        laser_ally_count = 0;

        spawning_enemies = false;
    }
};


//--------- helper function declarations (defined at bottom of file)

/// Hue in range [0, 360)
Color color_from_hue(float hue);

bool modify_health(const Health& health, int modification);
void modify_score(const GameState& game_state, int modification);

glm::vec3 screen_position_to_world(
    const ScreenPosition& screen_position,
    const Aspect& aspect,
    const Transform& camera_transform,
    float fov
);

void start_next_wave(GameState& game_state);

void spawn_ally(const Engine&, GameState&, const glm::vec3& player_position, WeaponType, float random_scale);
void spawn_enemy(const Engine& engine, AssetId asset_id, const EnemyDescription& enemy_desc);
void spawn_explosion(const Engine& engine, const glm::vec3& position, const glm::vec3& color);
void spawn_laser(const Engine& engine, const glm::vec3& ally_position, int laser_index, int laser_count);
void spawn_menu_texture(const Engine& engine, const Transform& camera_transform, const char* asset_path);
void spawn_uber_laser(const Engine& engine, const glm::vec3& player_position);

void spawn_support_lasers(const Engine&, const SupportUnit&, Ally&, const Transform&, const Color&);
void spawn_support_grenades(const Engine&, const SupportUnit&, Ally&, const Transform&, const Color&);

float randf();


//--------- startup systems

SYSTEM_ONCE(init_world, const Engine& engine, GlobalLighting& lighting, GameState& game_state) {
    // spawn the sun
    const DirectionalLight sun = {
        .direction = { 0.717, -0.717, 0.0 },
        .intensity = glm::vec3(1.2f),
    };

    engine.spawn(sun);

    // set ambient lighting
    lighting.ambient_intensity = glm::vec3(0.05f);

    // spawn camera
    const Camera camera{
        .fov = 1.5f,
    };

    const glm::vec3 camera_pos = glm::vec3(0.f, 30.f, 0.f);
    const glm::vec3 camera_dir = glm::normalize(glm::vec3(0.f, 0.f, 19.f) - camera_pos);

    Transform camera_transform{
        .position = camera_pos,
        .rotation = glm::quatLookAt(camera_dir, { 0.f, 1.f, 0.f }),
    };

    engine.spawn(camera, camera_transform);

    // spawn initial starfield (background visual effect)
    for (int i = 0; i < 300; i++) {
        const Transform transform{
            .position = {
                randf() * 100.f - 50.f,
                randf() * -10.f - 5.f,
                randf() * 200.f
            },
            .scale = glm::vec3(randf() / 3.f),
        };

        engine.spawn(
            transform,
            Star{},
            Color{ glm::vec3(1.f) },
            DynamicStaticMesh{ engine.load_asset("sphere.glb") }
        );
    }

    // spawn health bar (x axis is right to left)

    const float segment_width = (STAGE_WIDTH - 2) / 100.f;

    for (int i = 0; i < 100; i++) {
        // offset is [-49.5, 49.5]
        const float offset = i - 49.5f;

        const Transform transform{
            .position = { -offset * segment_width, 0.f, -4.f },
            .scale = { segment_width, .25f, .25f },
        };

        engine.spawn(
            transform,
            HealthBarSegment{ i },
            DynamicStaticMesh{ engine.load_asset("cube.glb") }
        );
    }

    // spawn start screen texture

    spawn_menu_texture(engine, camera_transform, "menu_start.glb");
}


//--------- frame update systems

SYSTEM(player_movement,
    const InputState& input,
    const Aspect& aspect,
    const FrameConstants& constants,
    Query<const Camera&, const Transform&> query_camera,
    Query<Player&, Transform&> query_player
) {
    query_player.par_for_each([&](Player& player, Transform& transform) {
        const float old_x = transform.position.x;

        const Camera* camera = query_camera.get_first<Camera>();
        const Transform* camera_transform = query_camera.get_first<Transform>();

        // touch input

        if (input.touches_len > 0) {
            const glm::vec3 target_position = screen_position_to_world(
                input.touches[0].position,
                aspect,
                *camera_transform,
                camera->fov
            );

            transform.position = target_position;
        }

        // mouse input

        if (input.mouse.is_present) {
            const glm::vec3 target_position = screen_position_to_world(
                input.mouse.cursor.position,
                aspect,
                *camera_transform,
                camera->fov
            );

            transform.position = target_position;
        }

        // clamp movement to edges of stage

        transform.position.x = std::clamp(transform.position.x, -STAGE_HALF_WIDTH, STAGE_HALF_WIDTH);
        transform.position.z = std::clamp(transform.position.z, 0.f, 25.f);

        // set rotation (slerp, for smooth rotation)

        player.tilt_angle += (old_x - transform.position.x) * 0.1f;
        player.tilt_angle *= std::pow(0.005f, constants.delta_time);

        transform.rotation = glm::quat(glm::vec3(0.f, 0.f, player.tilt_angle));
    });
}

SYSTEM(update_support_units,
    const FrameConstants& constants,
    Query<const Player&, const Transform&> query_player,
    Query<SupportUnit&, Transform&> query_support
) {
    const Transform* player_transform = query_player.get_first<Transform>();

    if (!player_transform) {
        return;
    }

    query_support.par_for_each([&](SupportUnit& support_unit, Transform& transform) {
        support_unit.angle += constants.delta_time;

        const float x = std::sin(support_unit.angle) * 3.f;
        const float z = std::cos(support_unit.angle) * 3.f;

        transform.position.x = player_transform->position.x - x;
        transform.position.z = player_transform->position.z - z;
    });
}

SYSTEM(spawn_enemies,
    const Engine& engine,
    const FrameConstants& constants,
    const GameState& game_state
) {
    if (!game_state.spawning_enemies) {
        return;
    }

    for (auto enemy_desc = game_state.wave.enemies.cbegin(); enemy_desc != game_state.wave.enemies.cend(); enemy_desc++) {
        // load the asset here, once, to avoid repeated calls to load_asset
        const AssetId enemy_asset_id = engine.load_asset(enemy_desc->asset_path);

        const float expected_spawns = enemy_desc->spawn_rate * constants.delta_time;
        const int spawn_count = (int)expected_spawns + (randf() < std::fmod(expected_spawns, 1.f) ? 1 : 0);

        for (int i = 0; i < spawn_count; i++) {
            spawn_enemy(engine, enemy_asset_id, *enemy_desc);
        }
    }
}

SYSTEM(update_enemies,
    const Engine& engine,
    const FrameConstants& constants,
    const GameState& game_state,
    Query<Enemy&, Transform&, const EntityId&> query_enemy,
    Query<const Ally&, const Transform&, const Health&> query_ally
) {
    using namespace std::numbers;

    const Transform* homing_transform = query_ally.get_first<Transform>();

    if (!homing_transform) {
        return;
    }

    query_enemy.par_for_each([&](Enemy& enemy, Transform& transform, const EntityId& entity_id) {
        // move the enemy

        if (enemy.turn_rate > 0.f
            && transform.position.z < homing_transform->position.z + 30.f
            && transform.position.z > homing_transform->position.z
        ) {
            const float opp = transform.position.x - homing_transform->position.x;
            const float adj = transform.position.z - homing_transform->position.z;
            const float target_angle = std::atan(opp / adj);

            if (target_angle > enemy.angle) {
                enemy.angle = std::min(enemy.angle + enemy.turn_rate * constants.delta_time, target_angle);
            } else {
                enemy.angle = std::max(enemy.angle - enemy.turn_rate * constants.delta_time, target_angle);
            }

            enemy.angle = std::clamp(enemy.angle, -enemy.max_angle, enemy.max_angle);
        }

        transform.rotation = glm::angleAxis(enemy.angle + pi_v<float>, glm::vec3(0.f, 1.f, 0.f));

        const glm::vec3 velocity = glm::rotate(transform.rotation, glm::vec3(0.f, 0.f, enemy.speed));
        transform.position += velocity * constants.delta_time;

        // despawn the enemy when off the screen

        if (transform.position.z < -10.f) {
            engine.despawn(entity_id);
            return;
        }

        // check if the enemy hit the player or its allies

        const float damage_radius = ENEMY_DAMAGE_RADIUS * transform.scale.x;

        // return early/don't iterate allies if we're not near the player
        if (homing_transform->position.z + 3.f < transform.position.z - damage_radius) {
            return;
        }

        query_ally.par_for_each([&](const Ally&, const Transform& ally_transform, const Health& ally_health) {
            if (transform.position.z - damage_radius <= ally_transform.position.z
                && transform.position.z + damage_radius >= ally_transform.position.z
                && transform.position.x - damage_radius <= ally_transform.position.x
                && transform.position.x + damage_radius >= ally_transform.position.x
            ) {
                // inflict damage
                modify_health(ally_health, -enemy.damage);

                // despawn enemy
                spawn_explosion(engine, transform.position, EXPLOSION_COLOR);
                engine.despawn(entity_id);
            }
        });
    });
}

SYSTEM(spawn_player_weapons,
    const Engine& engine,
    const FrameConstants& constants,
    Query<const Player&, Ally&, const Transform&> query
) {
    query.par_for_each([&](const Player& player, Ally& ally, const Transform& transform) {
        ally.fire_timer += constants.delta_time;

        const float fire_rate_inverse = 1.f / player.fire_rate;

        while (ally.fire_timer >= fire_rate_inverse) {
            ally.fire_timer -= fire_rate_inverse;

            // spawn laser
            engine.spawn(
                Transform{
                    .position = transform.position + glm::vec3(0.f, 0.f, 1.f),
                    .scale = { 0.2f, 0.2f, 2.f },
                },
                Velocity{{ 0.f, 0.f, 100.f }},
                Laser{ player.damage },
                Color{{ 10.f, 0.f, 0.f }},
                DynamicStaticMesh{ engine.load_asset("cube.glb") }
            );
        }
    });
}

SYSTEM(spawn_support_weapons,
    const Engine& engine,
    const GameState& game_state,
    const FrameConstants& frame_constants,
    Query<const SupportUnit&, Ally&, const Transform&, const Color&> query
) {
    if (game_state.state != GameStates::Running) {
        return;
    }

    query.par_for_each([&](
        const SupportUnit& unit,
        Ally& ally,
        const Transform& transform,
        const Color& color
    ) {
        ally.fire_timer += frame_constants.delta_time;

        switch (unit.weapon) {
        case WeaponType::Laser:
            spawn_support_lasers(engine, unit, ally, transform, color);
            break;
        case WeaponType::Grenade:
            spawn_support_grenades(engine, unit, ally, transform, color);
            break;
        }
    });
}

SYSTEM(update_lasers,
    const Engine& engine,
    const GameState& game_state,
    const FrameConstants& constants,
    Query<const Laser&, Transform&, const Velocity&, const Color&, const EntityId&> query_laser,
    Query<const Enemy&, const Health&, const Transform&, const EntityId&> query_enemy
) {
    query_laser.par_for_each([&](
        const Laser& laser,
        Transform& transform,
        const Velocity& velocity,
        const Color& color,
        const EntityId& entity_id
    ) {
        // check if laser is beyond range
        if (transform.position.z >= LASER_DISTANCE) {
            engine.despawn(entity_id);
            return;
        }

        // calculate the updated position, so that we can check collisions on a line from the old to new position
        const glm::vec3 next_position = transform.position + velocity.value * constants.delta_time;

        // check for collisions with enemies
        query_enemy.par_for_each([&](
            const Enemy&,
            const Health& health,
            const Transform& enemy_transform,
            const EntityId& enemy_entity_id
        ) {
            // check horizontal distance
            if (std::fabs(enemy_transform.position.x - transform.position.x) > LASER_DAMAGE_RADIUS) {
                return;
            }

            // check vertical distance
            if ((enemy_transform.position.z < transform.position.z - LASER_DAMAGE_RADIUS)
                || (enemy_transform.position.z > next_position.z + LASER_DAMAGE_RADIUS)) {
                return;
            }

            // hit!

            // damage enemy
            if (modify_health(health, -laser.damage)) {
                // destroy enemy
                spawn_explosion(engine, enemy_transform.position, EXPLOSION_COLOR);
                engine.despawn(enemy_entity_id);

                modify_score(game_state, 1);
            }

            // destroy laser
            spawn_explosion(engine, transform.position, color.value);
            engine.despawn(entity_id);
        });

        transform.position = next_position;
    });
}

SYSTEM(update_uber_lasers,
    const Engine& engine,
    const GameState& game_state,
    const FrameConstants& constants,
    Query<const UberLaser&, Transform&, const Velocity&, const EntityId&> query_laser,
    Query<const Enemy&, const Health&, const Transform&, const EntityId&> query_enemy
) {
    query_laser.par_for_each([&](
        const UberLaser& laser,
        Transform& transform,
        const Velocity& velocity,
        const EntityId& entity_id
    ) {
        // check if laser is beyond range
        if (transform.position.z >= STAGE_LENGTH) {
            engine.despawn(entity_id);
            return;
        }

        // calculate the updated position, so that we can check collisions on a line from the old to new position
        const glm::vec3 next_position = transform.position + velocity.value * constants.delta_time;

        // check for collisions with enemies
        query_enemy.par_for_each([&](
            const Enemy&,
            const Health& health,
            const Transform& enemy_transform,
            const EntityId& enemy_entity_id
        ) {
            // check vertical distance
            if (enemy_transform.position.z > next_position.z + LASER_DAMAGE_RADIUS) {
                return;
            }

            // hit!

            // instantly kill enemy (still need to check in case it was already dead)
            if (modify_health(health, -laser.damage)) {
                spawn_explosion(engine, enemy_transform.position, EXPLOSION_COLOR);
                engine.despawn(enemy_entity_id);

                modify_score(game_state, 1);
            }
        });

        transform.position = next_position;
    });
}

SYSTEM(update_grenades,
    const Engine& engine,
    const GameState& game_state,
    const FrameConstants& constants,
    Query<Grenade&, Transform&, Velocity&, const Color&, const EntityId&> query_grenade,
    Query<const Enemy&, const Health&, const Transform&, const EntityId&> query_enemy
) {
    const glm::quat rotation = glm::quat(glm::vec3(1.f, 2.3f, 0.4f) * constants.delta_time);

    query_grenade.par_for_each([&](
        Grenade& grenade,
        Transform& transform,
        Velocity& velocity,
        const Color& color,
        const EntityId& entity_id
    ) {
        transform.position += velocity.value * constants.delta_time;
        transform.rotation *= rotation;

        velocity.value.y -= 40.f * constants.delta_time;

        if (transform.position.y < 0.f) {
            transform.position.y = 0.f;

            // check for collisions with enemies
            query_enemy.par_for_each([&](
                const Enemy&,
                const Health& health,
                const Transform& enemy_transform,
                const EntityId& enemy_entity_id
            ) {
                if (glm::distance(transform.position, enemy_transform.position) <= grenade.damage_radius) {
                    // instantly kill enemy (still need to check in case it was already dead)
                    if (modify_health(health, -grenade.damage)) {
                        spawn_explosion(engine, enemy_transform.position, EXPLOSION_COLOR);
                        engine.despawn(enemy_entity_id);

                        modify_score(game_state, 1);
                    }
                }
            });

            // spawn multiple explosions
            spawn_explosion(engine, transform.position, color.value);

            for (int i = 0; i < 2; i++) {
                const glm::vec3 offset{
                    randf() * 6.f - 3.f,
                    randf() * 6.f - 3.f,
                    randf() * 6.f - 3.f,
                };

                spawn_explosion(engine, transform.position + offset, color.value);
            }

            engine.despawn(entity_id);
        }
    });
}

SYSTEM(spawn_upgrades,
    const Engine& engine,
    const FrameConstants& constants,
    GameState& game_state,
    Query<const Player&, Health&> query_player
) {
    if (game_state.state != GameStates::Running) {
        return;
    }

    for (auto desc = game_state.wave.upgrades.cbegin(); desc != game_state.wave.upgrades.cend(); desc++) {
        if (desc->type == UpgradeType::Laser && game_state.laser_ally_count >= MAX_LASER_ALLY_COUNT) {
            continue;
        }

        if (desc->type == UpgradeType::Health) {
            auto* health = query_player.get_first_mut<Health>();
            if (health && health->value.get_mut() == 100) {
                continue;
            }
        }

        float& spawn_timer = game_state.upgrade_timers[static_cast<int>(desc->type)];
        spawn_timer += constants.delta_time;

        const float spawn_rate_inverse = 1.f / desc->spawn_rate;

        if (spawn_timer >= spawn_rate_inverse) {
            spawn_timer = 0.f;

            // spawn upgrade

            Transform transform{
                .position = {
                    randf() * (STAGE_WIDTH + 1) - STAGE_HALF_WIDTH,
                    0.f,
                    STAGE_LENGTH,
                },
                .scale = glm::vec3(2.f),
            };

            Upgrade upgrade{
                .speed = desc->speed_min + (randf() * (desc->speed_max - desc->speed_min)),
                .type = desc->type,
            };

            AssetId asset_id;
            switch (upgrade.type) {
                case UpgradeType::Health:
                    asset_id = engine.load_asset("powerup_health.glb");
                    break;
                case UpgradeType::UberLaser:
                    asset_id = engine.load_asset("powerup_uber_laser.glb");
                    break;
                case UpgradeType::Laser:
                case UpgradeType::Grenade:
                    asset_id = engine.load_asset("cube.glb");
                    break;
            }

            Color color{};
            if (upgrade.type == UpgradeType::Laser || upgrade.type == UpgradeType::Grenade) {
                upgrade.random_scale = randf();
                color = color_from_hue(upgrade.random_scale * 360.f);
                color.value *= 3.f;
            }

            engine.spawn(
                transform,
                upgrade,
                color,
                DynamicStaticMesh{ asset_id },
                DespawnOnGameRestart{}
            );
        }
    }
}

SYSTEM(update_upgrades,
    const Engine& engine,
    const FrameConstants& constants,
    GameState& game_state,
    Query<const Upgrade&, Transform&, const Color&, const EntityId&> query_upgrade,
    Query<Player&, const Transform&, Health&> query_player
) {
    Player* player = query_player.get_first_mut<Player>();

    if (!player) {
        return;
    }

    const Transform* player_transform = query_player.get_first<Transform>();

    query_upgrade.for_each([&](
        const Upgrade& upgrade,
        Transform& transform,
        const Color& color,
        const EntityId& entity_id
    ) {
        // move the upgrade down the screen
        transform.position.z -= upgrade.speed * constants.delta_time;

        // rotate the upgrade
        transform.rotation *= glm::slerp(
            glm::quat{ 1.f, 0.f, 0.f, 0.f },
            glm::quat(glm::vec3(1.f, 1.f, 0.f)),
            constants.delta_time
        );

        const float distance_to_player = glm::distance(transform.position, player_transform->position);

        // check player-gate collision
        if (distance_to_player <= UPGRADE_RADIUS) {
            switch (upgrade.type) {
                case UpgradeType::Health: {
                    int& health = query_player.get_first_mut<Health>()->value.get_mut();
                    health = std::min(health + 50, 100);
                    break;
                }
                case UpgradeType::UberLaser:
                    spawn_uber_laser(engine, player_transform->position);
                    break;
                case UpgradeType::Laser:
                case UpgradeType::Grenade:
                    spawn_ally(
                        engine,
                        game_state,
                        player_transform->position,
                        upgrade.type == UpgradeType::Laser ? WeaponType::Laser : WeaponType::Grenade,
                        upgrade.random_scale
                    );
                    break;
            }

            spawn_explosion(engine, transform.position, color.value);
            engine.despawn(entity_id);
        }

        // despawn gate when off-camera
        if (transform.position.z < -15.f) {
            engine.despawn(entity_id);
        }
    });
}

SYSTEM(update_explosion_particles,
    const Engine& engine,
    const FrameConstants& constants,
    Query<Explosion&, Transform&, const Velocity&, const EntityId&> query
) {
    query.par_for_each([&](
        Explosion& particle,
        Transform& transform,
        const Velocity& velocity,
        const EntityId& entity_id
    ) {
        particle.timer += constants.delta_time;

        if (particle.timer >= EXPLOSION_DURATION) {
            engine.despawn(entity_id);
        }

        transform.position += velocity.value * constants.delta_time;
        transform.scale = glm::vec3((1.f - particle.timer / EXPLOSION_DURATION) * EXPLOSION_SIZE);
    });
}

SYSTEM(spawn_stars, const Engine& engine, StarSpawnTimer& spawn_timer, const FrameConstants& constants) {
    const float spawn_rate = 1.f / 10.f;

    DynamicStaticMesh static_mesh{ engine.load_asset("sphere.glb") };

    spawn_timer.value += constants.delta_time;

    while (spawn_timer.value >= spawn_rate) {
        spawn_timer.value -= spawn_rate;

        Transform transform{
            .position = {
                randf() * 100.f - 50.f,
                randf() * -10.f - 5.f,
                200.f
            },
            .scale = glm::vec3(randf() / 3.f),
        };

        engine.spawn(
            transform,
            static_mesh,
            Star{},
            Color{ glm::vec3(1.f) }
        );
    }
}

SYSTEM(move_stars, const Engine& engine, const FrameConstants& constants, Query<const Star&, Transform&, const EntityId&> query) {
    query.par_for_each([&](const Star&, Transform& transform, const EntityId& entity_id) {
        transform.position.z -= constants.delta_time * 10.f;

        // despawn when off-camera
        if (transform.position.z < -10.f) {
            engine.despawn(entity_id);
        }
    });
}

SYSTEM(update_ally_health,
    const Engine& engine,
    GameState& game_state,
    Query<const Player&, Health&, const EntityId&> query_player,
    Query<const SupportUnit&, Health&, const EntityId&> query_support
) {
    query_player.for_each([&](const Player&, Health& health, const EntityId& entity_id) {
        if (health.value.get_mut() <= 0) {
            engine.despawn(entity_id);
        }
    });

    query_support.for_each([&](const SupportUnit& unit, Health& health, const EntityId& entity_id) {
        if (health.value.get_mut() <= 0) {
            engine.despawn(entity_id);

            if (unit.weapon == WeaponType::Laser) {
                game_state.laser_ally_count--;
            }
        }
    });
}

SYSTEM(update_game_state,
    const Engine& engine,
    const FrameConstants& constants,
    GameState& game_state,
    const InputState& input,
    Query<const Player&> query_player,
    Query<const Camera&, const Transform&> query_camera,
    Query<const DespawnOnGameRestart&, const EntityId&> query_despawn
) {
    if (game_state.state != GameStates::Running) {
        // check for game start
        if (input.key_space.pressed_this_frame
            || (input.touches_len == 1 && input.touches[0].phase == TouchPhase::Began)
        ) {
            // despawn the menu
            query_despawn.par_for_each([&](const DespawnOnGameRestart&, const EntityId& entity_id) {
                engine.despawn(entity_id);
            });

            game_state.start();

            // spawn player
            engine.spawn(
                Transform{ .scale = glm::vec3(1.f) },
                Color{ glm::vec3(0.5f) },
                DynamicStaticMesh{ engine.load_asset("player.glb") },
                Player{},
                Ally{},
                Health{ 100 },
                DespawnOnGameRestart{}
            );
        }
    } else {
        // update wave status
        game_state.wave_timer += constants.delta_time;

        game_state.spawning_enemies = game_state.wave_timer >= 0.f;

        if (game_state.wave_timer > game_state.wave.duration) {
            start_next_wave(game_state);
        }

        // check for game over (player dead)
        if (game_state.state == GameStates::Running && !query_player.get_first<Player>()) {
            game_state.state = GameStates::Ended;
            game_state.spawning_enemies = false;

            // despawn the world
            query_despawn.par_for_each([&](const DespawnOnGameRestart&, const EntityId& entity_id) {
                engine.despawn(entity_id);
            });

            // spawn game over menu
            spawn_menu_texture(engine, *query_camera.get_first<Transform>(), "menu_restart.glb");
        }
    }
}

SYSTEM(update_health_bar,
    const Engine& engine,
    const GameState& game_state,
    Query<const Player&, Health&> query_player,
    Query<const HealthBarSegment&, Color&, Transform&> query_health_bar
) {
    int health;
    if (Health* player_health = query_player.get_first_mut<Health>()) {
        health = player_health->value.get_mut();
    } else {
        health = game_state.state == GameStates::Start ? 100 : 0;
    }

    // set the color of each segment
    query_health_bar.par_for_each([&](const HealthBarSegment& segment, Color& color, Transform& transform) {
        if (segment.index < health) {
            color.value = { 0.f, 1.f, 0.f }; // green
        } else {
            color.value = { 1.f, 0.f, 0.f }; //red
        }
    });
}

const char* DIGIT_ASSET_PATHS[] = {
    "Number_0.glb",
    "Number_1.glb",
    "Number_2.glb",
    "Number_3.glb",
    "Number_4.glb",
    "Number_5.glb",
    "Number_6.glb",
    "Number_7.glb",
    "Number_8.glb",
    "Number_9.glb",
};

SYSTEM(update_score,
    const Engine& engine,
    const Aspect& aspect,
    GameState& game_state,
    Query<const Score&, const EntityId&> query_score,
    Query<const Camera&, const Transform&> query_camera
) {
    const Camera* camera = query_camera.get_first<Camera>();
    const Transform* camera_transform = query_camera.get_first<Transform>();

    if (!camera) {
        return;
    }

    int score = game_state.score.get_mut();

    // despawn old score

    query_score.par_for_each([&](const Score&, const EntityId& entity_id) {
        engine.despawn(entity_id);
    });

    // create new score

    const float char_width = 1.f; // from the mesh

    const glm::vec3 offset_local{
        std::tan(camera->fov / 2.f - 0.1f) * (aspect.x / aspect.y) * 2.f,
        std::tan(camera->fov / 2.f - 0.1f) * 2.f,
        -2.f,
    };

    const glm::vec3 offset =
        camera_transform->position + glm::rotate(camera_transform->rotation, offset_local);

    const int digit_len = score > 0 ? std::log10(score) + 1 : 1;

    for (int i = 0; i < digit_len; i++) {
        const int digit_value = score % 10;
        score /= 10;

        engine.spawn(
            Score{},
            Transform{
                .position = offset + glm::vec3(i * char_width * 0.05f * 2.f, 0.f, 0.f),
                .rotation = glm::quat(glm::vec3(-0.6f, std::numbers::pi_v<float>, 0.f)),
                .scale = { 0.05f, 0.05f, 0.001f },
            },
            DynamicStaticMesh{ engine.load_asset(DIGIT_ASSET_PATHS[digit_value])}
        );
    }
}


//--------- helper function definitions

Color color_from_hue(float hue) {
    const float x = 1.f - std::fabs(std::fmod(hue / 60.f, 2.f) - 1.f);

    if (hue < 60.f) {
        return {{ 1.f, x, 0.f }};
    } else if (hue < 120.f) {
        return {{ x, 1.f, 0.f }};
    } else if (hue < 180.f) {
        return {{ 0.f, 1.f, x }};
    } else if (hue < 240.f) {
        return {{ 0.f, x, 1.f }};
    } else if (hue < 300.f) {
        return {{ x, 0.f, 1.f }};
    } else {
        return {{ 1.f, 0.f, x }};
    }
}

/// Atomically modifies the health and returns if it resulted in the health dropping to 0.
bool modify_health(const Health& health, int modification) {
    const int prev = health.value.fetch_add(modification);
    return prev > 0 && prev + modification <= 0;
}

void modify_score(const GameState& game_state, int modification) {
    game_state.score.fetch_add(modification);
}

glm::vec3 screen_position_to_world(
    const ScreenPosition& screen_position,
    const Aspect& aspect,
    const Transform& camera_transform,
    float fov
) {
    const float screen_x = (screen_position.x - 0.5f) * 2.f;
    const float screen_y = -(screen_position.y - 0.5f) * 2.f;

    const glm::vec3 ray_dir_local{
        std::atan(std::tan(fov / 2.f) * (aspect.x / aspect.y) * screen_x),
        std::atan(std::tan(fov / 2.f) * screen_y),
        -1.f
    };

    const glm::vec3 ray_dir = glm::normalize(glm::rotate(camera_transform.rotation, ray_dir_local));

    // line plane intersection
    const float ndotu = glm::dot(ray_dir, { 0.f, 1.f, 0.f });
    const float si = glm::dot(camera_transform.position, { 0.f, -1.f, 0.f }) / ndotu;
    const glm::vec3 point = camera_transform.position + si * ray_dir;

    return point;
}

void start_next_wave(GameState& game_state) {
    game_state.wave_timer = -SECONDS_BETWEEN_WAVES;
    game_state.wave_count++;
    game_state.spawning_enemies = false;

    if (game_state.wave_count < WAVE_DESCRIPTIONS.size()) {
        game_state.wave = WAVE_DESCRIPTIONS[game_state.wave_count];
    }
}

void spawn_ally(
    const Engine& engine,
    GameState& game_state,
    const glm::vec3& player_position,
    WeaponType weapon_type,
    float random_scale
) {
    Transform transform {
        .position = player_position - glm::vec3(0.f, 0.f, 2.f),
        .scale = glm::vec3(0.8f),
    };

    Color color = color_from_hue(random_scale * 360.f);
    color.value *= 3.f;

    engine.spawn(
        transform,
        color,
        DynamicStaticMesh{ engine.load_asset("support.glb") },
        SupportUnit{
            .weapon = weapon_type,
            .random_scale = random_scale,
        },
        Ally{},
        Health{ 10 },
        DespawnOnGameRestart{}
    );

    if (weapon_type == WeaponType::Laser) {
        game_state.laser_ally_count++;
    }
}

void spawn_enemy(const Engine& engine, AssetId asset_id, const EnemyDescription& enemy_desc) {
    using namespace std::numbers;

    Transform transform{
        .position = {
            randf() * STAGE_WIDTH - STAGE_HALF_WIDTH,
            0.f,
            STAGE_LENGTH,
        },
        .rotation = glm::quat(glm::vec3(0.f, pi_v<float>, 0.f)),
        .scale = glm::vec3(enemy_desc.scale),
    };

    // randomize enemy speed
    const float speed = enemy_desc.speed_min + (randf() * (enemy_desc.speed_max - enemy_desc.speed_min));

    Enemy enemy{
        .damage = enemy_desc.damage,
        .speed = speed,
        .turn_rate = enemy_desc.turn_rate,
        .max_angle = enemy_desc.max_angle,
    };

    engine.spawn(
        transform,
        enemy,
        Health{ enemy_desc.health },
        DynamicStaticMesh{ asset_id },
        DespawnOnGameRestart{}
    );
}

void spawn_explosion(const Engine& engine, const glm::vec3& position, const glm::vec3& color) {
    const DynamicStaticMesh static_mesh{ engine.load_asset("sphere.glb") };

    // spawn explosion particles

    for (int i = 0; i < EXPLOSION_PARTICLE_COUNT; i++) {
        const Transform transform{
            .position = position,
        };

        const float speed = randf() * 30.f;

        const glm::vec3 direction{
            randf() - 0.5f,
            randf() - 0.5f,
            randf() - 0.5f,
        };

        engine.spawn(
            transform,
            static_mesh,
            Explosion{},
            Velocity{ glm::normalize(direction) * speed },
            Color{ color }
        );
    }
}

void spawn_laser(const Engine& engine, const glm::vec3& ally_position, int laser_index, int laser_count) {
    Transform transform {
        .position = ally_position + glm::vec3(0.f, 0.f, 1.f),
        .scale = { 0.2f, 0.2f, 2.f },
    };

    Velocity velocity{{ 0.f, 0.f, 1.f }};

    int half_laser_count = laser_count / 2;

    if (laser_count % 2 == 0) {
        if (laser_index == half_laser_count - 1) {
            transform.position.x += 0.5;
        } else if (laser_index == half_laser_count) {
            transform.position.x -= 0.5;
        } else if (laser_index < half_laser_count - 1) {
            velocity.value.x = (float(laser_index) - half_laser_count + 1) / half_laser_count * 0.3f;
        } else {
            velocity.value.x = (float(laser_index) - half_laser_count) / half_laser_count * 0.3f;
        }
    } else if (laser_count > 1) {
        velocity.value.x = (float(laser_index) - half_laser_count) / half_laser_count * 0.3f;
    }

    // set laser in direction of velocity
    transform.rotation = glm::quat(glm::vec3(0.f, velocity.value.x, 0.f));

    velocity.value = glm::normalize(velocity.value);

    // offset laser one unit along velocity vector
    transform.position += velocity.value;

    // velocity of 100 units per second
    velocity.value *= 100.f;

    engine.spawn(
        transform,
        velocity,
        Laser{},
        Color{{ 2.f, 0.f, 0.f }},
        DynamicStaticMesh{ engine.load_asset("cube.glb") }
    );
}

void spawn_menu_texture(const Engine& engine, const Transform& camera_transform, const char* asset_path) {
    using namespace std::numbers;

    const glm::vec3 position = camera_transform.position + glm::rotate(camera_transform.rotation, { 0.f, 0.f, -5.f });

    Transform transform {
        .position = position,
        .rotation = camera_transform.rotation * glm::quat(glm::vec3(pi_v<float> / 2.f, 0.f, 0.f)),
        .scale = glm::vec3(2.f),
    };

    engine.spawn(
        transform,
        DespawnOnGameRestart{},
        DynamicStaticMesh{ engine.load_asset(asset_path) }
    );
}

void spawn_uber_laser(const Engine& engine, const glm::vec3& player_position) {
    Transform transform {
        .position = glm::vec3(0.f, 0.f, -5.f),
        .scale = { STAGE_WIDTH + 5.f, 0.2f, 0.2f },
    };

    Velocity velocity{{ 0.f, 0.f, 50.f }};

    engine.spawn(
        transform,
        velocity,
        UberLaser{},
        Color{{ 2.f, 0.f, 0.f }},
        DynamicStaticMesh{ engine.load_asset("cube.glb") }
    );
}

void spawn_support_lasers(
    const Engine& engine,
    const SupportUnit& unit,
    Ally& ally,
    const Transform& transform,
    const Color& color
) {
    const float fire_rate_inverse = 1.f / (2.f + unit.random_scale * 4.f);

    while (ally.fire_timer >= fire_rate_inverse) {
        ally.fire_timer -= fire_rate_inverse;

        const int damage = 50.f + (1.f - unit.random_scale) * 150.f;
        const float speed = 75.f + unit.random_scale * 175.f;

        engine.spawn(
            color,
            Transform{
                .position = transform.position + glm::vec3(0.f, 0.f, 1.f),
                .scale = { 0.2f, 0.2f, 2.f },
            },
            Velocity{{ 0.f, 0.f, speed }},
            Laser{ .damage = damage },
            DynamicStaticMesh{ engine.load_asset("cube.glb") }
        );
    }
}

void spawn_support_grenades(
    const Engine& engine,
    const SupportUnit& unit,
    Ally& ally,
    const Transform& transform,
    const Color& color
) {
    const float FIRE_RATE_INVERSE = 1.f / 1.2f;

    while (ally.fire_timer >= FIRE_RATE_INVERSE) {
        ally.fire_timer -= FIRE_RATE_INVERSE;

        const float speed = 5.f + unit.random_scale * 30.f;
        const float radius = 5.f + (1.f - unit.random_scale) * 5.f;

        engine.spawn(
            color,
            Transform{
                .position = transform.position,
                .scale = glm::vec3(0.5f),
            },
            Velocity{{ 0.f, 25.f, speed }},
            Grenade{ .damage_radius = radius },
            DynamicStaticMesh{ engine.load_asset("cube.glb") }
        );
    }
}

float randf() {
    static std::random_device r;
    static std::default_random_engine engine(r());
    static std::uniform_real_distribution<float> uniform_dist(0.f, 1.f);
    return uniform_dist(engine);
}
