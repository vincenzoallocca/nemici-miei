use crate::assets::Assets;
use crate::hud::testo_con_ombra;
use crate::settings::{ALTEZZA, LUNGHEZZA};
use macroquad::prelude::*;

pub struct Win {
    pub credit_offset: f32,
    pub fine_offset: f32,
    pub line_spacing: f32,
    pub credit_speed: f32,
    pub lines: Vec<String>,
}

impl Win {
    pub fn new(monete: i32) -> Self {
        let lines = vec![
            "HAI VINTO!".to_string(),
            "".to_string(),
            format!("Monete raccolte: {}", monete),
            "".to_string(),
            "Sviluppato da: Rock A' Raso GAME LLC".to_string(),
            "".to_string(),
            "Allocca Vincenzo".to_string(),
            "Cerciello Antonio".to_string(),
            "Cestie' Augusto".to_string(),
            "Pellegrino Andrea".to_string(),
            "Prisco Carmine".to_string(),
            "Sdino Raffaele".to_string(),
            "".to_string(),
            "Grazie per aver giocato!".to_string(),
        ];

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
        clear_background(BLACK);

        for (i, line) in self.lines.iter().enumerate() {
            let y = self.credit_offset + i as f32 * self.line_spacing;
            if y > -50.0 && y < ALTEZZA + 50.0 {
                let col = if line == "HAI VINTO!" {
                    Color::from_rgba(250, 210, 60, 255)
                } else {
                    WHITE
                };
                let font_size = 32;
                let dims = measure_text(line, Some(&assets.arcade_font), font_size, 1.0);
                draw_text_ex(
                    line,
                    LUNGHEZZA / 2.0 - dims.width / 2.0,
                    y - dims.height / 2.0 + dims.offset_y,
                    TextParams {
                        font: Some(&assets.arcade_font),
                        font_size,
                        color: col,
                        ..Default::default()
                    },
                );
            }
        }

        // Header banner
        draw_rectangle(0.0, 0.0, LUNGHEZZA, 90.0, BLACK);
        testo_con_ombra(
            &assets.dungeon_font,
            "Crediti",
            LUNGHEZZA / 2.0,
            45.0,
            60,
            Color::from_rgba(255, 0, 0, 255),
            "center",
        );

        if self.finito() {
            testo_con_ombra(
                &assets.arcade_font,
                "Premi INVIO o tocca lo schermo per tornare al menu",
                LUNGHEZZA / 2.0,
                ALTEZZA - 40.0,
                26,
                WHITE,
                "center",
            );
        }
    }

    pub fn handle_events(&self, clicked: bool) -> bool {
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Escape) {
            return true;
        }
        if clicked && self.finito() {
            return true;
        }
        false
    }
}
