//! Sfondi pixel-art generati dal codice, uno per ambientazione.
//!
//! La parte fissa (cielo a bande, sole o luna, montagne, colline) viene disegnata una volta
//! sola in una texture 240x180 e ingrandita 4x; sopra si disegnano le parti animate
//! (nuvole, riflessi, lucciole, braci) allineate alla stessa griglia di 4 pixel.

use crate::livelli::Tema;
use crate::settings::{ALTEZZA, LUNGHEZZA};
use macroquad::prelude::*;

const W: u16 = 240;
const H: u16 = 180;
/// Ogni pixel dello sfondo è grande 4x4 pixel di gioco.
const PIX: f32 = 4.0;

pub struct Sfondi {
    prato: Texture2D,
    spiaggia: Texture2D,
    rovine: Texture2D,
    arena: Texture2D,
}

// ---------- utilità ----------

fn hash(x: u32, y: u32, seme: u32) -> u32 {
    let mut h = x.wrapping_mul(374_761_393) ^ y.wrapping_mul(668_265_263) ^ seme.wrapping_mul(2_246_822_519);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^ (h >> 16)
}

/// Numero pseudo-casuale in [0, 1), sempre uguale per gli stessi argomenti.
fn caso(x: u32, y: u32, seme: u32) -> f32 {
    (hash(x, y, seme) % 10_000) as f32 / 10_000.0
}

fn hex(c: u32) -> Color {
    Color::from_rgba((c >> 16) as u8, (c >> 8) as u8, c as u8, 255)
}

fn mescola(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, 1.0)
}

fn gradiente(tappe: &[(f32, u32)], t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    for coppia in tappe.windows(2) {
        let (t0, c0) = coppia[0];
        let (t1, c1) = coppia[1];
        if t <= t1 {
            return mescola(hex(c0), hex(c1), (t - t0) / (t1 - t0).max(1e-6));
        }
    }
    hex(tappe[tappe.len() - 1].1)
}

const BAYER: [[f32; 4]; 4] = [
    [0.0, 8.0, 2.0, 10.0],
    [12.0, 4.0, 14.0, 6.0],
    [3.0, 11.0, 1.0, 9.0],
    [15.0, 7.0, 13.0, 5.0],
];

/// Gradiente a bande con retinatura: l'effetto "cielo a 16 colori" dei giochi pixel-art.
fn cielo(tappe: &[(f32, u32)], t: f32, x: u16, y: u16, bande: f32) -> Color {
    let soglia = BAYER[(y % 4) as usize][(x % 4) as usize] / 16.0;
    let k = (t * bande + soglia - 0.5).round().clamp(0.0, bande);
    gradiente(tappe, k / bande)
}

/// Profilo di montagne o colline: somma di onde.
fn profilo(x: f32, base: f32, onde: &[(f32, f32, f32)]) -> f32 {
    base + onde.iter().map(|&(amp, freq, fase)| amp * (x * freq + fase).sin()).sum::<f32>()
}

fn disco(img: &mut Image, cx: f32, cy: f32, r: f32, colore: impl Fn(u16, u16, f32) -> Option<Color>) {
    let (x0, x1) = ((cx - r).floor().max(0.0) as u16, ((cx + r).ceil() as u16).min(W - 1));
    let (y0, y1) = ((cy - r).floor().max(0.0) as u16, ((cy + r).ceil() as u16).min(H - 1));
    for y in y0..=y1 {
        for x in x0..=x1 {
            let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
            if d <= r {
                if let Some(c) = colore(x, y, d / r) {
                    img.set_pixel(x as u32, y as u32, c);
                }
            }
        }
    }
}

fn stelle(img: &mut Image, densita: f32, fino_a: u16, seme: u32) {
    for y in 0..fino_a {
        for x in 0..W {
            let r = caso(x as u32, y as u32, seme);
            if r < densita {
                let c = if r < densita * 0.3 { hex(0xffffff) } else { hex(0xb8b4d8) };
                img.set_pixel(x as u32, y as u32, c);
            }
        }
    }
}

fn in_texture(img: Image) -> Texture2D {
    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Nearest);
    tex
}

// ---------- sfondi fissi ----------

