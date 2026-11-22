//! Definizione dei livelli: mappa, personaggi, nemici e obiettivo.
//!
//! Legenda dei tile più usati nelle mappe:
//! 1-2 terra, 3-5 erba, 7-8 acqua bassa (calpestabile, rallenta), 9-10 acqua alta,
//! 11-13 pietra, 18 fontanella, 19 erba intorno alla fontanella, 20 erba "sicura"
//! (niente nemici), 23 roccia, 24 cespuglio, 25 pietra "sicura".

use crate::assets::MusicTrack;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Obiettivo {
    /// Parla con Ugo, bevi alla fontanella ed entra nel portale.
    ParlaEBevi,
    /// Sconfiggi tutti i pipistrelli per aprire il portale.
    SconfiggiPipistrelli,
    /// Raccogli tutte le monete sparse per aprire il portale.
    RaccogliMonete,
    /// Sconfiggi il boss.
    SconfiggiBoss,
}

/// Ambientazione: decide lo sfondo.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tema {
    Prato,
    Spiaggia,
    Rovine,
    Arena,
}

pub struct DefLivello {
    pub nome: &'static str,
    pub tema: Tema,
    pub mappa: &'static [&'static [u8]],
    pub spawn: (i32, i32),
    pub npc: Option<(i32, i32)>,
    pub portale: Option<(i32, i32)>,
    pub boss: Option<(i32, i32)>,
    pub obiettivo: Obiettivo,
    /// Quanti pipistrelli (minimo, massimo).
    pub pipistrelli: (usize, usize),
    pub vita_pipistrelli: i32,
    pub danno_pipistrelli: i32,
    /// Monete sparse sulla mappa.
    pub monete: usize,
    /// Cuori sparsi sulla mappa (curano 25 punti).
    pub cuori: usize,
    pub musica: MusicTrack,
    /// Suggerimento mostrato all'inizio. `{ATK}` diventa il comando di attacco.
    pub intro: &'static str,
}

const PRATO: &[&[u8]] = &[
    &[3, 7, 7, 7, 7, 7, 7, 20, 20, 20],
    &[4, 8, 8, 8, 8, 8, 8, 3, 3, 3],
    &[3, 3, 8, 8, 8, 8, 3, 3, 3, 3],
    &[3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
    &[3, 3, 3, 3, 19, 19, 4, 3, 3, 3],
    &[3, 3, 4, 3, 19, 18, 4, 3, 3, 3],
    &[3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
    &[3, 3, 3, 3, 4, 3, 3, 3, 4, 3],
    &[3, 3, 3, 4, 3, 3, 3, 3, 3, 3],
    &[3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
];

const ARENA: &[&[u8]] = &[
    &[11, 11, 11, 11, 12, 11, 11, 11, 11, 11],
    &[11, 11, 11, 11, 11, 11, 11, 11, 11, 11],
    &[11, 11, 11, 12, 11, 11, 11, 11, 11, 11],
    &[11, 12, 11, 11, 11, 11, 11, 11, 11, 11],
    &[11, 11, 11, 11, 11, 11, 12, 11, 11, 11],
    &[11, 11, 12, 11, 11, 11, 12, 11, 11, 11],
    &[11, 11, 11, 11, 11, 11, 11, 11, 11, 11],
    &[11, 11, 11, 11, 12, 11, 11, 11, 12, 11],
    &[11, 11, 11, 12, 11, 11, 11, 11, 11, 11],
    &[11, 11, 11, 11, 11, 11, 11, 11, 11, 11],
];

pub const LIVELLI: &[DefLivello] = &[
    DefLivello {
        nome: "Il prato di Ugo",
        tema: Tema::Prato,
        mappa: PRATO,
        spawn: (0, 0),
        npc: Some((8, 8)),
        portale: Some((8, 0)),
        boss: None,
        obiettivo: Obiettivo::ParlaEBevi,
        pipistrelli: (3, 5),
        vita_pipistrelli: 50,
        danno_pipistrelli: 10,
        monete: 0,
        cuori: 0,
        musica: MusicTrack::Soundtrack,
        intro: "Muoviti con WASD o le frecce. Raggiungi Ugo, il mago con il punto esclamativo.",
    },
    DefLivello {
        nome: "L'arena",
        tema: Tema::Arena,
        mappa: ARENA,
        spawn: (8, 0),
        npc: None,
        portale: None,
        boss: Some((0, 0)),
        obiettivo: Obiettivo::SconfiggiBoss,
        pipistrelli: (0, 0),
        vita_pipistrelli: 50,
        danno_pipistrelli: 10,
        monete: 0,
        cuori: 0,
        musica: MusicTrack::Boss,
        intro: "Sconfiggi il boss! Attacca con {ATK} e allontanati quando trema.",
    },
];
