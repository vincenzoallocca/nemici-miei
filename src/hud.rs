use crate::assets::Assets;
use crate::boss::Boss;
use crate::effetti::Effetti;
use crate::npc::Npc;
use crate::player::Giocatore;
use crate::settings::*;
use crate::world::World;
use macroquad::prelude::*;

// ---------- testo ----------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ancora {
    /// (x, y) è l'angolo in alto a sinistra.
    Sinistra,
    /// (x, y) è il centro.
    Centro,
    /// (x, y) è l'angolo in alto a destra.
    Destra,
}

/// Parametri per disegnare il testo alla risoluzione reale dello schermo (vedi `scala_testo`).
pub fn parametri_testo(font: &Font, font_size: u16, color: Color) -> TextParams<'_> {
    let s = scala_per_dimensione(font_size);
    TextParams {
        font: Some(font),
        font_size: ((font_size as f32 * s).round() as u16).max(1),
        font_scale: 1.0 / s,
        color,
        ..Default::default()
    }
}

pub fn misura_testo(testo: &str, font: &Font, font_size: u16) -> TextDimensions {
    let s = scala_per_dimensione(font_size);
    measure_text(testo, Some(font), ((font_size as f32 * s).round() as u16).max(1), 1.0 / s)
}

pub fn dividi_testo(testo: &str, font: &Font, font_size: u16, larghezza_max: f32) -> Vec<String> {
    let mut righe = Vec::new();
    let mut riga = String::new();

    for parola in testo.split_whitespace() {
        let prova = if riga.is_empty() {
            parola.to_string()
        } else {
            format!("{} {}", riga, parola)
        };
        if misura_testo(&prova, font, font_size).width <= larghezza_max {
            riga = prova;
        } else {
            if !riga.is_empty() {
                righe.push(riga);
            }
            riga = parola.to_string();
        }
    }
    if !riga.is_empty() {
        righe.push(riga);
    }
    righe
}

pub fn testo_con_ombra(font: &Font, testo: &str, x: f32, y: f32, font_size: u16, colore: Color, ancora: Ancora) {
    let dims = misura_testo(testo, font, font_size);
    let (rx, ry) = match ancora {
        Ancora::Centro => (x - dims.width / 2.0, y + dims.offset_y / 2.0),
        Ancora::Destra => (x - dims.width, y + dims.offset_y),
        Ancora::Sinistra => (x, y + dims.offset_y),
    };
    let (rx, ry) = (rx.round(), ry.round());
    let ombra = Color::new(0.0, 0.0, 0.0, colore.a);
    draw_text_ex(testo, rx + 2.0, ry + 2.0, parametri_testo(font, font_size, ombra));
    draw_text_ex(testo, rx, ry, parametri_testo(font, font_size, colore));
}

pub fn barra(x: f32, y: f32, w: f32, h: f32, percentuale: f32, colore: Color, sfondo: Color) {
    draw_rectangle(x, y, w, h, sfondo);
    let piena_w = (w * percentuale.clamp(0.0, 1.0)).round();
    if piena_w > 0.0 {
        draw_rectangle(x, y, piena_w, h, colore);
        // riflesso in alto per dare volume
        draw_rectangle(x, y, piena_w, (h / 4.0).round(), Color::new(1.0, 1.0, 1.0, 0.25));
    }
    draw_rectangle_lines(x, y, w, h, 2.0, BLACK);
}

pub const GIALLO: Color = Color::new(250.0 / 255.0, 210.0 / 255.0, 60.0 / 255.0, 1.0);
pub const ROSSO: Color = Color::new(1.0, 60.0 / 255.0, 60.0 / 255.0, 1.0);

// ---------- HUD ----------

pub struct HUD {
    pub dialogue_index: usize,
    pub giaparlato: bool,
    pub touch: bool,
    pub messaggio: Option<String>,
    pub messaggio_timer: i32,
}

impl Default for HUD {
    fn default() -> Self {
        Self::new()
    }
}

impl HUD {
    pub fn new() -> Self {
        Self {
            dialogue_index: 0,
            giaparlato: false,
            touch: false,
            messaggio: None,
            messaggio_timer: 0,
        }
    }

    pub fn mostra_messaggio(&mut self, testo: &str, secondi: f32) {
        self.messaggio = Some(testo.to_string());
        self.messaggio_timer = (secondi * FPS) as i32;
    }

    pub fn interagisci_npc(&mut self, npc: &mut Npc, player: &mut Giocatore, effetti: &mut Effetti) {
        if self.giaparlato {
            return;
        }
        if !npc.parlando {
            npc.parlando = true;
            self.dialogue_index = 0;
        } else {
            self.dialogue_index += 1;
            if self.dialogue_index >= STRING_DIALOGUE.len() {
                npc.parlando = false;
                self.giaparlato = true;
                return;
            }
        }

        if self.dialogue_index == 1 && !player.ugo_vita_bonus {
            player.ugo_vita_bonus = true;
            player.cura(15, effetti);
        }
    }

