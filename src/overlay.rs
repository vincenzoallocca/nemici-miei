use crate::assets::Assets;
use crate::hud::testo_con_ombra;
use crate::menu::Bottone;
use crate::settings::{ALTEZZA, CONSIGLI, LUNGHEZZA};
use macroquad::prelude::*;

pub enum GameOverAction {
    Riprova,
    Menu,
    None,
}

pub struct SchermataMorte {
    pub btn_riprova: Bottone,
    pub btn_menu: Bottone,
    pub consiglio_index: usize,
    pub touch: bool,
}

impl SchermataMorte {
    pub fn new() -> Self {
        Self {
            btn_riprova: Bottone::new("RIPROVA", LUNGHEZZA / 2.0, ALTEZZA / 2.0 + 20.0, 220.0, 54.0),
            btn_menu: Bottone::new("MENU", LUNGHEZZA / 2.0, ALTEZZA / 2.0 + 90.0, 220.0, 54.0),
            consiglio_index: 0,
            touch: false,
        }
    }

    pub fn nuovo_consiglio(&mut self) {
        self.consiglio_index = (self.consiglio_index + 1) % CONSIGLI.len();
    }

    pub fn draw(&self, mouse_pos: (f32, f32), assets: &Assets) {
        draw_rectangle(0.0, 0.0, LUNGHEZZA, ALTEZZA, Color::new(0.0, 0.0, 0.0, 150.0 / 255.0));

        testo_con_ombra(
            &assets.dungeon_font,
            "Sei Morto",
            LUNGHEZZA / 2.0,
            ALTEZZA / 2.0 - 110.0,
            80,
            Color::from_rgba(255, 0, 0, 255),
            "center",
        );

        if !self.touch {
            testo_con_ombra(
                &assets.arcade_font,
                "Premi R per riprovare",
                LUNGHEZZA / 2.0,
                ALTEZZA / 2.0 - 55.0,
                26,
                WHITE,
                "center",
            );
        }

        self.btn_riprova.draw(mouse_pos, assets);
        self.btn_menu.draw(mouse_pos, assets);

        testo_con_ombra(
            &assets.arcade_font,
            CONSIGLI[self.consiglio_index],
            LUNGHEZZA / 2.0,
            ALTEZZA / 2.0 + 160.0,
            24,
            Color::from_rgba(250, 210, 60, 255),
            "center",
        );
    }

    pub fn handle_events(&self, mouse_pos: (f32, f32), clicked: bool) -> GameOverAction {
        if is_key_pressed(KeyCode::R) || is_key_pressed(KeyCode::Enter) {
            return GameOverAction::Riprova;
        }
        if self.btn_riprova.cliccato(mouse_pos, clicked) {
            return GameOverAction::Riprova;
        }
        if self.btn_menu.cliccato(mouse_pos, clicked) {
            return GameOverAction::Menu;
        }
        GameOverAction::None
    }
}

impl Default for SchermataMorte {
    fn default() -> Self {
        Self::new()
    }
}

pub enum PausaAction {
    Riprendi,
    Audio,
    Menu,
    None,
}

pub struct SchermataPausa {
    pub btn_riprendi: Bottone,
    pub btn_audio: Bottone,
    pub btn_menu: Bottone,
    pub touch: bool,
}

impl SchermataPausa {
    pub fn new() -> Self {
        Self {
            btn_riprendi: Bottone::new("RIPRENDI", LUNGHEZZA / 2.0, ALTEZZA / 2.0 - 10.0, 260.0, 54.0),
            btn_audio: Bottone::new("AUDIO: SI", LUNGHEZZA / 2.0, ALTEZZA / 2.0 + 55.0, 260.0, 54.0),
            btn_menu: Bottone::new("MENU", LUNGHEZZA / 2.0, ALTEZZA / 2.0 + 120.0, 260.0, 54.0),
            touch: false,
        }
    }

    pub fn aggiorna_audio(&mut self, attivo: bool) {
        self.btn_audio.testo = if attivo {
            "AUDIO: SI".to_string()
        } else {
            "AUDIO: NO".to_string()
        };
    }

    pub fn draw(&self, mouse_pos: (f32, f32), assets: &Assets) {
        draw_rectangle(0.0, 0.0, LUNGHEZZA, ALTEZZA, Color::new(0.0, 0.0, 0.0, 150.0 / 255.0));

        testo_con_ombra(
            &assets.dungeon_font,
            "Pausa",
            LUNGHEZZA / 2.0,
            ALTEZZA / 2.0 - 110.0,
            80,
            Color::from_rgba(255, 0, 0, 255),
            "center",
        );

        if !self.touch {
            testo_con_ombra(
                &assets.arcade_font,
                "ESC per riprendere, M per l'audio",
                LUNGHEZZA / 2.0,
                ALTEZZA / 2.0 - 55.0,
                26,
                WHITE,
                "center",
            );
        }

        self.btn_riprendi.draw(mouse_pos, assets);
        self.btn_audio.draw(mouse_pos, assets);
        self.btn_menu.draw(mouse_pos, assets);
    }

    pub fn handle_events(&self, mouse_pos: (f32, f32), clicked: bool) -> PausaAction {
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P) {
            return PausaAction::Riprendi;
        }
        if self.btn_riprendi.cliccato(mouse_pos, clicked) {
            return PausaAction::Riprendi;
        }
        if self.btn_audio.cliccato(mouse_pos, clicked) {
            return PausaAction::Audio;
        }
        if self.btn_menu.cliccato(mouse_pos, clicked) {
            return PausaAction::Menu;
        }
        PausaAction::None
    }
}

impl Default for SchermataPausa {
    fn default() -> Self {
        Self::new()
    }
}
