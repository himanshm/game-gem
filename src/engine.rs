//! Engine core: Context, game loop, and window configuration.
//!
//! ## Why game-gem > macroquad (architecture)
//!
//! | Feature               | macroquad                  | game-gem                            |
//! |-----------------------|----------------------------|-------------------------------------|
//! | Entry point           | `#[macroquad::main]` proc macro | Plain `fn main()` + `Game::run()`  |
//! | State management      | Global variables           | Trait-based `GameState`              |
//! | Error handling        | Panics                     | `Result` types                      |
//! | Context passing       | Global functions           | Explicit `&mut Context` parameter    |
//! | Modularity            | All-or-nothing             | Feature-gated modules               |
//! | Multiple windows      | No                         | Planned                             |
//! | Custom game loop      | No                         | Fixed timestep, variable, or custom |
//!
//! ## Minimal example
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
//!     fn render(&mut self, ctx: &mut Context) {
//!         ctx.graphics.clear(Color::SKY_BLUE);
//!         ctx.graphics.draw_circle(ctx, 400.0, 300.0, 50.0, Color::RED);
//!     }
//! }
//!
//! fn main() {
//!     Game::new()
//!         .window_title("My Game")
//!         .window_size(800, 600)
//!         .run(MyGame);
//! }
//! ```

use crate::time::Time;
use crate::input::InputState;
use crate::camera::Camera;
use crate::color::Color;
use crate::math::Vec2;

#[cfg(feature = "audio")]
use crate::audio::AudioManager;
#[cfg(feature = "ui")]
use crate::ui::{UiState, UiTheme};
#[cfg(feature = "particles")]
use crate::particles::ParticleEmitter;
#[cfg(feature = "tween")]
use crate::tween::TweenManager;

// ─────────────────────────────────────────────
// Window configuration
// ─────────────────────────────────────────────

/// Window configuration builder.
///
/// Use `Game::new()` to create one, then chain methods to configure.
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// Window title.
    pub title: String,
    /// Window width in pixels.
    pub width: u32,
    /// Window height in pixels.
    pub height: u32,
    /// Whether the window is resizable.
    pub resizable: bool,
    /// Whether to start in fullscreen.
    pub fullscreen: bool,
    /// Whether to show the window (can start hidden).
    pub visible: bool,
    /// Minimum window size.
    pub min_size: Option<(u32, u32)>,
    /// Maximum window size.
    pub max_size: Option<(u32, u32)>,
    /// Whether to enable vsync.
    pub vsync: bool,
    /// MSAA sample count (0 = disabled).
    pub msaa_samples: u32,
    /// High-DPI / retina support.
    pub high_dpi: bool,
    /// Icon path (set after window creation).
    pub icon_path: Option<String>,
    /// Target FPS cap (0 = unlimited).
    pub target_fps: u32,
    /// Background color when the window is cleared.
    pub clear_color: Color,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "game-gem".to_string(),
            width: 800,
            height: 600,
            resizable: true,
            fullscreen: false,
            visible: true,
            min_size: None,
            max_size: None,
            vsync: true,
            msaa_samples: 0,
            high_dpi: true,
            icon_path: None,
            target_fps: 0,
            clear_color: Color::from_hex("#1A1A2E").unwrap(),
        }
    }
}

// ─────────────────────────────────────────────
// Graphics context (drawing commands stored here)
// ─────────────────────────────────────────────

/// Drawing operations that the game can issue each frame.
///
/// In the real rendering backend, these would be batched and sent to the GPU.
/// For the API definition, we store the draw commands for deferred rendering.
#[derive(Debug, Clone)]
pub enum DrawCommand {
    Clear { color: Color },
    DrawCircle { x: f32, y: f32, radius: f32, color: Color },
    DrawRect { x: f32, y: f32, w: f32, h: f32, color: Color },
    DrawLine { x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color },
    DrawText { text: String, x: f32, y: f32, size: f32, color: Color },
    SetCamera { camera: Camera },
    DrawTriangle { x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32, color: Color },
    DrawPoly { points: Vec<Vec2>, color: Color },
    DrawEllipse { x: f32, y: f32, rx: f32, ry: f32, color: Color },
    DrawRing { x: f32, y: f32, inner_radius: f32, outer_radius: f32, color: Color },
    DrawArc { x: f32, y: f32, radius: f32, start_angle: f32, end_angle: f32, color: Color },
}

