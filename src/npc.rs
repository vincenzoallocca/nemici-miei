use crate::assets::Assets;
use crate::hud::{misura_testo, parametri_testo};
use crate::settings::SCALA_PIXEL;
use crate::world::World;
use macroquad::prelude::*;

/// Ugo, il mago: sta fermo su un tile e blocca il passaggio.
pub struct Npc {
    pub tile_x: i32,
    pub tile_y: i32,
    pub parlando: bool,
    /// Punto dove poggiano i piedi.
    pub x: f32,
    pub y: f32,
}

impl Npc {
    pub fn new(tile_x: i32, tile_y: i32, world: &mut World) -> Self {
        world.blocca(tile_x, tile_y);
        let (x, y) = world.piedi_tile(tile_x, tile_y);
        Self {
            tile_x,
            tile_y,
            parlando: false,
            x: x.round(),
            y: y.round(),
        }
    }

    pub fn profondita(&self) -> f32 {
        self.y
    }

    fn rect(&self, assets: &Assets) -> Rect {
        let w = assets.npc_image.width() * SCALA_PIXEL;
        let h = assets.npc_image.height() * SCALA_PIXEL;
        // le ultime 3 righe dello sprite sono vuote
        Rect::new(self.x - (w / 2.0).round(), self.y + 3.0 * SCALA_PIXEL - h, w, h)
    }

    pub fn draw(&self, assets: &Assets) {
        let r = self.rect(assets);
        draw_ellipse(self.x, self.y, 14.0, 5.0, 0.0, Color::new(0.0, 0.0, 0.0, 70.0 / 255.0));
        draw_texture_ex(
            &assets.npc_image,
            r.x,
            r.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(r.w, r.h)),
                ..Default::default()
            },
        );
    }

    /// Il "!" giallo sopra la testa.
    pub fn draw_avviso(&self, assets: &Assets) {
        let r = self.rect(assets);
        let center_x = r.x + r.w / 2.0;
        let bob = (get_time() as f32 * 5.0).sin() * 3.0;
        let center_y = (r.y + 2.0 + bob).round();

        draw_circle(center_x, center_y, 13.0, BLACK);
        draw_circle(center_x, center_y, 11.0, Color::from_rgba(250, 210, 60, 255));

        let font = &assets.arcade_font;
        let dims = misura_testo("!", font, 22);
        draw_text_ex(
            "!",
            (center_x - dims.width / 2.0).round(),
            (center_y + dims.height / 2.0).round(),
            parametri_testo(font, 22, BLACK),
        );
    }
}
