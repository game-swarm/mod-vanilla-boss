# vanilla-boss

Boss NPC 遭遇战模组。管理世界 Boss 和 Arena Boss。

## 职责

- 世界 Boss：在指定房间周期性生成，全服玩家可挑战
- Arena Boss：Arena 模式中按轮次出现
- Boss 特性：多阶段（phases）、血条分段、特殊掉落
- Boss 掉落：击杀后掉落稀有资源/物品
- Boss AI：阶段状态机（Phase 1: 普通攻击 → Phase 2: 范围攻击 → Phase 3: 狂暴）
- Boss 生成定时器：typed setting 位于 `world.toml [mods.vanilla-boss].boss_spawn_interval`

## 依赖

- bevy
- combat-core（使用同类 combat/damage 系统）
- serde

## 配置

typed gameplay config 来自 `world.toml [mods.vanilla-boss]`；`mods.lock [plugins.vanilla-boss]` 只保存 runtime policy/provenance。native register 当前保持 defaults-only parity，并用 versioned defaults 构造 `VanillaBossPlugin` 与其 mod-local `WorldConfig`。

**有效运行配置 (Effective)**:
- `world_bosses_enabled`: 是否启用世界 Boss。
- `arena_bosses_enabled`: 是否启用竞技场 Boss。
- `boss_spawn_interval`: Boss 生成间隔 (ticks)。

这些 resolved 字段进入 Engine config/replay identity；native constructor override 仍只接收 defaults-only register config。

`world.toml` 示例：
```toml
[mods.vanilla-boss]
world_bosses_enabled = true
arena_bosses_enabled = true
boss_spawn_interval = 5000
```

## 事件

- 读取: `WorldConfig`, `Tick`
- 写入: `Drone`（Boss 实体）, `BossAI`（阶段状态机）, `Resource`（掉落物）

## Standalone Development

This crate pins `swarm-engine-api` and `swarm-engine-plugin-sdk` to canonical source `https://github.com/game-swarm/engine-api.git`, exact version `0.1.0`, and identical full revision `0d97444af0c8f8c563bbe58837a4fdf8753630cf`. Cargo fetches both crates directly from that revision.

```sh
git clone <this-mod-repository-url> vanilla-boss
cd vanilla-boss
cargo check
cargo test
```

To adopt a later API/SDK release, update both canonical URLs, both exact versions, and both full Git revisions in `Cargo.toml` together, then regenerate `Cargo.lock` and verify both packages resolve to the same commit.