/// Graphics context that collects draw commands.
///
/// In a full implementation, this wraps the miniquad rendering pipeline
/// with batched rendering, GPU resource management, and shader programs.
pub struct GraphicsContext {
    /// Draw commands for the current frame.
    commands: Vec<DrawCommand>,
    /// Current camera.
    pub camera: Camera,
    /// Default camera.
    default_camera: Camera,
    /// Screen size in pixels.
    pub screen_size: Vec2,
    /// DPI scale factor.
    pub dpi_scale: f32,
}

impl GraphicsContext {
    fn new(width: u32, height: u32) -> Self {
        let screen_size = Vec2::new(width as f32, height as f32);
        let default_camera = Camera::centered();
        Self {
            commands: Vec::with_capacity(1024),
            camera: default_camera.clone(),
            default_camera,
            screen_size,
            dpi_scale: 1.0,
        }
    }

    /// Clear the screen with a color.
    pub fn clear(&mut self, color: Color) {
        self.commands.push(DrawCommand::Clear { color });
    }

    /// Draw a filled circle.
    pub fn draw_circle(&mut self, x: f32, y: f32, radius: f32, color: Color) {
        self.commands.push(DrawCommand::DrawCircle { x, y, radius, color });
    }

    /// Draw a filled rectangle.
    pub fn draw_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        self.commands.push(DrawCommand::DrawRect { x, y, w, h, color });
    }

    /// Draw a line segment.
    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
        self.commands.push(DrawCommand::DrawLine { x1, y1, x2, y2, thickness, color });
    }

    /// Draw text (placeholder — real impl uses font rendering).
    pub fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: Color) {
        self.commands.push(DrawCommand::DrawText {
            text: text.to_string(),
            x, y, size, color,
        });
    }

    /// Draw a filled triangle.
    pub fn draw_triangle(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32, color: Color) {
        self.commands.push(DrawCommand::DrawTriangle { x1, y1, x2, y2, x3, y3, color });
    }

    /// Draw a filled polygon.
    pub fn draw_poly(&mut self, points: Vec<Vec2>, color: Color) {
        self.commands.push(DrawCommand::DrawPoly { points, color });
    }

    /// Draw a filled ellipse.
    pub fn draw_ellipse(&mut self, x: f32, y: f32, rx: f32, ry: f32, color: Color) {
        self.commands.push(DrawCommand::DrawEllipse { x, y, rx, ry, color });
    }

    /// Draw a ring (annulus).
    pub fn draw_ring(&mut self, x: f32, y: f32, inner_radius: f32, outer_radius: f32, color: Color) {
        self.commands.push(DrawCommand::DrawRing { x, y, inner_radius, outer_radius, color });
    }

    /// Draw an arc.
    pub fn draw_arc(&mut self, x: f32, y: f32, radius: f32, start_angle: f32, end_angle: f32, color: Color) {
        self.commands.push(DrawCommand::DrawArc { x, y, radius, start_angle, end_angle, color });
    }

    /// Set the active camera for subsequent draw calls.
    pub fn set_camera(&mut self, camera: Camera) {
        self.commands.push(DrawCommand::SetCamera { camera });
    }

    /// Reset to the default camera.
    pub fn reset_camera(&mut self) {
        self.commands.push(DrawCommand::SetCamera {
            camera: self.default_camera.clone(),
        });
    }

    /// Flush all commands (called at the end of each frame by the engine).
    fn flush(&mut self) -> Vec<DrawCommand> {
        std::mem::take(&mut self.commands)
    }
}

// ─────────────────────────────────────────────
// Context (passed to all game state methods)
// ─────────────────────────────────────────────