fn prato() -> Image {
    let mut img = Image::gen_image_color(W, H, BLACK);
    let tappe = [(0.0, 0x3f7fd6), (0.55, 0x8cc6f2), (1.0, 0xd9f1ff)];
    for y in 0..H {
        for x in 0..W {
            img.set_pixel(x as u32, y as u32, cielo(&tappe, y as f32 / 130.0, x, y, 10.0));
        }
    }
    // sole con alone
    disco(&mut img, 196.0, 30.0, 22.0, |x, y, d| {
        if d < 0.55 {
            Some(hex(0xfff6c0))
        } else if BAYER[(y % 4) as usize][(x % 4) as usize] / 16.0 > d {
            Some(hex(0xfdf0b0))
        } else {
            None
        }
    });
    for x in 0..W {
        let xf = x as f32;
        let lontane = profilo(xf, 112.0, &[(10.0, 0.045, 1.0), (6.0, 0.11, 2.0), (2.0, 0.37, 0.5)]);
        let colline = profilo(xf, 140.0, &[(7.0, 0.07, 0.0), (4.0, 0.19, 1.0)]);
        for y in 0..H {
            let yf = y as f32;
            let c = if yf > colline {
                gradiente(&[(0.0, 0x4f9a4a), (1.0, 0x2f6b35)], (yf - colline) / 40.0)
            } else if yf > lontane {
                // montagne lontane, più chiare in cima (foschia)
                mescola(hex(0x8fb0d4), hex(0x6f8fb8), (yf - lontane) / 30.0)
            } else {
                continue;
            };
            img.set_pixel(x as u32, y as u32, c);
        }
        // alberelli sulle colline
        if hash(x as u32 / 7, 0, 11).is_multiple_of(3) && x % 7 == 3 {
            let alt = 6 + (hash(x as u32, 1, 11) % 6) as i32;
            let base = colline as i32 + 1;
            for dy in 0..alt {
                let mezza = (alt - dy) / 3;
                for dx in -mezza..=mezza {
                    let (px, py) = (x as i32 + dx, base - dy);
                    if (0..W as i32).contains(&px) && (0..H as i32).contains(&py) {
                        img.set_pixel(px as u32, py as u32, hex(0x2a5e33));
                    }
                }
            }
        }
    }
    img
}

const ORIZZONTE: f32 = 116.0;

fn spiaggia() -> Image {
    let mut img = Image::gen_image_color(W, H, BLACK);
    let cielo_t = [(0.0, 0x2b1d52), (0.45, 0x7a3b78), (0.78, 0xe0645a), (1.0, 0xffb35c)];
    let mare_t = [(0.0, 0x5a3a78), (0.3, 0x2c3a72), (1.0, 0x132a55)];
    for y in 0..H {
        for x in 0..W {
            let yf = y as f32;
            let c = if yf < ORIZZONTE {
                cielo(&cielo_t, yf / ORIZZONTE, x, y, 12.0)
            } else {
                cielo(&mare_t, (yf - ORIZZONTE) / (H as f32 - ORIZZONTE), x, y, 8.0)
            };
            img.set_pixel(x as u32, y as u32, c);
        }
    }
    // sole "retro" a strisce che tramonta sull'orizzonte
    disco(&mut img, 120.0, ORIZZONTE, 30.0, |_, y, _| {
        let yf = y as f32;
        if yf >= ORIZZONTE {
            return None;
        }
        let sotto = ORIZZONTE - yf;
        if sotto < 16.0 && (sotto as i32) % 5 < 2 {
            return None;
        }
        Some(gradiente(&[(0.0, 0xffe58a), (1.0, 0xff7a4a)], 1.0 - sotto / 30.0))
    });
    // riflesso del sole sull'acqua
    for y in ORIZZONTE as u16..H {
        let profondita = (y as f32 - ORIZZONTE) / (H as f32 - ORIZZONTE);
        let mezza = 30.0 * (1.0 - profondita * 0.7);
        for x in 0..W {
            let dx = (x as f32 - 120.0).abs();
            if dx < mezza && y % 3 == 0 && caso(x as u32 / 3, y as u32, 5) < 0.55 * (1.0 - dx / mezza) + 0.1 {
                img.set_pixel(x as u32, y as u32, hex(0xff9e5e));
            }
        }
    }
    // isolotto lontano
    for x in 8..80u16 {
        let cima = profilo(x as f32, ORIZZONTE - 4.0, &[(-6.0, 0.06, -0.4), (-2.0, 0.2, 0.0)]);
        let cima = cima.min(ORIZZONTE - 1.0);
        for y in cima.max(0.0) as u16..ORIZZONTE as u16 {
            img.set_pixel(x as u32, y as u32, hex(0x3a2450));
        }
    }
    img
}

