use crate::assets::{Assets, Direction, Suono};
use crate::effetti::Effetti;
use crate::entita::Entita;
use crate::settings::*;
use crate::world::World;
use macroquad::prelude::*;

pub struct Giocatore {
    pub entita: Entita,
    pub thirsty: bool,
    pub ugo_vita_bonus: bool,
    pub vita: i32,
    pub vita_max: i32,
    pub monete: i32,
    pub level: i32,
    pub danno: i32,
    /// Passi che mancano prima di poter attaccare di nuovo.
    pub cooldown_attacco: i32,
    pub attaccato_timer: i32,
}

impl Giocatore {
    pub fn new(tile_x: i32, tile_y: i32, world: &World) -> Self {
        Self {
            entita: Entita::new(tile_x, tile_y, world, VELOCITA_NORMALE),
            thirsty: true,
            ugo_vita_bonus: false,
            vita: VITA_GIOCATORE,
            vita_max: VITA_GIOCATORE,
            monete: 0,
            level: 1,
            danno: DANNO_GIOCATORE,
            cooldown_attacco: 0,
            attaccato_timer: 0,
        }
    }

    pub fn vivo(&self) -> bool {
        self.vita > 0
    }

    /// Un passo di logica. `passo` è la direzione richiesta sulla griglia (tastiera o joystick).
    pub fn update(
        &mut self,
        world: &World,
        occupati: &[(i32, i32)],
        passo: (i32, i32),
        direzione: Option<Direction>,
        effetti: &mut Effetti,
    ) {
        if self.cooldown_attacco > 0 {
            self.cooldown_attacco -= 1;
        }
        if self.attaccato_timer > 0 {
            self.attaccato_timer -= 1;
        }

        if self.vivo() {
            if self.entita.in_movimento() {
                self.entita.aggiorna_movimento();
            } else {
                self.gestisci_input(world, occupati, passo, direzione, effetti);
            }
        }

        let velocita_anim = if self.entita.in_movimento() { 8 } else { 20 };
        self.entita.avanza_animazione(4, velocita_anim);
    }

    fn gestisci_input(
        &mut self,
        world: &World,
        occupati: &[(i32, i32)],
        (dx, dy): (i32, i32),
        direzione: Option<Direction>,
        effetti: &mut Effetti,
    ) {
        if dx == 0 && dy == 0 {
            return;
        }
        self.entita.direction = direzione.unwrap_or_else(|| Self::direzione_passo(dx, dy));
        if self
            .entita
            .muovi_verso(self.entita.tile_x + dx, self.entita.tile_y + dy, world, occupati)
        {
            self.suono_passo(world, effetti);
        }
    }

    /// Verso quale lato dello schermo guarda chi fa il passo (dx, dy) sulla griglia.
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
        } else if dx > 0 {
            Direction::Right
        } else {
            Direction::Left
        }
    }

    pub fn suono_passo(&mut self, world: &World, effetti: &mut Effetti) {
        let tile = world.tipo_tile(self.entita.tile_x, self.entita.tile_y);
        self.entita.velocita = VELOCITA_NORMALE;

        if let Some(t) = tile {
            if is_tile_erba(t) {
                effetti.suona(Suono::Erba, 0.15);
            } else if is_tile_acqua(t) {
                effetti.suona(Suono::Acqua, 0.15);
                self.entita.velocita = VELOCITA_ACQUA;
            } else if is_tile_pietra(t) {
                effetti.suona(Suono::Pietra, 0.15);
            }
        }
    }

    pub fn sul_tile(&self, tile: (i32, i32)) -> bool {
        self.entita.tile() == tile && !self.entita.in_movimento()
    }

    pub fn is_near_fountain(&self, world: &World) -> bool {
        if let Some((fx, fy)) = world.trova_fontanella() {
            for x in (fx - 1)..=(fx + 1) {
                for y in (fy - 1)..=(fy + 1) {
                    if let Some(t) = world.tipo_tile(x, y) {
                        if (t == TILE_FONTANELLA || t == TILE_BORDO_FONTANA) && self.entita.distanza_tiles(x, y) == 1 {
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

    pub fn bevi(&mut self, effetti: &mut Effetti) {
        self.thirsty = false;
        self.monete += 1;
        effetti.suona(Suono::Moneta, 1.0);
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

    pub fn take_damage(&mut self, amount: i32, effetti: &mut Effetti) {
        if !self.vivo() {
            return;
        }
        self.vita = (self.vita - amount).max(0);
        effetti.suona(Suono::DannoGiocatore, 0.35);
        self.attaccato_timer = 15;
        effetti.add_text(
            &format!("-{}", amount),
            self.entita.x,
            self.entita.y - 60.0,
            Color::from_rgba(255, 80, 80, 255),
        );
        effetti.scintille(self.entita.x, self.entita.y - 30.0, Color::from_rgba(220, 40, 40, 255), 8);
    }

    /// Da 0 (appena attaccato) a 1 (pronto).
    pub fn ricarica_attacco(&self) -> f32 {
        1.0 - (self.cooldown_attacco as f32 / COOLDOWN_ATTACCO as f32).clamp(0.0, 1.0)
    }

    pub fn puo_attaccare(&self) -> bool {
        self.cooldown_attacco <= 0
    }

    pub fn esegui_attacco(&mut self, effetti: &mut Effetti) -> bool {
        if !self.puo_attaccare() || !self.vivo() {
            return false;
        }
        self.cooldown_attacco = COOLDOWN_ATTACCO;
        effetti.add_fendente(self.entita.x, self.entita.y - 25.0, self.entita.direction);
        effetti.suona(Suono::Attacco, 0.5);
        true
    }

    pub fn draw(&self, assets: &Assets) {
        let frames = if self.entita.in_movimento() {
            assets.player_walk.get(self.entita.direction)
        } else {
            assets.player_idle.get(self.entita.direction)
        };
        let tex = self.entita.frame(frames);

        self.entita.disegna_ombra(30.0);

        let tint = (self.attaccato_timer > 0).then(|| Color::from_rgba(255, 100, 100, 255));
        // l'ultima riga dello sprite è vuota: lo abbassiamo di un pixel (2 a schermo)
        self.entita.disegna_sprite(tex, -SCALA_PIXEL, SCALA_PIXEL, tint, 0.0, assets);
    }
}
