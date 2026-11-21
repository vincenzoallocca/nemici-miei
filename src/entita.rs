use crate::assets::Direction;
use crate::world::World;
use macroquad::prelude::*;
use std::collections::HashSet;

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

    pub fn in_movimento(&self) -> bool {
        (self.x - self.dest_x).abs() > 0.001 || (self.y - self.dest_y).abs() > 0.001
    }

    pub fn profondita(&self) -> f32 {
        self.y
    }

    pub fn distanza_tiles(&self, other_tx: i32, other_ty: i32) -> i32 {
        (self.tile_x - other_tx).abs().max((self.tile_y - other_ty).abs())
    }

    pub fn direzione_verso(&self, dx: i32, dy: i32) -> Direction {
        if dx.abs() > dy.abs() {
            if dx > 0 {
                Direction::Right
            } else {
                Direction::Left
            }
        } else {
            if dy > 0 {
                Direction::Down
            } else {
                Direction::Up
            }
        }
    }

    pub fn muovi_verso(&mut self, tx: i32, ty: i32, world: &World, occupati: &HashSet<(i32, i32)>) -> bool {
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

    pub fn insegui(&mut self, target_x: i32, target_y: i32, world: &World, occupati: &HashSet<(i32, i32)>) -> bool {
        let dx = target_x - self.tile_x;
        let dy = target_y - self.tile_y;
        if dx == 0 && dy == 0 {
            return false;
        }

        let passo_x = if dx != 0 {
            Some((if dx > 0 { 1 } else { -1 }, 0))
        } else {
            None
        };
        let passo_y = if dy != 0 {
            Some((0, if dy > 0 { 1 } else { -1 }))
        } else {
            None
        };

        let tentativi = if dx.abs() > dy.abs() {
            [passo_x, passo_y]
        } else {
            [passo_y, passo_x]
        };

        for (pdx, pdy) in tentativi.into_iter().flatten() {
            if self.muovi_verso(self.tile_x + pdx, self.tile_y + pdy, world, occupati) {
                self.direction = self.direzione_verso(pdx, pdy);
                return true;
            }
        }
        false
    }

    pub fn anima<'a>(&mut self, frames: &'a [Texture2D], velocita_anim: i32) -> &'a Texture2D {
        self.frame_counter += 1;
        if self.frame_counter >= velocita_anim {
            self.frame_counter = 0;
            self.frame_index = (self.frame_index + 1) % frames.len();
        }
        if self.frame_index >= frames.len() {
            self.frame_index = 0;
        }
        &frames[self.frame_index]
    }

    pub fn disegna_ombra(&self, larghezza: f32) {
        let altezza = (larghezza / 3.0).max(4.0);
        let color = Color::new(0.0, 0.0, 0.0, 70.0 / 255.0);
        draw_ellipse(self.x.round(), self.y.round(), larghezza / 2.0, altezza / 2.0, 0.0, color);
    }

    pub fn disegna_sprite(&mut self, tex: &Texture2D, alzo: f32, scale: f32, extra_tint: Option<Color>) -> Rect {
        let w = tex.width() * scale;
        let h = tex.height() * scale;
        let draw_x = self.x.round() - w / 2.0;
        let draw_y = self.y.round() - alzo - h;

        let mut tint = extra_tint.unwrap_or(WHITE);
        if self.flash_timer > 0 {
            self.flash_timer -= 1;
            tint = Color::new(1.8, 1.8, 1.8, 1.0);
        }

        draw_texture_ex(
            tex,
            draw_x,
            draw_y,
            tint,
            DrawTextureParams {
                dest_size: Some(vec2(w, h)),
                ..Default::default()
            },
        );

        Rect::new(draw_x, draw_y, w, h)
    }
}