/// The main context object passed to all game state methods.
///
/// Contains all engine subsystems: time, input, graphics, audio, etc.
/// This is the single point of access for everything the game needs each frame.
pub struct Context {
    /// Time and delta tracking.
    pub time: Time,
    /// Input state (keyboard, mouse, actions).
    pub input: InputState,
    /// Graphics context (drawing commands).
    pub graphics: GraphicsContext,
    /// Audio manager (feature-gated).
    #[cfg(feature = "audio")]
    pub audio: AudioManager,
    /// UI state and theme (feature-gated).
    #[cfg(feature = "ui")]
    pub ui_state: UiState,
    #[cfg(feature = "ui")]
    pub ui_theme: UiTheme,
    /// Active particle emitters (feature-gated).
    #[cfg(feature = "particles")]
    pub particles: Vec<ParticleEmitter>,
    /// Active tweens (feature-gated).
    #[cfg(feature = "tween")]
    pub tweens: TweenManager,
    /// Window configuration.
    pub window: WindowConfig,
    /// Whether the game should exit.
    should_quit: bool,
    /// Whether the game is currently paused (time still advances but update is skipped).
    paused: bool,
}

impl Default for Context {
    fn default() -> Self {
        Self::new(WindowConfig::default())
    }
}

impl Context {
    /// Create a new context from window configuration.
    pub(crate) fn new(config: WindowConfig) -> Self {
        let graphics = GraphicsContext::new(config.width, config.height);
        Self {
            time: Time::new(),
            input: InputState::default(),
            graphics,
            #[cfg(feature = "audio")]
            audio: AudioManager::new(),
            #[cfg(feature = "ui")]
            ui_state: UiState::default(),
            #[cfg(feature = "ui")]
            ui_theme: UiTheme::default(),
            #[cfg(feature = "particles")]
            particles: Vec::new(),
            #[cfg(feature = "tween")]
            tweens: TweenManager::new(),
            window: config,
            should_quit: false,
            paused: false,
        }
    }

    /// Request the game to exit (will close after the current frame).
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Check if the game is about to exit.
    pub fn is_quitting(&self) -> bool {
        self.should_quit
    }

    /// Pause the game (skips update but still renders).
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume from pause.
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Check if the game is paused.
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Toggle pause.
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Current screen width.
    pub fn screen_width(&self) -> f32 {
        self.graphics.screen_size.x
    }

    /// Current screen height.
    pub fn screen_height(&self) -> f32 {
        self.graphics.screen_size.y
    }

    /// Current screen size as Vec2.
    pub fn screen_size(&self) -> Vec2 {
        self.graphics.screen_size
    }

    /// Handle window resize.
    pub fn handle_resize(&mut self, width: u32, height: u32) {
        self.graphics.screen_size = Vec2::new(width as f32, height as f32);
        self.window.width = width;
        self.window.height = height;
    }
}

// ─────────────────────────────────────────────
// GameState trait
// ─────────────────────────────────────────────

/// The main game state trait. Implement this for your game.
///
/// The engine calls these methods every frame in order:
/// 1. [`on_enter`](GameState::on_enter) (once, when the game starts)
/// 2. [`fixed_update`](GameState::fixed_update) (zero or more times, if fixed timestep is enabled)
/// 3. [`update`](GameState::update) (once per frame)
/// 4. [`render`](GameState::render) (once per frame)
///
/// To add scene management, use [`crate::scene::SceneManager`] inside your game state.
pub trait GameState {
    /// Called once when the game starts (after window creation).
    fn on_enter(&mut self, _ctx: &mut Context) {}

    /// Called when the game is about to exit.
    fn on_exit(&mut self, _ctx: &mut Context) {}

    /// Fixed-rate update (for physics, networking, etc.).
    /// Called zero or more times per frame at the fixed timestep rate.
    fn fixed_update(&mut self, _ctx: &mut Context) {}

    /// Main update logic. Called once per frame.
    fn update(&mut self, ctx: &mut Context);

    /// Render the game. Called once per frame, after update.
    fn render(&mut self, ctx: &mut Context);
}

// ─────────────────────────────────────────────
// Game builder
// ─────────────────────────────────────────────

/// The game builder. Configure your game and run it.
///
/// # Example
/// ```no_run
/// use game_gem::prelude::*;
///
/// struct MyGame;
/// impl GameState for MyGame {
///     fn update(&mut self, ctx: &mut Context) {
///         if ctx.input.keyboard.is_pressed(KeyCode::Escape) { ctx.quit(); }
///     }
///     fn render(&mut self, ctx: &mut Context) {
///         ctx.graphics.clear(Color::BLACK);
///     }
/// }
///
/// fn main() {
///     Game::new()
///         .window_title("Hello game-gem!")
///         .window_size(1024, 768)
///         .clear_color(Color::from_hex("#1A1A2E").unwrap())
///         .target_fps(60)
///         .run(MyGame);
/// }
/// ```
pub struct Game {
    config: WindowConfig,
}

