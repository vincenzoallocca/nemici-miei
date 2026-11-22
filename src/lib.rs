//! Nemici Miei: piccolo gioco d'avventura isometrico.
//!
//! La logica (`Gioco::update`) gira a passo fisso ed è separata dal disegno (`Gioco::draw`)
//! e dall'audio (comandi in coda): per questo si può testare senza aprire una finestra.

pub mod assets;
pub mod boss;
pub mod effetti;
pub mod enemy;
pub mod entita;
pub mod game;
pub mod hud;
pub mod input;
pub mod livelli;
pub mod menu;
pub mod npc;
pub mod overlay;
pub mod pickup;
pub mod player;
pub mod portal;
pub mod settings;
pub mod touch;
pub mod win;
pub mod world;
