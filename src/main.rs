use macroquad::prelude::*;

const SCREEN_W: f32 = 800.0;
const SCREEN_H: f32 = 600.0;

const NEUTRAL_ZONE_X: f32 = SCREEN_W / 2.0 - 150.0;
const NEUTRAL_ZONE_W: f32 = 150.0;

const SHIELD_X: f32 = SCREEN_W - 80.0;
const SHIELD_COLS: usize = 2;
const SHIELD_ROWS: usize = 12;
const CELL_W: f32 = 16.0;
const CELL_H: f32 = (SCREEN_H - 80.0) / SHIELD_ROWS as f32;
const SHIELD_Y: f32 = 40.0;

const YAR_SIZE: f32 = 32.0;
const YAR_SPEED: f32 = 4.0;

const QOTILE_X: f32 = SCREEN_W - 30.0;

const MISSILE_SPEED: f32 = 8.0;
const CANNON_SPEED: f32 = 10.0;
const SWIRL_SPEED: f32 = 2.5;

#[derive(Clone, Copy, PartialEq)]
enum GameState {
    Playing,
    Win,
    Lose,
}

enum NibbleResult {
    Miss,
    Hit(f32, f32), // (dx, dy) bounce offset
    Consumed,
}

struct Shield {
    cells: [[u8; SHIELD_COLS]; SHIELD_ROWS], // 0 = gone, 1 = damaged, 2 = intact
}

impl Shield {
    fn new() -> Self {
        Self {
            cells: [[2; SHIELD_COLS]; SHIELD_ROWS],
        }
    }

    fn draw(&self) {
        for row in 0..SHIELD_ROWS {
            for col in 0..SHIELD_COLS {
                if self.cells[row][col] > 0 {
                    let x = SHIELD_X + col as f32 * CELL_W;
                    let y = SHIELD_Y + row as f32 * CELL_H;
                    let color = if self.cells[row][col] == 2 { YELLOW } else { ORANGE };
                    draw_rectangle(x, y, CELL_W - 1.0, CELL_H - 1.0, color);
                }
            }
        }
    }

    // Rectangle overlap check for Yar's body — returns bounce direction
    fn nibble_contact(&mut self, yar_x: f32, yar_y: f32, yar_w: f32, yar_h: f32) -> NibbleResult {
        for row in 0..SHIELD_ROWS {
            for col in 0..SHIELD_COLS {
                if self.cells[row][col] > 0 {
                    let cx = SHIELD_X + col as f32 * CELL_W;
                    let cy = SHIELD_Y + row as f32 * CELL_H;
                    if yar_x < cx + CELL_W && yar_x + yar_w > cx
                        && yar_y < cy + CELL_H && yar_y + yar_h > cy
                    {
                        // Find the shallowest overlap to determine which side was hit
                        let from_left   = (yar_x + yar_w) - cx;
                        let from_right  = (cx + CELL_W) - yar_x;
                        let from_top    = (yar_y + yar_h) - cy;
                        let from_bottom = (cy + CELL_H) - yar_y;
                        let min = from_left.min(from_right).min(from_top).min(from_bottom);

                        let bounce = if min == from_left {
                            (-28.0, 0.0)
                        } else if min == from_right {
                            (28.0, 0.0)
                        } else if min == from_top {
                            (0.0, -28.0)
                        } else {
                            (0.0, 28.0)
                        };

                        self.cells[row][col] -= 1;
                        if self.cells[row][col] == 0 {
                            return NibbleResult::Consumed;
                        } else {
                            return NibbleResult::Hit(bounce.0, bounce.1);
                        }
                    }
                }
            }
        }
        NibbleResult::Miss
    }

    // Point check for projectiles — one hit destroys the cell
    fn nibble_missile(&mut self, x: f32, y: f32) -> bool {
        for row in 0..SHIELD_ROWS {
            for col in 0..SHIELD_COLS {
                if self.cells[row][col] > 0 {
                    let cx = SHIELD_X + col as f32 * CELL_W;
                    let cy = SHIELD_Y + row as f32 * CELL_H;
                    if x >= cx && x <= cx + CELL_W && y >= cy && y <= cy + CELL_H {
                        self.cells[row][col] = 0;
                        return true;
                    }
                }
            }
        }
        false
    }

    fn is_column_clear(&self, col: usize) -> bool {
        for row in 0..SHIELD_ROWS {
            if self.cells[row][col] > 0 {
                return false;
            }
        }
        true
    }
}

// Regular torp — always available, destroyed by shield cells
struct Missile {
    x: f32,
    y: f32,
    active: bool,
}

impl Missile {
    fn new() -> Self {
        Self { x: 0.0, y: 0.0, active: false }
    }

