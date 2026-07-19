# vanilla-boss

Boss NPC 遭遇战模组。管理世界 Boss 和 Arena Boss。

## 职责

- 世界 Boss：在指定房间周期性生成，全服玩家可挑战
- Arena Boss：Arena 模式中按轮次出现
- Boss 特性：多阶段（phases）、血条分段、特殊掉落
- Boss 掉落：击杀后掉落稀有资源/物品
- Boss AI：阶段状态机（Phase 1: 普通攻击 → Phase 2: 范围攻击 → Phase 3: 狂暴）
- Boss 生成定时器：通过 `engine/mods.lock` 的 `plugins.vanilla-boss.config.boss_spawn_interval` 配置

## 依赖

- bevy
- combat-core（使用同类 combat/damage 系统）
- serde

## 配置

当前有效配置来自 `engine/mods.lock` 的 `plugins.vanilla-boss.config`。`world.toml [[mods]]` 不是当前 `WorldConfig` schema。

**有效运行配置 (Effective)**:
- `world_bosses_enabled`: 是否启用世界 Boss。
- `arena_bosses_enabled`: 是否启用竞技场 Boss。
- `boss_spawn_interval`: Boss 生成间隔 (ticks)。

这些字段由 `engine/src/main.rs` 读取并注入 `VanillaBossPlugin`。

`engine/mods.lock` 示例：
```toml
[plugins.vanilla-boss]
enabled = true
config = { world_bosses_enabled = true, arena_bosses_enabled = true, boss_spawn_interval = 5000 }
```

## 事件

- 读取: `WorldConfig`, `Tick`
- 写入: `Drone`（Boss 实体）, `BossAI`（阶段状态机）, `Resource`（掉落物）

## Standalone Development

This crate pins `swarm-engine-api` and `swarm-engine-plugin-sdk` to version `0.1.0` at the `v0.1.0` engine API Git tag. Cargo fetches both crates directly from that release.

```sh
git clone <this-mod-repository-url> vanilla-boss
cd vanilla-boss
cargo check
cargo test
```

To adopt a later API/SDK release, update both exact versions and both Git tags in `Cargo.toml` together.
