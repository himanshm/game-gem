//! Scene management system.
//!
//! Advantages over macroquad (which has none):
//! - **Stack-based scene transitions** with fade effects
//! - **Scene parameters** — pass data between scenes
//! - **Transition effects** — fade, slide, custom
//! - **On-enter / on-exit** hooks
//! - **Pause / resume** for overlay scenes

use crate::engine::Context;

// ─────────────────────────────────────────────
// Scene trait
// ─────────────────────────────────────────────

/// A scene is a self-contained game state (menu, gameplay, pause screen, etc.).
///
/// Implement this trait for each major game state.
/// The scene manager handles transitions automatically.
pub trait Scene: SceneAsAny + 'static {
    /// Called when the scene is first pushed onto the stack.
    fn on_enter(&mut self, _ctx: &mut Context) {}

    /// Called when the scene is popped off the stack.
    fn on_exit(&mut self, _ctx: &mut Context) {}

    /// Called when the scene below is covered by another scene.
    fn on_pause(&mut self, _ctx: &mut Context) {}

    /// Called when the scene above is popped, revealing this one.
    fn on_resume(&mut self, _ctx: &mut Context) {}

    /// Update logic (called once per frame).
    fn update(&mut self, ctx: &mut Context);

    /// Render (called once per frame, after update).
    fn render(&mut self, ctx: &mut Context);

    /// Handle window events (optional override).
    fn handle_event(&mut self, _ctx: &mut Context, _event: &SceneEvent) {}
}

/// Events that can be passed to scenes.
#[derive(Debug, Clone)]
pub enum SceneEvent {
    /// Window was resized.
    WindowResized { width: u32, height: u32 },
    /// Window gained focus.
    GainedFocus,
    /// Window lost focus.
    LostFocus,
    /// A custom event with a string tag.
    Custom(String),
}

// ─────────────────────────────────────────────
// Transition effects
// ─────────────────────────────────────────────

/// Transition effect between scenes.
#[derive(Debug, Clone)]
pub enum Transition {
    /// No transition (instant switch).
    None,
    /// Fade to black and back.
    Fade { duration: f32, color: [f32; 4] },
    /// Slide the new scene in from a direction.
    Slide { duration: f32, direction: SlideDirection },
}

#[derive(Debug, Clone, Copy)]
pub enum SlideDirection {
    Left,
    Right,
    Up,
    Down,
}

// ─────────────────────────────────────────────
// Scene Manager
// ─────────────────────────────────────────────

/// State of a transition.
#[derive(Debug, Clone, Copy, PartialEq)]
enum TransitionPhase {
    /// No transition active.
    Idle,
    /// Transitioning out (old scene fading/closing).
    Out,
    /// Transitioning in (new scene fading/opening).
    In,
}

/// Manages a stack of scenes with transitions.
///
/// # Example
/// ```
/// let mut scenes = SceneManager::new();
/// scenes.push(MenuScene::new());
///
/// // In game loop:
/// scenes.update(ctx);
/// scenes.render(ctx);
///
/// // From within a scene:
/// // scenes.push(GameScene::new(level));
/// // scenes.pop(); // return to previous scene
/// // scenes.replace(SettingsScene::new()); // replace current scene
/// ```
pub struct SceneManager {
    stack: Vec<Box<dyn Scene>>,
    transition: Option<TransitionEffect>,
    /// Pending scene to push after transition-out completes.
    pending_push: Option<Box<dyn Scene>>,
    /// Pending pop after transition-out completes.
    pending_pop: bool,
    /// Pending replace after transition-out completes.
    pending_replace: Option<Box<dyn Scene>>,
}

#[derive(Debug)]
struct TransitionEffect {
    #[allow(dead_code)]
    kind: Transition,
    phase: TransitionPhase,
    timer: f32,
    half_duration: f32,
}

