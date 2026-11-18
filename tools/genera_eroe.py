"""Genera gli sprite pixel-art dell'eroe (camminata e idle, 4 direzioni).

Uso:  python tools/genera_eroe.py [anteprima.png]
Le immagini vengono salvate in assets/[EROE]/...
Ogni sprite è disegnato come griglia di caratteri (vedi PALETTE): basta modificare
le griglie qui sotto e rilanciare lo script per cambiare l'aspetto del personaggio.
"""
import os
import sys

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
import pygame  # noqa: E402

BASE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
USCITA = os.path.join(BASE, "assets", "[EROE]")
LARGHEZZA, ALTEZZA = 20, 26
RIGA_GAMBE = 18

PALETTE = {
    "O": (34, 28, 45),      # contorno
    "H": (112, 64, 38),     # capelli
    "h": (160, 98, 56),     # riflesso capelli
    "S": (247, 202, 162),   # pelle
    "s": (212, 158, 120),   # pelle in ombra
    "R": (238, 146, 136),   # guance
    "E": (28, 30, 52),      # occhi
    "e": (255, 255, 255),   # luce negli occhi
    "T": (54, 112, 196),    # tunica
    "t": (36, 76, 140),     # tunica in ombra
    "C": (204, 50, 58),     # mantello
    "c": (138, 30, 42),     # mantello in ombra
    "B": (112, 72, 42),     # cintura
    "G": (248, 204, 72),    # fibbia / dettagli dorati
    "P": (232, 222, 200),   # pantaloni
    "p": (186, 174, 156),   # pantaloni in ombra
    "K": (92, 56, 34),      # stivali
    "k": (64, 38, 24),      # stivali in ombra
    "M": (214, 222, 236),   # metallo spada
    "m": (140, 150, 170),   # metallo in ombra
    "L": (126, 84, 50),     # impugnatura
}

# ---------------------------------------------------------------- busto (righe 0-17)
BUSTO = {
    "down": [
        "......OOOOOOOO......",
        ".....OhhHHHHHHO.....",
        "....OhHHHHHHHHHO....",
        "....OHHHHHHHHHHO....",
        "....OHHHSHHSHHHO....",
        "....OHSSSSSSSSHO....",
        "....OHSSeSSeSSHO....",
        "....OsSSESSESSsO....",
        "....OsSRSSSSRSsO....",
        ".....OsSSSSSSsO.....",
        "......OOssssOOLG....",
        "...OCOTTTTTTTTOMO...",
        "...OCtTTTGGTTTtCO...",
        "...OCtTTTTTTTTtCO...",
        "...OCtTTTTTTTTtCO...",
        "...OCSBBBGGBBBSCO...",
        "...OCOtTTTTTTtOCO...",
        "...OccOTTTTTTOccO...",
    ],
    "up": [
        "......OOOOOOOO......",
        ".....OhhHHHHHHO.....",
        "....OhHHHHHHHHHO....",
        "....OHHHHHHhHHHO....",
        "....OHHHHHHHHHHO....",
        "....OHHHHHHHHHHO....",
        "....OHhHHHHHHHHO....",
        "....OHHHHHHHHHHO....",
        "....OsHHHHHHHHsO....",
        ".....OHHHHHHHHO.....",
        "....GLOOssssOO......",
        "...OCOMCCCCCCCOCO...",
        "...OCtCMCCCCCCtCO...",
        "...OCtCCMCCcCCtCO...",
        "...OCtCcCmCCCCtCO...",
        "...OCSCCCCmCCCSCO...",
        "...OCOCCcCCCcCOCO...",
        "...OccOCCCCCCOccO...",
    ],
    "right": [
        "......OOOOOOOO......",
        ".....OhhHHHHHHO.....",
        "....OhHHHHHHHHHO....",
        "....OHHHHHHHHHHO....",
        "....OHHHHHHSSHHO....",
        "....OHHHHSSSSSSO....",
        "....OHHHHsSSSeSO....",
        "....OHHHHSSSSESO....",
        "....OHHHsSSSRSSO....",
        ".....OHHsSSSSSO.....",
        "....GLOOOssssOO.....",
        "....OCCOTTTTTTO.....",
        "...OCCCOTTtTTTO.....",
        "...OCCCOTTtTTTO.....",
        "...OCcCOTTtTTTO.....",
        "...OCcCOBBSBGBO.....",
        "...OCccOTTTTTTO.....",
        "...OOccOOTTTTOO.....",
    ],
}

# ---------------------------------------------------------------- gambe (righe 18-25)
GAMBE = {
    "fronte": [
        "...OcOOPPPPPPOOcO...",
        "......OPPOOPPO......",
        "......OPPOOPPO......",
        "......OPpOOpPO......",
        "......OKKOOKKO......",
        ".....OKKkOOkKKO.....",
        ".....OOOOOOOOOO.....",
        "....................",
    ],
    "lato": [
        "....OcOOPPPPO.......",
        ".......OPPPPO.......",
        ".......OPPpPO.......",
        ".......OPpPPO.......",
        ".......OKKKKO.......",
        ".......OKKKKKO......",
        ".......OOOOOOO......",
        "....................",
    ],
    "lato_passo": [
        "....OcOOPPPPO.......",
        ".......OPPPPO.......",
        "......OPPOOPPO......",
        "......OPpO.OPPO.....",
        ".....OKKkO.OKKKO....",
        ".....OOOOO.OKKKKO...",
        "...........OOOOOO...",
        "....................",
    ],
}


