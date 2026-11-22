use crate::assets::{Assets, Suono};
use crate::effetti::Effetti;
use crate::entita::Entita;
use crate::player::Giocatore;
use crate::settings::SCALA_PIXEL;
use crate::world::World;
use macroquad::prelude::*;

/// Passi di "tremore" prima del colpo: il momento per allontanarsi.
pub const CARICA_NORMALE: i32 = 35;
pub const CARICA_FURIA: i32 = 22;
pub const DURATA_RITIRATA: i32 = 180;
pub const VITA_BOSS: f32 = 150.0;
pub const DANNO_BOSS: i32 = 10;
pub const VELOCITA_BOSS: f32 = 2.7;
pub const VELOCITA_BOSS_FURIA: f32 = 4.0;

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
    /// Vero per un passo quando entra in furia: il gioco può chiamare rinforzi.
    pub appena_infuriato: bool,
}

impl Boss {
    pub fn new(tile_x: i32, tile_y: i32, world: &World) -> Self {
        Self {
            entita: Entita::new(tile_x, tile_y, world, VELOCITA_BOSS),
            damage: DANNO_BOSS,
            hp: VITA_BOSS,
            hp_max: VITA_BOSS,
            soglia_furia: VITA_BOSS / 2.0,
            furia: false,
            stato: BossStato::Insegui,
            timer: 0,
            rigenerazione: 0.05,
            appena_infuriato: false,
        }
    }

    pub fn vivo(&self) -> bool {
        self.hp > 0.0
    }

    /// Solo animazione ed effetti: usato anche durante la tregua.
    pub fn anima(&mut self, effetti: &mut Effetti) {
        self.entita.avanza_animazione(4, 10);
        self.timer_visivo(effetti);
    }

    fn timer_visivo(&mut self, effetti: &mut Effetti) {
        if self.furia && self.vivo() && rand::gen_range(0, 3) == 0 {
            let x = self.entita.x + rand::gen_range(-18.0, 18.0);
            let y = self.entita.y - rand::gen_range(10.0, 70.0);
            effetti.brace(x, y, Color::from_rgba(255, rand::gen_range(60, 160), 40, 255));
        }
    }

    pub fn update(&mut self, world: &World, player: &mut Giocatore, effetti: &mut Effetti, occupati: &[(i32, i32)]) {
        self.appena_infuriato = false;
        if !self.vivo() {
            return;
        }

        if !self.furia && self.hp <= self.soglia_furia {
            self.entra_in_furia(effetti);
        }

        if self.entita.in_movimento() {
            self.entita.aggiorna_movimento();
        }

        let giocatore = player.entita.tile();
        match self.stato {
            BossStato::Ritirata => {
                self.timer += 1;
                let limite = if self.furia { self.soglia_furia } else { self.hp_max };
                self.hp = (self.hp + self.rigenerazione).min(limite);
                if !self.entita.in_movimento() {
                    let rifugio = world.punto_lontano(giocatore.0, giocatore.1);
                    self.entita.vai_verso(rifugio, 0, world, occupati);
                }
                if self.timer >= DURATA_RITIRATA {
                    self.stato = BossStato::Insegui;
                }
            }
            BossStato::Carica => {
                self.timer += 1;
                let limite_carica = if self.furia { CARICA_FURIA } else { CARICA_NORMALE };
                if self.timer >= limite_carica {
                    if self.entita.distanza_tiles(giocatore.0, giocatore.1) <= 1 {
                        player.take_damage(self.damage, effetti);
                    }
                    self.stato = BossStato::Ritirata;
                    self.timer = 0;
                }
            }
            BossStato::Insegui => {
                if !self.entita.in_movimento() {
                    if self.entita.distanza_tiles(giocatore.0, giocatore.1) <= 1 {
                        self.stato = BossStato::Carica;
                        self.timer = 0;
                        self.entita.direction = self
                            .entita
                            .direzione_verso(giocatore.0 - self.entita.tile_x, giocatore.1 - self.entita.tile_y);
                    } else {
                        self.entita.insegui(giocatore.0, giocatore.1, world, occupati);
                    }
                }
            }
        }
    }

    pub fn entra_in_furia(&mut self, effetti: &mut Effetti) {
        self.furia = true;
        self.appena_infuriato = true;
        self.entita.velocita = VELOCITA_BOSS_FURIA;
        self.damage = (self.damage as f32 * 1.5) as i32;
        effetti.add_text("FURIA!", self.entita.x, self.entita.y - 120.0, Color::from_rgba(255, 60, 60, 255));
        effetti.scintille(self.entita.x, self.entita.y - 50.0, Color::from_rgba(255, 80, 40, 255), 30);
    }

    pub fn take_damage(&mut self, amount: i32, effetti: &mut Effetti) {
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
        effetti.scintille(self.entita.x, self.entita.y - 50.0, Color::from_rgba(150, 100, 50, 255), 8);
        if !self.vivo() {
            effetti.suona(Suono::MorteBoss, 0.5);
            effetti.scintille(self.entita.x, self.entita.y - 50.0, Color::from_rgba(250, 210, 60, 255), 40);
        }
    }

    pub fn draw(&self, assets: &Assets) {
        if !self.vivo() {
            return;
        }

        if self.furia {
            // alone rosso che pulsa sotto i piedi
            let pulsazione = (get_time() as f32 * 6.0).sin() * 0.5 + 0.5;
            draw_ellipse(
                self.entita.x.round(),
                self.entita.y.round(),
                34.0 + pulsazione * 6.0,
                13.0 + pulsazione * 2.0,
                0.0,
                Color::new(1.0, 0.15, 0.1, 0.25 + 0.2 * pulsazione),
            );
        }
        self.entita.disegna_ombra(if self.furia { 48.0 } else { 38.0 });

        let tint = self.furia.then(|| Color::from_rgba(255, 110, 110, 255));
        let frames = assets.boss_frames.get(self.entita.direction);

        if self.stato == BossStato::Carica {
            let scossa = if (self.timer / 3) % 2 == 0 { 2.0 } else { -2.0 };
            let rect = self
                .entita
                .disegna_sprite(&frames[0], 0.0, SCALA_PIXEL, tint, scossa, assets);

            // avviso rosso sopra la testa
            draw_circle(rect.x + rect.w / 2.0, rect.y - 12.0, 7.0, BLACK);
            draw_circle(rect.x + rect.w / 2.0, rect.y - 12.0, 5.0, Color::from_rgba(255, 60, 60, 255));
        } else {
            let tex = self.entita.frame(frames);
            self.entita.disegna_sprite(tex, 0.0, SCALA_PIXEL, tint, 0.0, assets);
        }
    }
}
