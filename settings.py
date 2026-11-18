import os
import sys

# cartella del progetto: permette di avviare il gioco da qualsiasi directory
BASE_DIR = os.path.dirname(os.path.abspath(__file__))


def percorso(*parti):
    """Restituisce il percorso assoluto di un file dentro la cartella assets."""
    return os.path.join(BASE_DIR, "assets", *parti)


# True quando il gioco gira nel browser (pygbag / WebAssembly)
WEB = sys.platform == "emscripten"



def dispositivo_touch():
    """True se il gioco gira nel browser di un telefono/tablet (schermo touch come puntatore principale)."""
    if not WEB:
        return False
    try:
        import platform  # nel browser è il modulo di pygbag che dà accesso a window
        return bool(platform.window.matchMedia("(pointer: coarse)").matches)
    except Exception:
        return False


# tasto pausa mostrato nelle scritte (nel browser ESC è spesso intercettato dalla pagina)
TASTO_PAUSA = "P" if WEB else "ESC"

# finestra
LUNGHEZZA = 960
ALTEZZA = 720
FPS = 60
TITOLO = "Nemici Miei | Rock A' Raso"

# mondo isometrico
GRANDEZZA_TILES = 72
HUD_HEIGHT = 100

# colori
ROSSO = (255, 0, 0)
ROSSO_SCURO = (140, 20, 20)
VERDE = (60, 200, 80)
GIALLO = (250, 210, 60)
BLU = (0, 0, 255)
BIANCO = (255, 255, 255)
NERO = (0, 0, 0)
GRIGIO = (50, 50, 50)
GRIGIO_CHIARO = (120, 120, 120)
TRASPARENTE = (0, 0, 0, 128)
