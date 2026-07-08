//! Scene management example with transitions.
//!
//! Run with: `cargo run --example scene_demo`

use game_gem::prelude::*;

// ─────────────────────────────────────────────
// Menu Scene
// ─────────────────────────────────────────────

struct MenuScene {
    selected: usize,
    options: Vec<String>,
}

impl MenuScene {
    fn new() -> Self {
        Self {
            selected: 0,
            options: vec![
                "Play Game".to_string(),
                "Settings".to_string(),
                "Quit".to_string(),
            ],
        }
    }
}

impl Scene for MenuScene {
    fn on_enter(&mut self, _ctx: &mut Context) {
        self.selected = 0;
    }

    fn update(&mut self, ctx: &mut Context) {
        if ctx.input.keyboard.is_pressed(KeyCode::Up) || ctx.input.keyboard.is_pressed(KeyCode::W) {
            self.selected = self.selected.saturating_sub(1);
        }
        if ctx.input.keyboard.is_pressed(KeyCode::Down) || ctx.input.keyboard.is_pressed(KeyCode::S) {
            self.selected = (self.selected + 1).min(self.options.len() - 1);
        }
        if ctx.input.keyboard.is_pressed(KeyCode::Enter) {
            match self.selected {
                0 => {
                    // Transition to game scene
                    let _transition = Transition::Fade {
                        duration: 0.5,
                        color: [0.0, 0.0, 0.0, 1.0],
                    };
                    // ctx.scene.push_with_transition(GameScene::new(), transition);
                }
                2 => {
                    ctx.quit();
                }
                _ => {}
            }
        }
    }

    fn render(&mut self, ctx: &mut Context) {
        ctx.graphics.clear(Color::from_hex("#16213E").unwrap());

        // Title
        ctx.graphics.draw_text("game-gem Scene Demo", 250.0, 150.0, 36.0, Color::GOLD);

        // Menu options
        for (i, option) in self.options.iter().enumerate() {
            let color = if i == self.selected {
                Color::GOLD
            } else {
                Color::LIGHT_GRAY
            };
            let prefix = if i == self.selected { "> " } else { "  " };
            ctx.graphics.draw_text(
                &format!("{}{}", prefix, option),
                320.0, 250.0 + i as f32 * 50.0,
                24.0, color,
            );
        }

        ctx.graphics.draw_text("Arrow Keys / WASD to navigate, Enter to select", 200.0, 500.0, 14.0, Color::GRAY);
    }
}

// ─────────────────────────────────────────────
// Main
// ─────────────────────────────────────────────

struct SceneDemo {
    scenes: SceneManager,
}

impl GameState for SceneDemo {
    fn on_enter(&mut self, _ctx: &mut Context) {
        self.scenes.push(MenuScene::new());
    }

    fn update(&mut self, ctx: &mut Context) {
        self.scenes.update(ctx);
    }

    fn render(&mut self, ctx: &mut Context) {
        self.scenes.render(ctx);
    }
}

fn main() {
    let example = SceneDemo {
        scenes: SceneManager::new(),
    };

    Game::new()
        .window_title("game-gem: Scene Demo")
        .window_size(800, 600)
        .run(example);
}