fn rovine() -> Image {
    let mut img = Image::gen_image_color(W, H, BLACK);
    let tappe = [(0.0, 0x120c26), (0.5, 0x3b2250), (0.85, 0x7d3f63), (1.0, 0xb8606a)];
    for y in 0..H {
        for x in 0..W {
            img.set_pixel(x as u32, y as u32, cielo(&tappe, y as f32 / 140.0, x, y, 10.0));
        }
    }
    stelle(&mut img, 0.006, 90, 3);
    // luna con crateri
    disco(&mut img, 192.0, 34.0, 12.0, |x, y, d| {
        let cratere = caso(x as u32 / 2, y as u32 / 2, 9) < 0.18;
        Some(if d > 0.85 {
            hex(0xd8c8a8)
        } else if cratere {
            hex(0xd9cba8)
        } else {
            hex(0xf3e6c8)
        })
    });
    for x in 0..W {
        let xf = x as f32;
        let lontane = profilo(xf, 104.0, &[(14.0, 0.03, 0.0), (8.0, 0.09, 3.0), (3.0, 0.31, 1.0)]);
        let vicine = profilo(xf, 134.0, &[(10.0, 0.05, 2.0), (5.0, 0.15, 0.0)]);
        // colonne in rovina sul profilo vicino
        let colonna = x % 37 < 4 && hash(x as u32 / 37, 2, 7).is_multiple_of(2);
        let altezza_colonna = 10.0 + (hash(x as u32 / 37, 3, 7) % 14) as f32;
        for y in 0..H {
            let yf = y as f32;
            let c = if yf > vicine || (colonna && yf > vicine - altezza_colonna) {
                hex(0x24142f)
            } else if yf > lontane {
                mescola(hex(0x5a3468), hex(0x3e2350), (yf - lontane) / 30.0)
            } else {
                continue;
            };
            img.set_pixel(x as u32, y as u32, c);
        }
    }
    img
}

fn arena() -> Image {
    let mut img = Image::gen_image_color(W, H, BLACK);
    let tappe = [(0.0, 0x07040f), (0.55, 0x1c0b24), (0.85, 0x4a0f22), (1.0, 0x7a1a1a)];
    for y in 0..H {
        for x in 0..W {
            img.set_pixel(x as u32, y as u32, cielo(&tappe, y as f32 / 150.0, x, y, 10.0));
        }
    }
    stelle(&mut img, 0.01, 120, 1);
    // luna rossa
    disco(&mut img, 176.0, 46.0, 24.0, |x, y, d| {
        let macchia = caso(x as u32 / 3, y as u32 / 3, 4) < 0.25;
        Some(if d > 0.9 {
            hex(0xa8302c)
        } else if macchia {
            hex(0xb83a32)
        } else {
            hex(0xd8473c)
        })
    });
    // guglie di roccia frastagliate
    for x in 0..W {
        let xf = x as f32;
        let picco = (caso(x as u32 / 4, 0, 8) * 18.0).round();
        let guglia = if x % 53 < 5 { 22.0 - (x % 53) as f32 * 2.0 } else { 0.0 };
        let cima = profilo(xf, 146.0, &[(6.0, 0.06, 1.0), (3.0, 0.2, 0.0)]) - picco * 0.4 - guglia;
        for y in cima.max(0.0) as u16..H {
            let c = mescola(hex(0x1e0c1c), hex(0x0c0610), (y as f32 - cima) / 30.0);
            img.set_pixel(x as u32, y as u32, c);
        }
    }
    img
}

// ---------- parti animate ----------

fn quadretto(x: f32, y: f32, w: f32, h: f32, c: Color) {
    // allineato alla griglia di 4 pixel dello sfondo
    let gx = (x / PIX).round() * PIX;
    let gy = (y / PIX).round() * PIX;
    draw_rectangle(gx, gy, (w / PIX).round().max(1.0) * PIX, (h / PIX).round().max(1.0) * PIX, c);
}

fn nuvole(t: f32, colore: Color, ombra: Color, quante: u32, y_max: f32, velocita: f32) {
    for i in 0..quante {
        let scala = 0.7 + caso(i, 0, 21) * 0.8;
        let larghezza = 120.0 * scala;
        let x = (caso(i, 1, 21) * (LUNGHEZZA + larghezza) + t * velocita * scala) % (LUNGHEZZA + larghezza) - larghezza;
        let y = 30.0 + caso(i, 2, 21) * y_max;
        let h = 20.0 * scala;
        quadretto(x, y + h, larghezza, 8.0, ombra);
        quadretto(x + 8.0, y + 8.0, larghezza - 16.0, h, colore);
        quadretto(x + larghezza * 0.2, y, larghezza * 0.35, h, colore);
        quadretto(x + larghezza * 0.45, y - 8.0 * scala, larghezza * 0.3, h, colore);
        quadretto(x, y + h * 0.5, larghezza, h * 0.6, colore);
    }
}