    fn fire(&mut self, from_x: f32, from_y: f32) {
        if !self.active {
            self.x = from_x;
            self.y = from_y;
            self.active = true;
        }
    }

    fn update(&mut self, shield: &mut Shield) {
        if self.active {
            self.x += MISSILE_SPEED;
            if self.x > SCREEN_W {
                self.active = false;
                return;
            }
            if shield.nibble_missile(self.x, self.y) {
                self.active = false;
            }
        }
    }

    fn draw(&self) {
        if self.active {
            draw_rectangle(self.x, self.y - 2.0, 8.0, 4.0, SKYBLUE);
        }
    }
}

// Zorlon Cannon — charged only by nibbling the shield by contact.
// Icon sits on the left edge of the screen tracking Yar's Y.
// When fired it travels from the left all the way across, destroying
// shield cells, the Swirl, and the Qotile.
struct ZorlonCannon {
    shot_x: f32,
    shot_y: f32,
    firing: bool,
    pub charged: bool,
}

impl ZorlonCannon {
    fn new() -> Self {
        Self { shot_x: 0.0, shot_y: 0.0, firing: false, charged: false }
    }

    fn charge(&mut self) {
        self.charged = true;
    }

    fn fire(&mut self, yar_center_y: f32) {
        if self.charged && !self.firing {
            self.shot_x = 4.0;
            self.shot_y = yar_center_y;
            self.firing = true;
            self.charged = false;
        }
    }

    // Destroys any shield cell the beam passes through. Returns true if
    // the shot reached the Qotile area.
    fn update(&mut self, shield: &mut Shield) -> bool {
        if self.firing {
            self.shot_x += CANNON_SPEED;
            if shield.nibble_missile(self.shot_x, self.shot_y) {
                self.firing = false;
                return false;
            }
            if self.shot_x >= QOTILE_X - 10.0 {
                self.firing = false;
                return true;
            }
            if self.shot_x > SCREEN_W {
                self.firing = false;
            }
        }
        false
    }

    fn hit_swirl(&self, swirl: &Swirl) -> bool {
        if !self.firing || !swirl.active { return false; }
        let dy = (self.shot_y - swirl.y).abs();
        self.shot_x >= swirl.x - 12.0 && self.shot_x <= swirl.x + 12.0 && dy < 12.0
    }

    fn draw(&self, yar_center_y: f32) {
        // Icon on left edge when charged, tracking Yar
        if self.charged {
            draw_rectangle(4.0, yar_center_y - 6.0, 24.0, 12.0, WHITE);
            draw_rectangle(6.0, yar_center_y - 4.0, 20.0, 8.0, YELLOW);
            draw_rectangle(8.0, yar_center_y - 2.0, 16.0, 4.0, WHITE);
        }
        // Shot in flight — same shape, travels right
        if self.firing {
            draw_rectangle(self.shot_x, self.shot_y - 6.0, 24.0, 12.0, WHITE);
            draw_rectangle(self.shot_x + 2.0, self.shot_y - 4.0, 20.0, 8.0, YELLOW);
            draw_rectangle(self.shot_x + 4.0, self.shot_y - 2.0, 16.0, 4.0, WHITE);
        }
    }
}

struct Swirl {
    x: f32,
    y: f32,
    active: bool,
    angle: f32,
}

impl Swirl {
    fn new() -> Self {
        Self { x: QOTILE_X, y: SCREEN_H / 2.0, active: false, angle: 0.0 }
    }

    fn launch(&mut self, target_y: f32) {
        if !self.active {
            self.x = QOTILE_X;
            self.y = SCREEN_H / 2.0;
            self.active = true;
            self.angle = (target_y - self.y).atan2(-self.x);
        }
    }

    fn update(&mut self, target_x: f32, target_y: f32) {
        if self.active {
            let dx = target_x - self.x;
            let dy = target_y - self.y;
            let dist = (dx * dx + dy * dy).sqrt().max(1.0);
            self.x += (dx / dist) * SWIRL_SPEED;
            self.y += (dy / dist) * SWIRL_SPEED;
            self.angle += 0.15;
            if self.x < 0.0 {
                self.active = false;
            }
        }
    }

    fn draw(&self) {
        if self.active {
            let pts = 6;
            for i in 0..pts {
                let a = self.angle + (i as f32 / pts as f32) * std::f32::consts::TAU;
                let r = 8.0;
                let x2 = self.x + a.cos() * r;
                let y2 = self.y + a.sin() * r;
                draw_circle(x2, y2, 3.0, ORANGE);
            }
            draw_circle(self.x, self.y, 4.0, RED);
        }
    }
}

