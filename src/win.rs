use crate::assets::Assets;
use crate::hud::{testo_con_ombra, Ancora, GIALLO};
use crate::input::Input;
use crate::settings::{ALTEZZA, AUTORI, LUNGHEZZA};
use macroquad::prelude::*;

/// Titoli di coda.
pub struct Win {
    pub credit_offset: f32,
    pub fine_offset: f32,
    pub line_spacing: f32,
    pub credit_speed: f32,
    pub lines: Vec<String>,
}

impl Win {
    pub fn new(monete: i32) -> Self {
        let mut lines = vec![
            "HAI VINTO!".to_string(),
            String::new(),
            format!("Monete raccolte: {}", monete),
            String::new(),
            "Sviluppato da: Rock A' Raso GAME LLC".to_string(),
            String::new(),
        ];
        lines.extend(AUTORI.iter().map(|a| a.to_string()));
        lines.push(String::new());
        lines.push("Grazie per aver giocato!".to_string());

        let line_spacing = 44.0;
        let fine_offset = ALTEZZA / 2.0 - (lines.len() as f32 - 1.0) * line_spacing;

        Self {
            credit_offset: ALTEZZA,
            fine_offset,
            line_spacing,
            credit_speed: 1.0,
            lines,
        }
    }

    pub fn finito(&self) -> bool {
        self.credit_offset <= self.fine_offset
    }

    pub fn update(&mut self) {
        if !self.finito() {
            self.credit_offset -= self.credit_speed;
        }
    }

    pub fn draw(&self, assets: &Assets) {
        draw_rectangle(0.0, 0.0, LUNGHEZZA, ALTEZZA, BLACK);

        for (i, line) in self.lines.iter().enumerate() {
            let y = self.credit_offset + i as f32 * self.line_spacing;
            if y > -50.0 && y < ALTEZZA + 50.0 {
                let col = if i == 0 { GIALLO } else { WHITE };
                testo_con_ombra(&assets.arcade_font, line, LUNGHEZZA / 2.0, y, 32, col, Ancora::Centro);
            }
        }

        // intestazione
        draw_rectangle(0.0, 0.0, LUNGHEZZA, 90.0, BLACK);
        testo_con_ombra(
            &assets.dungeon_font,
            "Crediti",
            LUNGHEZZA / 2.0,
            45.0,
            60,
            Color::from_rgba(255, 0, 0, 255),
            Ancora::Centro,
        );

        if self.finito() {
            testo_con_ombra(
                &assets.arcade_font,
                "Premi INVIO o tocca lo schermo per tornare al menu",
                LUNGHEZZA / 2.0,
                ALTEZZA - 40.0,
                26,
                WHITE,
                Ancora::Centro,
            );
        }
    }

    /// Vero quando si vuole tornare al menu (i tasti saltano i titoli, il click solo alla fine).
    pub fn handle_events(&self, input: &Input) -> bool {
        input.conferma() || input.premuti.esc || (input.premuti.click && self.finito())
    }
}
