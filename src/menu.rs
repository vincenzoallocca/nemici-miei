use crate::assets::Assets;
use crate::hud::{testo_con_ombra, Ancora};
use crate::input::{Input, NOME_TASTO_PAUSA, WEB};
use crate::settings::{ALTEZZA, LUNGHEZZA};
use macroquad::prelude::*;

pub struct Bottone {
    pub testo: String,
    pub rect: Rect,
}

impl Bottone {
    pub fn new(testo: &str, cx: f32, cy: f32, w: f32, h: f32) -> Self {
        Self {
            testo: testo.to_string(),
            rect: Rect::new(cx - w / 2.0, cy - h / 2.0, w, h),
        }
    }

    pub fn sopra(&self, mouse_pos: (f32, f32)) -> bool {
        self.rect.contains(vec2(mouse_pos.0, mouse_pos.1))
    }

    pub fn draw(&self, mouse_pos: (f32, f32), assets: &Assets) {
        let r = self.rect;
        let col_bg = if self.sopra(mouse_pos) {
            Color::from_rgba(255, 0, 0, 255)
        } else {
            Color::from_rgba(140, 20, 20, 255)
        };
        // ombra, corpo e bordo
        draw_rectangle(r.x + 4.0, r.y + 4.0, r.w, r.h, Color::new(0.0, 0.0, 0.0, 0.5));
        draw_rectangle(r.x, r.y, r.w, r.h, col_bg);
        draw_rectangle(r.x, r.y, r.w, 4.0, Color::new(1.0, 1.0, 1.0, 0.2));
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 3.0, WHITE);

        testo_con_ombra(
            &assets.arcade_font,
            &self.testo,
            r.x + r.w / 2.0,
            r.y + r.h / 2.0,
            50,
            WHITE,
            Ancora::Centro,
        );
    }

    pub fn cliccato(&self, input: &Input) -> bool {
        input.premuti.click && self.sopra(input.mouse)
    }
}

/// Disegna un'immagine a tutto schermo mantenendo le proporzioni (ritaglia i bordi in eccesso).
pub fn disegna_copertina(tex: &Texture2D) {
    let (tw, th) = (tex.width(), tex.height());
    let scala = (LUNGHEZZA / tw).max(ALTEZZA / th);
    let (sw, sh) = (LUNGHEZZA / scala, ALTEZZA / scala);
    draw_texture_ex(
        tex,
        0.0,
        0.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(LUNGHEZZA, ALTEZZA)),
            source: Some(Rect::new((tw - sw) / 2.0, (th - sh) / 2.0, sw, sh)),
            ..Default::default()
        },
    );
}

pub enum MenuAction {
    Start,
    Esci,
    None,
}

pub struct Menu {
    pub btn_start: Bottone,
    pub btn_esci: Option<Bottone>,
    pub touch: bool,
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}

impl Menu {
    pub fn new() -> Self {
        // nel browser non si può chiudere la finestra: niente "ESCI"
        let (btn_start, btn_esci) = if WEB {
            (Bottone::new("START", LUNGHEZZA / 2.0, ALTEZZA / 2.0 + 40.0, 220.0, 54.0), None)
        } else {
            (
                Bottone::new("START", LUNGHEZZA / 2.0, ALTEZZA / 2.0 + 20.0, 220.0, 54.0),
                Some(Bottone::new("ESCI", LUNGHEZZA / 2.0, ALTEZZA / 2.0 + 90.0, 220.0, 54.0)),
            )
        };
        Self {
            btn_start,
            btn_esci,
            touch: false,
        }
    }

    pub fn draw(&self, mouse_pos: (f32, f32), assets: &Assets) {
        disegna_copertina(&assets.bg3);
        // velatura per far risaltare titolo e pulsanti
        draw_rectangle(0.0, 0.0, LUNGHEZZA, ALTEZZA, Color::new(0.0, 0.0, 0.0, 0.25));

        let ondeggio = ((get_time() as f32 * 2.0).sin() * 4.0).round();
        testo_con_ombra(
            &assets.dungeon_font,
            "Nemici Miei",
            LUNGHEZZA / 2.0,
            ALTEZZA / 2.0 - 110.0 + ondeggio,
            90,
            Color::from_rgba(255, 30, 30, 255),
            Ancora::Centro,
        );

        self.btn_start.draw(mouse_pos, assets);
        if let Some(ref b) = self.btn_esci {
            b.draw(mouse_pos, assets);
        }

        let pausa = format!("{}  pausa      M  audio", NOME_TASTO_PAUSA);
        let righe: Vec<&str> = if self.touch {
            vec![
                "Trascina il dito a sinistra per muoverti",
                "ATK  attacca      E  parla e interagisci",
                "II  pausa",
            ]
        } else if WEB {
            vec!["WASD / Frecce  muoviti", "Click / Spazio  attacca", "E  parla e interagisci", &pausa]
        } else {
            vec![
                "WASD / Frecce  muoviti",
                "Click / Spazio  attacca",
                "E  parla e interagisci",
                "ESC  pausa      M  audio      F11  schermo intero",
            ]
        };

        let box_h = righe.len() as f32 * 28.0 + 20.0;
        let box_w = 600.0;
        let mut y = ALTEZZA - 40.0 - righe.len() as f32 * 28.0;

        draw_rectangle(
            LUNGHEZZA / 2.0 - box_w / 2.0,
            y - 10.0,
            box_w,
            box_h,
            Color::new(0.0, 0.0, 0.0, 150.0 / 255.0),
        );

        for riga in righe {
            testo_con_ombra(&assets.arcade_font, riga, LUNGHEZZA / 2.0, y + 12.0, 24, WHITE, Ancora::Centro);
            y += 28.0;
        }
    }

    pub fn handle_events(&mut self, input: &Input) -> MenuAction {
        if input.conferma() || self.btn_start.cliccato(input) {
            return MenuAction::Start;
        }
        if self.btn_esci.as_ref().is_some_and(|b| b.cliccato(input)) {
            return MenuAction::Esci;
        }
        MenuAction::None
    }
}
