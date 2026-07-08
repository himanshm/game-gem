//! Immediate-mode UI toolkit.
//!
//! Unlike macroquad's `ui` which requires a separate `Ui` root and `widgets` module,
//! game-gem's UI is designed to be:
//! - **Drop-in simple**: call `ui::button(ctx, "Click me", rect)` — done
//! - **No tree building**: no `begin()` / `end()` pairs required
//! - **Styling**: global theme + per-widget overrides
//! - **Layout helpers**: vertical/horizontal stacking, spacing, alignment
//! - **Keyboard navigation**: tab between widgets, Enter to activate

use crate::math::{Vec2, Rect};
use crate::color::Color;
use crate::input::{InputState, KeyCode, MouseButton};

// ─────────────────────────────────────────────
// Theme
// ─────────────────────────────────────────────

/// Global UI theme.
#[derive(Debug, Clone)]
pub struct UiTheme {
    /// Primary color (buttons, sliders, etc.).
    pub primary: Color,
    /// Secondary / background color.
    pub secondary: Color,
    /// Text color.
    pub text: Color,
    /// Text color when hovered.
    pub text_hovered: Color,
    /// Background color for input fields.
    pub input_bg: Color,
    /// Border color.
    pub border: Color,
    /// Border color when focused.
    pub border_focused: Color,
    /// Font size in pixels.
    pub font_size: f32,
    /// Corner radius for rounded rectangles.
    pub corner_radius: f32,
    /// Padding inside widgets.
    pub padding: Vec2,
    /// Spacing between widgets.
    pub spacing: f32,
    /// Animation speed for hover/press transitions.
    pub animation_speed: f32,
    /// Whether to show focus outlines.
    pub show_focus: bool,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            primary: Color::from_hex("#4A90D9").unwrap(),
            secondary: Color::from_hex("#2C2C3E").unwrap(),
            text: Color::WHITE,
            text_hovered: Color::new(1.0, 1.0, 0.8, 1.0),
            input_bg: Color::from_hex("#1E1E2E").unwrap(),
            border: Color::from_hex("#444466").unwrap(),
            border_focused: Color::from_hex("#6CA0DC").unwrap(),
            font_size: 16.0,
            corner_radius: 6.0,
            padding: Vec2::new(12.0, 8.0),
            spacing: 8.0,
            animation_speed: 8.0,
            show_focus: true,
        }
    }
}

impl UiTheme {
    /// Create a dark theme.
    pub fn dark() -> Self {
        Self::default()
    }

    /// Create a light theme.
    pub fn light() -> Self {
        Self {
            primary: Color::from_hex("#3B82F6").unwrap(),
            secondary: Color::from_hex("#F3F4F6").unwrap(),
            text: Color::from_hex("#111827").unwrap(),
            text_hovered: Color::from_hex("#1D4ED8").unwrap(),
            input_bg: Color::WHITE,
            border: Color::from_hex("#D1D5DB").unwrap(),
            border_focused: Color::from_hex("#3B82F6").unwrap(),
            ..Self::default()
        }
    }
}

// ─────────────────────────────────────────────
// UI State (hover, active, focus tracking)
// ─────────────────────────────────────────────

/// Tracks per-widget interaction state across frames.
#[derive(Debug, Default)]
pub struct UiState {
    /// Currently hovered widget ID.
    hovered_id: Option<u64>,
    /// Currently active (pressed) widget ID.
    active_id: Option<u64>,
    /// Currently focused widget ID (for keyboard input).
    focused_id: Option<u64>,
    /// Hot (mouse-down) widget ID.
    hot_id: Option<u64>,
    /// Animation states per widget (for hover transitions).
    hover_animations: std::collections::HashMap<u64, f32>,
    /// Next widget Z-index (auto-incremented).
    z_index: u32,
}

impl UiState {
    /// Generate a stable ID from a label string.
    pub fn id_from_label(label: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        label.hash(&mut hasher);
        hasher.finish()
    }

    /// Check if a widget is hovered.
    pub fn is_hovered(&self, id: u64) -> bool {
        self.hovered_id == Some(id)
    }

    /// Check if a widget is active (being pressed).
    pub fn is_active(&self, id: u64) -> bool {
        self.active_id == Some(id)
    }

    /// Check if a widget is focused (for text input).
    pub fn is_focused(&self, id: u64) -> bool {
        self.focused_id == Some(id)
    }

    /// Get the hover animation value (0.0–1.0) for a widget.
    pub fn hover_t(&self, id: u64) -> f32 {
        self.hover_animations.get(&id).copied().unwrap_or(0.0)
    }

