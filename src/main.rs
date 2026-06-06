use macroquad::prelude::*;

const SCREEN_W: f32 = 800.0;
const SCREEN_H: f32 = 600.0;

const NEUTRAL_ZONE_X: f32 = SCREEN_W / 2.0 - 20.0;
const NEUTRAL_ZONE_W: f32 = 40.0;

const SHIELD_X: f32 = SCREEN_W - 80.0;
const SHIELD_COLS: usize = 2;
const SHIELD_ROWS: usize = 12;
const CELL_W: f32 = 16.0;
const CELL_H: f32 = (SCREEN_H - 80.0) / SHIELD_ROWS as f32;
const SHIELD_Y: f32 = 40.0;

const YAR_SIZE: f32 = 16.0;
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

struct Shield {
    cells: [[bool; SHIELD_COLS]; SHIELD_ROWS],
}

impl Shield {
    fn new() -> Self {
        Self {
            cells: [[true; SHIELD_COLS]; SHIELD_ROWS],
        }
    }

    fn draw(&self) {
        for row in 0..SHIELD_ROWS {
            for col in 0..SHIELD_COLS {
                if self.cells[row][col] {
                    let x = SHIELD_X + col as f32 * CELL_W;
                    let y = SHIELD_Y + row as f32 * CELL_H;
                    draw_rectangle(x, y, CELL_W - 1.0, CELL_H - 1.0, YELLOW);
                }
            }
        }
    }

    fn nibble(&mut self, x: f32, y: f32) -> bool {
        for row in 0..SHIELD_ROWS {
            for col in 0..SHIELD_COLS {
                if self.cells[row][col] {
                    let cx = SHIELD_X + col as f32 * CELL_W;
                    let cy = SHIELD_Y + row as f32 * CELL_H;
                    if x >= cx && x <= cx + CELL_W && y >= cy && y <= cy + CELL_H {
                        self.cells[row][col] = false;
                        return true;
                    }
                }
            }
        }
        false
    }

    fn blocks(&self, x: f32, y: f32) -> bool {
        for row in 0..SHIELD_ROWS {
            for col in 0..SHIELD_COLS {
                if self.cells[row][col] {
                    let cx = SHIELD_X + col as f32 * CELL_W;
                    let cy = SHIELD_Y + row as f32 * CELL_H;
                    if x >= cx && x <= cx + CELL_W && y >= cy && y <= cy + CELL_H {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn is_column_clear(&self, col: usize) -> bool {
        for row in 0..SHIELD_ROWS {
            if self.cells[row][col] {
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

    // Returns true if it destroyed a shield cell
    fn update(&mut self, shield: &mut Shield) -> bool {
        if self.active {
            self.x += MISSILE_SPEED;
            if self.x > SCREEN_W {
                self.active = false;
                return false;
            }
            if shield.nibble(self.x, self.y) {
                self.active = false;
                return true;
            }
        }
        false
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
            shield.nibble(self.shot_x, self.shot_y);
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
            yar_y = yar_y.clamp(0.0, SCREEN_H - YAR_SIZE);

            let in_neutral_zone = yar_x + YAR_SIZE > NEUTRAL_ZONE_X
                && yar_x < NEUTRAL_ZONE_X + NEUTRAL_ZONE_W;

            let yar_center_y = yar_y + YAR_SIZE / 2.0;

            // Nibble shield by contact — ONLY way to charge the Zorlon Cannon
            if yar_x + YAR_SIZE >= SHIELD_X {
                if shield.nibble(yar_x + YAR_SIZE, yar_center_y) {
                    score += 10;
                    zorlon.charge();
                }
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

            // Zorlon Cannon shot destroys the Swirl
            if zorlon.hit_swirl(&swirl) {
                swirl.active = false;
                score += 250;
            }

            // Swirl launch timer
            swirl_timer += 1;
            if swirl_timer > 180 {
                swirl_timer = 0;
                swirl.launch(yar_y + YAR_SIZE / 2.0);
            }

            swirl.update(yar_x + YAR_SIZE / 2.0, yar_y + YAR_SIZE / 2.0);

            // Swirl hits Yar
            if swirl.active {
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
            Color::new(0.3, 1.0, 0.3, 0.5)
        } else {
            GREEN
        };
        draw_rectangle(yar_x, yar_y, YAR_SIZE, YAR_SIZE, yar_color);
        draw_triangle(
            Vec2::new(yar_x + YAR_SIZE, yar_y + YAR_SIZE / 2.0),
            Vec2::new(yar_x, yar_y),
            Vec2::new(yar_x, yar_y + YAR_SIZE),
            yar_color,
        );

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
