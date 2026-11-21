use crate::assets::Assets;
use crate::settings::{ALTEZZA, GRANDEZZA_TILES, HUD_HEIGHT, LUNGHEZZA};
use macroquad::prelude::*;

pub const PASSI: [(i32, i32); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

pub fn vettore_schermo(dx: i32, dy: i32) -> (f32, f32) {
    (
        (dx - dy) as f32 * GRANDEZZA_TILES / 2.0,
        (dx + dy) as f32 * GRANDEZZA_TILES / 4.0,
    )
}

pub struct Pulsante {
    pub centro: (f32, f32),
    pub raggio: f32,
    pub etichetta: &'static str,
    pub premuto: bool,
    pub touch_id: Option<u64>,
}

impl Pulsante {
    pub fn new(centro: (f32, f32), raggio: f32, etichetta: &'static str) -> Self {
        Self {
            centro,
            raggio,
            etichetta,
            premuto: false,
            touch_id: None,
        }
    }

    pub fn contiene(&self, pos: (f32, f32)) -> bool {
        let dx = pos.0 - self.centro.0;
        let dy = pos.1 - self.centro.1;
        (dx * dx + dy * dy).sqrt() <= self.raggio * 1.15
    }

    pub fn draw(&self, assets: &Assets) {
        let alpha = if self.premuto { 150.0 / 255.0 } else { 80.0 / 255.0 };
        draw_circle(self.centro.0, self.centro.1, self.raggio, Color::new(1.0, 1.0, 1.0, alpha));
        draw_circle_lines(self.centro.0, self.centro.1, self.raggio, 3.0, Color::new(0.0, 0.0, 0.0, 160.0 / 255.0));

        let font_size = 30;
        let dims = measure_text(self.etichetta, Some(&assets.arcade_font), font_size, 1.0);
        draw_text_ex(
            self.etichetta,
            self.centro.0 - dims.width / 2.0,
            self.centro.1 + dims.offset_y / 2.0,
            TextParams {
                font: Some(&assets.arcade_font),
                font_size,
                color: Color::from_rgba(20, 20, 20, 255),
                ..Default::default()
            },
        );
    }
}

pub struct ControlliTouch {
    pub attivo: bool,
    pub attacco: Pulsante,
    pub interagisci: Pulsante,
    pub pausa: Pulsante,
    pub joy_touch_id: Option<u64>,
    pub joy_centro: (f32, f32),
    pub joy_pos: (f32, f32),
}

pub const RAGGIO_JOYSTICK: f32 = 70.0;
pub const ZONA_MORTA: f32 = 18.0;

impl ControlliTouch {
    pub fn new() -> Self {
        let base = ALTEZZA - HUD_HEIGHT;
        let attacco = Pulsante::new((LUNGHEZZA - 105.0, base - 95.0), 62.0, "ATK");
        let interagisci = Pulsante::new((LUNGHEZZA - 235.0, base - 55.0), 44.0, "E");
        let pausa = Pulsante::new((LUNGHEZZA - 38.0, 40.0), 26.0, "II");

        Self {
            attivo: false,
            attacco,
            interagisci,
            pausa,
            joy_touch_id: None,
            joy_centro: (130.0, base - 110.0),
            joy_pos: (130.0, base - 110.0),
        }
    }

    pub fn reset(&mut self) {
        self.joy_touch_id = None;
        self.attacco.premuto = false;
        self.attacco.touch_id = None;
        self.interagisci.premuto = false;
        self.interagisci.touch_id = None;
        self.pausa.premuto = false;
        self.pausa.touch_id = None;
    }

    pub fn handle_input(&mut self, _mouse_world: (f32, f32)) -> (bool, bool, bool) {
        let mut azione_attacca = false;
        let mut azione_interagisci = false;
        let mut azione_pausa = false;

        let cur_touches = touches();
        if !cur_touches.is_empty() {
            self.attivo = true;
        }

        for t in &cur_touches {
            let pos = (t.position.x, t.position.y);
            match t.phase {
                TouchPhase::Started => {
                    if self.interagisci.touch_id.is_none() && self.interagisci.contiene(pos) {
                        self.interagisci.touch_id = Some(t.id);
                        self.interagisci.premuto = true;
                        azione_interagisci = true;
                    } else if self.pausa.touch_id.is_none() && self.pausa.contiene(pos) {
                        self.pausa.touch_id = Some(t.id);
                        self.pausa.premuto = true;
                        azione_pausa = true;
                    } else if self.attacco.touch_id.is_none() && self.attacco.contiene(pos) {
                        self.attacco.touch_id = Some(t.id);
                        self.attacco.premuto = true;
                    } else if self.joy_touch_id.is_none()
                        && pos.0 < LUNGHEZZA * 0.55
                        && pos.1 < ALTEZZA - HUD_HEIGHT
                    {
                        self.joy_touch_id = Some(t.id);
                        self.joy_centro = pos;
                        self.joy_pos = pos;
                    }
                }
                TouchPhase::Moved | TouchPhase::Stationary => {
                    if Some(t.id) == self.joy_touch_id {
                        self.joy_pos = pos;
                    }
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    if Some(t.id) == self.joy_touch_id {
                        self.joy_touch_id = None;
                    }
                    if Some(t.id) == self.attacco.touch_id {
                        self.attacco.touch_id = None;
                        self.attacco.premuto = false;
                    }
                    if Some(t.id) == self.interagisci.touch_id {
                        self.interagisci.touch_id = None;
                        self.interagisci.premuto = false;
                    }
                    if Some(t.id) == self.pausa.touch_id {
                        self.pausa.touch_id = None;
                        self.pausa.premuto = false;
                    }
                }
            }
        }

        if self.attacco.premuto {
            azione_attacca = true;
        }

        (azione_attacca, azione_interagisci, azione_pausa)
    }

    pub fn passo(&self) -> Option<(i32, i32)> {
        self.joy_touch_id?;
        let vx = self.joy_pos.0 - self.joy_centro.0;
        let vy = self.joy_pos.1 - self.joy_centro.1;
        let dist = (vx * vx + vy * vy).sqrt();
        if dist < ZONA_MORTA {
            return None;
        }

        let mut best_passo = None;
        let mut best_score = f32::NEG_INFINITY;

        for p in &PASSI {
            let (sx, sy) = vettore_schermo(p.0, p.1);
            let s_len = (sx * sx + sy * sy).sqrt();
            let score = (sx * vx + sy * vy) / s_len;
            if score > best_score {
                best_score = score;
                best_passo = Some(*p);
            }
        }

        best_passo
    }

    pub fn draw(&self, assets: &Assets) {
        if !self.attivo {
            return;
        }

        let centro = if self.joy_touch_id.is_some() {
            self.joy_centro
        } else {
            (130.0, ALTEZZA - HUD_HEIGHT - 110.0)
        };

        // Joystick base
        draw_circle(centro.0, centro.1, RAGGIO_JOYSTICK, Color::new(1.0, 1.0, 1.0, 60.0 / 255.0));
        draw_circle_lines(centro.0, centro.1, RAGGIO_JOYSTICK, 3.0, Color::new(0.0, 0.0, 0.0, 120.0 / 255.0));

        // Knob
        let mut pomello = centro;
        if self.joy_touch_id.is_some() {
            let vx = self.joy_pos.0 - centro.0;
            let vy = self.joy_pos.1 - centro.1;
            let d = (vx * vx + vy * vy).sqrt();
            if d > RAGGIO_JOYSTICK {
                pomello = (centro.0 + vx / d * RAGGIO_JOYSTICK, centro.1 + vy / d * RAGGIO_JOYSTICK);
            } else {
                pomello = (centro.0 + vx, centro.1 + vy);
            }
        }
        draw_circle(pomello.0, pomello.1, 28.0, WHITE);
        draw_circle_lines(pomello.0, pomello.1, 28.0, 3.0, BLACK);

        // Buttons
        self.attacco.draw(assets);
        self.interagisci.draw(assets);
        self.pausa.draw(assets);
    }
}

impl Default for ControlliTouch {
    fn default() -> Self {
        Self::new()
    }
}
