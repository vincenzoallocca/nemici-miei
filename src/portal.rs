use crate::assets::Assets;
use crate::effetti::Effetti;
use crate::settings::SCALA_PIXEL;
use crate::world::World;
use macroquad::prelude::*;

pub struct Portale {
    pub tile_x: i32,
    pub tile_y: i32,
    pub frame_index: usize,
    pub frame_counter: i32,
    /// Punto dove poggia la base del portale.
    pub x: f32,
    pub y: f32,
    pub aperto: bool,
}

impl Portale {
    pub fn new(tile_x: i32, tile_y: i32, world: &World) -> Self {
        let (x, y) = world.piedi_tile(tile_x, tile_y);
        Self {
            tile_x,
            tile_y,
            frame_index: 0,
            frame_counter: 0,
            x: x.round(),
            y: y.round(),
            aperto: false,
        }
    }

    pub fn tile(&self) -> (i32, i32) {
        (self.tile_x, self.tile_y)
    }

    pub fn profondita(&self) -> f32 {
        self.y
    }

    pub fn apri(&mut self, effetti: &mut Effetti) {
        if !self.aperto {
            self.aperto = true;
            effetti.scintille(self.x, self.y - 30.0, Color::from_rgba(120, 255, 140, 255), 24);
        }
    }

    pub fn update(&mut self, effetti: &mut Effetti) {
        self.frame_counter += 1;
        if self.frame_counter >= 10 {
            self.frame_counter = 0;
            self.frame_index = (self.frame_index + 1) % 6;
        }
        if self.aperto && rand::gen_range(0, 6) == 0 {
            let x = self.x + rand::gen_range(-12.0, 12.0);
            effetti.brace(x, self.y - rand::gen_range(0.0, 50.0), Color::from_rgba(140, 255, 160, 255));
        }
    }

    /// Cima del portale: dove punta la freccia dell'obiettivo.
    pub fn cima(&self) -> (f32, f32) {
        (self.x, self.y - 30.0 * SCALA_PIXEL)
    }

    pub fn draw(&self, assets: &Assets) {
        let alpha = if self.aperto { 1.0 } else { 110.0 / 255.0 };
        if let Some(tex) = assets.portal_frames.get(self.frame_index) {
            // 11x30 disegnato 2x: le proporzioni restano quelle dello sprite
            let w = tex.width() * SCALA_PIXEL;
            let h = tex.height() * SCALA_PIXEL;
            if self.aperto {
                draw_ellipse(self.x, self.y, 18.0, 7.0, 0.0, Color::new(0.4, 1.0, 0.5, 0.35));
            }
            draw_texture_ex(
                tex,
                self.x - (w / 2.0).round(),
                self.y - h + 4.0,
                Color::new(1.0, 1.0, 1.0, alpha),
                DrawTextureParams {
                    dest_size: Some(vec2(w, h)),
                    ..Default::default()
                },
            );
        }
    }
}
