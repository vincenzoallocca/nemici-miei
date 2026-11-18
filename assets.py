import random
import pygame
from pygame import mixer
from settings import LUNGHEZZA, ALTEZZA, GRANDEZZA_TILES, percorso

# fattori di scala degli sprite (mantengono le proporzioni originali)
SCALA_PLAYER = 2  # sprite dell'eroe 20x26 -> 40x52
SCALA_BAT = 1.5
SCALA_BOSS = 2.0
SCALA_BOSS_FURIA = 2.6
SCALA_NPC = 2.5

DIREZIONI = ("down", "up", "left", "right")
CARTELLE_DIREZIONI = {"down": "[DOWN]", "up": "[UP]", "left": "[LEFT]", "right": "[RIGHT]"}

sounds = {}


def carica_img(*parti):
    """Carica un'immagine con trasparenza ottimizzata per lo schermo."""
    return pygame.image.load(percorso(*parti)).convert_alpha()


def scala(img, fattore):
    w, h = img.get_size()
    return pygame.transform.scale(img, (max(1, round(w * fattore)), max(1, round(h * fattore))))


def carica_animazione(cartella, nomi, fattore):
    """Carica i frame di un'animazione per tutte e quattro le direzioni.

    Nei nomi dei file, {d} viene sostituito con l'iniziale della direzione (d, u, l, r).
    """
    return {
        d: [scala(carica_img(*cartella, CARTELLE_DIREZIONI[d], nome.format(d=d[0])), fattore) for nome in nomi]
        for d in DIREZIONI
    }


def carica_suono(nome, volume):
    # senza scheda audio il gioco funziona lo stesso, solo muto
    if mixer.get_init() is None:
        return None
    try:
        s = mixer.Sound(percorso("audio", nome))
    except pygame.error:
        return None
    s.set_volume(volume)
    return s


def suona(nome, **kwargs):
    """Riproduce un effetto sonoro, scegliendo a caso tra le varianti disponibili."""
    varianti = [s for s in sounds.get(nome, []) if s is not None]
    if varianti:
        random.choice(varianti).play(**kwargs)


def suona_musica(nome, volume):
    """Cambia la musica di sottofondo (in loop)."""
    if mixer.get_init() is None:
        return
    try:
        mixer.music.load(percorso("audio", nome))
        mixer.music.set_volume(volume)
        mixer.music.play(-1)
    except pygame.error:
        pass


