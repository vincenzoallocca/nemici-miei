pub const LUNGHEZZA: f32 = 960.0;
pub const ALTEZZA: f32 = 720.0;
pub const HUD_HEIGHT: f32 = 100.0;
pub const GRANDEZZA_TILES: f32 = 72.0;
pub const OFFSET_PIEDI: f32 = 22.0;
pub const FPS: f64 = 60.0;
pub const TITOLO: &str = "Nemici Miei | Rock A' Raso (Rust High-Performance)";

pub const SPAWN_GIOCATORE: (i32, i32) = (0, 0);
pub const POSIZIONE_NPC: (i32, i32) = (8, 8);
pub const POSIZIONE_PORTALE: (i32, i32) = (8, 0);
pub const SPAWN_BOSS: (i32, i32) = (0, 0);
pub const FRAME_PRIMA_DEI_CREDITI: i32 = 120;
pub const FRAME_DI_TREGUA: i32 = 180;

pub const SCALA_PLAYER: f32 = 2.0;
pub const SCALA_BAT: f32 = 1.5;
pub const SCALA_BOSS: f32 = 2.0;
pub const SCALA_BOSS_FURIA: f32 = 2.6;
pub const SCALA_NPC: f32 = 2.5;

pub const VELOCITA_NORMALE: f32 = 4.0;
pub const VELOCITA_ACQUA: f32 = 2.0;

// Walkable tiles:
// {1, 2, 3, 4, 5, 6, 7, 8, 11, 12, 13, 14, 15, 16, 17, 20, 21}
pub fn is_tile_id_walkable(id: u8) -> bool {
    matches!(id, 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 20 | 21)
}

pub fn is_tile_erba(id: u8) -> bool {
    matches!(id, 3..=5)
}

pub fn is_tile_acqua(id: u8) -> bool {
    matches!(id, 7..=10)
}

pub fn is_tile_pietra(id: u8) -> bool {
    matches!(id, 11..=13)
}

pub const LIVELLO_1: [[u8; 10]; 10] = [
    [3, 7, 7, 7, 7, 7, 7, 20, 20, 20],
    [4, 8, 8, 8, 8, 8, 8, 3, 3, 3],
    [3, 3, 8, 8, 8, 8, 3, 3, 3, 3],
    [3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
    [3, 3, 3, 3, 19, 19, 4, 3, 3, 3],
    [3, 3, 4, 3, 19, 18, 4, 3, 3, 3],
    [3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
    [3, 3, 3, 3, 4, 3, 3, 3, 4, 3],
    [3, 3, 3, 4, 3, 3, 3, 3, 3, 3],
    [3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
];

pub const LIVELLO_2: [[u8; 10]; 10] = [
    [11, 11, 11, 11, 12, 11, 11, 11, 11, 11],
    [11, 11, 11, 11, 11, 11, 11, 11, 11, 11],
    [11, 11, 11, 12, 11, 11, 11, 11, 11, 11],
    [11, 12, 11, 11, 11, 11, 11, 11, 11, 11],
    [11, 11, 11, 11, 11, 11, 12, 11, 11, 11],
    [11, 11, 12, 11, 11, 11, 12, 11, 11, 11],
    [11, 11, 11, 11, 11, 11, 11, 11, 11, 11],
    [11, 11, 11, 11, 12, 11, 11, 11, 12, 11],
    [11, 11, 11, 12, 11, 11, 11, 11, 11, 11],
    [11, 11, 11, 11, 11, 11, 11, 11, 11, 11],
];

pub const STRING_DIALOGUE: &[(&str, &str)] = &[
    ("Me", "CHI SEI?! DOVE MI TROVO???"),
    ("Ugo Tognazzi", "No dico noi siamo in due, come se fosse Antani, prefettura?"),
    ("Me", "(???)"),
    ("Ugo Tognazzi", "Dico la supercazzola e' subatomica ma.."),
    ("Ugo Tognazzi", "..prematurata in acqua!"),
    ("Me", "Cosa mi significa?"),
    ("Ugo Tognazzi", "TERAPIATAPIOCO! Bevi alla fontanella... se te ne vuoi andare!"),
];

pub const CONSIGLI: &[&str] = &[
    "Consiglio: colpisci i pipistrelli appena ti arrivano accanto",
    "Consiglio: bere alla fontanella ti cura",
    "Consiglio: quando il boss trema, allontanati di un passo",
    "Consiglio: Ugo ti regala un po' di vita se ci parli",
];

pub const CREDITS_LINES: &[&str] = &[
    "HAI VINTO!",
    "",
    "Sviluppato da: Rock A' Raso GAME LLC",
    "",
    "Allocca Vincenzo",
    "Cerciello Antonio",
    "Cestie' Augusto",
    "Pellegrino Andrea",
    "Prisco Carmine",
    "Sdino Raffaele",
    "",
    "Grazie per aver giocato!",
];
