use macroquad::audio::{load_sound, play_sound, stop_sound, PlaySoundParams, Sound};
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

/// Barra di caricamento: con più di cento file (soprattutto nel browser) serve un segno di vita.
struct Caricamento {
    fatti: usize,
    totale: usize,
}

impl Caricamento {
    async fn avanza(&mut self, quanti: usize) {
        self.fatti += quanti;
        let progresso = (self.fatti as f32 / self.totale as f32).min(1.0);
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
        draw_rectangle(bx, by, bw * progresso, 16.0, Color::from_rgba(250, 210, 60, 255));
        draw_rectangle_lines(bx, by, bw, 16.0, 2.0, WHITE);
        next_frame().await;
    }
}

async fn load_tex(rel: &str) -> Texture2D {
    let p = asset_path(rel);
    let tex = load_texture(&p)
        .await
        .unwrap_or_else(|e| panic!("Errore caricamento texture {}: {:?}", p, e));
    tex.set_filter(FilterMode::Nearest);
    tex
}

async fn load_snd(rel: &str) -> Option<Sound> {
    let p = asset_path(rel);
    match load_sound(&p).await {
        Ok(s) => Some(s),
        Err(e) => {
            eprintln!("Attenzione: impossibile caricare il suono {}: {:?}", p, e);
            None
        }
    }
}

async fn load_anim(folder: &str, file_pattern: &str, num_frames: usize) -> AnimFrames {
    let mut frames: [Vec<Texture2D>; 4] = Default::default();
    let dirs = [Direction::Down, Direction::Up, Direction::Left, Direction::Right];
    for (dir, lista) in dirs.iter().zip(frames.iter_mut()) {
        for i in 1..=num_frames {
            let filename = file_pattern
                .replace("{d}", &dir.char_code().to_string())
                .replace("{i}", &i.to_string());
            lista.push(load_tex(&format!("{}/{}/{}", folder, dir.folder(), filename)).await);
        }
    }
    let [down, up, left, right] = frames;
    AnimFrames { down, up, left, right }
}

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

pub struct Assets {
    pub materiale_flash: Material,

    // Fonts
    pub arcade_font: Font,
    pub dungeon_font: Font,

    // Backgrounds
    pub bg1: Texture2D,
    pub bg2: Texture2D,
    pub bg3: Texture2D,

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
        let mut c = Caricamento { fatti: 0, totale: 12 };
        c.avanza(0).await;

        let mut arcade_font = load_ttf_font(&asset_path("Arcade.ttf")).await.expect("Arcade.ttf");
        let mut dungeon_font = load_ttf_font(&asset_path("DungeonFont.ttf"))
            .await
            .expect("DungeonFont.ttf");
        arcade_font.set_filter(FilterMode::Nearest);
        dungeon_font.set_filter(FilterMode::Nearest);

        let bg1 = load_tex("[SFONDI]/bg.png").await;
        let bg2 = load_tex("[SFONDI]/bg2.png").await;
        let bg3 = load_tex("[SFONDI]/bg3.png").await;
        bg3.set_filter(FilterMode::Linear);
        c.avanza(1).await;

        let icon_vita = load_tex("[ITEMS]/health.png").await;
        let icon_moneta = load_tex("[ITEMS]/coin.png").await;
        let npc_image = load_tex("[NPC]/1.png").await;
        let mut portal_frames = Vec::with_capacity(6);
        for i in 1..=6 {
            portal_frames.push(load_tex(&format!("[PORTALE]/portal{}.png", i)).await);
        }
        c.avanza(1).await;

        let max_id = TILE_FILES.iter().map(|t| t.0).max().unwrap_or(0) as usize;
        let mut tile_textures = vec![None; max_id + 1];
        for &(id, file) in TILE_FILES {
            tile_textures[id as usize] = Some(load_tex(&format!("[BLOCCHI]/{}", file)).await);
        }
        c.avanza(1).await;

        let player_walk = load_anim("[EROE]/[MOVEMENT]", "walk{d}{i}.png", 4).await;
        c.avanza(1).await;
        let player_idle = load_anim("[EROE]/[IDLE]", "idle{i}.png", 4).await;
        c.avanza(1).await;
        let bat_frames = load_anim("[MOBS]/[BAT]", "{i}.png", 3).await;
        c.avanza(1).await;
        let boss_frames = load_anim("[MOBS]/[BOSS]", "{i}.png", 4).await;
        c.avanza(1).await;

        let snd_grass = load_snd("audio/grass-001.ogg").await;
        let snd_coin = load_snd("audio/coin.ogg").await;
        let snd_morte = load_snd("audio/boss_morte.ogg").await;
        let snd_stone = load_snd("audio/stone.ogg").await;
        c.avanza(1).await;
        let snd_acqua = load_snd("audio/acqua.ogg").await;
        let snd_attack = load_snd("audio/spada-004.ogg").await;
        let snd_player_dmg = load_snd("audio/player_dmg.ogg").await;
        let snd_bat_morte = load_snd("audio/bat_mobile.ogg").await;
        c.avanza(1).await;

        let mus_soundtrack = load_snd("audio/soundtrack.ogg").await;
        c.avanza(1).await;
        let mus_boss = load_snd("audio/boss_soundtrack.ogg").await;
        c.avanza(1).await;

        Self {
            materiale_flash: crea_materiale_flash(),
            arcade_font,
            dungeon_font,
            bg1,
            bg2,
            bg3,
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
            mus_soundtrack,
            mus_boss,
            audio_attivo: true,
            current_music_track: MusicTrack::None,
            current_music: None,
            current_music_volume: 0.2,
        }
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
        if self.current_music_track == track {
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
