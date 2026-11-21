<h1 align="center" id="title">Nemici Miei</h1>

<p align="center" id="description">PCTO 2024-2025 - nato in Pygame, ora riscritto in Rust</p>

Un piccolo gioco d'avventura in vista isometrica scritto in Rust: parla con Ugo, bevi alla fontanella, attraversa il portale e sconfiggi il boss.

<h2>🎮 Gioca online</h2>

**[Gioca nel browser](https://vincenzoallocca.github.io/nemici-miei/)**: funziona anche da telefono (meglio in orizzontale), con joystick e pulsanti sullo schermo.

<h2>🚀 Demo</h2>

[https://streamable.com/46ll9t](https://streamable.com/46ll9t)

<h2>🧐 Funzionalità</h2>

*   Vista isometrica con personaggi ordinati per profondità
*   Animazioni idle e di movimento in 4 direzioni
*   Pipistrelli che inseguono il giocatore senza sovrapporsi
*   Boss con fase di furia, attacco caricato e rigenerazione
*   HUD con barra della vita, vita del boss, ricarica dell'attacco e obiettivo corrente
*   Menu, pausa, game over con "riprova" e titoli di coda
*   Effetti sonori, musica diversa per menu, livello 1 e boss

<h2>🎮 Comandi</h2>

| Tasto | Azione |
| --- | --- |
| WASD / Frecce | Muoversi (due tasti insieme per muoversi in orizzontale/verticale sullo schermo) |
| Click sinistro / Spazio | Attaccare |
| E | Parlare / interagire |
| ESC / P | Pausa (nel browser usa P) |
| M | Audio sì/no |
| F11 | Schermo intero (solo PC) |

Da telefono: trascina il dito nella metà sinistra dello schermo per muoverti (joystick), usa **ATK** per attaccare, **E** per interagire e **II** per la pausa (dove c'è anche il pulsante dell'audio).

In gioco l'obiettivo del momento è sempre in alto a sinistra, una freccia gialla indica dove andare e all'inizio di ogni livello hai qualche secondo di tregua per leggere i suggerimenti.

<h2>🛠️ Compilare e avviare</h2>

Il gioco è scritto in **Rust** con [macroquad](https://macroquad.rs): un solo eseguibile, accelerazione GPU e la stessa base di codice per PC e browser.

1. Installa Rust da [rustup.rs](https://rustup.rs) (su Linux servono anche `libasound2-dev libx11-dev libxi-dev libgl1-mesa-dev`).
2. Clona il repository:

```
git clone https://github.com/vincenzoallocca/nemici-miei.git
cd nemici-miei
```

3. Avvia il gioco:

```
cargo run --release
```

<h2>🌐 Versione web</h2>

La versione per browser è compilata in WebAssembly e pubblicata su GitHub Pages a ogni push su `main` (workflow `.github/workflows/pages.yml`). Per provarla in locale:

```
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
mkdir dist
cp web/index.html target/wasm32-unknown-unknown/release/nemici-miei.wasm dist/
cp -r assets dist/
curl -o dist/mq_js_bundle.js https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js
python -m http.server 8000 --directory dist
```

e apri http://localhost:8000.

<h2>🧪 Test</h2>

I test controllano la logica di gioco senza aprire la finestra:

```
cargo test
```

<h2>📁 Struttura</h2>

| File | Contenuto |
| --- | --- |
| `src/main.rs` | Avvio, finestra e ciclo principale |
| `src/game.rs` | Gestione degli stati (menu, gioco, pausa, game over, vittoria) |
| `src/settings.rs` | Costanti, mappe dei livelli e dialoghi |
| `src/assets.rs` | Caricamento di immagini, suoni e font |
| `src/world.rs` | Mappa isometrica e conversione coordinate |
| `src/entita.rs` | Movimento e animazione comuni a giocatore e nemici |
| `src/player.rs`, `src/enemy.rs`, `src/boss.rs`, `src/npc.rs`, `src/portal.rs` | Personaggi e oggetti |
| `src/hud.rs`, `src/menu.rs`, `src/overlay.rs`, `src/win.rs`, `src/effetti.rs` | Interfaccia ed effetti |
| `src/touch.rs` | Joystick e pulsanti sullo schermo per giocare da telefono |
| `web/index.html` | Pagina della versione web |
| `tests/` | Test automatici |
| `tools/genera_eroe.py` | Disegna gli sprite pixel-art dell'eroe in `assets/[EROE]` (script Python opzionale, serve `pygame`) |

<h2>🛡️ Licenza</h2>

Questo progetto è distribuito con licenza GPL-3.0 (vedi [LICENSE](LICENSE)).

<h2>💖 Ti piace il progetto?</h2>

Lascia una stella al repository!
