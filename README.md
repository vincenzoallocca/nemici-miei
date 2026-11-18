<h1 align="center" id="title">Nemici Miei</h1>

<p align="center" id="description">PCTO 2024-2025 - PYGAME PROJECT</p>

Un piccolo gioco d'avventura in vista isometrica fatto con Pygame: parla con Ugo, bevi alla fontanella, attraversa il portale e sconfiggi il boss.

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
| ESC / P | Pausa |
| F11 | Schermo intero |

<h2>🛠️ Installazione</h2>

1. Clona il repository (o scarica lo zip ed estrailo):

```
git clone https://github.com/vincenzoallocca/nemici-miei.git
cd nemici-miei
```

2. Installa le dipendenze (serve Python 3.9 o superiore):

```
pip install -r requirements.txt
```

3. Avvia il gioco:

```
python main.py
```

<h2>🧪 Test</h2>

I test simulano una partita completa senza aprire la finestra:

```
python -m unittest discover -s tests -v
```

<h2>📁 Struttura</h2>

| File | Contenuto |
| --- | --- |
| `main.py` | Avvio e gestione degli stati (menu, gioco, pausa, game over, vittoria) |
| `settings.py` | Costanti e percorsi |
| `assets.py` | Caricamento di immagini, suoni, font, dialoghi e mappe dei livelli |
| `world.py` | Mappa isometrica e conversione coordinate |
| `entita.py` | Movimento e animazione comuni a giocatore e nemici |
| `player.py`, `enemy.py`, `boss.py`, `npc.py`, `portal.py` | Personaggi e oggetti |
| `hud.py`, `menu.py`, `riprova.py`, `win.py`, `effetti.py` | Interfaccia ed effetti |

<h2>🛡️ Licenza</h2>

Questo progetto è distribuito con licenza GPL-3.0 (vedi [LICENSE](LICENSE)).

<h2>💖 Ti piace il progetto?</h2>

Lascia una stella al repository!