    /// Un passo di logica: scadenza del messaggio e dialogo interrotto se ti allontani.
    pub fn update(&mut self, npc: Option<&mut Npc>, player: &Giocatore) {
        if self.messaggio_timer > 0 {
            self.messaggio_timer -= 1;
            if self.messaggio_timer == 0 {
                self.messaggio = None;
            }
        }
        if let Some(npc) = npc {
            if npc.parlando && player.entita.distanza_tiles(npc.tile_x, npc.tile_y) > 1 {
                npc.parlando = false;
                self.dialogue_index = 0;
            }
        }
    }

    pub fn mostra_suggerimento(&self, world: &World, player: &Giocatore, npc: Option<&Npc>) -> bool {
        let vicino_fontana = player.puo_bere(self.giaparlato, world);
        let vicino_npc = npc.is_some_and(|n| player.entita.distanza_tiles(n.tile_x, n.tile_y) == 1 && !self.giaparlato);
        vicino_fontana || vicino_npc
    }

    fn draw_messaggio(&self, assets: &Assets) {
        let Some(ref msg) = self.messaggio else {
            return;
        };
        if self.messaggio_timer <= 0 {
            return;
        }
        let alpha = (self.messaggio_timer as f32 / 30.0).clamp(0.0, 1.0);
        let font = &assets.dialogo_font;
        let righe = dividi_testo(msg, font, 22, 420.0);
        let h = righe.len() as f32 * 27.0 + 16.0;
        let (x, y, w) = (12.0, 46.0, 450.0);

        draw_rectangle(x, y, w, h, Color::new(0.0, 0.0, 0.0, 0.67 * alpha));
        draw_rectangle_lines(x, y, w, h, 2.0, Color::new(GIALLO.r, GIALLO.g, GIALLO.b, alpha));

        let mut text_y = y + 10.0;
        for riga in &righe {
            testo_con_ombra(font, riga, x + 14.0, text_y, 22, Color::new(1.0, 1.0, 1.0, alpha), Ancora::Sinistra);
            text_y += 27.0;
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        world: &World,
        player: &Giocatore,
        npc: Option<&Npc>,
        boss: Option<&Boss>,
        obiettivo: &str,
        livelli_totali: usize,
        assets: &Assets,
    ) {
        let top = ALTEZZA - HUD_HEIGHT;
        let font = &assets.dungeon_font;

        // pannello in basso
        draw_rectangle(0.0, top, LUNGHEZZA, HUD_HEIGHT, Color::from_rgba(42, 38, 48, 255));
        draw_rectangle(0.0, top, LUNGHEZZA, 4.0, Color::from_rgba(70, 64, 80, 255));
        draw_line(0.0, top, LUNGHEZZA, top, 3.0, Color::from_rgba(20, 18, 24, 255));

        // vita
        icona(&assets.icon_vita, 20.0, top + 12.0, SCALA_PIXEL);
        let colore_vita = if player.vita > 30 {
            Color::from_rgba(60, 200, 80, 255)
        } else {
            // lampeggia quando la vita è bassa
            let p = (get_time() as f32 * 8.0).sin() * 0.5 + 0.5;
            Color::new(1.0, 0.15 * p, 0.1 * p, 1.0)
        };
        barra(
            60.0,
            top + 18.0,
            170.0,
            24.0,
            player.vita as f32 / player.vita_max as f32,
            colore_vita,
            Color::from_rgba(30, 30, 30, 255),
        );
        testo_con_ombra(
            font,
            &format!("{} / {}", player.vita, player.vita_max),
            145.0,
            top + 30.0,
            22,
            WHITE,
            Ancora::Centro,
        );

        // monete (10x10 disegnata 3x)
        icona(&assets.icon_moneta, 21.0, top + 57.0, 3.0);
        testo_con_ombra(font, &format!("Monete: {}", player.monete), 60.0, top + 58.0, 28, WHITE, Ancora::Sinistra);

        // livello e ricarica dell'attacco
        testo_con_ombra(
            font,
            &format!("Livello: {}/{}", world.livello_id, livelli_totali),
            260.0,
            top + 14.0,
            28,
            WHITE,
            Ancora::Sinistra,
        );
        testo_con_ombra(font, "Attacco", 260.0, top + 50.0, 22, WHITE, Ancora::Sinistra);
        barra(340.0, top + 56.0, 100.0, 14.0, player.ricarica_attacco(), GIALLO, Color::from_rgba(30, 30, 30, 255));

        // a destra: dialogo, vita del boss o suggerimento
        let destra_x = 470.0;
        let destra_y = top + 10.0;
        let destra_w = LUNGHEZZA - 490.0;
        let destra_h = HUD_HEIGHT - 20.0;

        if npc.is_some_and(|n| n.parlando) {
            self.draw_dialogo(destra_x, destra_y, destra_w, destra_h, assets);
        } else if let Some(b) = boss.filter(|b| b.vivo()) {
            self.draw_vita_boss(destra_x, destra_y, destra_w, b, assets);
        } else if self.mostra_suggerimento(world, player, npc) {
            let tasto = if self.touch { "Premi E per interagire" } else { "Premi (E) per interagire" };
            testo_con_ombra(
                &assets.arcade_font,
                tasto,
                destra_x + destra_w / 2.0,
                destra_y + destra_h / 2.0,
                32,
                WHITE,
                Ancora::Centro,
            );
        }

        // obiettivo corrente
        testo_con_ombra(font, &format!("Obiettivo: {}", obiettivo), 16.0, 12.0, 30, GIALLO, Ancora::Sinistra);

        if !self.touch {
            testo_con_ombra(
                &assets.arcade_font,
                &format!("{}  pausa", crate::input::NOME_TASTO_PAUSA),
                LUNGHEZZA - 16.0,
                14.0,
                24,
                WHITE,
                Ancora::Destra,
            );
        }

        self.draw_messaggio(assets);
    }

    fn draw_dialogo(&self, x: f32, y: f32, w: f32, h: f32, assets: &Assets) {
        let Some(&(nome, testo)) = STRING_DIALOGUE.get(self.dialogue_index) else {
            return;
        };
        let font = &assets.dialogo_font;

        draw_rectangle(x, y, w, h, Color::from_rgba(25, 25, 25, 255));
        draw_rectangle_lines(x, y, w, h, 2.0, Color::from_rgba(90, 90, 90, 255));

        let colore_nome = if nome == "Me" { GIALLO } else { ROSSO };
        testo_con_ombra(font, nome, x + 10.0, y + 4.0, 24, colore_nome, Ancora::Sinistra);

        let mut text_y = y + 28.0;
        for riga in dividi_testo(testo, font, 22, w - 20.0).iter().take(2) {
            testo_con_ombra(font, riga, x + 10.0, text_y, 22, WHITE, Ancora::Sinistra);
            text_y += 24.0;
        }

        testo_con_ombra(
            &assets.arcade_font,
            "E >",
            x + w - 8.0,
            y + h - 22.0,
            24,
            Color::from_rgba(180, 180, 180, 255),
            Ancora::Destra,
        );
    }

    fn draw_vita_boss(&self, x: f32, y: f32, w: f32, boss: &Boss, assets: &Assets) {
        let font = &assets.dungeon_font;
        let (etichetta, col) = if boss.furia { ("BOSS - FURIA!", ROSSO) } else { ("BOSS", WHITE) };
        testo_con_ombra(font, etichetta, x, y + 2.0, 28, col, Ancora::Sinistra);

        barra(
            x,
            y + 36.0,
            w,
            22.0,
            boss.hp / boss.hp_max,
            Color::from_rgba(255, 0, 0, 255),
            Color::from_rgba(140, 20, 20, 255),
        );
        testo_con_ombra(
            font,
            &format!("{} / {}", boss.hp.ceil() as i32, boss.hp_max as i32),
            x + w / 2.0,
            y + 47.0,
            22,
            WHITE,
            Ancora::Centro,
        );
    }

    /// Freccia gialla che rimbalza sopra il prossimo obiettivo.
    pub fn draw_freccia_obiettivo(&self, bersaglio: Option<(f32, f32)>) {
        let Some((target_x, target_y)) = bersaglio else {
            return;
        };
        let target_x = target_x.round();
        let target_y = (target_y - 14.0 - (get_time() as f32 * 4.0).sin().abs() * 10.0).round();

        let tip = vec2(target_x, target_y);
        let left_corner = vec2(target_x - 13.0, target_y - 16.0);
        let inner_left = vec2(target_x - 5.0, target_y - 16.0);
        let stem_top_left = vec2(target_x - 5.0, target_y - 30.0);
        let stem_top_right = vec2(target_x + 5.0, target_y - 30.0);
        let inner_right = vec2(target_x + 5.0, target_y - 16.0);
        let right_corner = vec2(target_x + 13.0, target_y - 16.0);

        draw_triangle(tip, left_corner, right_corner, GIALLO);
        draw_rectangle(stem_top_left.x, stem_top_left.y, 10.0, 14.0, GIALLO);

        let punti = [
            tip,
            left_corner,
            inner_left,
            stem_top_left,
            stem_top_right,
            inner_right,
            right_corner,
            tip,
        ];
        for coppia in punti.windows(2) {
            draw_line(coppia[0].x, coppia[0].y, coppia[1].x, coppia[1].y, 2.0, BLACK);
        }
    }
}

fn icona(tex: &Texture2D, x: f32, y: f32, scala: f32) {
    draw_texture_ex(
        tex,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(tex.width() * scala, tex.height() * scala)),
            ..Default::default()
        },
    );
}
