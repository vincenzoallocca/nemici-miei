use crate::assets::Assets;
use crate::settings::GRANDEZZA_TILES;
use crate::world::World;
use macroquad::prelude::*;

pub struct Portale {
    pub tile_x: i32,
    pub tile_y: i32,
    pub frame_index: usize,
    pub frame_counter: i32,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub bottom: f32,
}

impl Portale {
    pub fn new(tile_x: i32, tile_y: i32, world: &World) -> Self {
        let (sx, sy) = world.tile_to_screen(tile_x, tile_y);
        let dim = (GRANDEZZA_TILES * 0.75).round(); // 54.0
        let bottom_center_x = sx + GRANDEZZA_TILES / 2.0;
        let bottom_center_y = sy + GRANDEZZA_TILES / 2.0;
        let x = bottom_center_x - dim / 2.0;
        let y = bottom_center_y - dim;
        Self {
            tile_x,
            tile_y,
            frame_index: 0,
            frame_counter: 0,
            x,
            y,
            w: dim,
            h: dim,
            bottom: bottom_center_y,
        }
    }

    pub fn profondita(&self) -> f32 {
        self.bottom
    }

    pub fn update(&mut self) {
        self.frame_counter += 1;
        if self.frame_counter >= 10 {
            self.frame_counter = 0;
            self.frame_index = (self.frame_index + 1) % 6;
        }
    }

    pub fn draw(&self, assets: &Assets, attivo: bool) {
        let alpha = if attivo { 1.0 } else { 110.0 / 255.0 };
        let tint = Color::new(1.0, 1.0, 1.0, alpha);
        if let Some(tex) = assets.portal_frames.get(self.frame_index) {
            draw_texture_ex(
                tex,
                self.x,
                self.y,
                tint,
                DrawTextureParams {
                    dest_size: Some(vec2(self.w, self.h)),
                    ..Default::default()
                },
            );
        }
    }
}
