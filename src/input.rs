//! Input handling: keyboard, mouse, and gamepad.
//!
//! Key advantages over macroquad:
//! - **Pressed/released detection per frame** (macroquad requires manual tracking)
//! - **Input actions** (remappable keybindings with a single name)
//! - **Mouse gesture detection** (drag, click, double-click)
//! - **Text input** support for UI
//!
//! Access via `ctx.input` inside your [`GameState`].

use crate::math::Vec2;

// ─────────────────────────────────────────────
// Key definitions
// ─────────────────────────────────────────────

/// Physical keyboard key.
///
/// Covers the common keyboard layout. For full coverage, use `KeyCode` from the
/// underlying windowing layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum KeyCode {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    // Numbers
    Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9, Key0,
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    // Special
    Space, Enter, Escape, Tab, Backspace, Delete, Insert,
    Home, End, PageUp, PageDown,
    // Arrows
    Up, Down, Left, Right,
    // Modifiers
    LeftShift, RightShift, LeftCtrl, RightCtrl,
    LeftAlt, RightAlt, LeftSuper, RightSuper,
    // Punctuation
    Semicolon, Comma, Period, Slash, Backslash,
    LeftBracket, RightBracket, Equals, Minus,
    Apostrophe, Backquote,
    // Other
    CapsLock, ScrollLock, NumLock, PrintScreen,
    Pause, ContextMenu,
    /// Unknown / unmapped key.
    Unknown,
}

// ─────────────────────────────────────────────
// Mouse
// ─────────────────────────────────────────────

/// Mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Button4,
    Button5,
}

/// Mouse state for the current frame.
#[derive(Debug, Clone)]
pub struct MouseState {
    /// Current pixel position.
    pub position: Vec2,
    /// Position at the start of the current drag (if any).
    pub drag_start: Option<Vec2>,
    /// Delta movement this frame.
    pub delta: Vec2,
    /// Scroll wheel delta this frame.
    pub scroll: Vec2,
    /// Which buttons are currently held down.
    pub held: std::collections::HashSet<MouseButton>,
    /// Buttons that were pressed this frame (down edge).
    pub pressed_this_frame: Vec<MouseButton>,
    /// Buttons that were released this frame (up edge).
    pub released_this_frame: Vec<MouseButton>,
    /// Whether the cursor is visible.
    pub cursor_visible: bool,
    /// Whether the cursor is locked (grabbed).
    pub cursor_grabbed: bool,
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            drag_start: None,
            delta: Vec2::ZERO,
            scroll: Vec2::ZERO,
            held: std::collections::HashSet::new(),
            pressed_this_frame: Vec::new(),
            released_this_frame: Vec::new(),
            cursor_visible: true,
            cursor_grabbed: false,
        }
    }
}

impl MouseState {
    /// Is the given button currently held?
    pub fn is_down(&self, button: MouseButton) -> bool {
        self.held.contains(&button)
    }

    /// Was the button pressed this frame?
    pub fn is_pressed(&self, button: MouseButton) -> bool {
        self.pressed_this_frame.contains(&button)
    }

    /// Was the button released this frame?
    pub fn is_released(&self, button: MouseButton) -> bool {
        self.released_this_frame.contains(&button)
    }

    /// Is any mouse button held?
    pub fn is_any_down(&self) -> bool {
        !self.held.is_empty()
    }

    /// Check if the user is dragging with the left button.
    pub fn is_dragging(&self) -> bool {
        self.held.contains(&MouseButton::Left) && self.drag_start.is_some()
    }

    /// Get the drag vector (from start to current position), or `None` if not dragging.
    pub fn drag_vector(&self) -> Option<Vec2> {
        self.drag_start.map(|start| self.position - start)
    }

    /// Set cursor visibility.
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
    }

    /// Set cursor grab (locks cursor to window).
    pub fn set_cursor_grabbed(&mut self, grabbed: bool) {
        self.cursor_grabbed = grabbed;
    }
}

// ─────────────────────────────────────────────
// Keyboard
// ─────────────────────────────────────────────

/// Keyboard state for the current frame.
#[derive(Debug, Clone)]
pub struct KeyboardState {
    /// Keys currently held down.
    held: std::collections::HashSet<KeyCode>,
    /// Keys pressed this frame.
    pressed_this_frame: Vec<KeyCode>,
    /// Keys released this frame.
    released_this_frame: Vec<KeyCode>,
    /// Currently buffered text input (from IME / key events).
    text_buffer: String,
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self {
            held: std::collections::HashSet::new(),
            pressed_this_frame: Vec::new(),
            released_this_frame: Vec::new(),
            text_buffer: String::new(),
        }
    }
}

