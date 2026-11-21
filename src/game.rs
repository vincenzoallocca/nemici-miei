use crate::assets::{Assets, MusicTrack};
use crate::boss::Boss;
use crate::effetti::Effetti;
use crate::enemy::Enemy;
use crate::hud::{testo_con_ombra, HUD};
use crate::menu::{Menu, MenuAction};
use crate::npc::Npc;
use crate::overlay::{GameOverAction, PausaAction, SchermataMorte, SchermataPausa};
use crate::player::Giocatore;
use crate::portal::Portale;
use crate::settings::*;
use crate::touch::ControlliTouch;
use crate::win::Win;
use crate::world::World;
use macroquad::prelude::*;
use std::collections::HashSet;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameState {
    Menu,
    Gioco,
    Pausa,
    Morto,
    Vittoria,
}

pub struct Gioco {
    pub stato: GameState,
    pub menu: Menu,
    pub schermata_morte: SchermataMorte,
    pub schermata_pausa: SchermataPausa,
    pub win: Option<Win>,
    pub touch: ControlliTouch,

    pub mondo: World,
    pub player: Giocatore,
    pub npc: Npc,
    pub portale: Portale,
    pub bats: Vec<Enemy>,
    pub boss: Option<Boss>,
    pub hud: HUD,
    pub effetti: Effetti,

    pub timer_vittoria: i32,
    pub tregua: i32,
    pub fullscreen: bool,
}

impl Gioco {
    pub fn new(assets: &Assets) -> Self {
        let menu = Menu::new();
        let schermata_morte = SchermataMorte::new();
        let schermata_pausa = SchermataPausa::new();
        let touch = ControlliTouch::new();

        let mut mondo = World::new(1);
        let player = Giocatore::new(SPAWN_GIOCATORE.0, SPAWN_GIOCATORE.1, &mondo);
        let npc = Npc::new(POSIZIONE_NPC.0, POSIZIONE_NPC.1, &mut mondo, assets);
        let portale = Portale::new(POSIZIONE_PORTALE.0, POSIZIONE_PORTALE.1, &mondo);
        let hud = HUD::new();
        let effetti = Effetti::new();

        

        Self {
            stato: GameState::Menu,
            menu,
            schermata_morte,
            schermata_pausa,
            win: None,
            touch,
            mondo,
            player,
            npc,
            portale,
            bats: Vec::new(),
            boss: None,
            hud,
            effetti,
            timer_vittoria: 0,
            tregua: FRAME_DI_TREGUA,
            fullscreen: false,
        }
    }

    pub fn vai_al_menu(&mut self, assets: &mut Assets) {
        self.stato = GameState::Menu;
        assets.play_music(MusicTrack::Soundtrack, 0.45);
    }

    pub fn nuova_partita(&mut self, assets: &mut Assets) {
        self.effetti = Effetti::new();
        self.mondo = World::new(1);
        self.player = Giocatore::new(SPAWN_GIOCATORE.0, SPAWN_GIOCATORE.1, &self.mondo);
        self.npc = Npc::new(POSIZIONE_NPC.0, POSIZIONE_NPC.1, &mut self.mondo, assets);
        self.portale = Portale::new(POSIZIONE_PORTALE.0, POSIZIONE_PORTALE.1, &self.mondo);

        let quanti_bats = rand::gen_range(3, 6);
        self.bats = self.genera_pipistrelli(quanti_bats);
        self.boss = None;

        self.hud = HUD::new();
        self.hud.touch = self.touch.attivo;
        self.touch.reset();

        if self.touch.attivo {
            self.hud.mostra_messaggio(
                "Trascina il dito a sinistra per muoverti. Raggiungi Ugo, il mago con il punto esclamativo.",
                7.0,
            );
        } else {
            self.hud.mostra_messaggio(
                "Muoviti con WASD o le frecce. Raggiungi Ugo, il mago con il punto esclamativo.",
                7.0,
            );
        }

        self.timer_vittoria = 0;
        self.tregua = FRAME_DI_TREGUA;
        self.stato = GameState::Gioco;
        assets.play_music(MusicTrack::Soundtrack, 0.45);
    }

    pub fn genera_pipistrelli(&self, quanti: usize) -> Vec<Enemy> {
        let (px, py) = SPAWN_GIOCATORE;
        let mut liberi = self.mondo.tiles_liberi(|tx, ty| {
            (tx - px).abs().max((ty - py).abs()) >= 4 && self.mondo.tipo_tile(tx, ty) != Some(20)
        });

        // Shuffle candidates
        for i in (1..liberi.len()).rev() {
            let j = rand::gen_range(0, i + 1);
            liberi.swap(i, j);
        }

        liberi
            .iter()
            .take(quanti)
            .map(|&(tx, ty)| Enemy::new(tx, ty, &self.mondo))
            .collect()
    }

