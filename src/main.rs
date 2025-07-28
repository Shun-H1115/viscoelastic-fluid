use macroquad::prelude::*;

const PARTICLE_RADIUS: f32 = 3.0;
const PARTICLE_COUNT: usize = 1000;
const BALLOON_RADIUS: f32 = 60.0;
const REST_LENGTH: f32 = 6.0;
const STIFFNESS_AFTER: f32 = 300.0;
const DAMPING_AFTER: f32 = 2.0;
const GRAVITY: Vec2 = vec2(0.0, 500.0);
const REBOUND: f32 = -0.3;

struct Particle {
    position: Vec2,
    velocity: Vec2,
    force: Vec2,
}

struct Bullet {
    position: Vec2,
    velocity: Vec2,
    radius: f32,
    active: bool,
}

impl Particle {
    fn new(pos: Vec2) -> Self {
        Self {
            position: pos,
            velocity: Vec2::ZERO,
            force: Vec2::ZERO,
        }
    }

    fn apply_force(&mut self, force: Vec2) {
        self.force += force;
    }

    fn reset_force(&mut self) {
        self.force = Vec2::ZERO;
    }

    fn apply_gravity(&mut self) {
        self.apply_force(GRAVITY);
    }

    fn update(&mut self, dt: f32) {
        self.velocity += self.force * dt;
        self.position += self.velocity * dt;
        self.reset_force();

        if self.position.y + PARTICLE_RADIUS > screen_height() {
            self.position.y = screen_height() - PARTICLE_RADIUS;
            self.velocity.y *= REBOUND;
        }
    }

    fn draw(&self) {
        draw_circle(self.position.x, self.position.y, PARTICLE_RADIUS, Color::new(0.4, 0.7, 1.0, 0.9));
    }
}

impl Bullet {
    fn new(start: Vec2, target: Vec2) -> Self {
        let dir = (target - start).normalize();
        let speed = 800.0;
        Self {
            position: start,
            velocity: dir * speed,
            radius: 5.0,
            active: true,
        }
    }

    fn update(&mut self, dt: f32) {
        self.position += self.velocity * dt;
        if self.position.x < 0.0
            || self.position.x > screen_width()
            || self.position.y < 0.0
            || self.position.y > screen_height()
        {
            self.active = false;
        }
    }

    fn draw(&self) {
        draw_circle(self.position.x, self.position.y, self.radius, RED);
    }

    fn collides_with(&self, particle: &Particle) -> bool {
        self.position.distance(particle.position) < (self.radius + PARTICLE_RADIUS)
    }
}

fn apply_spring_forces(particles: &mut Vec<Particle>) {
    let stiffness = STIFFNESS_AFTER;
    let damping = DAMPING_AFTER;

    let n = particles.len();
    for i in 0..n {
        for j in (i + 1)..n {
            let delta = particles[j].position - particles[i].position;
            let dist = delta.length();
            if dist < REST_LENGTH * 2.0 && dist > 0.01 {
                let dir = delta.normalize();
                let x = dist - REST_LENGTH;

                let f_spring = dir * (stiffness * x);
                let v_rel = particles[j].velocity - particles[i].velocity;
                let f_damp = dir * (v_rel.dot(dir) * damping);
                let force = f_spring + f_damp;

                particles[i].apply_force(force);
                particles[j].apply_force(-force);
            }
        }
    }
}

fn generate_spherical_particles(center: Vec2, count: usize, radius: f32) -> Vec<Particle> {
    let mut particles = Vec::with_capacity(count);
    let mut r = 0.0;

    while particles.len() < count {
        let layer_radius = r;
        let points_in_layer = ((2.0 * std::f32::consts::PI * layer_radius) / (PARTICLE_RADIUS * 2.5)).max(6.0) as usize;

        for i in 0..points_in_layer {
            if particles.len() >= count {
                break;
            }
            let theta = i as f32 / points_in_layer as f32 * 2.0 * std::f32::consts::PI;
            let offset = vec2(layer_radius * theta.cos(), layer_radius * theta.sin());
            particles.push(Particle::new(center + offset));
        }

        r += PARTICLE_RADIUS * 2.0;
        if r > radius {
            break;
        }
    }

    particles
}

#[macroquad::main("Perfect Spherical Water Balloon")]
async fn main() {
    let mut particles: Vec<Particle> = Vec::new();
    let mut bullets: Vec<Bullet> = Vec::new();
    let mut exploded = false;
    let mut initialized = false;

    loop {
        clear_background(BLACK);
        let dt = get_frame_time();

        if !initialized {
            let center = vec2(screen_width() / 2.0, screen_height() / 2.5);
            particles = generate_spherical_particles(center, PARTICLE_COUNT, BALLOON_RADIUS);
            initialized = true;
        }

        // 弾発射
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            bullets.push(Bullet::new(vec2(screen_width() / 2.0, screen_height()), vec2(mx, my)));
        }

        // 弾更新＆命中で爆発
        for bullet in bullets.iter_mut() {
            bullet.update(dt);
            if !exploded {
                if particles.iter().any(|p| bullet.collides_with(p)) {
                    exploded = true;
                }
            }
        }

        if exploded {
            for p in particles.iter_mut() {
                p.apply_gravity();
            }
            apply_spring_forces(&mut particles);
            for p in particles.iter_mut() {
                p.update(dt);
            }
        }

        for p in particles.iter() {
            p.draw();
        }

        bullets.retain(|b| b.active);
        for b in bullets.iter() {
            b.draw();
        }

        next_frame().await;
    }
}