    /// Update hover animations.
    pub fn update_animations(&mut self, dt: f32, speed: f32) {
        // Collect IDs first so we can mutate hover_animations without
        // aliasing the iterator's borrow.
        let ids: Vec<u64> = self.hover_animations.keys().copied().collect();
        let mut to_remove: Vec<u64> = Vec::new();
        for id in ids {
            let t = *self.hover_animations.get(&id).unwrap_or(&0.0);
            if self.hovered_id == Some(id) {
                let new_t = (t + dt * speed).min(1.0);
                self.hover_animations.insert(id, new_t);
            } else if t > 0.0 {
                let new_t = (t - dt * speed).max(0.0);
                if new_t <= 0.0 {
                    to_remove.push(id);
                } else {
                    self.hover_animations.insert(id, new_t);
                }
            } else {
                to_remove.push(id);
            }
        }

        for id in to_remove {
            self.hover_animations.remove(&id);
        }
    }

    /// Begin a new frame — reset per-frame state.
    pub fn begin_frame(&mut self) {
        self.hovered_id = None;
        self.hot_id = None;
        self.z_index = 0;
    }
}

// ─────────────────────────────────────────────
// Widget results
// ─────────────────────────────────────────────

/// Result of a UI interaction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiInteraction {
    /// The widget was clicked this frame.
    pub clicked: bool,
    /// The widget is being held down.
    pub pressed: bool,
    /// The widget was released this frame.
    pub released: bool,
    /// The mouse is hovering over the widget.
    pub hovered: bool,
    /// The widget just gained focus.
    pub focused: bool,
}

// ─────────────────────────────────────────────
// Button
// ─────────────────────────────────────────────

/// Draw a button and return whether it was clicked.
///
/// # Example
/// ```
/// if ui::button(ctx, "Start Game", Rect::new(100, 200, 200, 50)) {
///     start_game();
/// }
/// ```
pub fn button(
    input: &InputState,
    ui: &mut UiState,
    theme: &UiTheme,
    label: &str,
    rect: Rect,
) -> UiInteraction {
    let id = UiState::id_from_label(label);
    let _mouse_in_rect = input.mouse.is_down(MouseButton::Left) && rect.contains(input.mouse.position);
    let mouse_hovering = rect.contains(input.mouse.position);

    // Update hover
    if mouse_hovering {
        ui.hovered_id = Some(id);
        if !ui.hover_animations.contains_key(&id) {
            ui.hover_animations.insert(id, 0.0);
        }
    }

    // Track active state
    let was_active = ui.active_id == Some(id);
    if mouse_hovering && input.mouse.is_pressed(MouseButton::Left) {
        ui.active_id = Some(id);
        ui.hot_id = Some(id);
    }

    let clicked = was_active && input.mouse.is_released(MouseButton::Left) && mouse_hovering;
    let released = was_active && input.mouse.is_released(MouseButton::Left);
    let pressed = ui.active_id == Some(id);

    if released {
        ui.active_id = None;
    }

    // Compute visual state
    let hover_t = ui.hover_animations.get(&id).copied().unwrap_or(0.0);
    let color = if pressed {
        theme.primary.darkened(0.3)
    } else {
        theme.primary.lerp(theme.primary.lightened(0.15), hover_t)
    };

    let _ = (color, label); // In real impl, these would be passed to the renderer

    UiInteraction {
        clicked,
        pressed,
        released,
        hovered: mouse_hovering,
        focused: false,
    }
}

// ─────────────────────────────────────────────
// Slider
// ─────────────────────────────────────────────

/// Draw a horizontal slider and return the new value.
///
/// # Example
/// ```
/// let volume = ui::slider(ctx, "Volume", Rect::new(100, 300, 200, 20), volume, 0.0, 1.0);
/// ```
pub fn slider(
    input: &InputState,
    ui: &mut UiState,
    _theme: &UiTheme,
    label: &str,
    rect: Rect,
    current_value: f32,
    min: f32,
    max: f32,
) -> (f32, UiInteraction) {
    let id = UiState::id_from_label(label);
    let hovering = rect.contains(input.mouse.position);

    if hovering {
        ui.hovered_id = Some(id);
    }

    let mut value = current_value;
    let interaction = UiInteraction {
        clicked: false,
        pressed: ui.active_id == Some(id),
        released: false,
        hovered: hovering,
        focused: false,
    };

    if hovering && input.mouse.is_pressed(MouseButton::Left) {
        ui.active_id = Some(id);
    }

    if ui.active_id == Some(id) {
        if input.mouse.is_released(MouseButton::Left) {
            ui.active_id = None;
        } else {
            // Compute value from mouse position
            let t = ((input.mouse.position.x - rect.x) / rect.w).clamp(0.0, 1.0);
            value = min + (max - min) * t;
        }
    }

    (value, interaction)
}

// ─────────────────────────────────────────────
// Text Input
// ─────────────────────────────────────────────

