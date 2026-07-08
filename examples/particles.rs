//! Particle system example.
//!
//! Run with: `cargo run --example particles`

use game_gem::prelude::*;

struct ParticlesExample {
    emitters: Vec<ParticleEmitter>,
}

impl GameState for ParticlesExample {
    fn on_enter(&mut self, _ctx: &mut Context) {
        // Fire emitter at bottom center
        let mut fire = ParticleEmitter::new(vec2(400.0, 500.0));
        fire.configure(|cfg| {
            cfg
                .rate(80.0)
                .shape(EmitterShape::Line { x1: 370.0, y1: 500.0, x2: 430.0, y2: 500.0 })
                .speed(80.0, 200.0)
                .angle(std::f32::consts::PI * 1.1, std::f32::consts::PI * 1.9)
                .size(3.0, 10.0)
                .lifetime(0.3, 1.0)
                .gravity(0.0, -80.0)
                .color_start(Color::from_hex("#FF6600").unwrap())
                .color_end(Color::from_hex("#FFFF00").unwrap())
                .size_end(0.0)
                .drag(1.5);
        });
        self.emitters.push(fire);

        // Sparkle emitter
        let mut sparkles = ParticleEmitter::new(vec2(400.0, 300.0));
        sparkles.configure(|cfg| {
            cfg
                .rate(30.0)
                .shape(EmitterShape::Circle { cx: 400.0, cy: 300.0, radius: 150.0 })
                .speed(10.0, 50.0)
                .angle(0.0, std::f32::consts::TAU)
                .size(2.0, 6.0)
                .lifetime(0.5, 1.5)
                .gravity(0.0, 20.0)
                .color_start(Color::from_hex("#FFFFFF").unwrap())
                .color_end(Color::TRANSPARENT)
                .size_end(0.0);
        });
        self.emitters.push(sparkles);
    }

    fn update(&mut self, ctx: &mut Context) {
        let dt = ctx.time.delta() as f32;

        // Move fire emitter to mouse X
        self.emitters[0].position.x = ctx.input.mouse.position.x;
        self.emitters[0].position.y = ctx.screen_height() - 50.0;

        // Burst on click
        if ctx.input.mouse.is_pressed(MouseButton::Left) {
            self.emitters[1].position = ctx.input.mouse.position;
            self.emitters[1].burst_now(50);
        }

        // Update all emitters
        for emitter in &mut self.emitters {
            emitter.update(dt);
        }

        if ctx.input.keyboard.is_pressed(KeyCode::Escape) {
            ctx.quit();
        }
    }

    fn render(&mut self, ctx: &mut Context) {
        ctx.graphics.clear(Color::from_hex("#0A0A1A").unwrap());

        // Draw particles
        for emitter in &self.emitters {
            for particle in emitter.particles() {
                if !particle.alive {
                    continue;
                }
                ctx.graphics.draw_circle(
                    particle.position.x,
                    particle.position.y,
                    particle.size,
                    particle.color,
                );
            }
        }

        // HUD
        ctx.graphics.draw_text(
            &format!("Particles: {}", self.emitters.iter().map(|e| e.alive_count()).sum::<usize>()),
            10.0, 10.0, 18.0, Color::WHITE,
        );
        ctx.graphics.draw_text(
            "Move mouse = fire | Click = sparkles | Esc = quit",
            10.0, 35.0, 14.0, Color::LIGHT_GRAY,
        );
    }
}

fn main() {
    let example = ParticlesExample {
        emitters: Vec::new(),
    };

    Game::new()
        .window_title("game-gem: Particles Example")
        .window_size(800, 600)
        .clear_color(Color::from_hex("#0A0A1A").unwrap())
        .run(example);
}