fn scintillio_stelle(t: f32, quante: u32, y_max: f32, seme: u32) {
    for i in 0..quante {
        let fase = caso(i, 0, seme) * std::f32::consts::TAU;
        let luce = ((t * (1.0 + caso(i, 1, seme) * 2.0) + fase).sin() * 0.5 + 0.5).powi(3);
        if luce < 0.2 {
            continue;
        }
        let x = caso(i, 2, seme) * LUNGHEZZA;
        let y = caso(i, 3, seme) * y_max;
        let c = Color::new(1.0, 1.0, 1.0, luce);
        quadretto(x, y, PIX, PIX, c);
        if luce > 0.7 {
            // piccola croce sulle più luminose
            let a = Color::new(1.0, 1.0, 1.0, luce * 0.5);
            quadretto(x - PIX, y, PIX, PIX, a);
            quadretto(x + PIX, y, PIX, PIX, a);
            quadretto(x, y - PIX, PIX, PIX, a);
            quadretto(x, y + PIX, PIX, PIX, a);
        }
    }
}

fn particelle_che_salgono(t: f32, quante: u32, colore: impl Fn(f32) -> Color, velocita: f32, seme: u32) {
    for i in 0..quante {
        let v = velocita * (0.5 + caso(i, 0, seme));
        let periodo = ALTEZZA + 40.0;
        let y = ALTEZZA - (caso(i, 1, seme) * periodo + t * v) % periodo;
        let x = caso(i, 2, seme) * LUNGHEZZA + (t * 0.8 + i as f32).sin() * 12.0;
        let vita = (y / ALTEZZA).clamp(0.0, 1.0);
        quadretto(x, y, PIX, PIX, colore(vita));
    }
}

impl Sfondi {
    /// Va chiamato con il contesto grafico attivo (crea texture).
    pub fn genera() -> Self {
        Self {
            prato: in_texture(prato()),
            spiaggia: in_texture(spiaggia()),
            rovine: in_texture(rovine()),
            arena: in_texture(arena()),
        }
    }

    pub fn disegna(&self, tema: Tema, tempo: f64) {
        let t = tempo as f32;
        let tex = match tema {
            Tema::Prato => &self.prato,
            Tema::Spiaggia => &self.spiaggia,
            Tema::Rovine => &self.rovine,
            Tema::Arena => &self.arena,
        };
        draw_texture_ex(
            tex,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(LUNGHEZZA, ALTEZZA)),
                ..Default::default()
            },
        );

        match tema {
            Tema::Prato => nuvole(t, Color::new(1.0, 1.0, 1.0, 0.92), Color::from_rgba(190, 220, 245, 230), 6, 260.0, 9.0),
            Tema::Spiaggia => {
                nuvole(t, Color::from_rgba(255, 150, 140, 170), Color::from_rgba(160, 70, 120, 150), 4, 180.0, 5.0);
                // luccichii che corrono sull'acqua
                let mare = ORIZZONTE * PIX;
                for i in 0..26u32 {
                    let y = mare + 8.0 + caso(i, 0, 31) * (ALTEZZA - mare);
                    let x = (caso(i, 1, 31) * LUNGHEZZA + t * (6.0 + caso(i, 2, 31) * 10.0)) % LUNGHEZZA;
                    let luce = ((t * 2.0 + i as f32).sin() * 0.5 + 0.5) * 0.8;
                    quadretto(x, y, 12.0 + caso(i, 3, 31) * 20.0, PIX, Color::new(1.0, 0.75, 0.55, luce));
                }
            }
            Tema::Rovine => {
                scintillio_stelle(t, 30, 300.0, 5);
                // lucciole
                for i in 0..22u32 {
                    let bx = caso(i, 0, 41) * LUNGHEZZA;
                    let by = 280.0 + caso(i, 1, 41) * 380.0;
                    let x = bx + (t * 0.5 + i as f32 * 1.7).sin() * 30.0;
                    let y = by + (t * 0.7 + i as f32).cos() * 18.0;
                    let luce = ((t * 3.0 + i as f32 * 2.3).sin() * 0.5 + 0.5).powi(2);
                    quadretto(x - PIX, y - PIX, PIX * 3.0, PIX * 3.0, Color::new(0.8, 1.0, 0.4, luce * 0.25));
                    quadretto(x, y, PIX, PIX, Color::new(0.9, 1.0, 0.5, luce));
                }
            }
            Tema::Arena => {
                scintillio_stelle(t, 40, 420.0, 9);
                particelle_che_salgono(
                    t,
                    45,
                    |vita| Color::new(1.0, 0.35 + 0.4 * vita, 0.1, vita * 0.9),
                    28.0,
                    13,
                );
            }
        }
    }
}