    pub fn entra_livello_2(&mut self, assets: &mut Assets) {
        self.mondo.nuovo_livello(2);
        self.player.level = 2;
        self.player
            .entita
            .piazza(self.player.entita.tile_x, self.player.entita.tile_y, &self.mondo);
        self.bats.clear();
        self.boss = Some(Boss::new(SPAWN_BOSS.0, SPAWN_BOSS.1, &self.mondo));
        self.tregua = FRAME_DI_TREGUA;

        self.effetti.add_text(
            "LIVELLO 2",
            LUNGHEZZA / 2.0,
            ALTEZZA / 2.0 - 120.0,
            Color::from_rgba(255, 80, 80, 255),
        );

        let cmd = if self.touch.attivo {
            "ATK"
        } else {
            "Click o Spazio"
        };
        self.hud.mostra_messaggio(
            &format!("Sconfiggi il boss! Attacca con {} e allontanati quando trema.", cmd),
            7.0,
        );

        assets.play_music(MusicTrack::Boss, 1.0);
    }

    pub fn interagisci(&mut self, assets: &Assets) {
        if self.mondo.livello_id != 1 {
            return;
        }

        if self.player.entita.distanza_tiles(self.npc.tile_x, self.npc.tile_y) == 1 && !self.hud.giaparlato {
            self.hud.interagisci_npc(&mut self.npc, &mut self.player, &mut self.effetti);
            if self.hud.giaparlato {
                self.hud.mostra_messaggio("Vai alla fontanella e premi E per bere.", 5.0);
            }
        } else if self.player.puo_bere(self.hud.giaparlato, &self.mondo) {
            self.player.bevi(assets, &mut self.effetti);
            self.hud.mostra_messaggio("Il portale si e' aperto! Raggiungilo seguendo la freccia.", 6.0);
        }
    }

    pub fn attacca_nemici(&mut self, assets: &Assets) {
        if !self.player.esegui_attacco(assets, &mut self.effetti) {
            return;
        }

        if self.mondo.livello_id == 1 {
            for bat in &mut self.bats {
                if bat.vivo() && self.player.entita.distanza_tiles(bat.entita.tile_x, bat.entita.tile_y) <= 1 {
                    bat.take_damage(self.player.danno, &mut self.player, assets, &mut self.effetti);
                }
            }
        } else if let Some(ref mut boss) = self.boss {
            if boss.vivo() && self.player.entita.distanza_tiles(boss.entita.tile_x, boss.entita.tile_y) <= 1 {
                boss.take_damage(self.player.danno, assets, &mut self.effetti);
            }
        }
    }

    pub fn update(&mut self, assets: &mut Assets, mouse_world: (f32, f32)) -> bool {
        // Global Keys
        if is_key_pressed(KeyCode::M) {
            assets.toggle_audio();
            self.schermata_pausa.aggiorna_audio(assets.audio_attivo);
        }

        if is_key_pressed(KeyCode::F11) {
            // Note: toggle fullscreen
        }

        let clicked = is_mouse_button_pressed(MouseButton::Left);

        match self.stato {
            GameState::Menu => {
                match self.menu.handle_events(mouse_world, clicked) {
                    MenuAction::Start => self.nuova_partita(assets),
                    MenuAction::Esci => return false,
                    MenuAction::None => {}
                }
            }
            GameState::Gioco => {
                self.update_gioco(assets, mouse_world, clicked);
            }
            GameState::Pausa => {
                match self.schermata_pausa.handle_events(mouse_world, clicked) {
                    PausaAction::Riprendi => self.stato = GameState::Gioco,
                    PausaAction::Audio => {
                        assets.toggle_audio();
                        self.schermata_pausa.aggiorna_audio(assets.audio_attivo);
                    }
                    PausaAction::Menu => self.vai_al_menu(assets),
                    PausaAction::None => {}
                }
            }
            GameState::Morto => {
                match self.schermata_morte.handle_events(mouse_world, clicked) {
                    GameOverAction::Riprova => self.nuova_partita(assets),
                    GameOverAction::Menu => self.vai_al_menu(assets),
                    GameOverAction::None => {}
                }
            }
            GameState::Vittoria => {
                if let Some(ref mut w) = self.win {
                    w.update();
                    if w.handle_events(clicked) {
                        self.vai_al_menu(assets);
                    }
                }
            }
        }

        true
    }

    pub fn update_gioco(&mut self, assets: &mut Assets, mouse_world: (f32, f32), clicked: bool) {
        let (azione_attacca, azione_interagisci, azione_pausa) = self.touch.handle_input(mouse_world);
        self.hud.touch = self.touch.attivo;
        self.menu.touch = self.touch.attivo;
        self.schermata_morte.touch = self.touch.attivo;
        self.schermata_pausa.touch = self.touch.attivo;

        if azione_pausa || is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P) {
            self.touch.reset();
            self.stato = GameState::Pausa;
            return;
        }

        if azione_interagisci || is_key_pressed(KeyCode::E) {
            self.interagisci(assets);
        }

        if azione_attacca || is_key_pressed(KeyCode::Space) || (clicked && !self.touch.attivo) {
            self.attacca_nemici(assets);
        }

        // Build set of occupied tiles
        let mut tiles_nemici = HashSet::new();
        if self.mondo.livello_id == 1 {
            for b in &self.bats {
                if b.vivo() {
                    tiles_nemici.insert((b.entita.tile_x, b.entita.tile_y));
                }
            }
        } else if let Some(ref b) = self.boss {
            if b.vivo() {
                tiles_nemici.insert((b.entita.tile_x, b.entita.tile_y));
            }
        }