# funzione per caricare tutti gli assets (va chiamata dopo pygame.display.set_mode)
def load_assets():
    global sounds, TILE_IMAGES, PLAYER_FRAMES, PLAYER_IDLE, ENEMY_FRAMES, STRING_DIALOGUE
    global PORTAL_FRAMES, NPC_IMAGES, BOSS_FRAMES, BOSS_FRAMES_FURIA, SFONDO_IMAGES, FONT, ICONE

    SFONDO_IMAGES = {
        1: pygame.transform.scale(carica_img('[SFONDI]', 'bg.png'), (LUNGHEZZA, ALTEZZA)),
        2: pygame.transform.scale(carica_img('[SFONDI]', 'bg2.png'), (LUNGHEZZA, ALTEZZA)),
        3: pygame.transform.scale(carica_img('[SFONDI]', 'bg3.png'), (LUNGHEZZA, ALTEZZA)),
    }

    sounds = {
        "grass": [carica_suono(f'grass-00{i}.ogg', 0.1) for i in range(1, 5)],
        "coin": [carica_suono('coin.ogg', 1.0)],
        "morte": [carica_suono('boss_morte.ogg', 0.5)],
        "stone": [carica_suono('stone.ogg', 0.1)],
        "acqua": [carica_suono('acqua.ogg', 0.1)],
        "attack": [carica_suono(f'spada-00{i}.ogg', 0.4) for i in range(1, 5)],
        "player_dmg": [carica_suono('player_dmg.ogg', 0.2)],
        "bat_morte": [carica_suono('bat_mobile.ogg', 0.5)],
        "boss_attacco": [carica_suono(f'zombie-00{i}.ogg', 0.4) for i in range(1, 6)],
    }

    FONT = {
        "arcade_40": pygame.font.Font(percorso("Arcade.ttf"), 40),
        "arcade_30": pygame.font.Font(percorso("Arcade.ttf"), 30),
        "arcade_22": pygame.font.Font(percorso("Arcade.ttf"), 22),
        "dungeon_70": pygame.font.Font(percorso("DungeonFont.ttf"), 70),
        "dungeon_50": pygame.font.Font(percorso("DungeonFont.ttf"), 50),
        "dungeon_25": pygame.font.Font(percorso("DungeonFont.ttf"), 25),
        "dungeon_20": pygame.font.Font(percorso("DungeonFont.ttf"), 20),
    }

    ICONE = {
        "vita": pygame.transform.scale(carica_img('[ITEMS]', 'health.png'), (32, 32)),
        "moneta": pygame.transform.scale(carica_img('[ITEMS]', 'coin.png'), (32, 32)),
    }

    NPC_IMAGES = {
        1: scala(carica_img('[NPC]', '1.png'), SCALA_NPC)
    }

    TILE_IMAGES = {
        1: carica_img('[BLOCCHI]', 'tile_000.png'),     # terreno no erba
        2: carica_img('[BLOCCHI]', 'tile_003.png'),     # terreno no erba scuro
        3: carica_img('[BLOCCHI]', 'tile_022.png'),     # terreno erba
        4: carica_img('[BLOCCHI]', 'tile_023.png'),     # terreno erba 2
        5: carica_img('[BLOCCHI]', 'tile_036.png'),     # terreno erba alta
        6: carica_img('[BLOCCHI]', 'tile_041.png'),     # fiori
        7: carica_img('[BLOCCHI]', 'tile_105.png'),     # mare onda a destra
        8: carica_img('[BLOCCHI]', 'tile_104.png'),     # mare no onde
        9: carica_img('[BLOCCHI]', 'tile_109.png'),     # mare onda destra e sinistra
        10: carica_img('[BLOCCHI]', 'tile_106.png'),    # mare onda sinistra
        11: carica_img('[BLOCCHI]', 'tile_061.png'),    # ciottoli
        12: carica_img('[BLOCCHI]', 'tile_063.png'),    # ciottoli liscio
        13: carica_img('[BLOCCHI]', 'tile_064.png'),    # ciottoli pila
        14: carica_img('[BLOCCHI]', 'tile_077.png'),    # ciottoli in mare
        15: carica_img('[BLOCCHI]', 'tile_053.png'),    # ciottolo arancio
        16: carica_img('[BLOCCHI]', 'tile_049.png'),    # legno pila
        17: carica_img('[BLOCCHI]', 'tile_050.png'),    # legno singolo
        18: carica_img('[BLOCCHI]', 'fontanella.png'),  # autoesplicativo
        19: carica_img('[BLOCCHI]', 'tile_022.png'),    # terra non camminabile (attorno alla fontanella)
        20: carica_img('[BLOCCHI]', 'tile_022.png'),    # terra portale
        21: carica_img('[BLOCCHI]', 'tile_044.png'),    # fiori
        22: carica_img('[BLOCCHI]', 'tile_061.png'),    # ciottoli void
    }

    # il portale viene allargato per sembrare un vortice rotondo
    dim_portale = (int(GRANDEZZA_TILES * 0.75), int(GRANDEZZA_TILES * 0.75))
    PORTAL_FRAMES = [pygame.transform.scale(carica_img('[PORTALE]', f'portal{i}.png'), dim_portale) for i in range(1, 7)]

    PLAYER_FRAMES = carica_animazione(('[EROE]', '[MOVEMENT]'), [f'walk{{d}}{i}.png' for i in range(1, 5)], SCALA_PLAYER)
    PLAYER_IDLE = carica_animazione(('[EROE]', '[IDLE]'), [f'idle{i}.png' for i in range(1, 5)], SCALA_PLAYER)
    ENEMY_FRAMES = carica_animazione(('[MOBS]', '[BAT]'), [f'{i}.png' for i in range(1, 4)], SCALA_BAT)
    BOSS_FRAMES = carica_animazione(('[MOBS]', '[BOSS]'), [f'{i}.png' for i in range(1, 5)], SCALA_BOSS)
    BOSS_FRAMES_FURIA = carica_animazione(('[MOBS]', '[BOSS]'), [f'{i}.png' for i in range(1, 5)], SCALA_BOSS_FURIA)
    # nella fase di furia il boss diventa rossastro
    for frames in BOSS_FRAMES_FURIA.values():
        for img in frames:
            img.fill((70, 0, 0), special_flags=pygame.BLEND_RGB_ADD)

    STRING_DIALOGUE = [
        ("Me", "CHI SEI?! DOVE MI TROVO???"),
        ("Ugo Tognazzi", "No dico noi siamo in due, come se fosse Antani, prefettura?"),
        ("Me", "(???)"),
        ("Ugo Tognazzi", "Dico la supercazzola e' subatomica ma.."),
        ("Ugo Tognazzi", "..prematurata in acqua!"),
        ("Me", "Cosa mi significa?"),
        ("Ugo Tognazzi", "TERAPIATAPIOCO! Bevi alla fontanella... se te ne vuoi andare!"),
    ]


# matrici
livello_1 = [
    [3, 7, 7, 7, 7, 7, 7, 20, 20, 20],
    [4, 8, 8, 8, 8, 8, 8, 3, 3, 3],
    [3, 3, 8, 8, 8, 8, 3, 3, 3, 3],
    [3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
    [3, 3, 3, 3, 19, 19, 4, 3, 3, 3],
    [3, 3, 4, 3, 19, 18, 4, 3, 3, 3],
    [3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
    [3, 3, 3, 3, 4, 3, 3, 3, 4, 3],
    [3, 3, 3, 4, 3, 3, 3, 3, 3, 3],
    [3, 3, 3, 3, 3, 3, 3, 3, 3, 3]
]

livello_2 = [
    [11, 11, 11, 11, 12, 11, 11, 11, 11, 11],
    [11, 11, 11, 11, 11, 11, 11, 11, 11, 11],
    [11, 11, 11, 12, 11, 11, 11, 11, 11, 11],
    [11, 12, 11, 11, 11, 11, 11, 11, 11, 11],
    [11, 11, 11, 11, 11, 11, 12, 11, 11, 11],
    [11, 11, 12, 11, 11, 11, 12, 11, 11, 11],
    [11, 11, 11, 11, 11, 11, 11, 11, 11, 11],
    [11, 11, 11, 11, 12, 11, 11, 11, 12, 11],
    [11, 11, 11, 12, 11, 11, 11, 11, 11, 11],
    [11, 11, 11, 11, 11, 11, 11, 11, 11, 11]
]

livelli = {
    1: livello_1,
    2: livello_2,
}
