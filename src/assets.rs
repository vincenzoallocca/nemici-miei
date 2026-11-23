use macroquad::audio::{load_sound, play_sound, stop_sound, PlaySoundParams, Sound};
use macroquad::experimental::coroutines::{start_coroutine, Coroutine};
use std::collections::HashMap;
use crate::sfondo::Sfondi;
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Direction {
    Down,
    Up,
    Left,
    Right,
}

impl Direction {
    pub fn folder(&self) -> &'static str {
        match self {
            Direction::Down => "[DOWN]",
            Direction::Up => "[UP]",
            Direction::Left => "[LEFT]",
            Direction::Right => "[RIGHT]",
        }
    }

    pub fn char_code(&self) -> char {
        match self {
            Direction::Down => 'd',
            Direction::Up => 'u',
            Direction::Left => 'l',
            Direction::Right => 'r',
        }
    }
}

pub struct AnimFrames {
    pub down: Vec<Texture2D>,
    pub up: Vec<Texture2D>,
    pub left: Vec<Texture2D>,
    pub right: Vec<Texture2D>,
}

impl AnimFrames {
    pub fn get(&self, dir: Direction) -> &[Texture2D] {
        match dir {
            Direction::Down => &self.down,
            Direction::Up => &self.up,
            Direction::Left => &self.left,
            Direction::Right => &self.right,
        }
    }
}

// ---------- audio ----------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MusicTrack {
    Soundtrack,
    Boss,
    None,
}

/// Effetti sonori. La logica di gioco li mette in coda (vedi `Effetti::suona`) e il ciclo
/// principale li suona: così la logica non dipende dall'audio e si può testare senza finestra.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Suono {
    Erba,
    Acqua,
    Pietra,
    Moneta,
    Cuore,
    Attacco,
    DannoGiocatore,
    MortePipistrello,
    MorteBoss,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ComandoAudio {
    Suono(Suono, f32),
    Musica(MusicTrack, f32),
    Attiva(bool),
}

// ---------- percorsi ----------

#[cfg(not(target_arch = "wasm32"))]
pub fn asset_path(rel: &str) -> String {
    use std::path::Path;
    let p = Path::new("assets").join(rel);
    if p.exists() {
        return p.to_string_lossy().to_string();
    }
    // avviato da target/release o da un'altra cartella: cerca accanto all'eseguibile
    if let Ok(exe) = std::env::current_exe() {
        for dir in exe.ancestors().skip(1).take(3) {
            let p = dir.join("assets").join(rel);
            if p.exists() {
                return p.to_string_lossy().to_string();
            }
        }
    }
    format!("assets/{}", rel)
}

#[cfg(target_arch = "wasm32")]
pub fn asset_path(rel: &str) -> String {
    format!("assets/{}", rel)
}

// ---------- caricamento ----------

/// Barra di caricamento.
fn disegna_caricamento(progresso: f32) {
    set_default_camera();
    clear_background(BLACK);
    let (w, h) = (screen_width(), screen_height());
    let bw = (w * 0.5).min(480.0);
    let bx = (w - bw) / 2.0;
    let by = h / 2.0;
    let testo = "Caricamento...";
    let dim = measure_text(testo, None, 32, 1.0);
    draw_text(testo, (w - dim.width) / 2.0, by - 24.0, 32.0, WHITE);
    draw_rectangle(bx, by, bw, 16.0, Color::from_rgba(40, 40, 40, 255));
    draw_rectangle(bx, by, bw * progresso.clamp(0.0, 1.0), 16.0, Color::from_rgba(250, 210, 60, 255));
    draw_rectangle_lines(bx, by, bw, 16.0, 2.0, WHITE);
}

/// Aspetta un gruppo di coroutine mostrando la barra di caricamento.
async fn aspetta_tutte<T: 'static + std::any::Any>(lavori: Vec<Coroutine<T>>, fatti_prima: usize, totale: usize) -> Vec<T> {
    let mut risultati: Vec<Option<T>> = (0..lavori.len()).map(|_| None).collect();
    loop {
        for (lavoro, posto) in lavori.iter().zip(risultati.iter_mut()) {
            if posto.is_none() {
                *posto = lavoro.retrieve();
            }
        }
        let fatti = risultati.iter().filter(|r| r.is_some()).count();
        if fatti == risultati.len() {
            return risultati.into_iter().flatten().collect();
        }
        disegna_caricamento((fatti_prima + fatti) as f32 / totale as f32);
        next_frame().await;
    }
}

