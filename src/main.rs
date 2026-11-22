use macroquad::prelude::*;
use macroquad::experimental::coroutines::start_coroutine;
use nemici_miei::assets::{carica_brano, Assets, MusicTrack};
use nemici_miei::game::Gioco;
use nemici_miei::input::{Input, WEB};
use nemici_miei::settings::{imposta_scala_testo, ALTEZZA, DT, LUNGHEZZA, TITOLO};

fn window_conf() -> Conf {
    // solo per lo sviluppo: NEMICI_FINESTRA=1440x1080 per screenshot più grandi
    let (w, h) = std::env::var("NEMICI_FINESTRA")
        .ok()
        .and_then(|v| {
            let (w, h) = v.split_once('x')?;
            Some((w.parse().ok()?, h.parse().ok()?))
        })
        .unwrap_or((LUNGHEZZA as i32, ALTEZZA as i32));
    Conf {
        window_title: TITOLO.to_string(),
        window_width: w,
        window_height: h,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

/// Camera che mostra l'area di gioco 960x720 al centro della finestra, con bande nere ai lati.
/// Dove sta l'area di gioco 960x720 nella finestra (centrata, con bande nere ai lati).
///
/// Attenzione agli schermi ad alta densità (telefoni, Retina, Windows al 150%):
/// `screen_width()` e il mouse sono in pixel logici, mentre il viewport di OpenGL e i
/// tocchi sono in pixel fisici. Senza conversione il gioco finiva piccolo in un angolo.
struct Vista {
    camera: Camera2D,
    /// Pixel logici per unità di gioco.
    scala: f32,
    /// Angolo in alto a sinistra dell'area di gioco, in pixel logici.
    ox: f32,
    oy: f32,
    dpi: f32,
}

impl Vista {
    fn calcola() -> Self {
        let dpi = screen_dpi_scale().max(0.1);
        let (sw, sh) = (screen_width(), screen_height());
        let scala = (sw / LUNGHEZZA).min(sh / ALTEZZA);
        let (vp_w, vp_h) = (LUNGHEZZA * scala, ALTEZZA * scala);
        let (ox, oy) = ((sw - vp_w) / 2.0, (sh - vp_h) / 2.0);
        let fisico = |v: f32| (v * dpi).round() as i32;
        let camera = Camera2D {
            zoom: vec2(2.0 / LUNGHEZZA, 2.0 / ALTEZZA),
            target: vec2(LUNGHEZZA / 2.0, ALTEZZA / 2.0),
            // il viewport è in pixel fisici, con l'origine in basso: centrato, quindi oy va bene
            viewport: Some((fisico(ox), fisico(oy), fisico(vp_w), fisico(vp_h))),
            ..Default::default()
        };
        Self { camera, scala, ox, oy, dpi }
    }

    /// Da pixel logici a coordinate di gioco.
    fn da_logico(&self, p: Vec2) -> (f32, f32) {
        ((p.x - self.ox) / self.scala, (p.y - self.oy) / self.scala)
    }

    /// Da pixel fisici (tocchi) a coordinate di gioco.
    fn da_fisico(&self, p: Vec2) -> (f32, f32) {
        self.da_logico(p / self.dpi)
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand(macroquad::miniquad::date::now().to_bits());

    // la musica parte a scaricarsi subito, in parallelo con immagini ed effetti sonori
    let mut brani = vec![
        start_coroutine(carica_brano(MusicTrack::Soundtrack, true)),
        start_coroutine(carica_brano(MusicTrack::Boss, false)),
    ];
    let mut assets = Assets::load().await;
    assets.prepara_testo();
    let mut gioco = Gioco::new();
    let mut input = Input::default();
    let mut accumulatore = 0.0f32;
    let mut schermo_intero = false;
    let mut cattura = Cattura::da_ambiente(&mut gioco);

    loop {
        let vista = Vista::calcola();
        if imposta_scala_testo(vista.scala * vista.dpi) {
            // nuova risoluzione: i caratteri vanno preparati prima di disegnare
            assets.prepara_testo();
        }

        // input di questo frame, in coordinate di gioco
        input.unisci(Input::leggi(vista.da_logico(mouse_position().into())));
        let tocchi: Vec<_> = touches()
            .iter()
            .map(|t| (t.id, t.phase, vista.da_fisico(t.position)))
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

        for comando in gioco.audio.drain(..) {
            assets.esegui(comando);
        }
        for (suono, volume) in gioco.effetti.suoni.drain(..) {
            assets.play(suono, volume);
        }
        // dopo i comandi: il gioco ha già chiesto quale brano vuole
        brani.retain(|c| match c.retrieve() {
            Some(caricato) => {
                assets.imposta_brano(caricato);
                false
            }
            None => true,
        });

        clear_background(BLACK);
        set_camera(&vista.camera);
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
/// `NEMICI_POSIZIONE=8,7` mette l'eroe su quel tile.
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
            let posizione = std::env::var("NEMICI_POSIZIONE").ok().and_then(|v| {
                let (x, y) = v.split_once(',')?;
                Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
            });
            if let Some((x, y)) = posizione {
                gioco.player.entita.piazza(x, y, &gioco.mondo);
                gioco.player.entita.direction = nemici_miei::assets::Direction::Left;
            }
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