def gambe_fronte(alza=None):
    """Gambe viste di fronte/retro; alza = 'sx' o 'dx' solleva quella gamba di un pixel."""
    righe = [list(r) for r in GAMBE["fronte"]]
    if alza:
        colonne = range(0, 10) if alza == "sx" else range(10, 20)
        for c in colonne:
            colonna = [righe[r][c] for r in range(1, len(righe))]
            del colonna[1]          # toglie una riga di pantalone: lo stivale sale
            colonna.append(".")
            for r, ch in enumerate(colonna, start=1):
                righe[r][c] = ch
    return ["".join(r) for r in righe]


def componi(busto, gambe, abbassa_tutto=0, abbassa_busto=0, occhi_chiusi=False):
    """Unisce busto e gambe in una griglia ALTEZZA x LARGHEZZA."""
    griglia = [["."] * LARGHEZZA for _ in range(ALTEZZA)]

    def disegna(righe, riga0):
        for r, riga in enumerate(righe):
            y = riga0 + r
            if 0 <= y < ALTEZZA:
                for c, ch in enumerate(riga):
                    if ch != ".":
                        griglia[y][c] = ch

    if occhi_chiusi:
        # la riga con la luce negli occhi diventa pelle, la pupilla diventa una linea
        busto = [riga.replace("e", "S") for riga in busto]
        busto = [riga.replace("E", "O") for riga in busto]
    disegna(gambe, RIGA_GAMBE + abbassa_tutto)
    disegna(busto, abbassa_tutto + abbassa_busto)
    return griglia


def superficie(griglia, specchia=False):
    img = pygame.Surface((LARGHEZZA, ALTEZZA), pygame.SRCALPHA)
    for r, riga in enumerate(griglia):
        for c, ch in enumerate(riga):
            if ch != ".":
                img.set_at((c, r), PALETTE[ch])
    return pygame.transform.flip(img, True, False) if specchia else img


def controlla():
    for nome, griglie in (("busto", BUSTO), ("gambe", GAMBE)):
        for chiave, righe in griglie.items():
            for i, riga in enumerate(righe):
                assert len(riga) == LARGHEZZA, f"{nome}/{chiave} riga {i}: {len(riga)} caratteri"
                for ch in riga:
                    assert ch == "." or ch in PALETTE, f"{nome}/{chiave} riga {i}: colore '{ch}' sconosciuto"


def frames():
    """Restituisce {(tipo, direzione): [griglie]} per camminata e idle (destra; sinistra è specchiata)."""
    controlla()
    risultato = {}
    for d, busto in BUSTO.items():
        if d == "right":
            fermo, passo = GAMBE["lato"], GAMBE["lato_passo"]
            cammina = [componi(busto, fermo), componi(busto, passo),
                       componi(busto, fermo), componi(busto, passo)]
        else:
            fermo = gambe_fronte()
            cammina = [componi(busto, fermo), componi(busto, gambe_fronte("sx")),
                       componi(busto, fermo), componi(busto, gambe_fronte("dx"))]
        # respiro: il busto scende di un pixel a metà ciclo; nell'ultimo frame sbatte le palpebre
        idle = [componi(busto, fermo), componi(busto, fermo, abbassa_busto=1),
                componi(busto, fermo, abbassa_busto=1), componi(busto, fermo, occhi_chiusi=True)]
        risultato[("walk", d)] = cammina
        risultato[("idle", d)] = idle
    return risultato


CARTELLE = {"down": "[DOWN]", "up": "[UP]", "left": "[LEFT]", "right": "[RIGHT]"}


def salva():
    tutti = frames()
    for (tipo, d), griglie in tutti.items():
        varianti = [(d, False)] + ([("left", True)] if d == "right" else [])
        for direzione, specchia in varianti:
            cartella = os.path.join(USCITA, "[MOVEMENT]" if tipo == "walk" else "[IDLE]", CARTELLE[direzione])
            os.makedirs(cartella, exist_ok=True)
            for i, g in enumerate(griglie, start=1):
                nome = f"walk{direzione[0]}{i}.png" if tipo == "walk" else f"idle{i}.png"
                pygame.image.save(superficie(g, specchia), os.path.join(cartella, nome))


def anteprima(percorso_png, scala=8):
    """Salva un foglio con tutti i frame ingranditi, utile per controllare il disegno."""
    tutti = frames()
    righe = [("walk", "down"), ("walk", "up"), ("walk", "right"),
             ("idle", "down"), ("idle", "up"), ("idle", "right")]
    foglio = pygame.Surface((4 * (LARGHEZZA + 4) * scala, len(righe) * (ALTEZZA + 4) * scala))
    foglio.fill((120, 190, 110))
    for r, chiave in enumerate(righe):
        for i, g in enumerate(tutti[chiave]):
            img = pygame.transform.scale(superficie(g), (LARGHEZZA * scala, ALTEZZA * scala))
            foglio.blit(img, ((i * (LARGHEZZA + 4) + 2) * scala, (r * (ALTEZZA + 4) + 2) * scala))
    pygame.image.save(foglio, percorso_png)


if __name__ == "__main__":
    pygame.init()
    salva()
    if len(sys.argv) > 1:
        anteprima(sys.argv[1])
    print("Sprite generati in", USCITA)
