use crate::assets::Assets;
use crate::livelli::{DefLivello, LIVELLI};
use crate::settings::*;
use macroquad::prelude::*;
use std::collections::VecDeque;

/// Un tile (o una decorazione) da disegnare, con la posizione già calcolata.
pub struct TileItem {
    pub tile_id: u8,
    pub x: f32,
    pub y: f32,
}

/// Fontanella, rocce e cespugli: vanno ordinati per profondità insieme ai personaggi.
pub struct OggettoAlto {
    pub tile_id: u8,
    pub tile_x: i32,
    pub tile_y: i32,
    pub x: f32,
    pub y: f32,
    pub profondita: f32,
}

pub struct World {
    /// Numero del livello, da 1.
    pub livello_id: u32,
    pub num_righe: usize,
    pub num_colonne: usize,
    pub offset_x: f32,
    pub offset_y: f32,
    griglia: Vec<u8>,
    bloccati: Vec<bool>,
    pub tiles: Vec<TileItem>,
    pub decorazioni: Vec<TileItem>,
    pub oggetti: Vec<OggettoAlto>,
}

/// Distanza "a scacchiera": quanti passi servono muovendosi anche in diagonale.
pub fn distanza(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

impl World {
    /// Carica il livello `livello_id` (da 1) di [`LIVELLI`].
    pub fn new(livello_id: u32) -> Self {
        let indice = (livello_id.max(1) as usize - 1).min(LIVELLI.len() - 1);
        Self::da_definizione(livello_id, &LIVELLI[indice])
    }

    pub fn da_definizione(livello_id: u32, def: &DefLivello) -> Self {
        let num_righe = def.mappa.len();
        let num_colonne = def.mappa.iter().map(|r| r.len()).max().unwrap_or(0);
        let mut griglia = vec![0u8; num_righe * num_colonne];
        for (r, riga) in def.mappa.iter().enumerate() {
            for (c, &t) in riga.iter().enumerate() {
                griglia[r * num_colonne + c] = t;
            }
        }

        let mut world = Self {
            livello_id,
            num_righe,
            num_colonne,
            offset_x: 0.0,
            offset_y: 0.0,
            bloccati: vec![false; griglia.len()],
            griglia,
            tiles: Vec::with_capacity(num_righe * num_colonne),
            decorazioni: Vec::new(),
            oggetti: Vec::new(),
        };

        // niente decorazioni dove stanno il giocatore, Ugo, il portale e il boss
        let liberi_da_deco: Vec<(i32, i32)> = [Some(def.spawn), def.npc, def.portale, def.boss]
            .into_iter()
            .flatten()
            .collect();
        world.carica_mondo(&liberi_da_deco);
        world
    }

    fn indice(&self, tx: i32, ty: i32) -> usize {
        ty as usize * self.num_colonne + tx as usize
    }

    pub fn calcola_offset(&mut self) {
        let altezza_eff = ALTEZZA - HUD_HEIGHT;
        let mezzo = GRANDEZZA_TILES / 2.0;
        let quarto = GRANDEZZA_TILES / 4.0;
        let c = self.num_colonne as f32 - 1.0;
        let r = self.num_righe as f32 - 1.0;

        // angoli della mappa in coordinate isometriche: (0,0), (c,0), (0,r), (c,r)
        let min_x = -r * mezzo;
        let max_x = c * mezzo;
        let min_y = 0.0;
        let max_y = (c + r) * quarto;

        let centro_x = (min_x + max_x) / 2.0;
        let centro_y = (min_y + max_y) / 2.0;

        // offset interi: i tile cadono su pixel esatti, niente righe tra un blocco e l'altro
        self.offset_x = (LUNGHEZZA / 2.0 - centro_x - mezzo).round();
        self.offset_y = (altezza_eff / 2.0 - centro_y - quarto).round();
    }

    fn carica_mondo(&mut self, liberi_da_deco: &[(i32, i32)]) {
        self.calcola_offset();
        self.tiles.clear();
        self.decorazioni.clear();
        self.oggetti.clear();

        // (id tile, probabilità su 1000): fiori, tronchi, tulipani, ciottoli
        const DECORAZIONI: &[(u8, u32)] = &[(6, 100), (17, 50), (21, 30), (15, 20)];

        for r in 0..self.num_righe as i32 {
            for c in 0..self.num_colonne as i32 {
                let tile_val = self.griglia[self.indice(c, r)];
                if tile_val == 0 {
                    continue;
                }
                let (screen_x, screen_y) = self.tile_to_screen(c, r);

                if is_tile_acqua(tile_val) {
                    // la lastra d'acqua è sottile: sotto c'è un blocco di terra, così l'isola
                    // non ha buchi
                    self.tiles.push(TileItem {
                        tile_id: 2,
                        x: screen_x,
                        y: screen_y + 8.0 * SCALA_PIXEL,
                    });
                }

                self.tiles.push(TileItem {
                    tile_id: terreno_sotto(tile_val),
                    x: screen_x,
                    y: screen_y,
                });

                if is_oggetto_alto(tile_val) {
                    self.oggetti.push(OggettoAlto {
                        tile_id: tile_val,
                        tile_x: c,
                        tile_y: r,
                        x: screen_x,
                        y: screen_y - ALZO_OGGETTI,
                        profondita: screen_y + OFFSET_PIEDI,
                    });
                    continue;
                }

                if matches!(tile_val, 3..=5) && !liberi_da_deco.contains(&(c, r)) {
                    // pseudo-casuale ma sempre uguale per lo stesso livello
                    let seme = (c as u32).wrapping_mul(73_856_093)
                        ^ (r as u32).wrapping_mul(19_349_663)
                        ^ self.livello_id.wrapping_mul(83_492_791);
                    let tiro = seme % 1000;
                    let mut cumulativo = 0;
                    for &(deco_id, prob) in DECORAZIONI {
                        cumulativo += prob;
                        if tiro < cumulativo {
                            self.decorazioni.push(TileItem {
                                tile_id: deco_id,
                                x: screen_x,
                                y: screen_y - ALZO_OGGETTI,
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

    /// Punto dello schermo dove poggiano i piedi di chi sta sul tile.
    pub fn piedi_tile(&self, tx: i32, ty: i32) -> (f32, f32) {
        let (sx, sy) = self.tile_to_screen(tx, ty);
        let affondo = match self.tipo_tile(tx, ty) {
            Some(t) if is_tile_acqua(t) => ABBASSAMENTO_ACQUA,
            _ => 0.0,
        };
        (sx + GRANDEZZA_TILES / 2.0, sy + OFFSET_PIEDI + affondo)
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
            Some(self.griglia[self.indice(tx, ty)]).filter(|&t| t != 0)
        } else {
            None
        }
    }

    /// Segna un tile come occupato da qualcosa di solido (es. Ugo).
    pub fn blocca(&mut self, tx: i32, ty: i32) {
        if self.dentro_mappa(tx, ty) {
            let i = self.indice(tx, ty);
            self.bloccati[i] = true;
        }
    }

    pub fn is_tile_walkable(&self, tx: i32, ty: i32) -> bool {
        match self.tipo_tile(tx, ty) {
            Some(t) => is_tile_id_walkable(t) && !self.bloccati[self.indice(tx, ty)],
            None => false,
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
        self.oggetti
            .iter()
            .find(|o| o.tile_id == TILE_FONTANELLA)
            .map(|o| (o.tile_x, o.tile_y))
    }

    /// Primo passo (solo in orizzontale o verticale sulla griglia) per avvicinarsi a `verso`
    /// fino a distanza `raggio`, girando intorno a ostacoli e tile occupati (ricerca in ampiezza).
    /// `None` se si è già arrivati o se non c'è strada.
    pub fn passo_verso(
        &self,
        da: (i32, i32),
        verso: (i32, i32),
        raggio: i32,
        occupati: &[(i32, i32)],
    ) -> Option<(i32, i32)> {
        if distanza(da, verso) <= raggio || !self.dentro_mappa(da.0, da.1) {
            return None;
        }
        let arrivato = |p: (i32, i32)| distanza(p, verso) <= raggio && (raggio == 0 || p != verso);

        // prova prima le direzioni che avvicinano di più: il percorso resta "dritto"
        let mut direzioni = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        let (vx, vy) = (verso.0 - da.0, verso.1 - da.1);
        direzioni.sort_by_key(|&(dx, dy)| -(dx * vx + dy * vy));

        const NESSUNO: usize = usize::MAX;
        let mut precedente = vec![NESSUNO; self.griglia.len()];
        let partenza = self.indice(da.0, da.1);
        precedente[partenza] = partenza;
        let mut coda = VecDeque::with_capacity(32);
        coda.push_back(da);

        while let Some(cur) = coda.pop_front() {
            for &(dx, dy) in &direzioni {
                let n = (cur.0 + dx, cur.1 + dy);
                if !self.is_tile_walkable(n.0, n.1) || occupati.contains(&n) {
                    continue;
                }
                let i = self.indice(n.0, n.1);
                if precedente[i] != NESSUNO {
                    continue;
                }
                precedente[i] = self.indice(cur.0, cur.1);
                if arrivato(n) {
                    // risale il percorso fino al primo passo
                    let mut passo = i;
                    while precedente[passo] != partenza {
                        passo = precedente[passo];
                    }
                    let w = self.num_colonne;
                    return Some(((passo % w) as i32, (passo / w) as i32));
                }
                coda.push_back(n);
            }
        }
        None
    }

    /// Il tile calpestabile più lontano da (px, py): dove scappa il boss per riprendersi.
    pub fn punto_lontano(&self, px: i32, py: i32) -> (i32, i32) {
        self.tiles_liberi(|_, _| true)
            .into_iter()
            .max_by_key(|&(x, y)| ((x - px).abs() + (y - py).abs(), x, y))
            .unwrap_or((px, py))
    }

    pub fn draw_terreno(&self, assets: &Assets) {
        let dim = Some(vec2(GRANDEZZA_TILES, GRANDEZZA_TILES));
        for t in self.tiles.iter().chain(self.decorazioni.iter()) {
            if let Some(tex) = assets.tile(t.tile_id) {
                draw_texture_ex(
                    tex,
                    t.x,
                    t.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: dim,
                        ..Default::default()
                    },
                );
            }
        }
    }

    pub fn draw_oggetto(&self, oggetto: &OggettoAlto, assets: &Assets) {
        if let Some(tex) = assets.tile(oggetto.tile_id) {
            draw_texture_ex(
                tex,
                oggetto.x,
                oggetto.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(GRANDEZZA_TILES, GRANDEZZA_TILES)),
                    ..Default::default()
                },
            );
        }
    }
}
