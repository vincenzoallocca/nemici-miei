//! Input raccolto a ogni frame disegnato e consumato dai passi di logica a 60 Hz.
//!
//! La logica può girare 0, 1 o più volte per frame (dipende dal monitor): i tasti premuti
//! restano in attesa finché un passo di logica non li usa, così nessuna pressione va persa
//! e nessuna viene contata due volte.

use crate::assets::Direction;
use macroquad::prelude::*;

/// Nel browser ESC fa uscire dallo schermo intero: lì la pausa è su P.
pub const WEB: bool = cfg!(target_arch = "wasm32");
pub const NOME_TASTO_PAUSA: &str = if WEB { "P" } else { "ESC" };

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Tasti {
    pub invio: bool,
    pub spazio: bool,
    pub esc: bool,
    pub p: bool,
    pub e: bool,
    pub r: bool,
    pub m: bool,
    pub click: bool,
}

impl Tasti {
    fn unisci(&mut self, altri: Tasti) {
        self.invio |= altri.invio;
        self.spazio |= altri.spazio;
        self.esc |= altri.esc;
        self.p |= altri.p;
        self.e |= altri.e;
        self.r |= altri.r;
        self.m |= altri.m;
        self.click |= altri.click;
    }
}

#[derive(Clone, Default, Debug)]
pub struct Input {
    /// Passo richiesto sulla griglia con i tasti tenuti premuti.
    pub passo: (i32, i32),
    /// Verso dove guardare (l'ultimo tasto di movimento).
    pub direzione: Option<Direction>,
    /// Mouse in coordinate di gioco (960x720).
    pub mouse: (f32, f32),
    /// Premuti dall'ultimo passo di logica.
    pub premuti: Tasti,
    pub touch_interagisci: bool,
    pub touch_pausa: bool,
    /// Il passo viene da un tasto appena premuto e già rilasciato: va tenuto finché non lo
    /// usa un passo di logica, altrimenti un tocco rapidissimo andrebbe perso.
    pub passo_da_tocco_rapido: bool,
}

impl Input {
    /// Legge tastiera e mouse di questo frame.
    pub fn leggi(mouse: (f32, f32)) -> Self {
        let giu = |a: KeyCode, b: KeyCode| {
            is_key_down(a) || is_key_down(b) || is_key_pressed(a) || is_key_pressed(b)
        };
        let tenuto = [KeyCode::A, KeyCode::D, KeyCode::W, KeyCode::S]
            .into_iter()
            .chain([KeyCode::Left, KeyCode::Right, KeyCode::Up, KeyCode::Down])
            .any(is_key_down);
        let mut passo = (0, 0);
        let mut direzione = None;
        if giu(KeyCode::A, KeyCode::Left) {
            passo.0 -= 1;
            direzione = Some(Direction::Left);
        }
        if giu(KeyCode::D, KeyCode::Right) {
            passo.0 += 1;
            direzione = Some(Direction::Right);
        }
        if giu(KeyCode::W, KeyCode::Up) {
            passo.1 -= 1;
            direzione = Some(Direction::Up);
        }
        if giu(KeyCode::S, KeyCode::Down) {
            passo.1 += 1;
            direzione = Some(Direction::Down);
        }

        Self {
            passo,
            direzione,
            mouse,
            premuti: Tasti {
                invio: is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter),
                spazio: is_key_pressed(KeyCode::Space),
                esc: is_key_pressed(KeyCode::Escape),
                p: is_key_pressed(KeyCode::P),
                e: is_key_pressed(KeyCode::E),
                r: is_key_pressed(KeyCode::R),
                m: is_key_pressed(KeyCode::M),
                click: is_mouse_button_pressed(MouseButton::Left),
            },
            touch_interagisci: false,
            touch_pausa: false,
            passo_da_tocco_rapido: passo != (0, 0) && !tenuto,
        }
    }

    /// Aggiunge l'input di un nuovo frame a quello non ancora consumato.
    pub fn unisci(&mut self, nuovo: Input) {
        if nuovo.passo != (0, 0) || !self.passo_da_tocco_rapido {
            self.passo = nuovo.passo;
            self.direzione = nuovo.direzione;
            self.passo_da_tocco_rapido = nuovo.passo_da_tocco_rapido;
        }
        self.mouse = nuovo.mouse;
        self.premuti.unisci(nuovo.premuti);
        self.touch_interagisci |= nuovo.touch_interagisci;
        self.touch_pausa |= nuovo.touch_pausa;
    }

    /// Dopo un passo di logica: i tasti "premuti" sono stati usati.
    pub fn consuma(&mut self) {
        if self.passo_da_tocco_rapido {
            self.passo = (0, 0);
            self.direzione = None;
            self.passo_da_tocco_rapido = false;
        }
        self.premuti = Tasti::default();
        self.touch_interagisci = false;
        self.touch_pausa = false;
    }

    pub fn pausa(&self) -> bool {
        self.premuti.esc || self.premuti.p || self.touch_pausa
    }

    pub fn conferma(&self) -> bool {
        self.premuti.invio || self.premuti.spazio
    }
}
