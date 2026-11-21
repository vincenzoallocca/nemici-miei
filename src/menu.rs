use crate::assets::Assets;
use crate::hud::testo_con_ombra;
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

    pub fn draw(&self, mouse_pos: (f32, f32), assets: &Assets) {
        let sopra = self.rect.contains(vec2(mouse_pos.0, mouse_pos.1));
        let col_bg = if sopra {
            Color::from_rgba(255, 0, 0, 255)
        } else {
            Color::from_rgba(140, 20, 20, 255)
        };

        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, col_bg);
        draw_rectangle_lines(self.rect.x, self.rect.y, self.rect.w, self.rect.h, 3.0, WHITE);

        let font_size = 50;
        let dims = measure_text(&self.testo, Some(&assets.arcade_font), font_size, 1.0);
        let rx = self.rect.x + (self.rect.w - dims.width) / 2.0;
        let ry = self.rect.y + (self.rect.h - dims.height) / 2.0 + dims.offset_y;

        // Shadow
        draw_text_ex(
            &self.testo,
            rx + 2.0,
            ry + 2.0,
            TextParams {
                font: Some(&assets.arcade_font),
                font_size,
                color: BLACK,
                ..Default::default()
            },
        );

        // Main
        draw_text_ex(
            &self.testo,
            rx,
            ry,
            TextParams {
                font: Some(&assets.arcade_font),
                font_size,
                color: WHITE,
                ..Default::default()
            },
        );
    }

    pub fn cliccato(&self, mouse_pos: (f32, f32), pressed: bool) -> bool {
        pressed && self.rect.contains(vec2(mouse_pos.0, mouse_pos.1))
    }
}

pub enum MenuAction {
    Start,
    Esci,
    None,
}

pub struct Menu {
    pub btn_start: Bottone,
    pub btn_esci: Bottone,
    pub touch: bool,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            btn_start: Bottone::new("START", LUNGHEZZA / 2.0, ALTEZZA / 2.0 + 20.0, 220.0, 54.0),
            btn_esci: Bottone::new("ESCI", LUNGHEZZA / 2.0, ALTEZZA / 2.0 + 90.0, 220.0, 54.0),
            touch: false,
        }
    }

    pub fn draw(&self, mouse_pos: (f32, f32), assets: &Assets) {
        draw_texture_ex(
            &assets.bg3,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(LUNGHEZZA, ALTEZZA)),
                ..Default::default()
            },
        );

        testo_con_ombra(
            &assets.dungeon_font,
            "Nemici Miei",
            LUNGHEZZA / 2.0,
            ALTEZZA / 2.0 - 110.0,
            80,
            Color::from_rgba(255, 0, 0, 255),
            "center",
        );

        self.btn_start.draw(mouse_pos, assets);
        self.btn_esci.draw(mouse_pos, assets);

        let righe: &[&str] = if self.touch {
            &[
                "Trascina il dito a sinistra per muoverti",
                "ATK  attacca      E  parla e interagisci",
                "II  pausa",
            ]
        } else {
            &[
                "WASD / Frecce  muoviti",
                "Click / Spazio  attacca",
                "E  parla e interagisci",
                "ESC  pausa      M  audio      F11  schermo intero",
            ]
        };

        let box_h = righe.len() as f32 * 28.0 + 20.0;
        let box_w = 560.0;
        let mut y = ALTEZZA - 40.0 - righe.len() as f32 * 28.0;

        draw_rectangle(
            LUNGHEZZA / 2.0 - box_w / 2.0,
            y - 10.0,
            box_w,
            box_h,
            Color::new(0.0, 0.0, 0.0, 150.0 / 255.0),
        );

        for riga in righe {
            testo_con_ombra(
                &assets.arcade_font,
                riga,
                LUNGHEZZA / 2.0,
                y + 12.0,
                24,
                WHITE,
                "center",
            );
            y += 28.0;
        }
    }

    pub fn handle_events(&mut self, mouse_pos: (f32, f32), clicked: bool) -> MenuAction {
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            return MenuAction::Start;
        }
        if self.btn_start.cliccato(mouse_pos, clicked) {
            return MenuAction::Start;
        }
        if self.btn_esci.cliccato(mouse_pos, clicked) {
            return MenuAction::Esci;
        }
        MenuAction::None
    }
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}