impl KeyboardState {
    /// Is the key currently held?
    #[inline]
    pub fn is_down(&self, key: KeyCode) -> bool {
        self.held.contains(&key)
    }

    /// Was the key pressed this frame (down-edge)?
    #[inline]
    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed_this_frame.contains(&key)
    }

    /// Was the key released this frame (up-edge)?
    #[inline]
    pub fn is_released(&self, key: KeyCode) -> bool {
        self.released_this_frame.contains(&key)
    }

    /// Are all given keys held down simultaneously?
    pub fn are_all_down(&self, keys: &[KeyCode]) -> bool {
        keys.iter().all(|k| self.held.contains(k))
    }

    /// Is any of the given keys held down?
    pub fn is_any_down(&self, keys: &[KeyCode]) -> bool {
        keys.iter().any(|k| self.held.contains(k))
    }

    /// Take the text input buffer (clears it).
    pub fn take_text(&mut self) -> String {
        std::mem::take(&mut self.text_buffer)
    }

    /// Peek at the text buffer without clearing.
    pub fn text(&self) -> &str {
        &self.text_buffer
    }
}

// ─────────────────────────────────────────────
// Input Actions (remappable bindings)
// ─────────────────────────────────────────────

/// A named input action that can be bound to multiple keys/buttons.
///
/// # Example
/// ```
/// let mut jump = InputAction::new("jump");
/// jump.bind_key(KeyCode::Space);
/// jump.bind_key(KeyCode::Up);
/// jump.bind_mouse(MouseButton::Left);
///
/// // In update:
/// if jump.is_pressed(&ctx.input) { player.jump(); }
/// ```
#[derive(Debug, Clone)]
pub struct InputAction {
    /// Human-readable name.
    pub name: String,
    /// Bound keyboard keys.
    pub keys: Vec<KeyCode>,
    /// Bound mouse buttons.
    pub mouse_buttons: Vec<MouseButton>,
    /// Positive gamepad axis + threshold (axis_index, threshold).
    pub positive_axis: Vec<(u32, f32)>,
    /// Negative gamepad axis + threshold.
    pub negative_axis: Vec<(u32, f32)>,
    /// Gamepad buttons.
    pub gamepad_buttons: Vec<u32>,
}

impl InputAction {
    /// Create a new named input action.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            keys: Vec::new(),
            mouse_buttons: Vec::new(),
            positive_axis: Vec::new(),
            negative_axis: Vec::new(),
            gamepad_buttons: Vec::new(),
        }
    }

    /// Bind a keyboard key to this action.
    pub fn bind_key(&mut self, key: KeyCode) -> &mut Self {
        self.keys.push(key);
        self
    }

    /// Bind a mouse button to this action.
    pub fn bind_mouse(&mut self, button: MouseButton) -> &mut Self {
        self.mouse_buttons.push(button);
        self
    }

    /// Check if this action is currently active (held).
    pub fn is_down(&self, input: &InputState) -> bool {
        for key in &self.keys {
            if input.keyboard.is_down(*key) {
                return true;
            }
        }
        for btn in &self.mouse_buttons {
            if input.mouse.is_down(*btn) {
                return true;
            }
        }
        false
    }

    /// Check if this action was just activated (pressed edge).
    pub fn is_pressed(&self, input: &InputState) -> bool {
        for key in &self.keys {
            if input.keyboard.is_pressed(*key) {
                return true;
            }
        }
        for btn in &self.mouse_buttons {
            if input.mouse.is_pressed(*btn) {
                return true;
            }
        }
        false
    }
}

// ─────────────────────────────────────────────
// Unified input state
// ─────────────────────────────────────────────

/// Unified input state, accessible via `ctx.input`.
#[derive(Debug, Clone, Default)]
pub struct InputState {
    /// Keyboard state.
    pub keyboard: KeyboardState,
    /// Mouse state.
    pub mouse: MouseState,
    /// Registered input actions.
    actions: Vec<InputAction>,
}

impl InputState {
    /// Register a new input action.
    pub fn register_action(&mut self, action: InputAction) {
        self.actions.push(action);
    }

    /// Find an action by name.
    pub fn action(&self, name: &str) -> Option<&InputAction> {
        self.actions.iter().find(|a| a.name == name)
    }

    /// Check if a named action is currently active.
    pub fn is_action_down(&self, name: &str) -> bool {
        self.actions.iter().any(|a| a.name == name && a.is_down(self))
    }

    /// Check if a named action was just pressed.
    pub fn is_action_pressed(&self, name: &str) -> bool {
        self.actions.iter().any(|a| a.name == name && a.is_pressed(self))
    }
}