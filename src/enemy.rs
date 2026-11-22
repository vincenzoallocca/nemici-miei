use crate::assets::{Assets, Suono};
use crate::effetti::Effetti;
use crate::entita::Entita;
use crate::player::Giocatore;
use crate::settings::SCALA_PIXEL;
use crate::world::World;
use macroquad::prelude::*;

pub const VELOCITA_PIPISTRELLO: f32 = 1.8;
/// Passi tra un morso e l'altro.
pub const COOLDOWN_PIPISTRELLO: i32 = 150;
/// Il primo morso arriva un po' dopo che il pipistrello ti ha raggiunto: il tempo di reagire.
pub const PRIMO_MORSO: i32 = 42;

pub struct Enemy {
    pub entita: Entita,
    pub hp: i32,
    pub hp_max: i32,
    pub dmg: i32,
    pub cooldown: i32,
    pub timer_attacco: i32,
    pub vicino: bool,
    pub fase_volo: f32,
}

impl Enemy {
    pub fn new(tile_x: i32, tile_y: i32, world: &World) -> Self {
        Self::con_statistiche(tile_x, tile_y, world, 50, 10)
    }

    pub fn con_statistiche(tile_x: i32, tile_y: i32, world: &World, hp: i32, dmg: i32) -> Self {
        Self {
            entita: Entita::new(tile_x, tile_y, world, VELOCITA_PIPISTRELLO),
            hp,
            hp_max: hp,
            dmg,
            cooldown: COOLDOWN_PIPISTRELLO,
            timer_attacco: 0,
            vicino: false,
            fase_volo: (tile_x + tile_y) as f32,
        }
    }

    pub fn vivo(&self) -> bool {
        self.hp > 0
    }

    /// Solo animazione: usato anche durante la tregua, quando i pipistrelli stanno fermi.
    pub fn anima(&mut self) {
        self.fase_volo += 0.08;
        self.entita.avanza_animazione(3, 10);
    }

    pub fn update(&mut self, world: &World, player: &mut Giocatore, effetti: &mut Effetti, occupati: &[(i32, i32)]) {
        if !self.vivo() {
            return;
        }

        if self.entita.in_movimento() {
            self.entita.aggiorna_movimento();
        } else if self.entita.distanza_tiles(player.entita.tile_x, player.entita.tile_y) <= 1 {
            self.attacca(player, effetti);
        } else {
            self.vicino = false;
            self.entita
                .insegui(player.entita.tile_x, player.entita.tile_y, world, occupati);
        }
    }

    pub fn attacca(&mut self, player: &mut Giocatore, effetti: &mut Effetti) {
        if !self.vicino {
            self.vicino = true;
            self.timer_attacco = PRIMO_MORSO;
        }
        let dx = player.entita.tile_x - self.entita.tile_x;
        let dy = player.entita.tile_y - self.entita.tile_y;
        self.entita.direction = self.entita.direzione_verso(dx, dy);

        self.timer_attacco -= 1;
        if self.timer_attacco <= 0 {
            player.take_damage(self.dmg, effetti);
            self.timer_attacco = self.cooldown;
        }
    }

    pub fn take_damage(&mut self, amount: i32, player: &mut Giocatore, effetti: &mut Effetti) {
        if !self.vivo() {
            return;
        }
        self.hp = (self.hp - amount).max(0);
        self.entita.flash_timer = 8;
        effetti.add_text(
            &format!("-{}", amount),
            self.entita.x,
            self.entita.y - 60.0,
            Color::from_rgba(255, 230, 120, 255),
        );
        effetti.scintille(self.entita.x, self.entita.y - 40.0, Color::from_rgba(120, 90, 140, 255), 6);
        if !self.vivo() {
            effetti.suona(Suono::MortePipistrello, 1.0);
            player.monete += 1;
            effetti.suona(Suono::Moneta, 1.0);
            effetti.add_text(
                "+1 moneta",
                self.entita.x,
                self.entita.y - 85.0,
                Color::from_rgba(250, 210, 60, 255),
            );
            effetti.scintille(self.entita.x, self.entita.y - 40.0, Color::from_rgba(250, 210, 60, 255), 12);
        }
    }

    pub fn draw(&self, assets: &Assets) {
        if !self.vivo() {
            return;
        }
        self.entita.disegna_ombra(26.0);
        let alzo = 30.0 + (self.fase_volo.sin() * 4.0).round();
        let frames = assets.bat_frames.get(self.entita.direction);
        let tex = self.entita.frame(frames);
        self.entita.disegna_sprite(tex, alzo, SCALA_PIXEL, None, 0.0, assets);

        // barra della vita sopra la testa, solo se ferito
        if self.hp < self.hp_max {
            let w = 30.0;
            let x = (self.entita.x - w / 2.0).round();
            let y = (self.entita.y - alzo - tex.height() * SCALA_PIXEL - 8.0).round();
            draw_rectangle(x - 1.0, y - 1.0, w + 2.0, 6.0, BLACK);
            draw_rectangle(x, y, w * self.hp as f32 / self.hp_max as f32, 4.0, Color::from_rgba(220, 50, 50, 255));
        }
    }
}
