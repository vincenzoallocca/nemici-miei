use std::sync::atomic::{AtomicU32, Ordering};

// ---------- finestra ----------
pub const LUNGHEZZA: f32 = 960.0;
pub const ALTEZZA: f32 = 720.0;
pub const HUD_HEIGHT: f32 = 100.0;
pub const TITOLO: &str = "Nemici Miei | Rock A' Raso";

/// La logica gira sempre a 60 passi al secondo, qualunque sia la frequenza del monitor.
pub const FPS: f32 = 60.0;
pub const DT: f32 = 1.0 / FPS;

// ---------- griglia isometrica ----------
/// Gli sprite sono pixel-art: tutto viene disegnato a scala intera (2x) per restare nitido.
pub const SCALA_PIXEL: f32 = 2.0;
/// Un tile 32x32 disegnato 2x.
pub const GRANDEZZA_TILES: f32 = 32.0 * SCALA_PIXEL;
/// Distanza verticale tra l'angolo in alto a sinistra del tile e il centro della faccia superiore
/// (dove poggiano i piedi). Nei tile 32x32 il centro della faccia è a y = 15.
pub const OFFSET_PIEDI: f32 = 15.0 * SCALA_PIXEL;
/// L'acqua è una lastra più bassa del terreno: chi ci cammina "affonda" un po'.
pub const ABBASSAMENTO_ACQUA: f32 = 5.0 * SCALA_PIXEL;
/// Gli oggetti (fontanella, rocce, cespugli, fiori) hanno la base 8px più in basso della faccia
/// superiore di un blocco: vanno alzati di tanto per poggiarci sopra.
pub const ALZO_OGGETTI: f32 = 8.0 * SCALA_PIXEL;

// ---------- tempi (in passi da 1/60 s) ----------
pub const FRAME_PRIMA_DEI_CREDITI: i32 = 120;
/// All'inizio di ogni livello i nemici aspettano 3 secondi: il tempo di leggere il suggerimento.
pub const FRAME_DI_TREGUA: i32 = 180;
pub const COOLDOWN_ATTACCO: i32 = 36;

// ---------- giocatore ----------
pub const VITA_GIOCATORE: i32 = 100;
pub const DANNO_GIOCATORE: i32 = 20;
pub const VELOCITA_NORMALE: f32 = 3.6;
pub const VELOCITA_ACQUA: f32 = 1.8;

// ---------- tile speciali ----------
pub const TILE_FONTANELLA: u8 = 18;
/// Erba intorno alla fontanella: non calpestabile.
pub const TILE_BORDO_FONTANA: u8 = 19;
/// Erba dove non nascono nemici (davanti al portale).
pub const TILE_ERBA_SICURA: u8 = 20;
pub const TILE_ROCCIA: u8 = 23;
pub const TILE_CESPUGLIO: u8 = 24;
/// Pavimento di pietra dove non nascono nemici.
pub const TILE_PIETRA_SICURA: u8 = 25;

pub fn is_tile_id_walkable(id: u8) -> bool {
    matches!(id, 1..=8 | 11..=17 | 20..=22 | 25)
}

pub fn is_tile_erba(id: u8) -> bool {
    matches!(id, 3..=5 | 19 | 20 | 24)
}

pub fn is_tile_acqua(id: u8) -> bool {
    matches!(id, 7..=10)
}

pub fn is_tile_pietra(id: u8) -> bool {
    matches!(id, 11..=13 | 22 | 23 | 25)
}

/// Tile dove non devono comparire nemici o oggetti da raccogliere.
pub fn is_tile_sicuro(id: u8) -> bool {
    matches!(id, TILE_ERBA_SICURA | TILE_PIETRA_SICURA)
}

/// Oggetti alti: si disegnano ordinati per profondità insieme ai personaggi.
pub fn is_oggetto_alto(id: u8) -> bool {
    matches!(id, TILE_FONTANELLA | TILE_ROCCIA | TILE_CESPUGLIO)
}

/// Il blocco di terreno da disegnare sotto un tile speciale.
pub fn terreno_sotto(id: u8) -> u8 {
    match id {
        TILE_FONTANELLA | TILE_BORDO_FONTANA | TILE_ERBA_SICURA | TILE_CESPUGLIO => 3,
        TILE_ROCCIA | TILE_PIETRA_SICURA => 11,
        altro => altro,
    }
}

// ---------- scala del testo ----------
// Il testo viene rasterizzato alla risoluzione reale dello schermo (e non a 960x720 per poi
// essere ingrandito), così resta nitido anche a schermo intero.
static SCALA_TESTO: AtomicU32 = AtomicU32::new(0x3f80_0000); // 1.0

pub fn imposta_scala_testo(scala: f32) {
    SCALA_TESTO.store(scala.clamp(0.5, 4.0).to_bits(), Ordering::Relaxed);
}

pub fn scala_testo() -> f32 {
    f32::from_bits(SCALA_TESTO.load(Ordering::Relaxed))
}

// ---------- testi ----------
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
    "Consiglio: i cuori a terra ti curano, tienili per dopo",
    "Consiglio: in acqua sei lento, i pipistrelli no",
    "Consiglio: dietro le rocce i pipistrelli fanno il giro lungo",
];

pub const AUTORI: &[&str] = &[
    "Allocca Vincenzo",
    "Cerciello Antonio",
    "Cestie' Augusto",
    "Pellegrino Andrea",
    "Prisco Carmine",
    "Sdino Raffaele",
];