        if self.player.vivo() {
            let passo_touch = self.touch.passo();
            self.player.update(&self.mondo, assets, &tiles_nemici, passo_touch);
        }

        if self.tregua > 0 {
            self.tregua -= 1;
        } else {
            // Update enemies
            if self.mondo.livello_id == 1 {
                for i in 0..self.bats.len() {
                    if !self.bats[i].vivo() {
                        continue;
                    }
                    let mut occupati = HashSet::new();
                    occupati.insert((self.player.entita.tile_x, self.player.entita.tile_y));
                    for (j, altro) in self.bats.iter().enumerate() {
                        if i != j && altro.vivo() {
                            occupati.insert((altro.entita.tile_x, altro.entita.tile_y));
                        }
                    }
                    self.bats[i].update(&self.mondo, &mut self.player, assets, &mut self.effetti, &occupati);
                }
            } else if let Some(ref mut boss) = self.boss {
                let mut occupati = HashSet::new();
                occupati.insert((self.player.entita.tile_x, self.player.entita.tile_y));
                boss.update(&self.mondo, &mut self.player, assets, &mut self.effetti, &occupati);
            }
        }

        if self.mondo.livello_id == 1 {
            self.portale.update();
            if self.player.sul_portale(&self.mondo) && !self.player.thirsty {
                self.entra_livello_2(assets);
            }
        }

        if !self.player.vivo() {
            self.schermata_morte.nuovo_consiglio();
            self.stato = GameState::Morto;
        } else if let Some(ref b) = self.boss {
            if !b.vivo() {
                self.timer_vittoria += 1;
                if self.timer_vittoria >= FRAME_PRIMA_DEI_CREDITI {
                    self.win = Some(Win::new(self.player.monete));
                    self.stato = GameState::Vittoria;
                }
            }
        }
    }

    pub fn draw(&mut self, assets: &Assets, mouse_world: (f32, f32)) {
        match self.stato {
            GameState::Menu => {
                self.menu.draw(mouse_world, assets);
            }
            GameState::Gioco => {
                self.draw_gioco(assets, true);
            }
            GameState::Pausa => {
                self.draw_gioco(assets, false);
                self.schermata_pausa.draw(mouse_world, assets);
            }
            GameState::Morto => {
                self.draw_gioco(assets, false);
                self.schermata_morte.draw(mouse_world, assets);
            }
            GameState::Vittoria => {
                if let Some(ref w) = self.win {
                    w.draw(assets);
                }
            }
        }
    }

    pub fn draw_gioco(&mut self, assets: &Assets, aggiorna_animazioni: bool) {
        self.mondo.draw_terreno(assets);

        // Sorting depth items
        enum TallKind {
            Fountain,
            Npc,
            Portal,
            Bat(usize),
            Boss,
            Player,
        }

        let mut items: Vec<(f32, TallKind)> = Vec::with_capacity(16);

        if let Some(ref f) = self.mondo.fountain {
            items.push((f.bottom, TallKind::Fountain));
        }

        if self.mondo.livello_id == 1 {
            items.push((self.npc.profondita(), TallKind::Npc));
            items.push((self.portale.profondita(), TallKind::Portal));
            for (idx, b) in self.bats.iter().enumerate() {
                if b.vivo() {
                    items.push((b.entita.profondita(), TallKind::Bat(idx)));
                }
            }
        } else if let Some(ref b) = self.boss {
            if b.vivo() {
                items.push((b.entita.profondita(), TallKind::Boss));
            }
        }

        items.push((self.player.entita.profondita(), TallKind::Player));

        // Sort ascending by depth (lower y drawn first, higher y in front)
        items.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        for (_, kind) in items {
            match kind {
                TallKind::Fountain => self.mondo.draw_fountain(assets),
                TallKind::Npc => self.npc.draw(assets),
                TallKind::Portal => self.portale.draw(assets, !self.player.thirsty),
                TallKind::Bat(idx) => self.bats[idx].draw(assets),
                TallKind::Boss => {
                    if let Some(ref mut b) = self.boss {
                        b.draw(assets);
                    }
                }
                TallKind::Player => self.player.draw(assets),
            }
        }

        self.hud.draw_freccia_obiettivo(&self.mondo, &self.player, &self.portale);

        if aggiorna_animazioni {
            self.effetti.update_and_draw(assets);
        }

        if let Some(ref b) = self.boss {
            if !b.vivo() {
                testo_con_ombra(
                    &assets.dungeon_font,
                    "VITTORIA!",
                    LUNGHEZZA / 2.0,
                    ALTEZZA / 2.0 - 60.0,
                    80,
                    Color::from_rgba(250, 210, 60, 255),
                    "center",
                );
            }
        }

        self.hud.draw(
            &self.mondo,
            &self.player,
            &mut self.npc,
            self.boss.as_ref(),
            assets,
        );

        if self.stato == GameState::Gioco {
            self.touch.draw(assets);
        }
    }
}
