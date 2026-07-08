//! Basic example: a bouncing ball with camera shake and tweening.
//!
//! Run with: `cargo run --example basics`

use game_gem::prelude::*;

struct BouncingBall {
    position: Vec2,
    velocity: Vec2,
    radius: f32,
    color: Color,
    trail: Vec<(Vec2, f32)>, // (position, alpha)
}

impl BouncingBall {
    fn new() -> Self {
        Self {
            position: vec2(400.0, 300.0),
            velocity: vec2(250.0, -180.0),
            radius: 30.0,
            color: Color::GOLD,
            trail: Vec::with_capacity(60),
        }
    }
}

struct BasicsExample {
    ball: BouncingBall,
    hue: f32,
    click_count: u32,
}

impl GameState for BasicsExample {
    fn on_enter(&mut self, ctx: &mut Context) {
        // Register input actions
        let mut jump = InputAction::new("shake");
        jump.bind_key(KeyCode::Space);
        ctx.input.register_action(jump);

        // Set up camera with follow
        ctx.graphics.camera.follow(self.ball.position, 0.05);
    }

    fn update(&mut self, ctx: &mut Context) {
        let dt = ctx.time.delta() as f32;

        // --- Ball physics ---
        self.ball.velocity.y += 600.0 * dt; // Gravity
        self.ball.position += self.ball.velocity * dt;

        // Bounce off walls
        let w = ctx.screen_width();
        let h = ctx.screen_height();

        if self.ball.position.x - self.ball.radius < 0.0 {
            self.ball.position.x = self.ball.radius;
            self.ball.velocity.x = self.ball.velocity.x.abs();
        }
        if self.ball.position.x + self.ball.radius > w {
            self.ball.position.x = w - self.ball.radius;
            self.ball.velocity.x = -self.ball.velocity.x.abs();
        }
        if self.ball.position.y - self.ball.radius < 0.0 {
            self.ball.position.y = self.ball.radius;
            self.ball.velocity.y = self.ball.velocity.y.abs();
        }
        if self.ball.position.y + self.ball.radius > h {
            self.ball.position.y = h - self.ball.radius;
            self.ball.velocity.y = -self.ball.velocity.y.abs() * 0.95; // Damping
        }

        // Update trail
        self.ball.trail.push((self.ball.position, 1.0));
        if self.ball.trail.len() > 60 {
            self.ball.trail.remove(0);
        }
        for (_, alpha) in &mut self.ball.trail {
            *alpha -= dt * 2.0;
        }
        self.ball.trail.retain(|(_, a)| *a > 0.0);

        // --- Input ---
        if ctx.input.is_action_pressed("shake") {
            ctx.graphics.camera.shake(10.0, 0.3);
        }

        if ctx.input.mouse.is_pressed(MouseButton::Left) {
            self.click_count += 1;
            // Change ball color
            self.hue = (self.hue + 30.0) % 360.0;
            self.ball.color = Color::from_hsla(self.hue, 0.8, 0.6, 1.0);
            // Boost ball toward click
            let dir = ctx.input.mouse.position - self.ball.position;
            self.ball.velocity += dir.normalize_or_zero() * 200.0;
        }

        // Zoom with scroll
        let zoom = ctx.graphics.camera.zoom + ctx.input.mouse.scroll.y * 0.1;
        ctx.graphics.camera.set_zoom(zoom);

        // Escape to quit
        if ctx.input.keyboard.is_pressed(KeyCode::Escape) {
            ctx.quit();
        }

        // Update camera follow target
        ctx.graphics.camera.follow(self.ball.position, 0.05);
    }

    fn render(&mut self, ctx: &mut Context) {
        ctx.graphics.clear(ctx.window.clear_color);

        // Draw trail
        for &(pos, alpha) in &self.ball.trail {
            ctx.graphics.draw_circle(
                pos.x, pos.y,
                self.ball.radius * 0.5 * alpha,
                self.ball.color.with_alpha(alpha * 0.5),
            );
        }

        // Draw ball
        ctx.graphics.draw_circle(
            self.ball.position.x,
            self.ball.position.y,
            self.ball.radius,
            self.ball.color,
        );

        // Draw HUD
        ctx.graphics.reset_camera();
        ctx.graphics.draw_text(
            &format!("FPS: {:.0}", ctx.time.fps()),
            10.0, 10.0, 18.0, Color::WHITE,
        );
        ctx.graphics.draw_text(
            &format!("Clicks: {}", self.click_count),
            10.0, 35.0, 18.0, Color::WHITE,
        );
        ctx.graphics.draw_text(
            "Space = shake | Click = change color | Scroll = zoom | Esc = quit",
            10.0, 60.0, 14.0, Color::LIGHT_GRAY,
        );
    }
}

fn main() {
    let example = BasicsExample {
        ball: BouncingBall::new(),
        hue: 0.0,
        click_count: 0,
    };

    Game::new()
        .window_title("game-gem: Basics Example")
        .window_size(800, 600)
        .clear_color(Color::from_hex("#0F0F23").unwrap())
        .run(example);
}