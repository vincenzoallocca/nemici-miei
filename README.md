<h1 align="center" id="title">Nemici Miei</h1>

<p align="center" id="description">PCTO 2024-2025 - nato in Pygame, ora riscritto in Rust</p>

Un piccolo gioco d'avventura in vista isometrica scritto in Rust: parla con Ugo, bevi alla fontanella, attraversa la spiaggia e le rovine e sconfiggi il boss.

<h2>🎮 Gioca online</h2>

**[Gioca nel browser](https://vincenzoallocca.github.io/nemici-miei/)**: funziona anche da telefono (meglio in orizzontale), con joystick e pulsanti sullo schermo.

<h2>🚀 Demo</h2>

[https://streamable.com/46ll9t](https://streamable.com/46ll9t)

<h2>🗺️ Livelli</h2>

| # | Livello | Obiettivo |
| --- | --- | --- |
| 1 | Il prato di Ugo | Parla con Ugo, bevi alla fontanella ed entra nel portale |
| 2 | La spiaggia | Sconfiggi tutto lo stormo di pipistrelli (in acqua sei lento) |
| 3 | Le rovine | Raccogli 6 monete tra i muri di roccia |
| 4 | L'arena | Sconfiggi il boss (in furia chiama i pipistrelli) |

Se muori, "Riprova" riparte dall'inizio del livello con la vita piena.

<h2>🧐 Funzionalità</h2>

*   Vista isometrica con personaggi e oggetti ordinati per profondità
*   Pixel-art nitida: tutto disegnato a scala intera, testo alla risoluzione dello schermo
*   Sfondi pixel-art generati dal codice e animati, uno per ambientazione
*   Pipistrelli che inseguono il giocatore aggirando ostacoli e senza sovrapporsi
*   Boss con fase di furia, attacco caricato, rigenerazione e rinforzi
*   Monete e cuori da raccogliere (i cuori restano a terra finché non servono)
*   HUD con vita, monete, livello, ricarica dell'attacco, vita del boss e obiettivo corrente
*   Menu, pausa, game over con "riprova" e titoli di coda
*   Logica a 60 passi al secondo fissi: stessa velocità su qualunque monitor
*   Effetti sonori e musica diversa per i livelli e per il boss

<h2>🎮 Comandi</h2>

| Tasto | Azione |
| --- | --- |
| WASD / Frecce | Muoversi (due tasti insieme per muoversi in orizzontale/verticale sullo schermo) |
| Click sinistro / Spazio | Attaccare |
| E | Parlare / interagire |
| ESC / P | Pausa (nel browser usa P) |
| R / Invio | Riprovare il livello dopo il game over |
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

In alternativa `bash scripts/build-web.sh` prepara la cartella `dist` con solo i file usati. Nel browser i suoni sono MP3 (li decodificano tutti i browser, Safari compreso) e la colonna sonora parte appena è pronta, mentre il resto del gioco sta ancora caricando.

<h2>🧪 Test</h2>

I test controllano la logica di gioco senza aprire la finestra, compresa una partita completa simulata dal menu ai titoli di coda:

```
cargo test
```

Per lo sviluppo c'è anche una modalità screenshot: `NEMICI_LIVELLO=3 NEMICI_SCREENSHOT=foto.png cargo run` avvia il livello 3, salva uno screenshot dopo 90 frame ed esce. Con `NEMICI_POSIZIONE=8,7` l'eroe parte da quel tile e con `NEMICI_FINESTRA=1440x1080` la finestra è più grande.

<h2>📁 Struttura</h2>

| File | Contenuto |
| --- | --- |
| `src/main.rs` | Avvio, finestra e ciclo principale a passo fisso |
| `src/game.rs` | Stati (menu, gioco, pausa, game over, vittoria), livelli e obiettivi |
| `src/livelli.rs` | Mappe e definizione dei livelli |
| `src/settings.rs` | Costanti, tile e dialoghi |
| `src/input.rs` | Tastiera e mouse raccolti a ogni frame |
| `src/assets.rs` | Caricamento di immagini, suoni e font |
| `src/sfondo.rs` | Sfondi pixel-art generati e animati |
| `src/world.rs` | Mappa isometrica, coordinate e ricerca del percorso |
| `src/pickup.rs` | Monete e cuori da raccogliere |
| `src/entita.rs` | Movimento e animazione comuni a giocatore e nemici |
| `src/player.rs`, `src/enemy.rs`, `src/boss.rs`, `src/npc.rs`, `src/portal.rs` | Personaggi e oggetti |
| `src/hud.rs`, `src/menu.rs`, `src/overlay.rs`, `src/win.rs`, `src/effetti.rs` | Interfaccia ed effetti |
| `src/touch.rs` | Joystick e pulsanti sullo schermo per giocare da telefono |
| `web/index.html` | Pagina della versione web |
| `tests/` | Test automatici |
| `tools/converti_audio.py` | Crea gli MP3 leggeri della versione web in `assets/audio/web` (script Python opzionale) |
| `tools/genera_eroe.py` | Disegna gli sprite pixel-art dell'eroe in `assets/[EROE]` (script Python opzionale, serve `pygame`) |

<h2>🛡️ Licenza</h2>

Questo progetto è distribuito con licenza GPL-3.0 (vedi [LICENSE](LICENSE)).

<h2>💖 Ti piace il progetto?</h2>

Lascia una stella al repository!
