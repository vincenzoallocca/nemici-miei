use crate::assets::{Assets, Direction, Suono};
use crate::hud::{misura_testo, parametri_testo};
use macroquad::prelude::*;

pub struct FloatingText {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub color: Color,
    pub duration: i32,
    pub timer: i32,
}

impl FloatingText {
    pub fn new(text: String, x: f32, y: f32, color: Color) -> Self {
        Self {
            text,
            x,
            y,
            color,
            duration: 45,
            timer: 0,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.timer >= self.duration
    }

    pub fn update(&mut self) {
        self.timer += 1;
        self.y -= 0.8;
    }

    pub fn draw(&self, assets: &Assets) {
        let alpha = ((self.duration - self.timer) as f32 / self.duration as f32).clamp(0.0, 1.0);
        let shadow_color = Color::new(0.0, 0.0, 0.0, alpha);
        let text_color = Color::new(self.color.r, self.color.g, self.color.b, alpha);

        let font = &assets.dungeon_font;
        let dims = misura_testo(&self.text, font, 28);
        let draw_x = (self.x - dims.width / 2.0).round();
        let draw_y = (self.y - dims.height / 2.0 + dims.offset_y).round();
        draw_text_ex(&self.text, draw_x + 2.0, draw_y + 2.0, parametri_testo(font, 28, shadow_color));
        draw_text_ex(&self.text, draw_x, draw_y, parametri_testo(font, 28, text_color));
    }
}

/// L'arco bianco del colpo di spada.
pub struct Fendente {
    pub x: f32,
    pub y: f32,
    pub direction: Direction,
    pub duration: i32,
    pub timer: i32,
}

impl Fendente {
    pub fn new(x: f32, y: f32, direction: Direction) -> Self {
        Self {
            x,
            y,
            direction,
            duration: 10,
            timer: 0,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.timer >= self.duration
    }

    pub fn draw(&self) {
        let start_angle: f32 = match self.direction {
            Direction::Right => -0.6,
            Direction::Down => 0.9,
            Direction::Left => 2.5,
            Direction::Up => 4.0,
        };

        let progress = (self.timer as f32 / self.duration as f32).clamp(0.0, 1.0);
        let sweep = 0.6 + 1.6 * progress;
        let alpha = 1.0 - progress * 0.5;

        let rx = 35.0;
        let ry = 23.0;
        let segments = 16;
        let step = sweep / segments as f32;

        let mut prev = vec2(self.x + rx * start_angle.cos(), self.y - ry * start_angle.sin());
        for i in 1..=segments {
            let angle = start_angle + step * i as f32;
            let cur = vec2(self.x + rx * angle.cos(), self.y - ry * angle.sin());
            draw_line(prev.x, prev.y, cur.x, cur.y, 4.0, Color::new(1.0, 1.0, 1.0, alpha));
            prev = cur;
        }
    }
}

/// Piccoli quadrati che schizzano via: colpi, monete, furia del boss.
pub struct Particella {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub gravita: f32,
    pub vita: i32,
    pub durata: i32,
    pub colore: Color,
    pub dim: f32,
}

pub struct Effetti {
    pub testi: Vec<FloatingText>,
    pub fendenti: Vec<Fendente>,
    pub particelle: Vec<Particella>,
    /// Suoni da riprodurre: li svuota il ciclo principale.
    pub suoni: Vec<(Suono, f32)>,
}

impl Default for Effetti {
    fn default() -> Self {
        Self::new()
    }
}

impl Effetti {
    pub fn new() -> Self {
        Self {
            testi: Vec::with_capacity(32),
            fendenti: Vec::with_capacity(8),
            particelle: Vec::with_capacity(128),
            suoni: Vec::with_capacity(8),
        }
    }

    pub fn add_text(&mut self, text: &str, x: f32, y: f32, color: Color) {
        self.testi.push(FloatingText::new(text.to_string(), x, y, color));
    }

    pub fn add_fendente(&mut self, x: f32, y: f32, dir: Direction) {
        self.fendenti.push(Fendente::new(x, y, dir));
    }

    pub fn suona(&mut self, suono: Suono, volume: f32) {
        self.suoni.push((suono, volume));
    }

    /// Schizzo di `quante` particelle dal punto (x, y).
    pub fn scintille(&mut self, x: f32, y: f32, colore: Color, quante: usize) {
        for _ in 0..quante {
            let angolo = rand::gen_range(0.0, std::f32::consts::TAU);
            let forza = rand::gen_range(1.0, 3.2);
            let durata = rand::gen_range(18, 34);
            self.particelle.push(Particella {
                x,
                y,
                vx: angolo.cos() * forza,
                vy: angolo.sin() * forza - 1.5,
                gravita: 0.12,
                vita: durata,
                durata,
                colore,
                dim: if rand::gen_range(0, 3) == 0 { 4.0 } else { 2.0 },
            });
        }
    }

    /// Particella che sale lentamente (aura del boss in furia).
    pub fn brace(&mut self, x: f32, y: f32, colore: Color) {
        let durata = rand::gen_range(30, 50);
        self.particelle.push(Particella {
            x,
            y,
            vx: rand::gen_range(-0.3, 0.3),
            vy: rand::gen_range(-1.4, -0.6),
            gravita: 0.0,
            vita: durata,
            durata,
            colore,
            dim: 2.0,
        });
    }

    pub fn update(&mut self) {
        for f in &mut self.fendenti {
            f.timer += 1;
        }
        self.fendenti.retain(|f| !f.is_finished());

        for t in &mut self.testi {
            t.update();
        }
        self.testi.retain(|t| !t.is_finished());

        for p in &mut self.particelle {
            p.x += p.vx;
            p.y += p.vy;
            p.vy += p.gravita;
            p.vx *= 0.97;
            p.vita -= 1;
        }
        self.particelle.retain(|p| p.vita > 0);
    }

    pub fn draw(&self, assets: &Assets) {
        for p in &self.particelle {
            let alpha = (p.vita as f32 / p.durata as f32).clamp(0.0, 1.0);
            let c = Color::new(p.colore.r, p.colore.g, p.colore.b, p.colore.a * alpha);
            draw_rectangle(p.x.round(), p.y.round(), p.dim, p.dim, c);
        }
        for f in &self.fendenti {
            f.draw();
        }
        for t in &self.testi {
            t.draw(assets);
        }
    }
}
