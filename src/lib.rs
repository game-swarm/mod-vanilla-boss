use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use swarm_engine_api::prelude::{
    API_VERSION, BodyPart, ConfigFieldDescriptor, ConfigValidator, ConfigValueType,
    DESCRIPTOR_SCHEMA_VERSION, PluginDependency, PluginDescriptor, RoomId, SystemDescriptor,
    TickPhase,
};
use swarm_engine_plugin_sdk::components::{BodyPartRegistry, Drone, Position, Resource};
use swarm_engine_plugin_sdk::traits::SwarmPlugin;

#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct Tick(pub u64);

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    pub world_bosses_enabled: bool,
    pub arena_bosses_enabled: bool,
    pub boss_spawn_interval: u64,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            world_bosses_enabled: true,
            arena_bosses_enabled: true,
            boss_spawn_interval: 5_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BossTemplate {
    pub name: String,
    pub mode: BossMode,
    pub hits: u32,
    pub phases: Vec<u32>,
    pub drops: BTreeMap<String, u32>,
    pub spawn_position: Position,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BossMode {
    #[default]
    World,
    Arena,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct BossAI {
    pub name: String,
    pub mode: BossMode,
    pub phase: BossPhase,
    pub phase_thresholds: Vec<u32>,
    pub drops: BTreeMap<String, u32>,
    pub spawn_position: Position,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BossPhase {
    #[default]
    Phase1,
    Phase2,
    Phase3,
}

#[derive(Resource, Debug, Clone)]
pub struct VanillaBossConfig {
    pub boss_templates: Vec<BossTemplate>,
    pub arena_bosses_enabled: bool,
    pub world_bosses_enabled: bool,
    pub boss_spawn_interval: u64,
}

#[derive(Debug, Clone)]
pub struct VanillaBossPlugin {
    pub boss_templates: Vec<BossTemplate>,
    pub arena_bosses_enabled: bool,
    pub world_bosses_enabled: bool,
    pub boss_spawn_interval: u64,
}

impl Default for VanillaBossPlugin {
    fn default() -> Self {
        Self {
            boss_templates: vec![
                BossTemplate {
                    name: "world-alpha".to_string(),
                    mode: BossMode::World,
                    hits: 100_000,
                    phases: vec![75, 50, 25],
                    drops: BTreeMap::from([
                        ("Energy".to_string(), 5_000),
                        ("Mineral".to_string(), 100),
                    ]),
                    spawn_position: Position {
                        x: 25,
                        y: 25,
                        room: RoomId(0),
                    },
                },
                BossTemplate {
                    name: "arena-champion".to_string(),
                    mode: BossMode::Arena,
                    hits: 50_000,
                    phases: vec![50, 20],
                    drops: BTreeMap::from([("ArenaToken".to_string(), 1)]),
                    spawn_position: Position {
                        x: 25,
                        y: 25,
                        room: RoomId(1),
                    },
                },
            ],
            arena_bosses_enabled: true,
            world_bosses_enabled: true,
            boss_spawn_interval: 5_000,
        }
    }
}

impl Plugin for VanillaBossPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(VanillaBossConfig {
            boss_templates: self.boss_templates.clone(),
            arena_bosses_enabled: self.arena_bosses_enabled,
            world_bosses_enabled: self.world_bosses_enabled,
            boss_spawn_interval: self.boss_spawn_interval,
        })
        .init_resource::<WorldConfig>()
        .init_resource::<Tick>()
        .add_systems(
            Update,
            (
                boss_spawn_system,
                boss_phase_trigger_system,
                boss_ai_system,
                boss_drop_system,
            )
                .chain(),
        );
    }
}

impl SwarmPlugin for VanillaBossPlugin {
    fn descriptor() -> PluginDescriptor {
        PluginDescriptor {
            id: "vanilla-boss".to_string(),
            version: "0.1.0".to_string(),
            api_version: API_VERSION.to_string(),
            dependencies: vec![
                PluginDependency {
                    id: "pve-spawning".to_string(),
                    version_req: ">=0.1.0".to_string(),
                },
                PluginDependency {
                    id: "combat-core".to_string(),
                    version_req: ">=0.1.0".to_string(),
                },
            ],
            config: vec![
                ConfigFieldDescriptor {
                    key: "arena_bosses_enabled".to_string(),
                    value_type: ConfigValueType::Bool,
                    default: true.into(),
                    required: false,
                    validator: None,
                },
                ConfigFieldDescriptor {
                    key: "world_bosses_enabled".to_string(),
                    value_type: ConfigValueType::Bool,
                    default: true.into(),
                    required: false,
                    validator: None,
                },
                ConfigFieldDescriptor {
                    key: "boss_spawn_interval".to_string(),
                    value_type: ConfigValueType::U64,
                    default: 5_000_u64.into(),
                    required: false,
                    validator: Some(ConfigValidator::Positive),
                },
                ConfigFieldDescriptor {
                    key: "boss_templates".to_string(),
                    value_type: ConfigValueType::Array {
                        item_type: "BossTemplate".to_string(),
                    },
                    default: boss_template_defaults(),
                    required: false,
                    validator: Some(ConfigValidator::NonEmptyArray),
                },
            ],
            systems: vec![
                system_descriptor(
                    "vanilla-boss.spawn",
                    0,
                    &["Tick", "VanillaBossConfig", "WorldConfig", "BossAI"],
                    &["EntityLifecycle", "Drone", "Position", "BossAI"],
                ),
                system_descriptor("vanilla-boss.phase-trigger", 1, &["Drone"], &["BossAI"]),
                system_descriptor("vanilla-boss.ai", 2, &["BossAI"], &["Drone"]),
                system_descriptor(
                    "vanilla-boss.drop",
                    3,
                    &["BossAI", "Drone", "Position"],
                    &["EntityLifecycle", "Resource", "Position"],
                ),
            ],
            actions: Vec::new(),
            descriptor_schema_version: DESCRIPTOR_SCHEMA_VERSION.to_string(),
        }
    }
}

fn system_descriptor(
    system_id: &str,
    order: u32,
    reads: &[&str],
    writes: &[&str],
) -> SystemDescriptor {
    SystemDescriptor {
        system_id: system_id.to_string(),
        version: "0.1.0".to_string(),
        phase: TickPhase::Update,
        order,
        reads: reads.iter().map(|name| (*name).to_string()).collect(),
        writes: writes.iter().map(|name| (*name).to_string()).collect(),
        produces_buffers: Vec::new(),
        consumes_buffers: Vec::new(),
        deterministic_iteration: vec!["Entity".to_string()],
    }
}

fn boss_template_defaults() -> serde_json::Value {
    serde_json::json!([
        {
            "name": "world-alpha",
            "mode": "world",
            "hits": 100_000,
            "phases": [75, 50, 25],
            "drops": { "Energy": 5_000, "Mineral": 100 },
            "spawn_position": { "x": 25, "y": 25, "room": 0 }
        },
        {
            "name": "arena-champion",
            "mode": "arena",
            "hits": 50_000,
            "phases": [50, 20],
            "drops": { "ArenaToken": 1 },
            "spawn_position": { "x": 25, "y": 25, "room": 1 }
        }
    ])
}

pub fn boss_spawn_system(
    mut commands: Commands,
    tick: Res<Tick>,
    config: Res<VanillaBossConfig>,
    world: Res<WorldConfig>,
    bosses: Query<&BossAI>,
) {
    let interval = world
        .boss_spawn_interval
        .max(config.boss_spawn_interval)
        .max(1);
    if !tick.0.is_multiple_of(interval) {
        return;
    }
    for template in &config.boss_templates {
        let enabled = match template.mode {
            BossMode::World => config.world_bosses_enabled && world.world_bosses_enabled,
            BossMode::Arena => config.arena_bosses_enabled && world.arena_bosses_enabled,
        };
        if !enabled || bosses.iter().any(|boss| boss.name == template.name) {
            continue;
        }
        commands.spawn((
            boss_drone(template.hits),
            template.spawn_position,
            BossAI {
                name: template.name.clone(),
                mode: template.mode,
                phase: BossPhase::Phase1,
                phase_thresholds: template.phases.clone(),
                drops: template.drops.clone(),
                spawn_position: template.spawn_position,
            },
        ));
    }
}

pub fn boss_phase_trigger_system(mut bosses: Query<(&mut BossAI, &Drone)>) {
    for (mut boss, drone) in &mut bosses {
        let pct = if drone.hits_max == 0 {
            0
        } else {
            (drone.hits as u64 * 100 / drone.hits_max as u64) as u32
        };
        boss.phase = if pct <= *boss.phase_thresholds.get(2).unwrap_or(&25) {
            BossPhase::Phase3
        } else if pct <= *boss.phase_thresholds.get(1).unwrap_or(&50) {
            BossPhase::Phase2
        } else {
            BossPhase::Phase1
        };
    }
}

pub fn boss_ai_system(mut bosses: Query<(&BossAI, &mut Drone)>) {
    for (boss, mut drone) in &mut bosses {
        match boss.phase {
            BossPhase::Phase1 => {}
            BossPhase::Phase2 => {
                drone.hits = drone.hits.saturating_add(10).min(drone.hits_max);
            }
            BossPhase::Phase3 => {
                drone.hits_max = drone.hits_max.saturating_add(1);
            }
        }
    }
}

pub fn boss_drop_system(
    mut commands: Commands,
    bosses: Query<(Entity, &BossAI, &Drone, &Position)>,
) {
    for (entity, boss, drone, position) in &bosses {
        if drone.hits == 0 {
            commands.spawn((
                Resource {
                    amounts: boss.drops.clone().into_iter().collect(),
                },
                *position,
            ));
            commands.entity(entity).despawn();
        }
    }
}

fn boss_drone(hits: u32) -> Drone {
    let registry = BodyPartRegistry::default();
    let mut drone = Drone::new(0, vec![BodyPart::Tough, BodyPart::Attack], &registry);
    drone.hits = hits;
    drone.hits_max = hits;
    drone
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_plugin_defines_world_and_arena_bosses() {
        let plugin = VanillaBossPlugin::default();

        assert!(plugin.world_bosses_enabled);
        assert!(plugin.arena_bosses_enabled);
        assert_eq!(plugin.boss_templates.len(), 2);
        assert!(
            plugin
                .boss_templates
                .iter()
                .any(|boss| boss.mode == BossMode::World)
        );
        assert!(
            plugin
                .boss_templates
                .iter()
                .any(|boss| boss.mode == BossMode::Arena)
        );
    }

    #[test]
    fn boss_drone_uses_requested_hit_points() {
        let drone = boss_drone(123);

        assert_eq!(drone.hits, 123);
        assert_eq!(drone.hits_max, 123);
    }

    #[test]
    fn descriptor_is_valid_and_declares_builtin_dependencies() {
        let descriptor = VanillaBossPlugin::descriptor();
        swarm_engine_api::validation::assert_valid_descriptor(&descriptor);
        assert_eq!(descriptor.id, "vanilla-boss");
        assert_eq!(
            descriptor
                .dependencies
                .iter()
                .map(|dependency| dependency.id.as_str())
                .collect::<Vec<_>>(),
            ["pve-spawning", "combat-core"]
        );
        assert_eq!(descriptor.config.len(), 4);
        assert_eq!(descriptor.systems.len(), 4);
        assert_eq!(
            descriptor
                .config
                .iter()
                .map(|field| field.key.as_str())
                .collect::<Vec<_>>(),
            [
                "arena_bosses_enabled",
                "world_bosses_enabled",
                "boss_spawn_interval",
                "boss_templates"
            ]
        );
        assert_eq!(
            descriptor
                .systems
                .iter()
                .map(|system| system.system_id.as_str())
                .collect::<Vec<_>>(),
            [
                "vanilla-boss.spawn",
                "vanilla-boss.phase-trigger",
                "vanilla-boss.ai",
                "vanilla-boss.drop"
            ]
        );
    }
}
