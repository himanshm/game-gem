<div align="center">

# game-gem

**A blazing-fast, ergonomic 2D game library for Rust — no macros, no magic, just gems.**

[![Crates.io](https://img.shields.io/badge/crates.io-game--gem-orange)](https://crates.io/crates/game-gem)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange?logo=rust)](https://www.rust-lang.org)
[![Edition](https://img.shields.io/badge/edition-2024-red)](https://doc.rust-lang.org/edition-guide/)
[![Platforms](https://img.shields.io/badge/platforms-windows%20%7C%20macos%20%7C%20linux%20%7C%20wasm-lightgrey)](#platform-support)

A modern, modular alternative to `macroquad` — built on `miniquad` + `glam` + `rodio`,
with batteries included for scenes, particles, tweens, animation, collision, audio, and UI.

</div>

---

## Why game-gem?

`macroquad` is great, but it leans on proc macros, global state, and a monolithic API.
`game-gem` is a rethink for developers who want the ergonomics of macroquad with the
architecture of a real engine:

- **No proc macros** — Plain `trait GameState` + `Game::run()` instead of `#[macroquad::main]`.
- **Explicit context** — All engine state passed via `&mut Context`. No hidden globals, no surprises.
- **Feature-gated modules** — Compile only what you use. Disable `audio`, `ui`, `particles`, etc. and they vanish from your binary.
- **Batteries included** — systems that macroquad ships without:
  - Scene management with transitions (fade, slide, push/pop stack)
  - Particle system with emitters, shapes, and pooling
  - Tweening with 30+ easing functions
  - Sprite-sheet animation with a state machine
  - Collision detection (AABB, circle, ray casting, spatial hash)
  - Immediate-mode UI toolkit
  - Spatial 2D audio with distance falloff
  - Multi-camera system with shake, follow, and split-screen
  - Asset manager with reference-counted handles and hot-reload hooks
- **Proper error handling** — `Result` types instead of panics.
- **Zero-cost abstractions** — Feature flags mean unused modules don't exist in your binary.

---

## Quick Start

```rust
use game_gem::prelude::*;

struct MyGame;

impl GameState for MyGame {
    fn update(&mut self, ctx: &mut Context) {
        if ctx.input.keyboard.is_pressed(KeyCode::Escape) {
            ctx.quit();
        }
    }

    fn render(&mut self, ctx: &mut Context) {
        ctx.graphics.clear(Color::SKY_BLUE);
        ctx.graphics.draw_rect(100.0, 100.0, 200.0, 150.0, Color::GOLD);
        ctx.graphics.draw_text("Hello, game-gem!", 50.0, 50.0, 32.0, Color::WHITE);
    }
}

fn main() {
    Game::new()
        .window_title("Hello game-gem!")
        .window_size(800, 600)
        .run(MyGame);
}
```

That's it — no `#[macroquad::main("Conf")]`, no global functions, no hidden setup.
The `Context` is the single point of access for everything: time, input, graphics,
audio, UI, particles, tweens, and the window.

---

## Installation

Add `game-gem` to your `Cargo.toml`:

```toml
[dependencies]
game-gem = "0.1"
```

All optional modules are enabled by default. To slim down your build, disable what
you don't need:

```toml
game-gem = { version = "0.1", default-features = false, features = ["collision"] }
```

### Requirements

- **Rust 1.85+** (uses Rust 2024 edition)
- Platform toolchain for `miniquad`:
  - **Linux:** `libx11` / `libxi` / `libgl` dev packages
  - **macOS:** Xcode command line tools
  - **Windows:** MSVC build tools
  - **Web:** `wasm32-unknown-unknown` target + a JS bundler

---

## Examples

Three runnable examples ship in `examples/`. Clone the repo and try them:

```bash
# Bouncing ball with camera shake, trail, and tweening
cargo run --example basics

# Menu scene with stack-based navigation and transitions
cargo run --example scene_demo

# Fire + sparkle particle emitters reacting to the mouse
cargo run --example particles
```

| Example | What it demonstrates |
|---|---|
| `basics.rs` | Game loop, input actions, camera follow & shake, color HSL, mouse interaction, FPS HUD |
| `scene_demo.rs` | `SceneManager`, `Scene` trait, `Transition::Fade`, keyboard-driven menu |
| `particles.rs` | `ParticleEmitter` configuration, emitter shapes, color/size interpolation, bursts |

---

## Module Overview

| Module | Feature flag | What it gives you |
|---|---|---|
| [`math`]      | (always)    | `Vec2` / `Vec3` / `Mat4` (via `glam`), `Rect`, `Transform`, `Lerp`, `Angle`, shorthand constructors |
| [`color`]     | (always)    | `Color` with HSL/HSV, hex parsing, CSS named constants, lerp, alpha helpers |
| [`time`]      | (always)    | `Time` resource with delta time, FPS, `Timer`, fixed timestep, time scaling |
| [`input`]     | (always)    | Keyboard, mouse, gamepad state, remappable `InputAction`s, gestures |
| [`camera`]    | (always)    | Multi-camera, smooth follow with deadzone, screen shake, zoom clamps, split-screen viewports |
| [`engine`]    | (always)    | `Game` builder, `WindowConfig`, `Context`, `GameState` trait, main loop |
| [`scene`]     | (always)    | `Scene` trait, `SceneManager`, `Transition` (fade / slide), `SceneEvent`s |
| [`assets`]    | (always)    | `AssetHandle<T>`, `AssetManager`, `ImageAsset`, `SpriteSheet`, hot-reload hooks |
| [`collision`] | `collision` | AABB, circle, ray casting, `SpatialHash` broad phase, `CollisionInfo` with penetration & normal |
| [`particles`] | `particles` | `ParticleEmitter`, `EmitterShape` (point/line/circle/rect), per-particle easing, pooling |
| [`tween`]     | `tween`     | 30+ `Ease` functions, generic `Tween<T: Lerp>`, `TweenManager`, callbacks, chaining |
| [`animation`] | `animation` | `Animation` from sprite sheets, `AnimationPlayer`, state machine, frame events |
| [`ui`]        | `ui`        | Immediate-mode `button`, `slider`, `text_input`, `checkbox`, `progress_bar`, layouts |
| [`audio`]     | `audio`     | `Sound` builder, `AudioManager`, music/SFX channels, fade in/out, 2D spatial positioning |

Import everything with a single line:

```rust
use game_gem::prelude::*;
```

The prelude is feature-aware — only items from enabled modules are re-exported.

---

## Feature Flags

```toml
[features]
default = ["audio", "ui", "particles", "animation", "tween", "collision"]

audio      = ["dep:rodio"]
ui         = []
particles  = []
animation  = []
tween      = []
collision  = []
serde      = ["dep:serde", "glam/serde"]   # Serde derive for math + color types
debug      = ["collision"]                 # Extra debug overlays / inspectors
```

| Flag | Default | What it pulls in |
|---|:---:|---|
| `audio`      | ✅ | `rodio` — cross-platform audio output |
| `ui`         | ✅ | Immediate-mode UI toolkit |
| `particles`  | ✅ | Particle emitters & shapes |
| `animation`  | ✅ | Sprite-sheet animation system |
| `tween`      | ✅ | Tweening & easing functions |
| `collision`  | ✅ | Collision detection & spatial hashing |
| `serde`      | ❌ | `Serialize` / `Deserialize` for math & color types |
| `debug`      | ❌ | Enables `collision` + debug visualization helpers |

---

## game-gem vs macroquad

| Concern | `macroquad` | `game-gem` |
|---|---|---|
| Entry point | `#[macroquad::main]` proc macro | Plain `fn main()` + `Game::run(state)` |
| State management | Global variables | Trait-based `GameState` |
| Context passing | Global functions (`draw_circle(...)`) | Explicit `&mut Context` parameter |
| Error handling | Panics | `Result` types |
| Modularity | All-or-nothing | Feature-gated modules |
| Scene management | None built-in | Stack-based `SceneManager` with transitions |
| Particles | None built-in | `ParticleEmitter` with shapes, easing, pooling |
| Tweening | None built-in | 30+ easing functions, `TweenManager` |
| Sprite animation | DIY | `AnimationPlayer` with state machine & events |
| Collision | DIY | AABB / circle / ray / `SpatialHash` |
| UI | Separate `ui` crate, tree-based | Drop-in immediate-mode `button(ctx, ...)` |
| Audio | Basic | Spatial 2D audio, music/SFX channels, fades |
| Cameras | Single global camera | Multi-camera, follow, shake, split-screen |
| Binary size | Larger (everything linked) | Smaller (only pay for what you use) |

`game-gem` doesn't replace `macroquad` for every use case — if you want a 50-line
game jam with zero ceremony, macroquad is hard to beat. `game-gem` is for projects
that have outgrown globals and want a real architecture without leaving the
`miniquad` ecosystem.

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Your Game (GameState)                │
└──────────────────────────┬──────────────────────────────┘
                           │ &mut Context
┌──────────────────────────▼──────────────────────────────┐
│                       Context                           │
│  ┌─────────┐ ┌───────┐ ┌──────────┐ ┌────────┐ ┌─────┐  │
│  │  Time   │ │ Input │ │ Graphics │ │ Camera │ │ ... │  │
│  └─────────┘ └───────┘ └──────────┘ └────────┘ └─────┘  │
│  ┌────────┐ ┌────────┐ ┌──────────┐ ┌────────┐ ┌─────┐  │
│  │  Audio │ │  UI    │ │ Particles│ │ Tweens │ │ ... │  │
│  └────────┘ └────────┘ └──────────┘ └────────┘ └─────┘  │
└──────────────────────────┬──────────────────────────────┘
                           │ Draw commands / events
┌──────────────────────────▼──────────────────────────────┐
│                Game loop (fixed + variable)             │
└──────────────────────────┬──────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────┐
│        miniquad (rendering + windowing + GL)            │
└─────────────────────────────────────────────────────────┘
```

The engine runs a standard `fixed_update` → `update` → `render` loop with a
configurable fixed timestep. Each frame:

1. `Time::tick()` advances clock, FPS, and the fixed-step accumulator.
2. Per-frame input deltas (pressed-this-frame, scroll, mouse delta) are cleared.
3. `GameState::fixed_update()` runs zero or more times at the fixed timestep.
4. `GameState::update()` runs once with variable `dt` (skipped while paused).
5. Engine subsystems (`audio`, `particles`, `tweens`, `ui`, `camera`) are updated.
6. `GameState::render()` issues draw commands into `GraphicsContext`.
7. Commands are flushed and submitted to the GPU.
8. If `ctx.is_quitting()`, the loop exits and `on_exit()` is called.

---

## Platform Support

| Platform | Status |
|---|---|
| Linux (X11/Wayland) | ✅ Supported (via `miniquad`) |
| macOS | ✅ Supported (via `miniquad`) |
| Windows | ✅ Supported (via `miniquad`) |
| Web (wasm32) | ✅ Supported (via `miniquad` + `rodio` web audio) |
| Android | 🔧 Planned (miniquad supports it, engine plumbing WIP) |
| iOS | 🔧 Planned |

---

## Project Status

`game-gem` is actively developed. The public API surface (`GameState`, `Context`,
`Game` builder, all feature-gated modules) is stable in shape, but the underlying
`miniquad` rendering bridge is still being filled in — the `GraphicsContext`
currently records draw commands and the GPU submission path is being completed
upstream. If you want to build on `game-gem` today, expect minor API churn on
the rendering side; the higher-level systems (scenes, particles, tweens, audio,
collision, input, UI) are stable.

### Roadmap

- [x] Core engine: `Game`, `Context`, `GameState`, `WindowConfig`
- [x] Math (via `glam`), `Color`, `Time`, `Input`, `Camera`
- [x] Scene management with transitions
- [x] Particle system with emitters
- [x] Tweening with 30+ easing functions
- [x] Sprite-sheet animation
- [x] Collision detection & spatial hashing
- [x] Immediate-mode UI toolkit
- [x] Spatial audio manager
- [ ] Full `miniquad` rendering bridge (batched draw calls, textures, fonts)
- [ ] Asset hot-reload on debug builds
- [ ] Android & iOS support
- [ ] Multi-window support
- [ ] Custom shader pipelines
- [ ] Scene Inspector / debug overlays (`debug` feature)

---

## Contributing

Contributions are welcome — bug reports, feature requests, docs, and PRs.

```bash
git clone https://github.com/himanshmewada/game-gem
cd game-gem
cargo build
cargo test
cargo run --example basics
```

### Guidelines

- Follow the existing code style — no proc macros, explicit `&mut Context`, no panics in public APIs.
- New modules should be feature-gated behind a flag in `Cargo.toml` and re-exported conditionally from `prelude.rs`.
- Document every public item with `///` doc comments and at least one `# Examples` block where reasonable.
- Run `cargo fmt` and `cargo clippy -- -D warnings` before submitting.

---

## License

Licensed under the **MIT License** — see [LICENSE](LICENSE) for the full text.

Copyright © 2026 Himansh Mewada.

---

## Acknowledgements

- [`miniquad`](https://github.com/not-fl3/miniquad) — the cross-platform rendering & windowing foundation.
- [`glam`](https://github.com/bitshifter/glam-rs) — fast, ergonomic math types.
- [`rodio`](https://github.com/RustAudio/rodio) — cross-platform audio playback.
- [`macroquad`](https://github.com/not-fl3/macroquad) — the inspiration for this project's ergonomics goals.

<div align="center">

**[Report Bug](https://github.com/himanshmewada/game-gem/issues)** ·
**[Request Feature](https://github.com/himanshmewada/game-gem/issues)** ·
**[Crates.io](https://crates.io/crates/game-gem)** ·
**[Docs](https://docs.rs/game-gem)**

Made with Rust and care.

</div>