/// State for a text input widget.
#[derive(Debug, Clone)]
pub struct TextInputState {
    /// Current text content.
    pub text: String,
    /// Cursor position (character index).
    pub cursor: usize,
    /// Selection start (None = no selection).
    pub selection_start: Option<usize>,
    /// Whether the cursor blink is visible.
    pub cursor_visible: bool,
    /// Cursor blink timer.
    blink_timer: f32,
    /// Scroll offset for long text.
    pub scroll_offset: f32,
}

impl Default for TextInputState {
    fn default() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            selection_start: None,
            cursor_visible: true,
            blink_timer: 0.0,
            scroll_offset: 0.0,
        }
    }
}

impl TextInputState {
    /// Create a text input with initial text.
    pub fn new(text: &str) -> Self {
        let len = text.len();
        Self {
            text: text.to_string(),
            cursor: len,
            ..Self::default()
        }
    }

    /// Handle text input events (called when focused).
    pub fn handle_text_input(&mut self, input_text: &str) {
        if self.selection_start.is_some() {
            self.delete_selection();
        }
        self.text.insert_str(self.cursor, input_text);
        self.cursor += input_text.len();
    }

    /// Handle a key press event.
    pub fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match key {
            KeyCode::Backspace => {
                if self.selection_start.is_some() {
                    self.delete_selection();
                } else if self.cursor > 0 {
                    // Find previous character boundary
                    let prev = self.text[..self.cursor]
                    .char_indices()
                    .next_back()
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                    self.text.drain(prev..self.cursor);
                    self.cursor = prev;
                }
            }
            KeyCode::Delete => {
                if self.selection_start.is_some() {
                    self.delete_selection();
                } else if self.cursor < self.text.len() {
                    let next = self.text[self.cursor..]
                    .char_indices()
                    .nth(1)
                    .map(|(i, _)| self.cursor + i)
                    .unwrap_or(self.text.len());
                    self.text.drain(self.cursor..next);
                }
            }
            KeyCode::Left => {
                if modifiers.shift {
                    self.selection_start = Some(self.selection_start.unwrap_or(self.cursor));
                } else {
                    self.selection_start = None;
                }
                if self.cursor > 0 {
                    self.cursor = self.text[..self.cursor]
                    .char_indices()
                    .next_back()
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                }
            }
            KeyCode::Right => {
                if modifiers.shift {
                    self.selection_start = Some(self.selection_start.unwrap_or(self.cursor));
                } else {
                    self.selection_start = None;
                }
                if self.cursor < self.text.len() {
                    self.cursor = self.text[self.cursor..]
                    .char_indices()
                    .nth(1)
                    .map(|(i, _)| self.cursor + i)
                    .unwrap_or(self.text.len());
                }
            }
            KeyCode::Home => {
                self.cursor = 0;
                self.selection_start = None;
            }
            KeyCode::End => {
                self.cursor = self.text.len();
                self.selection_start = None;
            }
            KeyCode::A if modifiers.ctrl => {
                self.selection_start = Some(0);
                self.cursor = self.text.len();
            }
            KeyCode::C if modifiers.ctrl => {
                // Copy — handled at a higher level with clipboard access
            }
            KeyCode::V if modifiers.ctrl => {
                // Paste — handled at a higher level with clipboard access
            }
            KeyCode::X if modifiers.ctrl => {
                // Cut — handled at a higher level with clipboard access
            }
            KeyCode::Enter => {
                // Typically handled by the caller (form submission, etc.)
            }
            _ => {}
        }
    }

    fn delete_selection(&mut self) {
        if let Some(start) = self.selection_start {
            let (lo, hi) = if start < self.cursor {
                (start, self.cursor)
            } else {
                (self.cursor, start)
            };
            self.text.drain(lo..hi);
            self.cursor = lo;
            self.selection_start = None;
        }
    }

    /// Update cursor blink.
    pub fn update(&mut self, dt: f32) {
        self.blink_timer += dt;
        if self.blink_timer >= 0.5 {
            self.blink_timer = 0.0;
            self.cursor_visible = !self.cursor_visible;
        }
    }
}

/// Keyboard modifier state.
#[derive(Debug, Clone, Copy, Default)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub super_key: bool,
}

/// Draw a text input field.
pub fn text_input(
    input: &InputState,
    ui: &mut UiState,
    theme: &UiTheme,
    label: &str,
    rect: Rect,
    state: &mut TextInputState,
) -> UiInteraction {
    let id = UiState::id_from_label(label);
    let hovering = rect.contains(input.mouse.position);

    if hovering {
        ui.hovered_id = Some(id);
    }

    // Focus on click
    if hovering && input.mouse.is_pressed(MouseButton::Left) {
        ui.focused_id = Some(id);
        // Set cursor to click position (approximate)
        let rel_x = input.mouse.position.x - rect.x;
        let char_width = theme.font_size * 0.6; // Approximate
        state.cursor = (rel_x / char_width).max(0.0) as usize;
        state.cursor = state.cursor.min(state.text.len());
        state.blink_timer = 0.0;
        state.cursor_visible = true;
    }

    // Handle text input if focused
    if ui.focused_id == Some(id) {
        let text = input.keyboard.text();
        if !text.is_empty() {
            state.handle_text_input(text);
        }
    }

    let focused = ui.focused_id == Some(id);
    let border_color = if focused { theme.border_focused } else { theme.border };

    let _ = (border_color, state, label, rect); // Real impl draws to screen

    UiInteraction {
        clicked: false,
        pressed: false,
        released: false,
        hovered: hovering,
        focused,
    }
}

