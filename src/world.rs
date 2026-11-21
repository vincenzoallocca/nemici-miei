use crate::assets::Assets;
use crate::settings::*;
use macroquad::prelude::*;
use std::collections::HashSet;

pub struct TileItem {
    pub tile_id: u8,
    pub x: f32,
    pub y: f32,
}

pub struct DecoItem {
    pub tile_id: u8,
    pub x: f32,
    pub y: f32,
}

pub struct FountainItem {
    pub x: f32,
    pub y: f32,
    pub bottom: f32,
}

pub struct World {
    pub livello_id: u32,
    pub num_righe: usize,
    pub num_colonne: usize,
    pub offset_x: f32,
    pub offset_y: f32,
    pub bloccati: HashSet<(i32, i32)>,
    pub tiles: Vec<TileItem>,
    pub decorazioni: Vec<DecoItem>,
    pub fountain: Option<FountainItem>,
}

impl World {
    pub fn new(livello_id: u32) -> Self {
        let mut world = Self {
            livello_id,
            num_righe: 10,
            num_colonne: 10,
            offset_x: 0.0,
            offset_y: 0.0,
            bloccati: HashSet::new(),
            tiles: Vec::with_capacity(100),
            decorazioni: Vec::new(),
            fountain: None,
        };
        world.carica_mondo();
        world
    }

    pub fn calcola_offset(&mut self) {
        let altezza_eff = ALTEZZA - HUD_HEIGHT;
        let tile_w = GRANDEZZA_TILES;

        let angoli = [
            (0.0, 0.0),
            ((self.num_colonne - 1) as f32 * (tile_w / 2.0), (self.num_colonne - 1) as f32 * (tile_w / 4.0)),
            (-((self.num_righe - 1) as f32) * (tile_w / 2.0), (self.num_righe - 1) as f32 * (tile_w / 4.0)),
            (
                ((self.num_colonne - 1) as f32 - (self.num_righe - 1) as f32) * (tile_w / 2.0),
                ((self.num_colonne - 1) as f32 + (self.num_righe - 1) as f32) * (tile_w / 4.0),
            ),
        ];

        let min_x = angoli.iter().map(|p| p.0).fold(f32::INFINITY, f32::min);
        let max_x = angoli.iter().map(|p| p.0).fold(f32::NEG_INFINITY, f32::max);
        let min_y = angoli.iter().map(|p| p.1).fold(f32::INFINITY, f32::min);
        let max_y = angoli.iter().map(|p| p.1).fold(f32::NEG_INFINITY, f32::max);

        let centro_x = (min_x + max_x) / 2.0;
        let centro_y = (min_y + max_y) / 2.0;

        self.offset_x = LUNGHEZZA / 2.0 - centro_x - tile_w / 2.0;
        self.offset_y = altezza_eff / 2.0 - centro_y - tile_w / 4.0;
    }

