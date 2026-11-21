use crate::assets::Assets;
use crate::boss::Boss;
use crate::effetti::Effetti;
use crate::npc::Npc;
use crate::player::Giocatore;
use crate::portal::Portale;
use crate::settings::*;
use crate::world::World;
use macroquad::prelude::*;

pub fn dividi_testo(testo: &str, font: &Font, font_size: u16, larghezza_max: f32) -> Vec<String> {
    let mut righe = Vec::new();
    let mut riga = String::new();

    for parola in testo.split_whitespace() {
        let prova = if riga.is_empty() {
            parola.to_string()
        } else {
            format!("{} {}", riga, parola)
        };
        let dims = measure_text(&prova, Some(font), font_size, 1.0);
        if dims.width <= larghezza_max {
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

pub fn testo_con_ombra(
    font: &Font,
    testo: &str,
    x: f32,
    y: f32,
    font_size: u16,
    colore: Color,
    ancora: &str, // "topleft", "center", "topright"
) {
    let dims = measure_text(testo, Some(font), font_size, 1.0);
    let (rx, ry) = match ancora {
        "center" => (x - dims.width / 2.0, y + dims.offset_y / 2.0),
        "topright" => (x - dims.width, y + dims.offset_y),
        _ => (x, y + dims.offset_y), // "topleft"
    };

    // Shadow
    draw_text_ex(
        testo,
        rx + 2.0,
        ry + 2.0,
        TextParams {
            font: Some(font),
            font_size,
            color: BLACK,
            ..Default::default()
        },
    );

    // Main
    draw_text_ex(
        testo,
        rx,
        ry,
        TextParams {
            font: Some(font),
            font_size,
            color: colore,
            ..Default::default()
        },
    );
}

pub fn barra(x: f32, y: f32, w: f32, h: f32, percentuale: f32, colore: Color, sfondo: Color) {
    draw_rectangle(x, y, w, h, sfondo);
    let piena_w = (w * percentuale.clamp(0.0, 1.0)).max(0.0);
    if piena_w > 0.0 {
        draw_rectangle(x, y, piena_w, h, colore);
    }
    draw_rectangle_lines(x, y, w, h, 2.0, BLACK);
}

pub struct HUD {
    pub dialogue_index: usize,
    pub giaparlato: bool,
    pub touch: bool,
    pub messaggio: Option<String>,
    pub messaggio_timer: i32,
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
        self.messaggio_timer = (secondi * 60.0) as i32;
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

    pub fn aggiorna_dialogo(&mut self, npc: &mut Npc, player: &Giocatore) {
        if npc.parlando {
            let dist = player.entita.distanza_tiles(npc.tile_x, npc.tile_y);
            if dist > 1 {
                npc.parlando = false;
                self.dialogue_index = 0;
            }
        }
    }

    pub fn obiettivo(&self, world: &World, boss: Option<&Boss>) -> &'static str {
        if world.livello_id == 2 {
            if let Some(b) = boss {
                if b.vivo() {
                    "Sconfiggi il boss!"
                } else {
                    "Boss sconfitto!"
                }
            } else {
                "Sconfiggi il boss!"
            }
        } else if !self.giaparlato {
            "Parla con Ugo"
        } else if true /* thirsty check handled by caller or state */ {
            "Bevi alla fontanella"
        } else {
            "Entra nel portale"
        }
    }

    pub fn mostra_suggerimento(&self, world: &World, player: &Giocatore, npc: &Npc) -> bool {
        if world.livello_id != 1 {
            return false;
        }
        let vicino_fontana = player.puo_bere(self.giaparlato, world);
        let vicino_npc = player.entita.distanza_tiles(npc.tile_x, npc.tile_y) == 1 && !self.giaparlato;
        vicino_fontana || vicino_npc
    }

    pub fn draw_messaggio(&mut self, assets: &Assets) {
        if let Some(ref msg) = self.messaggio {
            if self.messaggio_timer <= 0 {
                return;
            }
            self.messaggio_timer -= 1;
            let alpha = (self.messaggio_timer as f32 / 30.0).clamp(0.0, 1.0);
            let righe = dividi_testo(msg, &assets.dungeon_font, 25, 420.0);
            let h = righe.len() as f32 * 30.0 + 16.0;
            let w = 450.0;
            let x = 12.0;
            let y = 46.0;

            draw_rectangle(x, y, w, h, Color::new(0.0, 0.0, 0.0, 0.67 * alpha));
            draw_rectangle_lines(x, y, w, h, 2.0, Color::new(250.0 / 255.0, 210.0 / 255.0, 60.0 / 255.0, alpha));

            let mut text_y = y + 10.0;
            for riga in &righe {
                let dims = measure_text(riga, Some(&assets.dungeon_font), 25, 1.0);
                draw_text_ex(
                    riga,
                    x + 14.0,
                    text_y + dims.height,
                    TextParams {
                        font: Some(&assets.dungeon_font),
                        font_size: 25,
                        color: Color::new(1.0, 1.0, 1.0, alpha),
                        ..Default::default()
                    },
                );
                text_y += 30.0;
            }
        }
    }

    pub fn draw(
        &mut self,
        world: &World,
        player: &Giocatore,
        npc: &mut Npc,
        boss: Option<&Boss>,
        assets: &Assets,
    ) {
        self.aggiorna_dialogo(npc, player);

        let top = ALTEZZA - HUD_HEIGHT;
        // Background HUD
        draw_rectangle(0.0, top, LUNGHEZZA, HUD_HEIGHT, Color::from_rgba(50, 50, 50, 255));
        draw_line(0.0, top, LUNGHEZZA, top, 3.0, Color::from_rgba(20, 20, 20, 255));

        // Health Icon & Bar
        draw_texture_ex(
            &assets.icon_vita,
            20.0,
            top + 14.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(32.0, 32.0)),
                ..Default::default()
            },
        );
        let health_color = if player.vita > 30 {
            Color::from_rgba(60, 200, 80, 255)
        } else {
            Color::from_rgba(255, 0, 0, 255)
        };
        barra(
            60.0,
            top + 18.0,
            170.0,
            24.0,
            player.vita as f32 / player.vita_max as f32,
            health_color,
            Color::from_rgba(30, 30, 30, 255),
        );
        testo_con_ombra(
            &assets.dungeon_font,
            &format!("{} / {}", player.vita, player.vita_max),
            145.0,
            top + 30.0,
            22,
            WHITE,
            "center",
        );

        // Coin Icon & Text
        draw_texture_ex(
            &assets.icon_moneta,
            20.0,
            top + 56.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(32.0, 32.0)),
                ..Default::default()
            },
        );
        testo_con_ombra(
            &assets.dungeon_font,
            &format!("Monete: {}", player.monete),
            60.0,
            top + 58.0,
            28,
            WHITE,
            "topleft",
        );

        // Level & Attack recharge
        testo_con_ombra(
            &assets.dungeon_font,
            &format!("Livello: {}", player.level),
            260.0,
            top + 14.0,
            28,
            WHITE,
            "topleft",
        );
        testo_con_ombra(
            &assets.dungeon_font,
            "Attacco",
            260.0,
            top + 50.0,
            22,
            WHITE,
            "topleft",
        );
        barra(
            340.0,
            top + 56.0,
            100.0,
            14.0,
            player.ricarica_attacco(),
            Color::from_rgba(250, 210, 60, 255),
            Color::from_rgba(30, 30, 30, 255),
        );

        // Right side: Dialogue, Boss Health, or Interaction Prompt
        let destra_x = 470.0;
        let destra_y = top + 10.0;
        let destra_w = LUNGHEZZA - 490.0;
        let destra_h = HUD_HEIGHT - 20.0;

        if npc.parlando && world.livello_id == 1 {
            self.draw_dialogo(destra_x, destra_y, destra_w, destra_h, assets);
        } else if world.livello_id == 2 && boss.is_some_and(|b| b.vivo()) {
            if let Some(b) = boss {
                self.draw_vita_boss(destra_x, destra_y, destra_w, destra_h, b, assets);
            }
        } else if self.mostra_suggerimento(world, player, npc) {
            testo_con_ombra(
                &assets.arcade_font,
                "Premi (E) per interagire",
                destra_x + destra_w / 2.0,
                destra_y + destra_h / 2.0,
                32,
                WHITE,
                "center",
            );
        }

        // Current Objective
        let obj_text = if world.livello_id == 2 {
            if boss.is_some_and(|b| b.vivo()) {
                "Sconfiggi il boss!"
            } else {
                "Boss sconfitto!"
            }
        } else if !self.giaparlato {
            "Parla con Ugo"
        } else if player.thirsty {
            "Bevi alla fontanella"
        } else {
            "Entra nel portale"
        };
        testo_con_ombra(
            &assets.dungeon_font,
            &format!("Obiettivo: {}", obj_text),
            16.0,
            12.0,
            30,
            Color::from_rgba(250, 210, 60, 255),
            "topleft",
        );

        if !self.touch {
            testo_con_ombra(
                &assets.arcade_font,
                "ESC  pausa",
                LUNGHEZZA - 16.0,
                14.0,
                24,
                WHITE,
                "topright",
            );
        }

        self.draw_messaggio(assets);

        // "!" indicator above NPC
        if world.livello_id == 1 && !self.giaparlato && !npc.parlando {
            npc.draw_avviso(assets);
        }
    }

    pub fn draw_dialogo(&self, x: f32, y: f32, w: f32, h: f32, assets: &Assets) {
        if self.dialogue_index >= STRING_DIALOGUE.len() {
            return;
        }
        let (nome, testo) = STRING_DIALOGUE[self.dialogue_index];

        draw_rectangle(x, y, w, h, Color::from_rgba(25, 25, 25, 255));
        draw_rectangle_lines(x, y, w, h, 2.0, Color::from_rgba(90, 90, 90, 255));

        let colore_nome = if nome == "Me" {
            Color::from_rgba(250, 210, 60, 255)
        } else {
            Color::from_rgba(255, 60, 60, 255)
        };
        testo_con_ombra(&assets.dungeon_font, nome, x + 10.0, y + 4.0, 24, colore_nome, "topleft");

        let righe = dividi_testo(testo, &assets.dungeon_font, 22, w - 20.0);
        let mut text_y = y + 28.0;
        for riga in righe.iter().take(2) {
            testo_con_ombra(&assets.dungeon_font, riga, x + 10.0, text_y, 22, WHITE, "topleft");
            text_y += 24.0;
        }

        testo_con_ombra(
            &assets.arcade_font,
            "E >",
            x + w - 8.0,
            y + h - 2.0,
            24,
            Color::from_rgba(180, 180, 180, 255),
            "topright",
        );
    }

    pub fn draw_vita_boss(&self, x: f32, y: f32, w: f32, _h: f32, boss: &Boss, assets: &Assets) {
        let etichetta = if boss.furia { "BOSS - FURIA!" } else { "BOSS" };
        let col = if boss.furia {
            Color::from_rgba(255, 60, 60, 255)
        } else {
            WHITE
        };
        testo_con_ombra(&assets.dungeon_font, etichetta, x, y + 2.0, 28, col, "topleft");

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
            &assets.dungeon_font,
            &format!("{} / {}", boss.hp as i32, boss.hp_max as i32),
            x + w / 2.0,
            y + 47.0,
            22,
            WHITE,
            "center",
        );
    }

    pub fn draw_freccia_obiettivo(&self, world: &World, player: &Giocatore, portal: &Portale) {
        if world.livello_id != 1 || !self.giaparlato {
            return;
        }

        let (target_x, mut target_y) = if player.thirsty {
            if let Some(ref f) = world.fountain {
                (f.x + 50.0, f.y + 6.0)
            } else {
                return;
            }
        } else {
            (portal.x + portal.w / 2.0, portal.y)
        };

        target_y -= 14.0 + (get_time() as f32 * 4.0).sin().abs() * 10.0;

        // Draw bouncing arrow pointing down
        let tip = (target_x, target_y);
        let left_corner = (target_x - 13.0, target_y - 16.0);
        let inner_left = (target_x - 5.0, target_y - 16.0);
        let stem_top_left = (target_x - 5.0, target_y - 30.0);
        let stem_top_right = (target_x + 5.0, target_y - 30.0);
        let inner_right = (target_x + 5.0, target_y - 16.0);
        let right_corner = (target_x + 13.0, target_y - 16.0);

        let arrow_color = Color::from_rgba(250, 210, 60, 255);

        // Fill arrow with triangles
        draw_triangle(
            vec2(tip.0, tip.1),
            vec2(left_corner.0, left_corner.1),
            vec2(right_corner.0, right_corner.1),
            arrow_color,
        );
        draw_rectangle(stem_top_left.0, stem_top_left.1, 10.0, 14.0, arrow_color);

        // Outline
        let points = [
            tip,
            left_corner,
            inner_left,
            stem_top_left,
            stem_top_right,
            inner_right,
            right_corner,
            tip,
        ];
        for i in 0..points.len() - 1 {
            draw_line(
                points[i].0,
                points[i].1,
                points[i + 1].0,
                points[i + 1].1,
                2.0,
                BLACK,
            );
        }
    }
}

impl Default for HUD {
    fn default() -> Self {
        Self::new()
    }
}
