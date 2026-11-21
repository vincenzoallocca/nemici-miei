use crate::assets::Assets;
use crate::effetti::Effetti;
use crate::entita::Entita;
use crate::player::Giocatore;
use crate::settings::SCALA_BAT;
use crate::world::World;
use macroquad::prelude::*;
use std::collections::HashSet;

pub struct Enemy {
    pub entita: Entita,
    pub hp: i32,
    pub hp_max: i32,
    pub cooldown: f64, // seconds: 2.500
    pub dmg: i32,
    pub last_attack_time: f64,
    pub vicino: bool,
    pub fase_volo: f32,
}

impl Enemy {
    pub fn new(tile_x: i32, tile_y: i32, world: &World) -> Self {
        Self {
            entita: Entita::new(tile_x, tile_y, world, 2.0),
            hp: 50,
            hp_max: 50,
            cooldown: 2.5,
            dmg: 10,
            last_attack_time: 0.0,
            vicino: false,
            fase_volo: (tile_x + tile_y) as f32,
        }
    }

    pub fn vivo(&self) -> bool {
        self.hp > 0
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

        if self.entita.in_movimento() {
            self.entita.aggiorna_movimento();
        } else if self.entita.distanza_tiles(player.entita.tile_x, player.entita.tile_y) <= 1 {
            self.attacca(player, assets, effetti);
        } else {
            self.vicino = false;
            self.entita.insegui(player.entita.tile_x, player.entita.tile_y, world, occupati);
        }
    }

    pub fn attacca(&mut self, player: &mut Giocatore, assets: &Assets, effetti: &mut Effetti) {
        let now = get_time();
        if !self.vicino {
            self.vicino = true;
            self.last_attack_time = now - self.cooldown + 0.700;
        }
        if now - self.last_attack_time >= self.cooldown {
            player.take_damage(self.dmg, assets, effetti);
            self.last_attack_time = now;
        }
    }

    pub fn take_damage(&mut self, amount: i32, player: &mut Giocatore, assets: &Assets, effetti: &mut Effetti) {
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
        if !self.vivo() {
            assets.play_sound(&assets.snd_bat_morte, 1.0);
            player.monete += 1;
            assets.play_sound(&assets.snd_coin, 1.0);
            effetti.add_text(
                "+1 moneta",
                self.entita.x,
                self.entita.y - 85.0,
                Color::from_rgba(250, 210, 60, 255),
            );
        }
    }

    pub fn draw(&mut self, assets: &Assets) {
        if !self.vivo() {
            return;
        }
        self.entita.disegna_ombra(26.0);
        self.fase_volo += 0.08;
        let alzo = 30.0 + (self.fase_volo.sin() * 4.0).round();
        let frames = assets.bat_frames.get(self.entita.direction);
        let tex = self.entita.anima(frames, 10);
        self.entita.disegna_sprite(tex, alzo, SCALA_BAT, None);
    }
}
