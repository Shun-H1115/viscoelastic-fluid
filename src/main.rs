use macroquad::prelude::*;

// ==================== 定数定義 ==================== //

const PARTICLE_RADIUS: f32 = 3.0;        // 各水粒子の半径
const PARTICLE_COUNT: usize = 1000;      // 水風船を構成する粒子の数
const BALLOON_RADIUS: f32 = 60.0;        // 初期の水風船（真球）の半径
const REST_LENGTH: f32 = 6.0;            // ばねの自然長（粒子間の理想距離）
const STIFFNESS_AFTER: f32 = 300.0;      // 爆発後のばね定数（弾性の強さ）
const DAMPING_AFTER: f32 = 2.0;          // 爆発後の減衰係数（粘性）
const GRAVITY: Vec2 = vec2(0.0, 500.0);  // 重力ベクトル（下向き）
const REBOUND: f32 = -0.3;               // 地面に当たったときの反発係数

// ==================== 構造体定義 ==================== //

/// 水粒子（位置・速度・外力）を表現する構造体
struct Particle {
    position: Vec2,
    velocity: Vec2,
    force: Vec2,
}

/// 弾丸（発射位置・速度・半径・有効フラグ）を表現する構造体
struct Bullet {
    position: Vec2,
    velocity: Vec2,
    radius: f32,
    active: bool,
}

// ==================== 水粒子の処理 ==================== //

impl Particle {
    fn new(pos: Vec2) -> Self {
        Self {
            position: pos,
            velocity: Vec2::ZERO,
            force: Vec2::ZERO,
        }
    }

    /// 外力を加える（重力・ばね力など）
    fn apply_force(&mut self, force: Vec2) {
        self.force += force;
    }

    /// フレーム更新後に力をリセット
    fn reset_force(&mut self) {
        self.force = Vec2::ZERO;
    }

    /// 重力を適用する
    fn apply_gravity(&mut self) {
        self.apply_force(GRAVITY);
    }

    /// 速度・位置を更新（半陰的オイラー法）
    fn update(&mut self, dt: f32) {
        self.velocity += self.force * dt;
        self.position += self.velocity * dt;
        self.reset_force();

        // 地面との衝突処理
        if self.position.y + PARTICLE_RADIUS > screen_height() {
            self.position.y = screen_height() - PARTICLE_RADIUS;
            self.velocity.y *= REBOUND;
        }
    }

    /// 粒子を描画（水色の円）
    fn draw(&self) {
        draw_circle(self.position.x, self.position.y, PARTICLE_RADIUS, Color::new(0.4, 0.7, 1.0, 0.9));
    }
}

// ==================== 弾の処理 ==================== //

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

    /// 弾の位置を更新
    fn update(&mut self, dt: f32) {
        self.position += self.velocity * dt;

        // 画面外に出たら非アクティブ化
        if self.position.x < 0.0 || self.position.x > screen_width()
            || self.position.y < 0.0 || self.position.y > screen_height()
        {
            self.active = false;
        }
    }

    /// 弾を描画（赤い円）
    fn draw(&self) {
        draw_circle(self.position.x, self.position.y, self.radius, RED);
    }

    /// 弾と粒子の衝突判定
    fn collides_with(&self, particle: &Particle) -> bool {
        self.position.distance(particle.position) < (self.radius + PARTICLE_RADIUS)
    }
}

// ==================== 粘弾性ばね力の適用 ==================== //

/// 近接する粒子間にフックの法則と粘性減衰を適用する
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

                // フックの法則によるばね力
                let f_spring = dir * (stiffness * x);

                // 相対速度によるダンピング
                let v_rel = particles[j].velocity - particles[i].velocity;
                let f_damp = dir * (v_rel.dot(dir) * damping);

                let force = f_spring + f_damp;

                particles[i].apply_force(force);
                particles[j].apply_force(-force);
            }
        }
    }
}

// ==================== 球状の粒子配置 ==================== //

/// 真球状に粒子を配置する（極座標を用いた同心円配置）
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

// ==================== メインループ ==================== //

#[macroquad::main("Perfect Spherical Water Balloon")]
async fn main() {
    let mut particles: Vec<Particle> = Vec::new();   // 水風船の粒子群
    let mut bullets: Vec<Bullet> = Vec::new();       // 弾の配列
    let mut exploded = false;                        // 爆発済みフラグ
    let mut initialized = false;                     // 初期化済みフラグ

    loop {
        clear_background(BLACK);
        let dt = get_frame_time();

        // 初期化（画面サイズ取得後に実行）
        if !initialized {
            let center = vec2(screen_width() / 2.0, screen_height() / 2.5);
            particles = generate_spherical_particles(center, PARTICLE_COUNT, BALLOON_RADIUS);
            initialized = true;
        }

        // マウスクリックで弾を発射
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            bullets.push(Bullet::new(vec2(screen_width() / 2.0, screen_height()), vec2(mx, my)));
        }

        // 弾と水風船の衝突判定（初回のみ）
        for bullet in bullets.iter_mut() {
            bullet.update(dt);
            if !exploded {
                if particles.iter().any(|p| bullet.collides_with(p)) {
                    exploded = true;
                }
            }
        }

        // 爆発後は物理シミュレーションを適用
        if exploded {
            for p in particles.iter_mut() {
                p.apply_gravity();
            }
            apply_spring_forces(&mut particles);
            for p in particles.iter_mut() {
                p.update(dt);
            }
        }

        // 粒子描画
        for p in particles.iter() {
            p.draw();
        }

        // 弾の描画と削除処理
        bullets.retain(|b| b.active);
        for b in bullets.iter() {
            b.draw();
        }

        next_frame().await;
    }
}
