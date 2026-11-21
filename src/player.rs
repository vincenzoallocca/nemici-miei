use crate::assets::{Assets, Direction};
use crate::effetti::Effetti;
use crate::entita::Entita;
use crate::settings::*;
use crate::world::World;
use macroquad::prelude::*;
use std::collections::HashSet;

pub struct Giocatore {
    pub entita: Entita,
    pub thirsty: bool,
    pub ugo_vita_bonus: bool,
    pub vita: i32,
    pub vita_max: i32,
    pub monete: i32,
    pub level: i32,
    pub danno: i32,
    pub last_attack_time: f64,
    pub attack_cooldown: f64, // seconds: 0.600
    pub attaccato_timer: i32,
}

impl Giocatore {
    pub fn new(tile_x: i32, tile_y: i32, world: &World) -> Self {
        Self {
            entita: Entita::new(tile_x, tile_y, world, VELOCITA_NORMALE),
            thirsty: true,
            ugo_vita_bonus: false,
            vita: 100,
            vita_max: 100,
            monete: 0,
            level: 1,
            danno: 20,
            last_attack_time: -100.0,
            attack_cooldown: 0.600,
            attaccato_timer: 0,
        }
    }

    pub fn vivo(&self) -> bool {
        self.vita > 0
    }

    pub fn update(
        &mut self,
        world: &World,
        assets: &Assets,
        occupati: &HashSet<(i32, i32)>,
        passo_touch: Option<(i32, i32)>,
    ) {
        if self.entita.in_movimento() {
            self.entita.aggiorna_movimento();
        } else {
            self.gestisci_input(world, assets, occupati, passo_touch);
        }
    }

    pub fn gestisci_input(
        &mut self,
        world: &World,
        assets: &Assets,
        occupati: &HashSet<(i32, i32)>,
        passo_touch: Option<(i32, i32)>,
    ) {
        let mut dx = 0;
        let mut dy = 0;
        let mut dir = None;

        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            dx -= 1;
            dir = Some(Direction::Left);
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            dx += 1;
            dir = Some(Direction::Right);
        }
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            dy -= 1;
            dir = Some(Direction::Up);
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            dy += 1;
            dir = Some(Direction::Down);
        }

        if dx == 0 && dy == 0 {
            if let Some((tdx, tdy)) = passo_touch {
                dx = tdx;
                dy = tdy;
                dir = Some(Self::direzione_passo(dx, dy));
            }
        }

        if let Some(d) = dir {
            self.entita.direction = d;
        }

        if (dx != 0 || dy != 0)
            && self
                .entita
                .muovi_verso(self.entita.tile_x + dx, self.entita.tile_y + dy, world, occupati)
        {
            self.suono_passo(world, assets);
        }
    }

    pub fn direzione_passo(dx: i32, dy: i32) -> Direction {
        if dx == -dy {
            if dx > 0 {
                Direction::Right
            } else {
                Direction::Left
            }
        } else if dy != 0 {
            if dy > 0 {
                Direction::Down
            } else {
                Direction::Up
            }
        } else {
            if dx > 0 {
                Direction::Right
            } else {
                Direction::Left
            }
        }
    }

    pub fn suono_passo(&mut self, world: &World, assets: &Assets) {
        let tile = world.tipo_tile(self.entita.tile_x, self.entita.tile_y);
        self.entita.velocita = VELOCITA_NORMALE;

        if let Some(t) = tile {
            if is_tile_erba(t) {
                assets.play_sound(&assets.snd_grass, 0.35);
            } else if is_tile_acqua(t) {
                assets.play_sound(&assets.snd_acqua, 0.35);
                self.entita.velocita = VELOCITA_ACQUA;
            } else if is_tile_pietra(t) {
                assets.play_sound(&assets.snd_stone, 0.35);
            }
        }
    }

    pub fn sul_portale(&self, world: &World) -> bool {
        world.tipo_tile(self.entita.tile_x, self.entita.tile_y) == Some(20) && !self.entita.in_movimento()
    }

    pub fn is_near_fountain(&self, world: &World) -> bool {
        if let Some((fx, fy)) = world.trova_fontanella() {
            for x in (fx - 1)..=(fx + 1) {
                for y in (fy - 1)..=(fy + 1) {
                    if let Some(t) = world.tipo_tile(x, y) {
                        if (t == 18 || t == 19)
                            && (self.entita.tile_x - x).abs().max((self.entita.tile_y - y).abs()) == 1
                        {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn puo_bere(&self, ha_parlato: bool, world: &World) -> bool {
        self.is_near_fountain(world) && self.thirsty && ha_parlato
    }

    pub fn bevi(&mut self, assets: &Assets, effetti: &mut Effetti) {
        self.thirsty = false;
        self.monete += 1;
        assets.play_sound(&assets.snd_coin, 1.0);
        effetti.add_text("+1 moneta", self.entita.x, self.entita.y - 80.0, Color::from_rgba(250, 210, 60, 255));
        self.cura(25, effetti);
    }

    pub fn cura(&mut self, quantita: i32, effetti: &mut Effetti) {
        let prima = self.vita;
        self.vita = (self.vita + quantita).min(self.vita_max);
        if self.vita > prima {
            effetti.add_text(
                &format!("+{}", self.vita - prima),
                self.entita.x,
                self.entita.y - 60.0,
                Color::from_rgba(90, 230, 90, 255),
            );
        }
    }

    pub fn take_damage(&mut self, amount: i32, assets: &Assets, effetti: &mut Effetti) {
        if !self.vivo() {
            return;
        }
        self.vita = (self.vita - amount).max(0);
        assets.play_sound(&assets.snd_player_dmg, 0.6);
        self.attaccato_timer = 15;
        effetti.add_text(
            &format!("-{}", amount),
            self.entita.x,
            self.entita.y - 60.0,
            Color::from_rgba(255, 80, 80, 255),
        );
    }

    pub fn ricarica_attacco(&self) -> f32 {
        let now = get_time();
        let elapsed = now - self.last_attack_time;
        ((elapsed / self.attack_cooldown) as f32).clamp(0.0, 1.0)
    }

    pub fn puo_attaccare(&self) -> bool {
        get_time() - self.last_attack_time >= self.attack_cooldown
    }

    pub fn esegui_attacco(&mut self, assets: &Assets, effetti: &mut Effetti) -> bool {
        if !self.puo_attaccare() {
            return false;
        }
        self.last_attack_time = get_time();
        effetti.add_fendente(self.entita.x, self.entita.y - 25.0, self.entita.direction);
        assets.play_sound(&assets.snd_attack, 0.7);
        true
    }

    pub fn draw(&mut self, assets: &Assets) {
        let frames = if self.entita.in_movimento() {
            assets.player_walk.get(self.entita.direction)
        } else {
            assets.player_idle.get(self.entita.direction)
        };
        let anim_speed = if self.entita.in_movimento() { 8 } else { 20 };
        let tex = self.entita.anima(frames, anim_speed);

        self.entita.disegna_ombra(30.0);

        let tint = if self.attaccato_timer > 0 {
            self.attaccato_timer -= 1;
            Some(Color::from_rgba(255, 100, 100, 255))
        } else {
            None
        };

        self.entita.disegna_sprite(tex, 4.0, SCALA_PLAYER, tint);
    }
}