// Side-view fly, facing right.
fn draw_yar(x: f32, y: f32, size: f32, color: Color) {
    let s  = size;
    let cy = y + s * 0.5;

    // Abdomen — tapered point on the left
    draw_triangle(
        Vec2::new(x,           cy),
        Vec2::new(x + s*0.30,  cy - s*0.14),
        Vec2::new(x + s*0.30,  cy + s*0.14),
        color,
    );

    // Thorax — main body block
    draw_rectangle(x + s*0.28, cy - s*0.19, s*0.36, s*0.38, color);

    // Head — circle on the right
    draw_circle(x + s*0.76, cy, s*0.15, color);

    // Proboscis / beak pointing right
    draw_triangle(
        Vec2::new(x + s*0.89, cy - s*0.04),
        Vec2::new(x + s*0.89, cy + s*0.04),
        Vec2::new(x + s,      cy),
        color,
    );

    // Upper wing — triangle pointing up from thorax
    draw_triangle(
        Vec2::new(x + s*0.33, cy - s*0.19),
        Vec2::new(x + s*0.60, cy - s*0.19),
        Vec2::new(x + s*0.30, y),
        color,
    );

    // Lower wing — triangle pointing down from thorax
    draw_triangle(
        Vec2::new(x + s*0.33, cy + s*0.19),
        Vec2::new(x + s*0.60, cy + s*0.19),
        Vec2::new(x + s*0.30, y + s),
        color,
    );

    // Eye — dark pupil on head
    draw_circle(x + s*0.74, cy - s*0.06, s*0.06, BLACK);

    // Legs — three per side from thorax
    let t = (s * 0.04).max(1.5);
    draw_line(x+s*0.38, cy-s*0.19, x+s*0.22, cy-s*0.40, t, color);
    draw_line(x+s*0.48, cy-s*0.19, x+s*0.40, cy-s*0.40, t, color);
    draw_line(x+s*0.58, cy-s*0.19, x+s*0.56, cy-s*0.37, t, color);
    draw_line(x+s*0.38, cy+s*0.19, x+s*0.22, cy+s*0.40, t, color);
    draw_line(x+s*0.48, cy+s*0.19, x+s*0.40, cy+s*0.40, t, color);
    draw_line(x+s*0.58, cy+s*0.19, x+s*0.56, cy+s*0.37, t, color);
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Yars' Revenge".to_owned(),
        window_width: SCREEN_W as i32,
        window_height: SCREEN_H as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut yar_x = 60.0_f32;
    let mut yar_y = SCREEN_H / 2.0;
    let mut shield = Shield::new();
    let mut missile = Missile::new();
    let mut zorlon = ZorlonCannon::new();
    let mut swirl = Swirl::new();
    let mut swirl_timer = 0u32;
    let mut state = GameState::Playing;
    let mut score = 0u32;

    loop {
        if state == GameState::Playing {
            // --- Input ---
            if is_key_down(KeyCode::Up)    { yar_y -= YAR_SPEED; }
            if is_key_down(KeyCode::Down)  { yar_y += YAR_SPEED; }
            if is_key_down(KeyCode::Left)  { yar_x -= YAR_SPEED; }
            if is_key_down(KeyCode::Right) { yar_x += YAR_SPEED; }

            yar_x = yar_x.clamp(0.0, SCREEN_W - YAR_SIZE);
            if yar_y < 0.0 { yar_y = SCREEN_H - YAR_SIZE; }
            if yar_y > SCREEN_H - YAR_SIZE { yar_y = 0.0; }

            let in_neutral_zone = yar_x + YAR_SIZE > NEUTRAL_ZONE_X
                && yar_x < NEUTRAL_ZONE_X + NEUTRAL_ZONE_W;

            let yar_center_y = yar_y + YAR_SIZE / 2.0;

            // Nibble shield by contact — ONLY way to charge the Zorlon Cannon.
            // First hit bounces Yar away; second hit consumes the block.
            match shield.nibble_contact(yar_x, yar_y, YAR_SIZE, YAR_SIZE) {
                NibbleResult::Hit(dx, dy) => {
                    yar_x = (yar_x + dx).clamp(0.0, SCREEN_W - YAR_SIZE);
                    yar_y = (yar_y + dy).clamp(0.0, SCREEN_H - YAR_SIZE);
                }
                NibbleResult::Consumed => {
                    score += 10;
                    zorlon.charge();
                }
                NibbleResult::Miss => {}
            }

            // Space: fire Zorlon Cannon if charged, otherwise fire missile
            if is_key_pressed(KeyCode::Space) {
                if zorlon.charged {
                    zorlon.fire(yar_center_y);
                } else {
                    missile.fire(yar_x + YAR_SIZE, yar_center_y);
                }
            }

            missile.update(&mut shield);

            if zorlon.update(&mut shield) {
                score += 1000;
                state = GameState::Win;
            }

            // Zorlon Cannon shot destroys the Swirl — both disappear
            if zorlon.hit_swirl(&swirl) {
                swirl.active = false;
                zorlon.firing = false;
                score += 250;
            }

            // Swirl launch timer
            swirl_timer += 1;
            if swirl_timer > 180 {
                swirl_timer = 0;
                swirl.launch(yar_y + YAR_SIZE / 2.0);
            }

            swirl.update(yar_x + YAR_SIZE / 2.0, yar_y + YAR_SIZE / 2.0);

            // Swirl hits Yar — neutral zone is safe
            let in_neutral_zone = yar_x + YAR_SIZE > NEUTRAL_ZONE_X
                && yar_x < NEUTRAL_ZONE_X + NEUTRAL_ZONE_W;
            if swirl.active && !in_neutral_zone {
                let dx = swirl.x - (yar_x + YAR_SIZE / 2.0);
                let dy = swirl.y - (yar_y + YAR_SIZE / 2.0);
                if (dx * dx + dy * dy).sqrt() < YAR_SIZE {
                    state = GameState::Lose;
                }
            }

            // If inner shield column is gone, Qotile can fire immediately
            if shield.is_column_clear(0) && !swirl.active {
                swirl.launch(yar_y + YAR_SIZE / 2.0);
            }
        }

        // --- Draw ---
        clear_background(BLACK);

        // Neutral zone
        draw_rectangle(NEUTRAL_ZONE_X, 0.0, NEUTRAL_ZONE_W, SCREEN_H,
            Color::new(0.05, 0.05, 0.25, 1.0));
        draw_line(NEUTRAL_ZONE_X, 0.0, NEUTRAL_ZONE_X, SCREEN_H, 1.0, DARKBLUE);
        draw_line(NEUTRAL_ZONE_X + NEUTRAL_ZONE_W, 0.0,
                  NEUTRAL_ZONE_X + NEUTRAL_ZONE_W, SCREEN_H, 1.0, DARKBLUE);

        shield.draw();

        // Qotile
        draw_circle(QOTILE_X, SCREEN_H / 2.0, 14.0, PURPLE);
        draw_circle(QOTILE_X, SCREEN_H / 2.0, 8.0, WHITE);

        missile.draw();
        zorlon.draw(yar_y + YAR_SIZE / 2.0);
        swirl.draw();

        // Yar (player)
        let in_nz = yar_x + YAR_SIZE > NEUTRAL_ZONE_X
            && yar_x < NEUTRAL_ZONE_X + NEUTRAL_ZONE_W;
        let yar_color = if in_nz {
            Color::new(1.0, 1.0, 1.0, 0.4)
        } else {
            WHITE
        };
        draw_yar(yar_x, yar_y, YAR_SIZE, yar_color);

        // HUD
        draw_text(&format!("SCORE: {}", score), 10.0, 24.0, 24.0, WHITE);

        let status_text = if zorlon.charged {
            "SPACE: fire Zorlon Cannon!  (nibble shield to recharge)"
        } else {
            "SPACE: missile  |  Nibble shield to charge Zorlon Cannon"
        };
        draw_text(status_text, 10.0, SCREEN_H - 10.0, 18.0, GRAY);

        match state {
            GameState::Win => {
                draw_rectangle(200.0, 220.0, 400.0, 120.0, BLACK);
                draw_text("QOTILE DESTROYED!", 230.0, 270.0, 36.0, YELLOW);
                draw_text(&format!("Score: {}  -  Press R to restart", score),
                    215.0, 310.0, 22.0, WHITE);
                if is_key_pressed(KeyCode::R) {
                    yar_x = 60.0; yar_y = SCREEN_H / 2.0;
                    shield = Shield::new(); missile = Missile::new();
                    zorlon = ZorlonCannon::new();
                    swirl = Swirl::new(); swirl_timer = 0; score = 0;
                    state = GameState::Playing;
                }
            }
            GameState::Lose => {
                draw_rectangle(200.0, 220.0, 400.0, 120.0, BLACK);
                draw_text("YAR DESTROYED!", 240.0, 270.0, 36.0, RED);
                draw_text(&format!("Score: {}  -  Press R to restart", score),
                    215.0, 310.0, 22.0, WHITE);
                if is_key_pressed(KeyCode::R) {
                    yar_x = 60.0; yar_y = SCREEN_H / 2.0;
                    shield = Shield::new(); missile = Missile::new();
                    zorlon = ZorlonCannon::new();
                    swirl = Swirl::new(); swirl_timer = 0; score = 0;
                    state = GameState::Playing;
                }
            }
            GameState::Playing => {}
        }

        next_frame().await;
    }
}
