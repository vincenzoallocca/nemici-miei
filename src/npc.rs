use crate::assets::Assets;
use crate::settings::SCALA_NPC;
use crate::world::World;
use macroquad::prelude::*;

pub struct Npc {
    pub tile_x: i32,
    pub tile_y: i32,
    pub parlando: bool,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Npc {
    pub fn new(tile_x: i32, tile_y: i32, world: &mut World, assets: &Assets) -> Self {
        world.bloccati.insert((tile_x, tile_y));
        let (px, py) = world.piedi_tile(tile_x, tile_y);
        let w = assets.npc_image.width() * SCALA_NPC;
        let h = assets.npc_image.height() * SCALA_NPC;
        let x = px.round() - w / 2.0;
        let y = py.round() + 2.0 - h;
        Self {
            tile_x,
            tile_y,
            parlando: false,
            x,
            y,
            w,
            h,
        }
    }

    pub fn profondita(&self) -> f32 {
        self.y + self.h
    }

    pub fn draw(&self, assets: &Assets) {
        draw_texture_ex(
            &assets.npc_image,
            self.x,
            self.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(self.w, self.h)),
                ..Default::default()
            },
        );
    }

    pub fn draw_avviso(&self, assets: &Assets) {
        let center_x = self.x + self.w / 2.0;
        let bob = (get_time() as f32 * 5.0).sin() * 3.0;
        let center_y = self.y - 16.0 + bob;

        draw_circle(center_x, center_y, 13.0, BLACK);
        draw_circle(center_x, center_y, 11.0, Color::from_rgba(250, 210, 60, 255));

        let font_size = 22;
        let text = "!";
        let dims = measure_text(text, Some(&assets.arcade_font), font_size, 1.0);
        draw_text_ex(
            text,
            center_x - dims.width / 2.0,
            center_y + dims.height / 2.0,
            TextParams {
                font: Some(&assets.arcade_font),
                font_size,
                color: BLACK,
                ..Default::default()
            },
        );
    }
}
