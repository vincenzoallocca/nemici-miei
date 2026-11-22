use crate::assets::{Assets, ComandoAudio, MusicTrack};
use crate::boss::Boss;
use crate::effetti::Effetti;
use crate::enemy::Enemy;
use crate::entita::Entita;
use crate::hud::{testo_con_ombra, Ancora, GIALLO, HUD};
use crate::input::Input;
use crate::livelli::{DefLivello, Obiettivo, LIVELLI};
use crate::menu::{disegna_copertina, Menu, MenuAction};
use crate::npc::Npc;
use crate::pickup::{Raccoglibile, TipoOggetto};
use crate::assets::Suono;
use crate::overlay::{GameOverAction, PausaAction, SchermataMorte, SchermataPausa};
use crate::player::Giocatore;
use crate::portal::Portale;
use crate::settings::*;
use crate::touch::ControlliTouch;
use crate::win::Win;
use crate::world::{distanza, World};
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameState {
    Menu,
    Gioco,
    Pausa,
    Morto,
    Vittoria,
}

/// Durata della scritta "LIVELLO N" all'inizio di un livello.
const DURATA_BANNER: i32 = 150;
const VOLUME_SOUNDTRACK: f32 = 0.45;
const VOLUME_BOSS: f32 = 1.0;

pub struct Gioco {
    pub stato: GameState,
    pub menu: Menu,
    pub schermata_morte: SchermataMorte,
    pub schermata_pausa: SchermataPausa,
    pub win: Option<Win>,
    pub touch: ControlliTouch,

    /// Indice in [`LIVELLI`].
    pub livello: usize,
    pub mondo: World,
    pub player: Giocatore,
    pub npc: Option<Npc>,
    pub portale: Option<Portale>,
    pub bats: Vec<Enemy>,
    pub boss: Option<Boss>,
    pub oggetti: Vec<Raccoglibile>,
    pub hud: HUD,
    pub effetti: Effetti,

    pub timer_vittoria: i32,
    pub tregua: i32,
    pub banner_timer: i32,
    /// Monete all'inizio del livello: "Riprova" riparte da qui.
    pub monete_inizio_livello: i32,

    pub audio_attivo: bool,
    /// Comandi audio per il ciclo principale (musica, suoni, audio sì/no).
    pub audio: Vec<ComandoAudio>,
}

impl Default for Gioco {
    fn default() -> Self {
        Self::new()
    }
}

impl Gioco {
    pub fn new() -> Self {
        let mondo = World::new(1);
        let player = Giocatore::new(LIVELLI[0].spawn.0, LIVELLI[0].spawn.1, &mondo);
        let mut gioco = Self {
            stato: GameState::Menu,
            menu: Menu::new(),
            schermata_morte: SchermataMorte::new(),
            schermata_pausa: SchermataPausa::new(),
            win: None,
            touch: ControlliTouch::new(),
            livello: 0,
            mondo,
            player,
            npc: None,
            portale: None,
            bats: Vec::new(),
            boss: None,
            oggetti: Vec::new(),
            hud: HUD::new(),
            effetti: Effetti::new(),
            timer_vittoria: 0,
            tregua: FRAME_DI_TREGUA,
            banner_timer: 0,
            monete_inizio_livello: 0,
            audio_attivo: true,
            audio: Vec::new(),
        };
        gioco.vai_al_menu();
        gioco
    }