// ─────────────────────────────────────────────
// Label
// ─────────────────────────────────────────────

/// Draw a text label. Returns its rect for layout purposes.
pub fn label(
    theme: &UiTheme,
    text: &str,
    position: Vec2,
    font_size: Option<f32>,
    color: Option<Color>,
) -> Rect {
    let size = font_size.unwrap_or(theme.font_size);
    let width = text.len() as f32 * size * 0.6;
    let height = size * 1.2;
    let _ = color;
    Rect::new(position.x, position.y, width, height)
}

// ─────────────────────────────────────────────
// Layout helpers
// ─────────────────────────────────────────────

/// Simple vertical layout helper.
#[derive(Debug, Clone)]
pub struct VerticalLayout {
    /// Starting position.
    pub origin: Vec2,
    /// Current Y cursor.
    cursor_y: f32,
    /// Widget width (0 = auto).
    pub width: f32,
    /// Spacing between widgets.
    pub spacing: f32,
}

impl VerticalLayout {
    /// Create a new vertical layout.
    pub fn new(origin: Vec2, width: f32, spacing: f32) -> Self {
        Self {
            origin,
            cursor_y: origin.y,
            width,
            spacing,
        }
    }

    /// Get the next widget's position and advance the cursor by `height`.
    pub fn next(&mut self, height: f32) -> Vec2 {
        let pos = Vec2::new(self.origin.x, self.cursor_y);
        self.cursor_y += height + self.spacing;
        pos
    }

    /// Get a rect for the next widget.
    pub fn next_rect(&mut self, height: f32) -> Rect {
        let pos = self.next(height);
        Rect::new(pos.x, pos.y, self.width, height)
    }

    /// Reset to origin.
    pub fn reset(&mut self) {
        self.cursor_y = self.origin.y;
    }
}

/// Simple horizontal layout helper.
#[derive(Debug, Clone)]
pub struct HorizontalLayout {
    /// Starting position.
    pub origin: Vec2,
    /// Current X cursor.
    cursor_x: f32,
    /// Widget height (0 = auto).
    pub height: f32,
    /// Spacing between widgets.
    pub spacing: f32,
}

impl HorizontalLayout {
    /// Create a new horizontal layout.
    pub fn new(origin: Vec2, height: f32, spacing: f32) -> Self {
        Self {
            origin,
            cursor_x: origin.x,
            height,
            spacing,
        }
    }

    /// Get the next widget's position and advance the cursor by `width`.
    pub fn next(&mut self, width: f32) -> Vec2 {
        let pos = Vec2::new(self.cursor_x, self.origin.y);
        self.cursor_x += width + self.spacing;
        pos
    }

    /// Get a rect for the next widget.
    pub fn next_rect(&mut self, width: f32) -> Rect {
        let pos = self.next(width);
        Rect::new(pos.x, pos.y, width, self.height)
    }

    /// Reset to origin.
    pub fn reset(&mut self) {
        self.cursor_x = self.origin.x;
    }
}

// ─────────────────────────────────────────────
// Checkbox
// ─────────────────────────────────────────────

/// Draw a checkbox and return whether it's checked.
pub fn checkbox(
    input: &InputState,
    ui: &mut UiState,
    theme: &UiTheme,
    label: &str,
    position: Vec2,
    checked: &mut bool,
) -> UiInteraction {
    let size = theme.font_size;
    let rect = Rect::new(position.x, position.y, size, size);

    let interaction = button(input, ui, theme, label, rect);
    if interaction.clicked {
        *checked = !*checked;
    }

    interaction
}

// ─────────────────────────────────────────────
// Progress bar
// ─────────────────────────────────────────────

/// Draw a progress bar.
pub fn progress_bar(
    _theme: &UiTheme,
    _rect: Rect,
    progress: f32,
    fill_color: Option<Color>,
    bg_color: Option<Color>,
) {
    let _fill = fill_color.unwrap_or(Color::from_hex("#4CAF50").unwrap());
    let _bg = bg_color.unwrap_or(Color::from_hex("#333333").unwrap());
    let _clamped_progress = progress.clamp(0.0, 1.0);
    // Real impl: draw bg rect, then fill rect with width * progress
}