    pub fn carica_mondo(&mut self) {
        self.calcola_offset();
        self.bloccati.clear();
        self.tiles.clear();
        self.decorazioni.clear();
        self.fountain = None;

        let matrice = if self.livello_id == 1 {
            &LIVELLO_1
        } else {
            &LIVELLO_2
        };

        let decorazioni_casuali: &[(u8, f32)] = &[
            (6, 0.10),
            (17, 0.05),
            (21, 0.03),
            (15, 0.02),
        ];

        for (r, riga) in matrice.iter().enumerate() {
            for (c, &tile_val) in riga.iter().enumerate() {
                let (screen_x, mut screen_y) = self.tile_to_screen(c as i32, r as i32);

                if is_tile_acqua(tile_val) {
                    screen_y -= 10.0;
                }

                if tile_val == 18 {
                    // Terreno sotto fontanella (tile 3)
                    self.tiles.push(TileItem {
                        tile_id: 3,
                        x: screen_x,
                        y: screen_y,
                    });

                    let bottom_center_x = screen_x + GRANDEZZA_TILES / 2.0;
                    let bottom_center_y = screen_y + GRANDEZZA_TILES / 2.0;
                    self.fountain = Some(FountainItem {
                        x: bottom_center_x - 50.0,
                        y: bottom_center_y - 100.0,
                        bottom: bottom_center_y,
                    });
                    continue;
                }

                self.tiles.push(TileItem {
                    tile_id: tile_val,
                    x: screen_x,
                    y: screen_y,
                });

                if is_tile_erba(tile_val) {
                    let seed = ((c * 17 + r * 31 + 42) % 1000) as f32 / 1000.0;
                    let mut cumulative = 0.0;
                    for &(deco_id, prob) in decorazioni_casuali {
                        cumulative += prob;
                        if seed < cumulative {
                            let center_x = screen_x + GRANDEZZA_TILES / 2.0;
                            let center_y = screen_y + GRANDEZZA_TILES / 2.0;
                            let deco_size = GRANDEZZA_TILES / 2.0;
                            self.decorazioni.push(DecoItem {
                                tile_id: deco_id,
                                x: center_x - deco_size / 2.0,
                                y: center_y - deco_size / 2.0,
                            });
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn tile_to_screen(&self, tx: i32, ty: i32) -> (f32, f32) {
        let screen_x = (tx - ty) as f32 * (GRANDEZZA_TILES / 2.0) + self.offset_x;
        let screen_y = (tx + ty) as f32 * (GRANDEZZA_TILES / 4.0) + self.offset_y;
        (screen_x, screen_y)
    }

    pub fn piedi_tile(&self, tx: i32, ty: i32) -> (f32, f32) {
        let (sx, sy) = self.tile_to_screen(tx, ty);
        (sx + GRANDEZZA_TILES / 2.0, sy + OFFSET_PIEDI)
    }

    pub fn screen_to_tile(&self, sx: f32, sy: f32) -> (i32, i32) {
        let a = (sx - self.offset_x - GRANDEZZA_TILES / 2.0) / (GRANDEZZA_TILES / 2.0);
        let b = (sy - self.offset_y - OFFSET_PIEDI) / (GRANDEZZA_TILES / 4.0);
        (((a + b) / 2.0).round() as i32, ((b - a) / 2.0).round() as i32)
    }

    pub fn dentro_mappa(&self, tx: i32, ty: i32) -> bool {
        tx >= 0 && tx < self.num_colonne as i32 && ty >= 0 && ty < self.num_righe as i32
    }

    pub fn tipo_tile(&self, tx: i32, ty: i32) -> Option<u8> {
        if self.dentro_mappa(tx, ty) {
            let matrice = if self.livello_id == 1 {
                &LIVELLO_1
            } else {
                &LIVELLO_2
            };
            Some(matrice[ty as usize][tx as usize])
        } else {
            None
        }
    }

    pub fn is_tile_walkable(&self, tx: i32, ty: i32) -> bool {
        if !self.dentro_mappa(tx, ty) || self.bloccati.contains(&(tx, ty)) {
            return false;
        }
        if let Some(t) = self.tipo_tile(tx, ty) {
            is_tile_id_walkable(t)
        } else {
            false
        }
    }

    pub fn tiles_liberi<F>(&self, condizione: F) -> Vec<(i32, i32)>
    where
        F: Fn(i32, i32) -> bool,
    {
        let mut liberi = Vec::new();
        for r in 0..self.num_righe as i32 {
            for c in 0..self.num_colonne as i32 {
                if self.is_tile_walkable(c, r) && condizione(c, r) {
                    liberi.push((c, r));
                }
            }
        }
        liberi
    }

    pub fn trova_fontanella(&self) -> Option<(i32, i32)> {
        for r in 0..self.num_righe as i32 {
            for c in 0..self.num_colonne as i32 {
                if self.tipo_tile(c, r) == Some(18) {
                    return Some((c, r));
                }
            }
        }
        None
    }

    pub fn nuovo_livello(&mut self, nuovo: u32) {
        self.livello_id = nuovo;
        self.carica_mondo();
    }

    pub fn draw_terreno(&self, assets: &Assets) {
        // Draw background
        let bg = if self.livello_id == 1 {
            &assets.bg1
        } else {
            &assets.bg2
        };
        draw_texture_ex(
            bg,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(LUNGHEZZA, ALTEZZA)),
                ..Default::default()
            },
        );

        // Draw tiles
        for t in &self.tiles {
            if let Some(tex) = assets.tile_textures.get(&t.tile_id) {
                draw_texture_ex(
                    tex,
                    t.x,
                    t.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(GRANDEZZA_TILES, GRANDEZZA_TILES)),
                        ..Default::default()
                    },
                );
            }
        }

        // Draw flat decorations
        let deco_size = GRANDEZZA_TILES / 2.0;
        for d in &self.decorazioni {
            if let Some(tex) = assets.tile_textures.get(&d.tile_id) {
                draw_texture_ex(
                    tex,
                    d.x,
                    d.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(deco_size, deco_size)),
                        ..Default::default()
                    },
                );
            }
        }
    }

    pub fn draw_fountain(&self, assets: &Assets) {
        if let Some(ref f) = self.fountain {
            if let Some(tex) = assets.tile_textures.get(&18) {
                draw_texture_ex(
                    tex,
                    f.x,
                    f.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(100.0, 100.0)),
                        ..Default::default()
                    },
                );
            }
        }
    }
}
