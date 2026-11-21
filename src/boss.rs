use crate::assets::Assets;
use crate::effetti::Effetti;
use crate::entita::Entita;
use crate::player::Giocatore;
use crate::settings::{SCALA_BOSS, SCALA_BOSS_FURIA};
use crate::world::World;
use macroquad::prelude::*;
use std::collections::HashSet;

pub const CARICA_NORMALE: i32 = 35;
pub const CARICA_FURIA: i32 = 22;
pub const DURATA_RITIRATA: i32 = 180;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BossStato {
    Insegui,
    Carica,
    Ritirata,
}

pub struct Boss {
    pub entita: Entita,
    pub damage: i32,
    pub hp: f32,
    pub hp_max: f32,
    pub soglia_furia: f32,
    pub furia: bool,
    pub stato: BossStato,
    pub timer: i32,
    pub rigenerazione: f32,
}

impl Boss {
    pub fn new(tile_x: i32, tile_y: i32, world: &World) -> Self {
        Self {
            entita: Entita::new(tile_x, tile_y, world, 3.0),
            damage: 10,
            hp: 150.0,
            hp_max: 150.0,
            soglia_furia: 75.0,
            furia: false,
            stato: BossStato::Insegui,
            timer: 0,
            rigenerazione: 0.05,
        }
    }

    pub fn vivo(&self) -> bool {
        self.hp > 0.0
    }

    pub fn update(
        &mut self,
        world: &World,
        player: &mut Giocatore,
        assets: &Assets,
        effetti: &mut Effetti,
        occupati: &HashSet<(i32, i32)>,
    ) {
        if !self.vivo() {
            return;
        }

        if !self.furia && self.hp <= self.soglia_furia {
            self.entra_in_furia(effetti);
        }

        if self.entita.in_movimento() {
            self.entita.aggiorna_movimento();
        }

        match self.stato {
            BossStato::Ritirata => {
                self.timer += 1;
                let limite = if self.furia {
                    self.soglia_furia
                } else {
                    self.hp_max
                };
                self.hp = (self.hp + self.rigenerazione).min(limite);
                if !self.entita.in_movimento() {
                    let angolo = self.angolo_lontano(player.entita.tile_x, player.entita.tile_y, world);
                    self.entita.insegui(angolo.0, angolo.1, world, occupati);
                }
                if self.timer >= DURATA_RITIRATA {
                    self.stato = BossStato::Insegui;
                }
            }
            BossStato::Carica => {
                self.timer += 1;
                let limite_carica = if self.furia {
                    CARICA_FURIA
                } else {
                    CARICA_NORMALE
                };
                if self.timer >= limite_carica {
                    if self.entita.distanza_tiles(player.entita.tile_x, player.entita.tile_y) <= 1 {
                        player.take_damage(self.damage, assets, effetti);
                    }
                    self.stato = BossStato::Ritirata;
                    self.timer = 0;
                }
            }
            BossStato::Insegui => {
                if !self.entita.in_movimento() {
                    if self.entita.distanza_tiles(player.entita.tile_x, player.entita.tile_y) <= 1 {
                        self.stato = BossStato::Carica;
                        self.timer = 0;
                        self.entita.direction = self.entita.direzione_verso(
                            player.entita.tile_x - self.entita.tile_x,
                            player.entita.tile_y - self.entita.tile_y,
                        );
                    } else {
                        self.entita.insegui(player.entita.tile_x, player.entita.tile_y, world, occupati);
                    }
                }
            }
        }
    }

    pub fn entra_in_furia(&mut self, effetti: &mut Effetti) {
        self.furia = true;
        self.entita.velocita = 4.5;
        self.damage = (self.damage as f32 * 1.5) as i32;
        effetti.add_text("FURIA!", self.entita.x, self.entita.y - 120.0, Color::from_rgba(255, 60, 60, 255));
    }

    pub fn angolo_lontano(&self, px: i32, py: i32, world: &World) -> (i32, i32) {
        let max_c = world.num_colonne as i32 - 1;
        let max_r = world.num_righe as i32 - 1;
        let angoli = [(0, 0), (max_c, 0), (0, max_r), (max_c, max_r)];
        *angoli
            .iter()
            .max_by_key(|a| (a.0 - px).abs() + (a.1 - py).abs())
            .unwrap()
    }

    pub fn take_damage(&mut self, amount: i32, assets: &Assets, effetti: &mut Effetti) {
        if !self.vivo() {
            return;
        }
        self.hp = (self.hp - amount as f32).max(0.0);
        self.entita.flash_timer = 8;
        effetti.add_text(
            &format!("-{}", amount),
            self.entita.x,
            self.entita.y - 100.0,
            Color::from_rgba(255, 230, 120, 255),
        );
        if !self.vivo() {
            assets.play_sound(&assets.snd_morte, 0.5);
        }
    }

    pub fn draw(&mut self, assets: &Assets) {
        if !self.vivo() {
            return;
        }

        let ombra_w = if self.furia { 48.0 } else { 38.0 };
        self.entita.disegna_ombra(ombra_w);

        let scale = if self.furia {
            SCALA_BOSS_FURIA
        } else {
            SCALA_BOSS
        };
        let tint = if self.furia {
            Some(Color::from_rgba(255, 110, 110, 255))
        } else {
            None
        };

        let frames = assets.boss_frames.get(self.entita.direction);

        if self.stato == BossStato::Carica {
            let scossa = if (self.timer / 3) % 2 == 0 { 2.0 } else { -2.0 };
            self.entita.x += scossa;
            let tex = &frames[0];
            let rect = self.entita.disegna_sprite(tex, 0.0, scale, tint);
            self.entita.x -= scossa;

            // Red warning indicator above head
            draw_circle(rect.x + rect.w / 2.0, rect.y - 12.0, 6.0, Color::from_rgba(255, 60, 60, 255));
        } else {
            let tex = self.entita.anima(frames, 10);
            self.entita.disegna_sprite(tex, 0.0, scale, tint);
        }
    }
}