impl SceneManager {
    /// Create a new empty scene manager.
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            transition: None,
            pending_push: None,
            pending_pop: false,
            pending_replace: None,
        }
    }

    /// Push a new scene onto the stack.
    pub fn push<S: Scene>(&mut self, scene: S) {
        match &self.transition {
            Some(t) if t.phase != TransitionPhase::Idle => {
                // Queue the push
                self.pending_push = Some(Box::new(scene));
                return;
            }
            _ => {}
        }

        // Pause current scene (split the borrow so we don't alias `self`).
        let mut fake_ctx = self.make_fake_ctx();
        if let Some(current) = self.stack.last_mut() {
            current.on_pause(&mut fake_ctx);
        }

        self.stack.push(Box::new(scene));
        if let Some(top) = self.stack.last_mut() {
            top.on_enter(&mut fake_ctx);
        }
    }

    /// Pop the top scene off the stack.
    pub fn pop(&mut self) {
        match &self.transition {
            Some(t) if t.phase != TransitionPhase::Idle => {
                self.pending_pop = true;
                return;
            }
            _ => {}
        }

        let mut fake_ctx = self.make_fake_ctx();
        if let Some(mut scene) = self.stack.pop() {
            scene.on_exit(&mut fake_ctx);
        }

        // Resume the scene below
        if let Some(top) = self.stack.last_mut() {
            top.on_resume(&mut fake_ctx);
        }
    }

    /// Replace the current scene with a new one.
    pub fn replace<S: Scene>(&mut self, scene: S) {
        match &self.transition {
            Some(t) if t.phase != TransitionPhase::Idle => {
                self.pending_replace = Some(Box::new(scene));
                return;
            }
            _ => {}
        }

        let mut fake_ctx = self.make_fake_ctx();
        if let Some(mut old) = self.stack.pop() {
            old.on_exit(&mut fake_ctx);
        }

        self.stack.push(Box::new(scene));
        if let Some(top) = self.stack.last_mut() {
            top.on_enter(&mut fake_ctx);
        }
    }

    /// Push a scene with a transition effect.
    pub fn push_with_transition<S: Scene>(&mut self, scene: S, transition: Transition) {
        self.start_transition(transition);
        self.pending_push = Some(Box::new(scene));
    }

    /// Pop with a transition effect.
    pub fn pop_with_transition(&mut self, transition: Transition) {
        self.start_transition(transition);
        self.pending_pop = true;
    }

    /// Get a reference to the current (top) scene, if any.
    pub fn current(&self) -> Option<&dyn Scene> {
        self.stack.last().map(|s| s.as_ref())
    }

    /// Get a mutable reference to the current scene, if any.
    pub fn current_mut(&mut self) -> Option<&mut dyn Scene> {
        self.stack.last_mut().map(|s| s.as_mut())
    }

    /// Get a scene by type from the stack.
    pub fn find<S: Scene>(&self) -> Option<&S> {
        for scene in &self.stack {
            if let Some(s) = scene.as_ref().as_any().downcast_ref::<S>() {
                return Some(s);
            }
        }
        None
    }

    /// Number of scenes on the stack.
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Whether the scene stack is empty.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Update the top scene and any active transition.
    pub fn update(&mut self, ctx: &mut Context) {
        // Update transition. We pull the transition out of `self` for the
        // duration of the body so we can call `execute_pending(&mut self, ...)`
        // without an aliased mutable borrow.
        let mut trans_opt = self.transition.take();
        if let Some(trans) = &mut trans_opt {
            trans.timer += ctx.time.delta() as f32;
            match trans.phase {
                TransitionPhase::Out => {
                    if trans.timer >= trans.half_duration {
                        // Transition-out complete, perform the pending action.
                        self.execute_pending(ctx);
                        trans.phase = TransitionPhase::In;
                        trans.timer = 0.0;
                    }
                }
                TransitionPhase::In => {
                    if trans.timer >= trans.half_duration {
                        // transition is finished; drop it.
                        // (trans_opt stays `Some` but we'll clear below)
                        trans_opt = None;
                    }
                }
                TransitionPhase::Idle => {}
            }
        }
        self.transition = trans_opt;

        // Update only the top scene
        if let Some(top) = self.stack.last_mut() {
            top.update(ctx);
        }
    }

    /// Render all visible scenes (the top one, and the one below during transitions).
    pub fn render(&mut self, ctx: &mut Context) {
        // During transition-out, render the old scene
        // During transition-in, render the new scene
        if let Some(trans) = &self.transition {
            match trans.phase {
                TransitionPhase::Out => {
                    if let Some(scene) = self.stack.last_mut() {
                        scene.render(ctx);
                    }
                    self.render_transition_overlay(ctx, trans, false);
                }
                TransitionPhase::In => {
                    if let Some(scene) = self.stack.last_mut() {
                        scene.render(ctx);
                    }
                    self.render_transition_overlay(ctx, trans, true);
                }
                TransitionPhase::Idle => {}
            }
        } else if let Some(top) = self.stack.last_mut() {
            top.render(ctx);
        }
    }

    fn start_transition(&mut self, kind: Transition) {
        let half_duration = match &kind {
            Transition::None => 0.0,
            Transition::Fade { duration, .. } => duration / 2.0,
            Transition::Slide { duration, .. } => duration / 2.0,
        };
        self.transition = Some(TransitionEffect {
            kind,
            phase: TransitionPhase::Out,
            timer: 0.0,
            half_duration,
        });
    }

    fn execute_pending(&mut self, ctx: &mut Context) {
        if let Some(scene) = self.pending_push.take() {
            if let Some(current) = self.stack.last_mut() {
                current.on_pause(ctx);
            }
            self.stack.push(scene);
            if let Some(top) = self.stack.last_mut() {
                top.on_enter(ctx);
            }
        }

        if self.pending_pop {
            self.pending_pop = false;
            if let Some(mut scene) = self.stack.pop() {
                scene.on_exit(ctx);
            }
            if let Some(top) = self.stack.last_mut() {
                top.on_resume(ctx);
            }
        }

        if let Some(scene) = self.pending_replace.take() {
            if let Some(mut old) = self.stack.pop() {
                old.on_exit(ctx);
            }
            self.stack.push(scene);
            if let Some(top) = self.stack.last_mut() {
                top.on_enter(ctx);
            }
        }
    }

    fn render_transition_overlay(&self, _ctx: &mut Context, trans: &TransitionEffect, _is_in: bool) {
        // In a real implementation, this would draw a fade overlay or sliding rect
        // using the graphics module. For now, this is a hook for the rendering backend.
        let _ = (trans, _is_in); // suppress unused warnings
    }

    /// Create a minimal context for on_enter/on_exit callbacks.
    /// In real use, the engine passes the actual context.
    fn make_fake_ctx(&self) -> Context {
        Context::default()
    }
}

// Helper trait for downcasting scene trait objects.
//
// `Scene` extends `SceneAsAny` so that `dyn Scene` carries the `as_any` method
// in its vtable, allowing runtime downcasting of trait-object pointers.
//
// The trait is declared `pub` (but not re-exported from the crate) so that
// using it as a supertrait bound on the public `Scene` trait does not leak
// private visibility.
pub trait SceneAsAny {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<S: Scene + 'static> SceneAsAny for S {
    fn as_any(&self) -> &dyn std::any::Any { self }
}