use macroquad::audio::{load_sound, play_sound, stop_sound, PlaySoundParams, Sound};
use macroquad::prelude::*;
use std::collections::HashMap;
use std::path::Path;

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

pub fn asset_path(rel: &str) -> String {
    let p = Path::new("assets").join(rel);
    if p.exists() {
        return p.to_string_lossy().to_string();
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let p2 = parent.join("assets").join(rel);
            if p2.exists() {
                return p2.to_string_lossy().to_string();
            }
            if let Some(grand) = parent.parent() {
                let p3 = grand.join("assets").join(rel);
                if p3.exists() {
                    return p3.to_string_lossy().to_string();
                }
            }
        }
    }
    format!("assets/{}", rel)
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
            eprintln!("Attenzione: impossibile caricare suono {}: {:?}", p, e);
            None
        }
    }
}

async fn load_anim(folder: &str, file_pattern: &str, num_frames: usize) -> AnimFrames {
    let mut down = Vec::with_capacity(num_frames);
    let mut up = Vec::with_capacity(num_frames);
    let mut left = Vec::with_capacity(num_frames);
    let mut right = Vec::with_capacity(num_frames);

    for dir in &[Direction::Down, Direction::Up, Direction::Left, Direction::Right] {
        for i in 1..=num_frames {
            let filename = file_pattern
                .replace("{d}", &dir.char_code().to_string())
                .replace("{i}", &i.to_string());
            let path_rel = format!("{}/{}/{}", folder, dir.folder(), filename);
            let tex = load_tex(&path_rel).await;
            match dir {
                Direction::Down => down.push(tex),
                Direction::Up => up.push(tex),
                Direction::Left => left.push(tex),
                Direction::Right => right.push(tex),
            }
        }
    }

    AnimFrames { down, up, left, right }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MusicTrack {
    Soundtrack,
    Boss,
    None,
}

pub struct Assets {
    // Fonts
    pub arcade_font: Font,
    pub dungeon_font: Font,

    // Backgrounds
    pub bg1: Texture2D,
    pub bg2: Texture2D,
    pub bg3: Texture2D,

    // Tiles
    pub tile_textures: HashMap<u8, Texture2D>,

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
        // Load fonts
        let mut arcade_font = load_ttf_font(&asset_path("Arcade.ttf")).await.expect("Arcade.ttf");
        let mut dungeon_font = load_ttf_font(&asset_path("DungeonFont.ttf")).await.expect("DungeonFont.ttf");
        arcade_font.set_filter(FilterMode::Nearest);
        dungeon_font.set_filter(FilterMode::Nearest);

        // Backgrounds
        let bg1 = load_tex("[SFONDI]/bg.png").await;
        let bg2 = load_tex("[SFONDI]/bg2.png").await;
        let bg3 = load_tex("[SFONDI]/bg3.png").await;

        // Items
        let icon_vita = load_tex("[ITEMS]/health.png").await;
        let icon_moneta = load_tex("[ITEMS]/coin.png").await;

        // NPC & Portal
        let npc_image = load_tex("[NPC]/1.png").await;
        let mut portal_frames = Vec::with_capacity(6);
        for i in 1..=6 {
            portal_frames.push(load_tex(&format!("[PORTALE]/portal{}.png", i)).await);
        }

        // Tiles
        let mut tile_textures = HashMap::new();
        let tile_mapping: &[(u8, &str)] = &[
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
            (19, "tile_022.png"),
            (20, "tile_022.png"),
            (21, "tile_044.png"),
            (22, "tile_061.png"),
        ];
        for (id, file) in tile_mapping {
            let tex = load_tex(&format!("[BLOCCHI]/{}", file)).await;
            tile_textures.insert(*id, tex);
        }

        // Character Animations
        let player_walk = load_anim("[EROE]/[MOVEMENT]", "walk{d}{i}.png", 4).await;
        let player_idle = load_anim("[EROE]/[IDLE]", "idle{i}.png", 4).await;
        let bat_frames = load_anim("[MOBS]/[BAT]", "{i}.png", 3).await;
        let boss_frames = load_anim("[MOBS]/[BOSS]", "{i}.png", 4).await;

        // Audio Effects
        let snd_grass = load_snd("audio/grass-001.ogg").await;
        let snd_coin = load_snd("audio/coin.ogg").await;
        let snd_morte = load_snd("audio/boss_morte.ogg").await;
        let snd_stone = load_snd("audio/stone.ogg").await;
        let snd_acqua = load_snd("audio/acqua.ogg").await;
        let snd_attack = load_snd("audio/spada-004.ogg").await;
        let snd_player_dmg = load_snd("audio/player_dmg.ogg").await;
        let snd_bat_morte = load_snd("audio/bat_mobile.ogg").await;

        // Music
        let mus_soundtrack = load_snd("audio/soundtrack.ogg").await;
        let mus_boss = load_snd("audio/boss_soundtrack.ogg").await;

        Self {
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
        self.current_music_volume = volume;
        if self.current_music_track == track && self.audio_attivo {
            return;
        }
        self.current_music_track = track;
        if let Some(ref cur) = self.current_music {
            stop_sound(cur);
        }
        let next = match track {
            MusicTrack::Soundtrack => self.mus_soundtrack.clone(),
            MusicTrack::Boss => self.mus_boss.clone(),
            MusicTrack::None => None,
        };
        self.current_music = next;
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
