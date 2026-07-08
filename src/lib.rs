//! # game-gem
//!
//! A blazing-fast, ergonomic 2D game library for Rust — **no macros, no magic, just gems.**
//!
//! ## Why game-gem?
//!
//! `game-gem` is designed as a modern, modular alternative to `macroquad` with:
//!
//! - **No proc macros** — Plain `trait GameState` + `Game::run()` instead of `#[macroquad::main]`
//! - **Explicit context** — All state passed via `&mut Context`, no global variables
//! - **Feature-gated modules** — Only compile what you use (audio, UI, particles, etc.)
//! - **Built-in systems** that macroquad lacks entirely:
//!   - Scene management with transitions
//!   - Particle system with emitters
//!   - Tweening with 30+ easing functions
//!   - Sprite animation with state machine
//!   - Collision detection (AABB, circle, ray casting, spatial hash)
//!   - Immediate-mode UI toolkit
//!   - Spatial audio with 2D positioning
//!   - Camera system with shake, follow, and split-screen
//!   - Asset management with hot-reload support
//! - **Zero-cost abstractions** — Feature flags mean unused modules don't exist in your binary
//! - **Proper error handling** — `Result` types instead of panics
//!
//! ## Quick Start
//!
//! ```no_run
//! use game_gem::prelude::*;
//!
//! struct MyGame;
//!
//! impl GameState for MyGame {
//!     fn update(&mut self, ctx: &mut Context) {
//!         if ctx.input.keyboard.is_pressed(KeyCode::Escape) {
//!             ctx.quit();
//!         }
//!     }
//!
//!     fn render(&mut self, ctx: &mut Context) {
//!         ctx.graphics.clear(Color::SKY_BLUE);
//!         ctx.graphics.draw_rect(100.0, 100.0, 200.0, 150.0, Color::GOLD);
//!         ctx.graphics.draw_text("Hello, game-gem!", 50.0, 50.0, 32.0, Color::WHITE);
//!     }
//! }
//!
//! fn main() {
//!     Game::new()
//!         .window_title("Hello game-gem!")
//!         .window_size(800, 600)
//!         .run(MyGame);
//! }
//! ```
//!
//! ## Module Overview
//!
//! | Module | Feature flag | Description |
//! |--------|-------------|-------------|
//! | [`math`] | (always) | Vec2, Rect, Transform, Angle, Lerp |
//! | [`color`] | (always) | Color with HSL/HSV, hex parsing, named constants |
//! | [`time`] | (always) | Delta time, FPS tracking, Timer, fixed timestep |
//! | [`input`] | (always) | Keyboard, mouse, gamepad, remappable actions |
//! | [`camera`] | (always) | Multi-camera, follow, shake, split-screen |
//! | [`engine`] | (always) | Game loop, Context, GameState trait, WindowConfig |
//! | [`collision`] | `collision` | AABB, circle, ray, spatial hash |
//! | [`particles`] | `particles` | Emitters, shapes, easing, pooling |
//! | [`tween`] | `tween` | 30+ easing functions, TweenManager |
//! | [`animation`] | `animation` | Sprite sheet animation, events |
//! | [`scene`] | (always) | Scene stack, transitions |
//! | [`ui`] | `ui` | Buttons, sliders, text input, layouts |
//! | [`assets`] | (always) | Asset handles, ImageAsset, SpriteSheet |
//! | [`audio`] | `audio` | Spatial audio, music/SFX channels, fade |
//!
//! ## Feature Flags
//!
//! ```toml
//! [dependencies]
//! game-gem = { version = "0.1", features = ["audio", "ui", "particles", "animation", "tween", "collision"] }
//! ```
//!
//! All features are enabled by default. Disable what you don't need:
//!
//! ```toml
//! game-gem = { version = "0.1", default-features = false, features = ["collision"] }
//! ```

// ─────────────────────────────────────────────
// Core modules (always compiled)
// ─────────────────────────────────────────────
pub mod math;
pub mod color;
pub mod time;
pub mod input;
pub mod camera;
pub mod engine;
pub mod scene;
pub mod assets;

// ─────────────────────────────────────────────
// Feature-gated modules
// ─────────────────────────────────────────────
#[cfg(feature = "collision")]
pub mod collision;

#[cfg(feature = "particles")]
pub mod particles;

#[cfg(feature = "tween")]
pub mod tween;

#[cfg(feature = "animation")]
pub mod animation;

#[cfg(feature = "ui")]
pub mod ui;

#[cfg(feature = "audio")]
pub mod audio;

// ─────────────────────────────────────────────
// Prelude
// ─────────────────────────────────────────────
pub mod prelude;