/// Come `aspetta_tutte`, ma dopo `limite` secondi smette di aspettare: chi non ha finito
/// resta `None`. Per gli effetti sonori: se un browser non riesce a decodificarne uno,
/// macroquad aspetterebbe per sempre e il gioco non partirebbe mai.
async fn aspetta_con_limite<T: 'static + std::any::Any>(
    lavori: Vec<Coroutine<T>>,
    limite: f64,
    fatti_prima: usize,
    totale: usize,
) -> Vec<Option<T>> {
    let mut risultati: Vec<Option<T>> = (0..lavori.len()).map(|_| None).collect();
    let inizio = get_time();
    loop {
        for (lavoro, posto) in lavori.iter().zip(risultati.iter_mut()) {
            if posto.is_none() {
                *posto = lavoro.retrieve();
            }
        }
        let fatti = risultati.iter().filter(|r| r.is_some()).count();
        if fatti == risultati.len() || get_time() - inizio > limite {
            return risultati;
        }
        disegna_caricamento((fatti_prima + fatti) as f32 / totale as f32);
        next_frame().await;
    }
}

/// Tutti i file scaricati, per percorso relativo ad assets/.
///
/// Il download avviene tutto in parallelo: nel browser scaricare un centinaio di file
/// uno alla volta (aspettando un frame per ciascuno) richiedeva secondi.
struct Archivio {
    file: HashMap<String, Vec<u8>>,
}

/// Scarica un file riprovando qualche volta: sul telefono, con una connessione instabile, una
/// richiesta su cento può fallire, e prima bastava quella per bloccare tutto il gioco.
async fn scarica_con_ritentativi(percorso: &str) -> Result<Vec<u8>, String> {
    let mut errore = String::new();
    for tentativo in 0..4 {
        if tentativo > 0 {
            // pausa crescente: 0,25 s, 0,5 s, 1 s
            let attesa = 0.25 * 2f64.powi(tentativo - 1);
            let inizio = get_time();
            while get_time() - inizio < attesa {
                next_frame().await;
            }
        }
        match load_file(percorso).await {
            Ok(dati) => return Ok(dati),
            Err(e) => errore = format!("{}: {:?}", percorso, e),
        }
    }
    Err(errore)
}

/// Schermata di errore al posto di un gioco congelato: non si può andare avanti, ma almeno
/// si sa cosa fare.
pub async fn mostra_errore(dettaglio: &str) -> ! {
    loop {
        set_default_camera();
        clear_background(Color::from_rgba(20, 10, 16, 255));
        let (w, h) = (screen_width(), screen_height());
        let righe = [
            ("Impossibile caricare il gioco", 30.0, Color::from_rgba(255, 90, 90, 255)),
            ("Controlla la connessione e ricarica la pagina.", 22.0, WHITE),
            (dettaglio, 16.0, GRAY),
        ];
        for (i, (testo, dim, colore)) in righe.iter().enumerate() {
            let m = measure_text(testo, None, *dim as u16, 1.0);
            draw_text(testo, ((w - m.width) / 2.0).max(8.0), h / 2.0 - 30.0 + i as f32 * 36.0, *dim, *colore);
        }
        next_frame().await;
    }
}

impl Archivio {
    async fn scarica(percorsi: Vec<String>, totale: usize) -> Self {
        let lavori = percorsi
            .iter()
            .map(|rel| {
                let pieno = asset_path(rel);
                start_coroutine(async move { scarica_con_ritentativi(&pieno).await })
            })
            .collect();
        let risultati = aspetta_tutte(lavori, 0, totale).await;
        let mut file = HashMap::with_capacity(percorsi.len());
        for (rel, dati) in percorsi.into_iter().zip(risultati) {
            match dati {
                Ok(dati) => {
                    file.insert(rel, dati);
                }
                Err(e) => mostra_errore(&e).await,
            }
        }
        Self { file }
    }

    fn dati(&self, rel: &str) -> &[u8] {
        self.file
            .get(rel)
            .unwrap_or_else(|| panic!("{} non è nella lista dei file da caricare", rel))
    }

