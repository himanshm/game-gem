//! Convenience prelude — import everything you need with one line.
//!
//! ```rust
//! use game_gem::prelude::*;
//! ```

// Math
pub use crate::math::{
    vec2, vec3, ivec2,
    Vec2, Vec2Ext, Vec3, Vec4, Mat4, Quat,
    Rect, Transform, Lerp, Angle, FloatExt,
    consts,
};

// Color
pub use crate::color::{Color, ColorParseError};

// Time
pub use crate::time::{Time, Timer};

// Input
pub use crate::input::{
    KeyCode, MouseButton, MouseState, KeyboardState,
    InputState, InputAction,
};

// Camera
pub use crate::camera::Camera;

// Engine
pub use crate::engine::{
    Context, GameState, Game, WindowConfig,
};

// Collision
#[cfg(feature = "collision")]
pub use crate::collision::{
    CollisionInfo, CircleCollider,
    circle_vs_circle, circle_vs_rect, rect_vs_rect,
    point_in_circle, point_in_rect,
    Ray, RaycastHit, ray_vs_rect, ray_vs_circle,
    SpatialHash, resolve_aabb_collision,
};

// Particles
#[cfg(feature = "particles")]
pub use crate::particles::{
    Easing as ParticleEasing, Particle, EmitterShape, ParticleEmitter, ParticleEmitterConfig,
};

// Tween
#[cfg(feature = "tween")]
pub use crate::tween::{
    Ease, Tween, TweenStatus, TweenManager,
};

// Animation
#[cfg(feature = "animation")]
pub use crate::animation::{
    Animation, AnimationFrame, AnimationPlayer, AnimationState,
};

// Scene
pub use crate::scene::{Scene, SceneManager, SceneEvent, Transition, SlideDirection};

// UI
#[cfg(feature = "ui")]
pub use crate::ui::{
    UiTheme, UiState, UiInteraction,
    button as ui_button, slider as ui_slider,
    text_input as ui_text_input, TextInputState,
    label as ui_label, checkbox as ui_checkbox,
    progress_bar as ui_progress_bar,
    VerticalLayout, HorizontalLayout,
    KeyModifiers,
};

// Assets
pub use crate::assets::{
    AssetHandle, AssetType, LoadStatus, AssetConfig, AssetManager,
    ImageAsset, PixelFormat, SpriteSheet,
};

// Audio
#[cfg(feature = "audio")]
pub use crate::audio::{
    Sound, AudioManager, AudioChannel, Fade,
};