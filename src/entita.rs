use crate::assets::{Assets, Direction};
use crate::world::{distanza, World};
use macroquad::prelude::*;

/// Posizione, movimento da un tile all'altro e animazione: comuni a giocatore e nemici.
#[derive(Clone, Debug)]
pub struct Entita {
    pub tile_x: i32,
    pub tile_y: i32,
    pub x: f32,
    pub y: f32,
    pub dest_x: f32,
    pub dest_y: f32,
    pub velocita: f32,
    pub direction: Direction,
    pub frame_index: usize,
    pub frame_counter: i32,
    /// Passi per cui lo sprite resta bianco dopo un colpo.
    pub flash_timer: i32,
}

impl Entita {
    pub fn new(tile_x: i32, tile_y: i32, world: &World, velocita: f32) -> Self {
        let (px, py) = world.piedi_tile(tile_x, tile_y);
        Self {
            tile_x,
            tile_y,
            x: px,
            y: py,
            dest_x: px,
            dest_y: py,
            velocita,
            direction: Direction::Down,
            frame_index: 0,
            frame_counter: 0,
            flash_timer: 0,
        }
    }

    pub fn piazza(&mut self, tile_x: i32, tile_y: i32, world: &World) {
        self.tile_x = tile_x;
        self.tile_y = tile_y;
        let (px, py) = world.piedi_tile(tile_x, tile_y);
        self.x = px;
        self.y = py;
        self.dest_x = px;
        self.dest_y = py;
    }

    pub fn tile(&self) -> (i32, i32) {
        (self.tile_x, self.tile_y)
    }

    pub fn in_movimento(&self) -> bool {
        (self.x - self.dest_x).abs() > 0.001 || (self.y - self.dest_y).abs() > 0.001
    }

    pub fn profondita(&self) -> f32 {
        self.y
    }

    pub fn distanza_tiles(&self, other_tx: i32, other_ty: i32) -> i32 {
        distanza(self.tile(), (other_tx, other_ty))
    }

    pub fn direzione_verso(&self, dx: i32, dy: i32) -> Direction {
        if dx.abs() > dy.abs() {
            if dx > 0 {
                Direction::Right
            } else {
                Direction::Left
            }
        } else if dy > 0 {
            Direction::Down
        } else {
            Direction::Up
        }
    }

    pub fn muovi_verso(&mut self, tx: i32, ty: i32, world: &World, occupati: &[(i32, i32)]) -> bool {
        if !world.is_tile_walkable(tx, ty) || occupati.contains(&(tx, ty)) {
            return false;
        }
        self.tile_x = tx;
        self.tile_y = ty;
        let (dest_x, dest_y) = world.piedi_tile(tx, ty);
        self.dest_x = dest_x;
        self.dest_y = dest_y;
        true
    }

    pub fn aggiorna_movimento(&mut self) {
        let dx = self.dest_x - self.x;
        let dy = self.dest_y - self.y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist <= self.velocita {
            self.x = self.dest_x;
            self.y = self.dest_y;
        } else {
            self.x += dx / dist * self.velocita;
            self.y += dy / dist * self.velocita;
        }
    }

    /// Fa un passo per avvicinarsi a `target` fino a distanza `raggio`, aggirando gli ostacoli.
    pub fn vai_verso(
        &mut self,
        target: (i32, i32),
        raggio: i32,
        world: &World,
        occupati: &[(i32, i32)],
    ) -> bool {
        let passo = world
            .passo_verso(self.tile(), target, raggio, occupati)
            .or_else(|| self.passo_diretto(target, world, occupati));
        match passo {
            Some((tx, ty)) => {
                let (dx, dy) = (tx - self.tile_x, ty - self.tile_y);
                self.muovi_verso(tx, ty, world, occupati);
                self.direction = self.direzione_verso(dx, dy);
                true
            }
            None => false,
        }
    }

    /// Insegue un bersaglio fermandosi accanto a lui.
    pub fn insegui(&mut self, target_x: i32, target_y: i32, world: &World, occupati: &[(i32, i32)]) -> bool {
        self.vai_verso((target_x, target_y), 1, world, occupati)
    }

    /// Se non c'è un percorso libero (es. tutti i tile intorno occupati) prova almeno ad
    /// avvicinarsi in linea retta.
    fn passo_diretto(&self, target: (i32, i32), world: &World, occupati: &[(i32, i32)]) -> Option<(i32, i32)> {
        let dx = target.0 - self.tile_x;
        let dy = target.1 - self.tile_y;
        let passo_x = (dx != 0).then(|| (self.tile_x + dx.signum(), self.tile_y));
        let passo_y = (dy != 0).then(|| (self.tile_x, self.tile_y + dy.signum()));
        let tentativi = if dx.abs() > dy.abs() {
            [passo_x, passo_y]
        } else {
            [passo_y, passo_x]
        };
        tentativi
            .into_iter()
            .flatten()
            .find(|&(x, y)| (x, y) != target && world.is_tile_walkable(x, y) && !occupati.contains(&(x, y)))
    }

    /// Avanza l'animazione di un passo (chiamato dalla logica, non dal disegno).
    pub fn avanza_animazione(&mut self, num_frames: usize, velocita_anim: i32) {
        self.frame_counter += 1;
        if self.frame_counter >= velocita_anim {
            self.frame_counter = 0;
            self.frame_index = (self.frame_index + 1) % num_frames.max(1);
        }
        if self.frame_index >= num_frames {
            self.frame_index = 0;
        }
        if self.flash_timer > 0 {
            self.flash_timer -= 1;
        }
    }

    pub fn frame<'a>(&self, frames: &'a [Texture2D]) -> &'a Texture2D {
        &frames[self.frame_index.min(frames.len() - 1)]
    }

    pub fn disegna_ombra(&self, larghezza: f32) {
        let altezza = (larghezza / 3.0).max(4.0);
        let color = Color::new(0.0, 0.0, 0.0, 70.0 / 255.0);
        draw_ellipse(self.x.round(), self.y.round(), larghezza / 2.0, altezza / 2.0, 0.0, color);
    }

    /// Disegna lo sprite con i piedi sul punto (x, y), alzato di `alzo` pixel.
    pub fn disegna_sprite(
        &self,
        tex: &Texture2D,
        alzo: f32,
        scale: f32,
        tint: Option<Color>,
        spostamento_x: f32,
        assets: &Assets,
    ) -> Rect {
        let w = tex.width() * scale;
        let h = tex.height() * scale;
        let draw_x = (self.x + spostamento_x).round() - (w / 2.0).round();
        let draw_y = (self.y - alzo).round() - h;
        let params = DrawTextureParams {
            dest_size: Some(vec2(w, h)),
            ..Default::default()
        };

        if self.flash_timer > 0 {
            // colpito: sagoma bianca
            gl_use_material(&assets.materiale_flash);
            draw_texture_ex(tex, draw_x, draw_y, WHITE, params);
            gl_use_default_material();
        } else {
            draw_texture_ex(tex, draw_x, draw_y, tint.unwrap_or(WHITE), params);
        }

        Rect::new(draw_x, draw_y, w, h)
    }
}