    fn tex(&self, rel: &str) -> Texture2D {
        let tex = Texture2D::from_file_with_format(self.dati(rel), None);
        tex.set_filter(FilterMode::Nearest);
        tex
    }

    fn font(&self, rel: &str) -> Font {
        let mut font = load_ttf_font_from_bytes(self.dati(rel)).unwrap_or_else(|e| panic!("{}: {:?}", rel, e));
        font.set_filter(FilterMode::Nearest);
        font
    }

    fn anim(&self, folder: &str, file_pattern: &str, num_frames: usize) -> AnimFrames {
        let [down, up, left, right] =
            percorsi_anim(folder, file_pattern, num_frames).map(|lista| lista.iter().map(|p| self.tex(p)).collect::<Vec<_>>());
        AnimFrames { down, up, left, right }
    }
}

fn percorsi_anim(folder: &str, file_pattern: &str, num_frames: usize) -> [Vec<String>; 4] {
    [Direction::Down, Direction::Up, Direction::Left, Direction::Right].map(|dir| {
        (1..=num_frames)
            .map(|i| {
                let filename = file_pattern
                    .replace("{d}", &dir.char_code().to_string())
                    .replace("{i}", &i.to_string());
                format!("{}/{}/{}", folder, dir.folder(), filename)
            })
            .collect()
    })
}

/// Percorso di un suono: nel browser MP3 leggeri (assets/audio/web, vedi
/// tools/converti_audio.py), che tutti i browser sanno decodificare, Safari compreso;
/// sul desktop gli .ogg originali.
pub fn percorso_suono(nome: &str) -> String {
    if cfg!(target_arch = "wasm32") {
        format!("audio/web/{}.mp3", nome)
    } else {
        format!("audio/{}.ogg", nome)
    }
}

async fn load_snd(nome: &str) -> Option<Sound> {
    let p = asset_path(&percorso_suono(nome));
    match load_sound(&p).await {
        Ok(s) => Some(s),
        Err(e) => {
            eprintln!("Attenzione: impossibile caricare il suono {}: {:?}", p, e);
            None
        }
    }
}

/// Effetti sonori, caricati tutti insieme in questo ordine.
const EFFETTI: [&str; 8] = [
    "grass-001",
    "coin",
    "boss_morte",
    "stone",
    "acqua",
    "spada-004",
    "player_dmg",
    "bat_mobile",
];

/// File di ogni tile, per id (l'indice dell'array è l'id usato nelle mappe).
const TILE_FILES: &[(u8, &str)] = &[
    (1, "tile_000.png"),
    (2, "tile_003.png"),
    (3, "tile_022.png"),
    (4, "tile_023.png"),
    (5, "tile_036.png"),
    (6, "tile_041.png"),
    (7, "tile_105.png"),
    (8, "tile_104.png"),
    (9, "tile_109.png"),
    (10, "tile_106.png"),
    (11, "tile_061.png"),
    (12, "tile_063.png"),
    (13, "tile_064.png"),
    (14, "tile_077.png"),
    (15, "tile_053.png"),
    (16, "tile_049.png"),
    (17, "tile_050.png"),
    (18, "fontanella.png"),
    (21, "tile_044.png"),
    (23, "tile_064.png"),
    (24, "tile_036.png"),
];

const FLASH_VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}
"#;

/// Disegna solo la sagoma dello sprite, tutta bianca: il "lampo" quando un nemico viene colpito.
const FLASH_FRAGMENT: &str = r#"#version 100
varying lowp vec4 color;
varying lowp vec2 uv;
uniform sampler2D Texture;
void main() {
    lowp vec4 c = texture2D(Texture, uv);
    gl_FragColor = vec4(1.0, 1.0, 1.0, c.a * color.a);
}
"#;