    pub fn def(&self) -> &'static DefLivello {
        &LIVELLI[self.livello]
    }

    // ---------- cambi di stato ----------

    pub fn vai_al_menu(&mut self) {
        self.stato = GameState::Menu;
        self.audio.push(ComandoAudio::Musica(MusicTrack::Soundtrack, VOLUME_SOUNDTRACK));
    }

    pub fn nuova_partita(&mut self) {
        self.player = Giocatore::new(0, 0, &self.mondo);
        self.carica_livello(0);
    }

    /// Riparte dall'inizio del livello in cui si è morti, con la vita piena.
    pub fn riprova(&mut self) {
        self.player.vita = self.player.vita_max;
        self.player.monete = self.monete_inizio_livello;
        self.carica_livello(self.livello);
    }

    pub fn carica_livello(&mut self, indice: usize) {
        let def = &LIVELLI[indice];
        self.livello = indice;
        self.mondo = World::da_definizione(indice as u32 + 1, def);
        self.effetti = Effetti::new();

        // il giocatore tiene vita e monete, ma riparte dallo spawn del livello
        self.player.entita = Entita::new(def.spawn.0, def.spawn.1, &self.mondo, VELOCITA_NORMALE);
        self.player.level = indice as i32 + 1;
        self.player.cooldown_attacco = 0;
        self.player.attaccato_timer = 0;
        if indice == 0 {
            self.player.thirsty = true;
            self.player.ugo_vita_bonus = false;
        }

        self.npc = def.npc.map(|(x, y)| Npc::new(x, y, &mut self.mondo));
        self.portale = def.portale.map(|(x, y)| Portale::new(x, y, &self.mondo));
        self.boss = def.boss.map(|(x, y)| Boss::new(x, y, &self.mondo));
        let (min, max) = def.pipistrelli;
        let quanti = if max > min { rand::gen_range(min, max + 1) } else { min };
        self.bats = self.genera_pipistrelli(quanti);
        self.oggetti = self.genera_oggetti(def.monete, def.cuori);

        self.hud = HUD::new();
        self.hud.touch = self.touch.attivo;
        self.touch.reset();

        let intro = if indice == 0 && self.touch.attivo {
            "Trascina il dito a sinistra per muoverti. Raggiungi Ugo, il mago con il punto esclamativo.".to_string()
        } else {
            def.intro.replace("{ATK}", self.comando_attacco())
        };
        self.hud.mostra_messaggio(&intro, 7.0);

        self.timer_vittoria = 0;
        self.tregua = FRAME_DI_TREGUA;
        self.banner_timer = DURATA_BANNER;
        self.monete_inizio_livello = self.player.monete;
        self.stato = GameState::Gioco;

        let volume = if def.musica == MusicTrack::Boss { VOLUME_BOSS } else { VOLUME_SOUNDTRACK };
        self.audio.push(ComandoAudio::Musica(def.musica, volume));
    }

    pub fn comando_attacco(&self) -> &'static str {
        if self.touch.attivo {
            "ATK"
        } else {
            "Click o Spazio"
        }
    }

    /// Tile liberi dove far comparire nemici: lontano dal giocatore, mai su portale o tile "sicuri".
    pub fn tiles_per_nemici(&self) -> Vec<(i32, i32)> {
        let spawn = self.player.entita.tile();
        let portale = self.portale.as_ref().map(|p| p.tile());
        let boss = self.boss.as_ref().map(|b| b.entita.tile());
        self.mondo.tiles_liberi(|tx, ty| {
            distanza((tx, ty), spawn) >= 4
                && Some((tx, ty)) != portale
                && Some((tx, ty)) != boss
                && !self.mondo.tipo_tile(tx, ty).is_some_and(is_tile_sicuro)
        })
    }

    pub fn genera_pipistrelli(&self, quanti: usize) -> Vec<Enemy> {
        let mut liberi = self.tiles_per_nemici();
        mescola(&mut liberi);
        let def = self.def();
        liberi
            .iter()
            .take(quanti)
            .map(|&(tx, ty)| Enemy::con_statistiche(tx, ty, &self.mondo, def.vita_pipistrelli, def.danno_pipistrelli))
            .collect()
    }

    /// Monete e cuori su tile raggiungibili, lontani dallo spawn e non sotto i pipistrelli.
    pub fn genera_oggetti(&self, monete: usize, cuori: usize) -> Vec<Raccoglibile> {
        let spawn = self.player.entita.tile();
        let pipistrelli: Vec<_> = self.bats.iter().map(|b| b.entita.tile()).collect();
        let mut liberi: Vec<_> = self
            .tiles_per_nemici()
            .into_iter()
            .filter(|t| !pipistrelli.contains(t) && self.mondo.passo_verso(spawn, *t, 0, &[]).is_some())
            .collect();
        mescola(&mut liberi);
        let tipi = std::iter::repeat_n(TipoOggetto::Moneta, monete).chain(std::iter::repeat_n(TipoOggetto::Cuore, cuori));
        liberi
            .into_iter()
            .zip(tipi)
            .map(|((x, y), tipo)| Raccoglibile::new(tipo, x, y, &self.mondo))
            .collect()
    }

    pub fn monete_rimaste(&self) -> usize {
        self.oggetti
            .iter()
            .filter(|o| o.tipo == TipoOggetto::Moneta && !o.preso)
            .count()
    }

    fn raccogli_oggetti(&mut self) {
        let pos = self.player.entita.tile();
        for o in self.oggetti.iter_mut().filter(|o| !o.preso && o.tile() == pos) {
            let (x, y) = (o.x, o.y - 30.0);
            match o.tipo {
                TipoOggetto::Moneta => {
                    o.preso = true;
                    self.player.monete += 1;
                    self.effetti.suona(Suono::Moneta, 1.0);
                    self.effetti.add_text("+1 moneta", x, y - 30.0, GIALLO);
                    self.effetti.scintille(x, y, GIALLO, 14);
                }
                // il cuore resta a terra finché non serve
                TipoOggetto::Cuore if self.player.vita < self.player.vita_max => {
                    o.preso = true;
                    self.effetti.suona(Suono::Cuore, 0.8);
                    self.player.cura(25, &mut self.effetti);
                    self.effetti.scintille(x, y, Color::from_rgba(240, 70, 90, 255), 14);
                }
                TipoOggetto::Cuore => {}
            }
        }
    }

    /// In furia il boss chiama due pipistrelli lontano dal giocatore.
    fn rinforzi_boss(&mut self) {
        let giocatore = self.player.entita.tile();
        let occupati: Vec<_> = self.bats.iter().filter(|b| b.vivo()).map(|b| b.entita.tile()).collect();
        let mut liberi: Vec<_> = self
            .tiles_per_nemici()
            .into_iter()
            .filter(|&t| distanza(t, giocatore) >= 3 && !occupati.contains(&t))
            .collect();
        mescola(&mut liberi);
        for &(x, y) in liberi.iter().take(2) {
            let bat = Enemy::con_statistiche(x, y, &self.mondo, 40, 8);
            self.effetti
                .scintille(bat.entita.x, bat.entita.y - 30.0, Color::from_rgba(170, 60, 220, 255), 16);
            self.bats.push(bat);
        }
        self.hud.mostra_messaggio("Il boss chiama i suoi pipistrelli!", 4.0);
    }

    // ---------- obiettivi ----------

    pub fn pipistrelli_vivi(&self) -> usize {
        self.bats.iter().filter(|b| b.vivo()).count()
    }

    pub fn obiettivo_completato(&self) -> bool {
        match self.def().obiettivo {
            Obiettivo::ParlaEBevi => !self.player.thirsty,
            Obiettivo::SconfiggiPipistrelli => self.pipistrelli_vivi() == 0,
            Obiettivo::RaccogliMonete => self.monete_rimaste() == 0,
            Obiettivo::SconfiggiBoss => self.boss.as_ref().is_none_or(|b| !b.vivo()),
        }
    }

    pub fn testo_obiettivo(&self) -> String {
        if self.portale.as_ref().is_some_and(|p| p.aperto) {
            return "Entra nel portale".to_string();
        }
        match self.def().obiettivo {
            Obiettivo::ParlaEBevi => {
                if !self.hud.giaparlato {
                    "Parla con Ugo".to_string()
                } else {
                    "Bevi alla fontanella".to_string()
                }
            }
            Obiettivo::SconfiggiPipistrelli => format!("Sconfiggi i pipistrelli ({} rimasti)", self.pipistrelli_vivi()),
            Obiettivo::RaccogliMonete => {
                let totale = self.def().monete;
                format!("Raccogli le monete ({}/{})", totale - self.monete_rimaste(), totale)
            }
            Obiettivo::SconfiggiBoss => {
                if self.obiettivo_completato() {
                    "Boss sconfitto!".to_string()
                } else {
                    "Sconfiggi il boss!".to_string()
                }
            }
        }
    }

    /// Dove punta la freccia gialla.
    pub fn bersaglio_freccia(&self) -> Option<(f32, f32)> {
        if let Some(p) = self.portale.as_ref().filter(|p| p.aperto) {
            return Some(p.cima());
        }
        match self.def().obiettivo {
            Obiettivo::ParlaEBevi if self.hud.giaparlato && self.player.thirsty => {
                let (fx, fy) = self.mondo.trova_fontanella()?;
                let (x, y) = self.mondo.piedi_tile(fx, fy);
                Some((x, y - 52.0))
            }
            Obiettivo::RaccogliMonete => {
                // la moneta più vicina
                let pos = self.player.entita.tile();
                self.oggetti
                    .iter()
                    .filter(|o| o.tipo == TipoOggetto::Moneta && !o.preso)
                    .min_by_key(|o| distanza(o.tile(), pos))
                    .map(|o| (o.x, o.y - 50.0))
            }
            _ => None,
        }
    }

    // ---------- logica ----------

    /// Elabora i tocchi di questo frame. Restituisce (interagisci, pausa).
    pub fn processa_tocchi(&mut self, tocchi: &[(u64, TouchPhase, (f32, f32))]) -> (bool, bool) {
        let azioni = self.touch.processa(tocchi);
        let touch = self.touch.attivo;
        self.menu.touch = touch;
        self.hud.touch = touch;
        self.schermata_morte.touch = touch;
        self.schermata_pausa.touch = touch;
        azioni
    }

    fn cambia_audio(&mut self) {
        self.audio_attivo = !self.audio_attivo;
        self.audio.push(ComandoAudio::Attiva(self.audio_attivo));
        self.schermata_pausa.aggiorna_audio(self.audio_attivo);
    }

    /// Un passo di logica (1/60 di secondo). Restituisce `false` quando il gioco deve chiudersi.
    pub fn update(&mut self, input: &Input) -> bool {
        if input.premuti.m {
            self.cambia_audio();
        }

        match self.stato {
            GameState::Menu => match self.menu.handle_events(input) {
                MenuAction::Start => self.nuova_partita(),
                MenuAction::Esci => return false,
                MenuAction::None => {}
            },
            GameState::Gioco => self.update_gioco(input),
            GameState::Pausa => match self.schermata_pausa.handle_events(input) {
                PausaAction::Riprendi => self.stato = GameState::Gioco,
                PausaAction::Audio => self.cambia_audio(),
                PausaAction::Menu => self.vai_al_menu(),
                PausaAction::None => {}
            },
            GameState::Morto => match self.schermata_morte.handle_events(input) {
                GameOverAction::Riprova => self.riprova(),
                GameOverAction::Menu => self.vai_al_menu(),
                GameOverAction::None => {}
            },
            GameState::Vittoria => {
                if let Some(ref mut w) = self.win {
                    w.update();
                    if w.handle_events(input) {
                        self.vai_al_menu();
                    }
                }
            }
        }

        true
    }

    pub fn interagisci(&mut self) {
        if let Some(ref mut npc) = self.npc {
            if self.player.entita.distanza_tiles(npc.tile_x, npc.tile_y) == 1 && !self.hud.giaparlato {
                self.hud.interagisci_npc(npc, &mut self.player, &mut self.effetti);
                if self.hud.giaparlato {
                    self.hud.mostra_messaggio("Vai alla fontanella e premi E per bere.", 5.0);
                }
                return;
            }
        }
        if self.player.puo_bere(self.hud.giaparlato, &self.mondo) {
            self.player.bevi(&mut self.effetti);
        }
    }

    pub fn attacca_nemici(&mut self) {
        if !self.player.esegui_attacco(&mut self.effetti) {
            return;
        }
        let (px, py) = self.player.entita.tile();
        for bat in &mut self.bats {
            if bat.vivo() && bat.entita.distanza_tiles(px, py) <= 1 {
                bat.take_damage(self.player.danno, &mut self.player, &mut self.effetti);
            }
        }
        if let Some(ref mut boss) = self.boss {
            if boss.vivo() && boss.entita.distanza_tiles(px, py) <= 1 {
                boss.take_damage(self.player.danno, &mut self.effetti);
            }
        }
    }

    /// Tile occupati dai nemici vivi (escluso il pipistrello `tranne`).
    fn tiles_nemici(&self, tranne: Option<usize>, occupati: &mut Vec<(i32, i32)>) {
        occupati.extend(
            self.bats
                .iter()
                .enumerate()
                .filter(|&(i, b)| Some(i) != tranne && b.vivo())
                .map(|(_, b)| b.entita.tile()),
        );
        if let Some(b) = self.boss.as_ref().filter(|b| b.vivo()) {
            occupati.push(b.entita.tile());
        }
    }

    pub fn update_gioco(&mut self, input: &Input) {
        if input.pausa() {
            self.touch.reset();
            self.stato = GameState::Pausa;
            return;
        }

        if input.premuti.e || input.touch_interagisci {
            self.interagisci();
        }

        let attacco_mouse = input.premuti.click && !self.touch.attivo;
        if input.premuti.spazio || attacco_mouse || self.touch.attacco_premuto() {
            self.attacca_nemici();
        }

        // giocatore
        let mut occupati = Vec::with_capacity(self.bats.len() + 2);
        self.tiles_nemici(None, &mut occupati);
        let (passo, direzione) = if input.passo != (0, 0) {
            (input.passo, input.direzione)
        } else {
            (self.touch.passo().unwrap_or((0, 0)), None)
        };
        self.player
            .update(&self.mondo, &occupati, passo, direzione, &mut self.effetti);

        // nemici (fermi durante la tregua iniziale)
        let giocatore = self.player.entita.tile();
        for i in 0..self.bats.len() {
            self.bats[i].anima();
            if self.tregua > 0 || !self.bats[i].vivo() {
                continue;
            }
            occupati.clear();
            occupati.push(giocatore);
            self.tiles_nemici(Some(i), &mut occupati);
            self.bats[i].update(&self.mondo, &mut self.player, &mut self.effetti, &occupati);
        }
        occupati.clear();
        occupati.push(giocatore);
        occupati.extend(self.bats.iter().filter(|b| b.vivo()).map(|b| b.entita.tile()));
        if let Some(ref mut boss) = self.boss {
            boss.anima(&mut self.effetti);
            if self.tregua == 0 {
                boss.update(&self.mondo, &mut self.player, &mut self.effetti, &occupati);
            }
        }
        if self.boss.as_ref().is_some_and(|b| b.appena_infuriato) {
            self.rinforzi_boss();
        }

        for o in &mut self.oggetti {
            o.update();
        }
        self.raccogli_oggetti();
        self.tregua = (self.tregua - 1).max(0);

        self.hud.update(self.npc.as_mut(), &self.player);
        self.banner_timer = (self.banner_timer - 1).max(0);

        // portale
        let completato = self.obiettivo_completato();
        if let Some(ref mut portale) = self.portale {
            portale.update(&mut self.effetti);
            if completato && !portale.aperto {
                portale.apri(&mut self.effetti);
                self.hud
                    .mostra_messaggio("Il portale si e' aperto! Raggiungilo seguendo la freccia.", 6.0);
            }
        }
        self.effetti.update();

        if !self.player.vivo() {
            self.effetti
                .scintille(self.player.entita.x, self.player.entita.y - 30.0, Color::from_rgba(200, 30, 30, 255), 30);
            self.schermata_morte.nuovo_consiglio();
            self.stato = GameState::Morto;
            return;
        }

        let sul_portale = self
            .portale
            .as_ref()
            .is_some_and(|p| p.aperto && self.player.sul_tile(p.tile()));
        if sul_portale {
            if self.livello + 1 < LIVELLI.len() {
                self.carica_livello(self.livello + 1);
            } else {
                self.vittoria();
            }
            return;
        }

        if self.def().obiettivo == Obiettivo::SconfiggiBoss && completato {
            self.timer_vittoria += 1;
            if self.timer_vittoria >= FRAME_PRIMA_DEI_CREDITI {
                self.vittoria();
            }
        }
    }

    fn vittoria(&mut self) {
        self.win = Some(Win::new(self.player.monete));
        self.stato = GameState::Vittoria;
    }

    // ---------- disegno ----------

    pub fn draw(&self, assets: &Assets, mouse: (f32, f32)) {
        match self.stato {
            GameState::Menu => self.menu.draw(mouse, assets),
            GameState::Gioco => self.draw_gioco(assets),
            GameState::Pausa => {
                self.draw_gioco(assets);
                self.schermata_pausa.draw(mouse, assets);
            }
            GameState::Morto => {
                self.draw_gioco(assets);
                self.schermata_morte.draw(mouse, assets);
            }
            GameState::Vittoria => {
                if let Some(ref w) = self.win {
                    w.draw(assets);
                }
            }
        }
    }

    pub fn draw_gioco(&self, assets: &Assets) {
        let sfondo = match self.def().tema {
            crate::livelli::Tema::Arena => &assets.bg2,
            _ => &assets.bg1,
        };
        disegna_copertina(sfondo);
        self.mondo.draw_terreno(assets);

        // tutto ciò che è "alto" si disegna dal fondo verso lo schermo
        enum Alto {
            Oggetto(usize),
            Npc,
            Portale,
            Bat(usize),
            Raccoglibile(usize),
            Boss,
            Player,
        }
        let mut items: Vec<(f32, Alto)> = Vec::with_capacity(self.mondo.oggetti.len() + self.bats.len() + 4);
        items.extend(self.mondo.oggetti.iter().enumerate().map(|(i, o)| (o.profondita, Alto::Oggetto(i))));
        if let Some(ref n) = self.npc {
            items.push((n.profondita(), Alto::Npc));
        }
        if let Some(ref p) = self.portale {
            items.push((p.profondita(), Alto::Portale));
        }
        for (i, b) in self.bats.iter().enumerate().filter(|(_, b)| b.vivo()) {
            items.push((b.entita.profondita(), Alto::Bat(i)));
        }
        for (i, o) in self.oggetti.iter().enumerate().filter(|(_, o)| !o.preso) {
            items.push((o.profondita(), Alto::Raccoglibile(i)));
        }
        if let Some(b) = self.boss.as_ref().filter(|b| b.vivo()) {
            items.push((b.entita.profondita(), Alto::Boss));
        }
        if self.player.vivo() {
            items.push((self.player.entita.profondita(), Alto::Player));
        }
        items.sort_by(|a, b| a.0.total_cmp(&b.0));

        for (_, item) in &items {
            match *item {
                Alto::Oggetto(i) => self.mondo.draw_oggetto(&self.mondo.oggetti[i], assets),
                Alto::Npc => {
                    if let Some(ref n) = self.npc {
                        n.draw(assets);
                    }
                }
                Alto::Portale => {
                    if let Some(ref p) = self.portale {
                        p.draw(assets);
                    }
                }
                Alto::Bat(i) => self.bats[i].draw(assets),
                Alto::Raccoglibile(i) => self.oggetti[i].draw(assets),
                Alto::Boss => {
                    if let Some(ref b) = self.boss {
                        b.draw(assets);
                    }
                }
                Alto::Player => self.player.draw(assets),
            }
        }

        if let Some(ref n) = self.npc {
            if !self.hud.giaparlato && !n.parlando {
                n.draw_avviso(assets);
            }
        }
        self.hud.draw_freccia_obiettivo(self.bersaglio_freccia());
        self.effetti.draw(assets);

        if self.def().obiettivo == Obiettivo::SconfiggiBoss && self.obiettivo_completato() {
            testo_con_ombra(
                &assets.dungeon_font,
                "VITTORIA!",
                LUNGHEZZA / 2.0,
                ALTEZZA / 2.0 - 60.0,
                80,
                GIALLO,
                Ancora::Centro,
            );
        }

        self.draw_banner(assets);

        self.hud.draw(
            &self.mondo,
            &self.player,
            self.npc.as_ref(),
            self.boss.as_ref(),
            &self.testo_obiettivo(),
            LIVELLI.len(),
            assets,
        );

        if self.stato == GameState::Gioco {
            self.touch.draw(assets);
        }
    }

    fn draw_banner(&self, assets: &Assets) {
        if self.banner_timer <= 0 {
            return;
        }
        let t = self.banner_timer as f32;
        let alpha = (t / 30.0).min(((DURATA_BANNER as f32 - t) / 15.0).max(0.0)).min(1.0);
        let y = ALTEZZA / 2.0 - 150.0;
        draw_rectangle(0.0, y - 50.0, LUNGHEZZA, 100.0, Color::new(0.0, 0.0, 0.0, 0.45 * alpha));
        testo_con_ombra(
            &assets.dungeon_font,
            &format!("LIVELLO {}", self.livello + 1),
            LUNGHEZZA / 2.0,
            y - 14.0,
            56,
            Color::new(1.0, 0.33, 0.33, alpha),
            Ancora::Centro,
        );
        testo_con_ombra(
            &assets.dungeon_font,
            self.def().nome,
            LUNGHEZZA / 2.0,
            y + 26.0,
            30,
            Color::new(GIALLO.r, GIALLO.g, GIALLO.b, alpha),
            Ancora::Centro,
        );
    }
}

/// Mescola una lista (Fisher-Yates) con il generatore di macroquad.
fn mescola<T>(lista: &mut [T]) {
    for i in (1..lista.len()).rev() {
        let j = rand::gen_range(0, i + 1);
        lista.swap(i, j);
    }
}
