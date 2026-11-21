use crate::assets::{Assets, Direction};
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

    pub fn update_and_draw(&mut self, assets: &Assets) {
        self.timer += 1;
        self.y -= 0.8;

        let alpha = ((self.duration - self.timer) as f32 / self.duration as f32).clamp(0.0, 1.0);
        let shadow_color = Color::new(0.0, 0.0, 0.0, alpha);
        let text_color = Color::new(self.color.r, self.color.g, self.color.b, alpha);

        let font_size = 28;
        let dims = measure_text(&self.text, Some(&assets.dungeon_font), font_size, 1.0);
        let draw_x = self.x - dims.width / 2.0;
        let draw_y = self.y - dims.height / 2.0 + dims.offset_y;

        // Shadow
        draw_text_ex(
            &self.text,
            draw_x + 2.0,
            draw_y + 2.0,
            TextParams {
                font: Some(&assets.dungeon_font),
                font_size,
                color: shadow_color,
                ..Default::default()
            },
        );

        // Main text
        draw_text_ex(
            &self.text,
            draw_x,
            draw_y,
            TextParams {
                font: Some(&assets.dungeon_font),
                font_size,
                color: text_color,
                ..Default::default()
            },
        );
    }
}

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

    pub fn update_and_draw(&mut self) {
        self.timer += 1;

        let start_angle: f32 = match self.direction {
            Direction::Right => -0.6,
            Direction::Down => 0.9,
            Direction::Left => 2.5,
            Direction::Up => 4.0,
        };

        let progress = (self.timer as f32 / self.duration as f32).clamp(0.0, 1.0);
        let sweep = 0.6 + 1.6 * progress;

        let rx = 35.0;
        let ry = 23.0;
        let segments = 16;
        let step = sweep / segments as f32;

        let mut prev_x = self.x + rx * start_angle.cos();
        let mut prev_y = self.y - ry * start_angle.sin(); // Pygame inverted y angle

        for i in 1..=segments {
            let angle = start_angle + step * i as f32;
            let cur_x = self.x + rx * angle.cos();
            let cur_y = self.y - ry * angle.sin();
            draw_line(prev_x, prev_y, cur_x, cur_y, 4.0, WHITE);
            prev_x = cur_x;
            prev_y = cur_y;
        }
    }
}

pub struct Effetti {
    pub testi: Vec<FloatingText>,
    pub fendenti: Vec<Fendente>,
}

impl Effetti {
    pub fn new() -> Self {
        Self {
            testi: Vec::with_capacity(32),
            fendenti: Vec::with_capacity(8),
        }
    }

    pub fn add_text(&mut self, text: &str, x: f32, y: f32, color: Color) {
        self.testi.push(FloatingText::new(text.to_string(), x, y, color));
    }

    pub fn add_fendente(&mut self, x: f32, y: f32, dir: Direction) {
        self.fendenti.push(Fendente::new(x, y, dir));
    }

    pub fn update_and_draw(&mut self, assets: &Assets) {
        for f in &mut self.fendenti {
            f.update_and_draw();
        }
        self.fendenti.retain(|f| !f.is_finished());

        for t in &mut self.testi {
            t.update_and_draw(assets);
        }
        self.testi.retain(|t| !t.is_finished());
    }
}

impl Default for Effetti {
    fn default() -> Self {
        Self::new()
    }
}
