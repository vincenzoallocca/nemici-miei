use macroquad::prelude::*;
use macroquad::experimental::coroutines::start_coroutine;
use nemici_miei::assets::{Assets, Musica};
use nemici_miei::game::Gioco;
use nemici_miei::input::{Input, WEB};
use nemici_miei::settings::{imposta_scala_testo, ALTEZZA, DT, LUNGHEZZA, TITOLO};

fn window_conf() -> Conf {
    Conf {
        window_title: TITOLO.to_string(),
        window_width: LUNGHEZZA as i32,
        window_height: ALTEZZA as i32,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

/// Camera che mostra l'area di gioco 960x720 al centro della finestra, con bande nere ai lati.
fn camera_gioco() -> (Camera2D, f32) {
    let sw = screen_width();
    let sh = screen_height();
    let scala = (sw / LUNGHEZZA).min(sh / ALTEZZA);
    let vp_w = (LUNGHEZZA * scala).round();
    let vp_h = (ALTEZZA * scala).round();
    let ox = ((sw - vp_w) / 2.0).round();
    let oy = ((sh - vp_h) / 2.0).round();

    let camera = Camera2D {
        zoom: vec2(2.0 / LUNGHEZZA, 2.0 / ALTEZZA),
        target: vec2(LUNGHEZZA / 2.0, ALTEZZA / 2.0),
        viewport: Some((ox as i32, oy as i32, vp_w as i32, vp_h as i32)),
        ..Default::default()
    };
    (camera, scala)
}

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand(macroquad::miniquad::date::now().to_bits());

    let mut assets = Assets::load().await;
    let mut musica = Some(start_coroutine(Musica::carica()));
    let mut gioco = Gioco::new();
    let mut input = Input::default();
    let mut accumulatore = 0.0f32;
    let mut schermo_intero = false;
    let mut cattura = Cattura::da_ambiente(&mut gioco);

    loop {
        let (camera, scala) = camera_gioco();
        imposta_scala_testo(scala * screen_dpi_scale().max(1.0));

        // input di questo frame, in coordinate di gioco
        let in_gioco = |p: Vec2| {
            let w = camera.screen_to_world(p);
            (w.x, w.y)
        };
        input.unisci(Input::leggi(in_gioco(mouse_position().into())));
        let tocchi: Vec<_> = touches()
            .iter()
            .map(|t| (t.id, t.phase, in_gioco(t.position)))
            .collect();
        let (interagisci, pausa) = gioco.processa_tocchi(&tocchi);
        input.touch_interagisci |= interagisci;
        input.touch_pausa |= pausa;

        if !WEB && is_key_pressed(KeyCode::F11) {
            schermo_intero = !schermo_intero;
            set_fullscreen(schermo_intero);
        }

        // logica a passo fisso: stessa velocità a 30, 60 o 144 Hz
        accumulatore += get_frame_time().min(0.25);
        let mut passi = 0;
        while accumulatore >= DT && passi < 5 {
            if !gioco.update(&input) {
                return;
            }
            input.consuma();
            accumulatore -= DT;
            passi += 1;
        }
        if passi == 5 {
            // la macchina non sta al passo: meglio rallentare che "saltare"
            accumulatore = 0.0;
        }

        if let Some(m) = musica.as_ref().and_then(|c| c.retrieve()) {
            assets.imposta_musica(m);
            musica = None;
        }
        for comando in gioco.audio.drain(..) {
            assets.esegui(comando);
        }
        for (suono, volume) in gioco.effetti.suoni.drain(..) {
            assets.play(suono, volume);
        }

        clear_background(BLACK);
        set_camera(&camera);
        gioco.draw(&assets, input.mouse);
        set_default_camera();

        if let Some(ref mut c) = cattura {
            if c.frame_dopo() {
                return;
            }
        }

        next_frame().await;
    }
}

/// Solo per lo sviluppo (desktop): `NEMICI_LIVELLO=3 NEMICI_SCREENSHOT=foto.png` avvia
/// direttamente il livello 3, salva uno screenshot dopo `NEMICI_FRAMES` frame (90) ed esce.
struct Cattura {
    file: String,
    frame: u32,
}

impl Cattura {
    fn da_ambiente(gioco: &mut Gioco) -> Option<Self> {
        if WEB {
            return None;
        }
        if let Some(n) = std::env::var("NEMICI_LIVELLO").ok().and_then(|v| v.parse::<usize>().ok()) {
            gioco.nuova_partita();
            let n = n.clamp(1, nemici_miei::livelli::LIVELLI.len());
            gioco.carica_livello(n - 1);
        }
        let file = std::env::var("NEMICI_SCREENSHOT").ok()?;
        let frame = std::env::var("NEMICI_FRAMES").ok().and_then(|v| v.parse().ok()).unwrap_or(90);
        Some(Self { file, frame })
    }

    fn frame_dopo(&mut self) -> bool {
        self.frame = self.frame.saturating_sub(1);
        if self.frame > 0 {
            return false;
        }
        get_screen_data().export_png(&self.file);
        true
    }
}