fn crea_materiale_flash() -> Material {
    use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation, PipelineParams};
    load_material(
        ShaderSource::Glsl {
            vertex: FLASH_VERTEX,
            fragment: FLASH_FRAGMENT,
        },
        MaterialParams {
            pipeline_params: PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("shader del lampo bianco")
}

pub const VOLUME_SOUNDTRACK: f32 = 0.45;
pub const VOLUME_BOSS: f32 = 1.0;

/// Un brano caricato in background.
pub struct BranoCaricato {
    pub brano: MusicTrack,
    pub suono: Option<Sound>,
    /// Vero se sta già suonando (la musica del menu parte appena pronta).
    pub in_riproduzione: bool,
}

/// Carica un brano. Va avviato in una coroutine all'avvio del gioco: così la musica si
/// scarica in parallelo a immagini e suoni. Con `suona_subito` parte appena è decodificata,
/// anche se il resto del gioco sta ancora caricando: è la musica del menu.
pub async fn carica_brano(brano: MusicTrack, suona_subito: bool) -> BranoCaricato {
    let (suono, volume) = match brano {
        MusicTrack::Soundtrack => (load_snd("soundtrack").await, VOLUME_SOUNDTRACK),
        MusicTrack::Boss => (load_snd("boss_soundtrack").await, VOLUME_BOSS),
        MusicTrack::None => (None, 0.0),
    };
    let in_riproduzione = suona_subito && suono.is_some();
    if let (true, Some(s)) = (in_riproduzione, &suono) {
        play_sound(s, PlaySoundParams { looped: true, volume });
    }
    BranoCaricato {
        brano,
        suono,
        in_riproduzione,
    }
}

pub struct Assets {
    pub materiale_flash: Material,

    // Fonts
    pub arcade_font: Font,
    pub dungeon_font: Font,
    /// Pixelify Sans (licenza OFL, vedi assets/PixelifySans-OFL.txt): dialoghi e messaggi.
    pub dialogo_font: Font,

    // Sfondi: copertina del menu e sfondi generati per i livelli
    pub bg3: Texture2D,
    pub sfondi: Sfondi,

    // Tiles, indicizzati per id
    tile_textures: Vec<Option<Texture2D>>,

    // Items
    pub icon_vita: Texture2D,
    pub icon_moneta: Texture2D,

    // NPC & Portal
    pub npc_image: Texture2D,
    pub portal_frames: Vec<Texture2D>,

    // Animations
    pub player_walk: AnimFrames,
    pub player_idle: AnimFrames,
    pub bat_frames: AnimFrames,
    pub boss_frames: AnimFrames,

    // Sounds
    pub snd_grass: Option<Sound>,
    pub snd_coin: Option<Sound>,
    pub snd_morte: Option<Sound>,
    pub snd_stone: Option<Sound>,
    pub snd_acqua: Option<Sound>,
    pub snd_attack: Option<Sound>,
    pub snd_player_dmg: Option<Sound>,
    pub snd_bat_morte: Option<Sound>,

    // Music
    pub mus_soundtrack: Option<Sound>,
    pub mus_boss: Option<Sound>,

    // Audio State
    pub audio_attivo: bool,
    pub current_music_track: MusicTrack,
    pub current_music: Option<Sound>,
    pub current_music_volume: f32,
}

impl Assets {
    pub async fn load() -> Self {
        let tutti_i_tile = TILE_FILES.iter().map(|(_, f)| format!("[BLOCCHI]/{}", f));
        let anim = [
            percorsi_anim("[EROE]/[MOVEMENT]", "walk{d}{i}.png", 4),
            percorsi_anim("[EROE]/[IDLE]", "idle{i}.png", 4),
            percorsi_anim("[MOBS]/[BAT]", "{i}.png", 3),
            percorsi_anim("[MOBS]/[BOSS]", "{i}.png", 4),
        ];
        let mut percorsi: Vec<String> = [
            "Arcade.ttf",
            "DungeonFont.ttf",
            "PixelifySans.ttf",
            "[SFONDI]/bg3.png",
            "[ITEMS]/health.png",
            "[ITEMS]/coin.png",
            "[NPC]/1.png",
        ]
        .iter()
        .map(|s| s.to_string())
        .chain((1..=6).map(|i| format!("[PORTALE]/portal{}.png", i)))
        .chain(tutti_i_tile)
        .chain(anim.into_iter().flatten().flatten())
        .collect();
        percorsi.sort();
        percorsi.dedup();

        let totale = percorsi.len() + EFFETTI.len();
        disegna_caricamento(0.0);
        next_frame().await;

        // effetti sonori: scaricati e decodificati in parallelo alle immagini
        let suoni = EFFETTI.iter().map(|&nome| start_coroutine(load_snd(nome))).collect::<Vec<_>>();
        let n_file = percorsi.len();
        let archivio = Archivio::scarica(percorsi, totale).await;

        let arcade_font = archivio.font("Arcade.ttf");
        let dungeon_font = archivio.font("DungeonFont.ttf");
        let dialogo_font = archivio.font("PixelifySans.ttf");
        let sfondi = Sfondi::genera();
        let bg3 = archivio.tex("[SFONDI]/bg3.png");
        bg3.set_filter(FilterMode::Linear);
        let icon_vita = archivio.tex("[ITEMS]/health.png");
        let icon_moneta = archivio.tex("[ITEMS]/coin.png");
        let npc_image = archivio.tex("[NPC]/1.png");
        let portal_frames = (1..=6)
            .map(|i| archivio.tex(&format!("[PORTALE]/portal{}.png", i)))
            .collect();

        let max_id = TILE_FILES.iter().map(|t| t.0).max().unwrap_or(0) as usize;
        let mut tile_textures = vec![None; max_id + 1];
        for &(id, file) in TILE_FILES {
            tile_textures[id as usize] = Some(archivio.tex(&format!("[BLOCCHI]/{}", file)));
        }

        let player_walk = archivio.anim("[EROE]/[MOVEMENT]", "walk{d}{i}.png", 4);
        let player_idle = archivio.anim("[EROE]/[IDLE]", "idle{i}.png", 4);
        let bat_frames = archivio.anim("[MOBS]/[BAT]", "{i}.png", 3);
        let boss_frames = archivio.anim("[MOBS]/[BOSS]", "{i}.png", 4);

        // il gioco parte comunque dopo 6 secondi, al massimo senza qualche effetto sonoro
        let effetti: [Option<Sound>; 8] = aspetta_con_limite(suoni, 6.0, n_file, totale)
            .await
            .into_iter()
            .map(Option::flatten)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_else(|_| unreachable!("un suono per ogni effetto"));
        let [snd_grass, snd_coin, snd_morte, snd_stone, snd_acqua, snd_attack, snd_player_dmg, snd_bat_morte] = effetti;

        Self {
            materiale_flash: crea_materiale_flash(),
            arcade_font,
            dungeon_font,
            dialogo_font,
            bg3,
            sfondi,
            tile_textures,
            icon_vita,
            icon_moneta,
            npc_image,
            portal_frames,
            player_walk,
            player_idle,
            bat_frames,
            boss_frames,
            snd_grass,
            snd_coin,
            snd_morte,
            snd_stone,
            snd_acqua,
            snd_attack,
            snd_player_dmg,
            snd_bat_morte,
            mus_soundtrack: None,
            mus_boss: None,
            audio_attivo: true,
            current_music_track: MusicTrack::None,
            current_music: None,
            current_music_volume: 0.2,
        }
    }

    /// Aggiunge un brano appena caricato; se era quello richiesto nel frattempo parte subito
    /// (o continua, se stava già suonando).
    pub fn imposta_brano(&mut self, caricato: BranoCaricato) {
        let BranoCaricato {
            brano,
            suono,
            in_riproduzione,
        } = caricato;
        match brano {
            MusicTrack::Soundtrack => self.mus_soundtrack = suono.clone(),
            MusicTrack::Boss => self.mus_boss = suono.clone(),
            MusicTrack::None => return,
        }
        let richiesto = self.current_music_track == brano && self.current_music.is_none();
        if in_riproduzione {
            if richiesto && self.audio_attivo {
                // sta già suonando: basta prenderne il controllo
                self.current_music = suono;
                return;
            }
            if let Some(ref s) = suono {
                stop_sound(s);
            }
        }
        if richiesto {
            let volume = self.current_music_volume;
            self.current_music_track = MusicTrack::None;
            self.play_music(brano, volume);
        }
    }

    /// Mette in cache, alla scala attuale, i caratteri che il gioco disegna davvero.
    ///
    /// Serve perché la cache dei glifi di macroquad, quando si riempie, raddoppia e ricopia
    /// tutto pixel per pixel: se succede a metà frame il testo si rovina per un frame. Ma va
    /// fatto con misura: precaricare tutte le dimensioni con tutti i caratteri, su uno schermo
    /// grande o su un telefono, bloccava l'avvio per un secondo (o molti di più su un telefono
    /// lento) e riempiva la memoria della GPU. Qui: set completi solo per il testo corrente,
    /// per titoli e pulsanti solo le lettere che compaiono.
    pub fn prepara_testo(&self) {
        let completo: Vec<char> = (' '..='~').chain("àèéìòù".chars()).collect();
        let solo = |testo: &str| -> Vec<char> {
            let mut c: Vec<char> = testo.chars().collect();
            c.sort_unstable();
            c.dedup();
            c
        };
        // stessa formula di parametri_testo, più la moltiplicazione per la densità dello schermo
        // che macroquad fa da solo: così i glifi in cache sono esattamente quelli disegnati
        let s = crate::settings::scala_testo();
        let dpi = screen_dpi_scale();
        let dim = |d: u16| (((d as f32 * s).round().max(1.0)) * dpi).ceil() as u16;

        for d in [22, 28, 30] {
            self.dungeon_font.populate_font_cache(&completo, dim(d));
        }
        self.dungeon_font.populate_font_cache(&solo("LIVELLO 0123456789"), dim(56));
        self.dungeon_font.populate_font_cache(&solo("Crediti"), dim(60));
        self.dungeon_font.populate_font_cache(&solo("Sei MortoPausaVITTORIA!"), dim(80));
        self.dungeon_font.populate_font_cache(&solo("Nemici Miei"), dim(90));

        self.dialogo_font.populate_font_cache(&completo, dim(22));
        self.dialogo_font.populate_font_cache(&solo("MeUgo Tognazzi"), dim(24));

        for d in [24, 26, 32] {
            self.arcade_font.populate_font_cache(&completo, dim(d));
        }
        self.arcade_font.populate_font_cache(&solo("!"), dim(22));
        self.arcade_font.populate_font_cache(&solo("ATKEII"), dim(30));
        self.arcade_font.populate_font_cache(&solo("STARTESCIRIPROVAMENURIPRENDIAUDIO: SINO"), dim(50));
    }

    pub fn tile(&self, id: u8) -> Option<&Texture2D> {
        self.tile_textures.get(id as usize).and_then(|t| t.as_ref())
    }

    pub fn esegui(&mut self, comando: ComandoAudio) {
        match comando {
            ComandoAudio::Suono(s, volume) => self.play(s, volume),
            ComandoAudio::Musica(track, volume) => self.play_music(track, volume),
            ComandoAudio::Attiva(attivo) => {
                if attivo != self.audio_attivo {
                    self.toggle_audio();
                }
            }
        }
    }

    pub fn play(&self, suono: Suono, volume: f32) {
        let snd = match suono {
            Suono::Erba => &self.snd_grass,
            Suono::Acqua => &self.snd_acqua,
            Suono::Pietra => &self.snd_stone,
            Suono::Moneta | Suono::Cuore => &self.snd_coin,
            Suono::Attacco => &self.snd_attack,
            Suono::DannoGiocatore => &self.snd_player_dmg,
            Suono::MortePipistrello => &self.snd_bat_morte,
            Suono::MorteBoss => &self.snd_morte,
        };
        self.play_sound(snd, volume);
    }

    pub fn play_sound(&self, snd: &Option<Sound>, volume: f32) {
        if !self.audio_attivo {
            return;
        }
        if let Some(ref s) = snd {
            play_sound(
                s,
                PlaySoundParams {
                    looped: false,
                    volume,
                },
            );
        }
    }

    pub fn play_music(&mut self, track: MusicTrack, volume: f32) {
        if self.current_music_track == track && self.current_music.is_some() {
            return;
        }
        self.current_music_volume = volume;
        self.current_music_track = track;
        if let Some(ref cur) = self.current_music {
            stop_sound(cur);
        }
        self.current_music = match track {
            MusicTrack::Soundtrack => self.mus_soundtrack.clone(),
            MusicTrack::Boss => self.mus_boss.clone(),
            MusicTrack::None => None,
        };
        if self.audio_attivo {
            if let Some(ref s) = self.current_music {
                play_sound(
                    s,
                    PlaySoundParams {
                        looped: true,
                        volume,
                    },
                );
            }
        }
    }

    pub fn toggle_audio(&mut self) {
        self.audio_attivo = !self.audio_attivo;
        if let Some(ref cur) = self.current_music {
            if self.audio_attivo {
                play_sound(
                    cur,
                    PlaySoundParams {
                        looped: true,
                        volume: self.current_music_volume,
                    },
                );
            } else {
                stop_sound(cur);
            }
        }
    }
}
