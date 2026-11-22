use crate::assets::Assets;
use crate::world::World;
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TipoOggetto {
    Moneta,
    /// Cura 25 punti. Si raccoglie solo se si è feriti: si può tenere per dopo.
    Cuore,
}

/// Monete e cuori sparsi sulla mappa.
pub struct Raccoglibile {
    pub tipo: TipoOggetto,
    pub tile_x: i32,
    pub tile_y: i32,
    /// Punto a terra sotto l'oggetto.
    pub x: f32,
    pub y: f32,
    pub fase: f32,
    pub preso: bool,
}

impl Raccoglibile {
    pub fn new(tipo: TipoOggetto, tile_x: i32, tile_y: i32, world: &World) -> Self {
        let (x, y) = world.piedi_tile(tile_x, tile_y);
        Self {
            tipo,
            tile_x,
            tile_y,
            x: x.round(),
            y: y.round(),
            fase: (tile_x * 7 + tile_y * 3) as f32,
            preso: false,
        }
    }

    pub fn tile(&self) -> (i32, i32) {
        (self.tile_x, self.tile_y)
    }

    pub fn profondita(&self) -> f32 {
        self.y
    }

    pub fn update(&mut self) {
        self.fase += 0.06;
    }

    pub fn draw(&self, assets: &Assets) {
        if self.preso {
            return;
        }
        let (tex, scala) = match self.tipo {
            TipoOggetto::Moneta => (&assets.icon_moneta, 3.0),
            TipoOggetto::Cuore => (&assets.icon_vita, 2.0),
        };
        let w = tex.width() * scala;
        let h = tex.height() * scala;
        let alzo = 12.0 + (self.fase.sin() * 4.0).round();

        // l'ombra si stringe quando l'oggetto sale
        let ombra = 10.0 - self.fase.sin() * 2.0;
        draw_ellipse(self.x, self.y, ombra, ombra / 3.0, 0.0, Color::new(0.0, 0.0, 0.0, 0.3));
        draw_texture_ex(
            tex,
            self.x - (w / 2.0).round(),
            self.y - alzo - h,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(w, h)),
                ..Default::default()
            },
        );
    }
}