impl Game {
    /// Create a new game with default configuration.
    pub fn new() -> Self {
        Self {
            config: WindowConfig::default(),
        }
    }

    /// Set the window title.
    pub fn window_title(mut self, title: &str) -> Self {
        self.config.title = title.to_string();
        self
    }

    /// Set the window size in pixels.
    pub fn window_size(mut self, width: u32, height: u32) -> Self {
        self.config.width = width;
        self.config.height = height;
        self
    }

    /// Set whether the window is resizable.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.config.resizable = resizable;
        self
    }

    /// Set whether to start in fullscreen.
    pub fn fullscreen(mut self, fullscreen: bool) -> Self {
        self.config.fullscreen = fullscreen;
        self
    }

    /// Set whether to enable vsync.
    pub fn vsync(mut self, vsync: bool) -> Self {
        self.config.vsync = vsync;
        self
    }

    /// Set MSAA samples (0 = disabled).
    pub fn msaa(mut self, samples: u32) -> Self {
        self.config.msaa_samples = samples;
        self
    }

    /// Set the clear/background color.
    pub fn clear_color(mut self, color: Color) -> Self {
        self.config.clear_color = color;
        self
    }

    /// Set target FPS cap (0 = unlimited).
    pub fn target_fps(mut self, fps: u32) -> Self {
        self.config.target_fps = fps;
        self
    }

    /// Set minimum window size.
    pub fn min_size(mut self, w: u32, h: u32) -> Self {
        self.config.min_size = Some((w, h));
        self
    }

    /// Set the window icon from a file path.
    pub fn icon(mut self, path: &str) -> Self {
        self.config.icon_path = Some(path.to_string());
        self
    }

    /// Run the game with the given game state.
    ///
    /// This is the main entry point. It creates the window, initializes
    /// the context, and runs the game loop until the game exits.
    pub fn run<S: GameState>(self, mut state: S) {
        let mut ctx = Context::new(self.config);

        state.on_enter(&mut ctx);

        // Main game loop
        // In a real implementation, this would use miniquad's event loop.
        // For the API definition, we simulate the loop structure.

        #[cfg(feature = "audio")]
        {
            // Initialize audio output (rodio)
        }

        loop {
            // 1. Tick time
            ctx.time.tick();

            // 2. Process input events
            ctx.input.keyboard.pressed_this_frame.clear();
            ctx.input.keyboard.released_this_frame.clear();
            ctx.input.mouse.pressed_this_frame.clear();
            ctx.input.mouse.released_this_frame.clear();
            ctx.input.mouse.scroll = Vec2::ZERO;
            ctx.input.mouse.delta = Vec2::ZERO;

            // 3. Fixed updates
            let fixed_steps = ctx.time.fixed_step_count();
            for _ in 0..fixed_steps {
                state.fixed_update(&mut ctx);
            }

            // 4. Variable update
            if !ctx.paused {
                state.update(&mut ctx);
            }

            // 5. Update subsystems
            #[cfg(feature = "audio")]
            ctx.audio.update(ctx.time.delta() as f32);

            #[cfg(feature = "particles")]
            for emitter in &mut ctx.particles {
                emitter.update(ctx.time.delta() as f32);
            }

            #[cfg(feature = "tween")]
            { let _ = ctx.tweens.update(ctx.time.delta() as f32); }

            #[cfg(feature = "ui")]
            ctx.ui_state.update_animations(
                ctx.time.delta() as f32,
                ctx.ui_theme.animation_speed,
            );

            // Update camera
            ctx.graphics.camera.update(ctx.time.delta() as f32);

            // 6. Render
            state.render(&mut ctx);

            // 7. Flush draw commands (in real impl, submit to GPU)
            let _commands = ctx.graphics.flush();

            // 8. Check quit
            if ctx.should_quit {
                break;
            }
        }

        state.on_exit(&mut ctx);
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}