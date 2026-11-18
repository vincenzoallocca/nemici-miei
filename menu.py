import pygame
import assets
from hud import testo_con_ombra
from settings import BIANCO, ROSSO, ROSSO_SCURO, LUNGHEZZA, ALTEZZA


class Bottone:
    def __init__(self, testo, centro, larghezza=220, altezza=54):
        self.testo = testo
        self.rect = pygame.Rect(0, 0, larghezza, altezza)
        self.rect.center = centro

    def draw(self, finestra):
        sopra = self.rect.collidepoint(pygame.mouse.get_pos())
        pygame.draw.rect(finestra, ROSSO if sopra else ROSSO_SCURO, self.rect, border_radius=8)
        pygame.draw.rect(finestra, BIANCO, self.rect, 3, border_radius=8)
        img = assets.FONT["arcade_40"].render(self.testo, True, BIANCO)
        finestra.blit(img, img.get_rect(center=self.rect.center))

    def cliccato(self, events):
        return any(e.type == pygame.MOUSEBUTTONDOWN and e.button == 1 and self.rect.collidepoint(e.pos)
                   for e in events)


COMANDI = [
    "WASD / Frecce  muoviti",
    "Click / Spazio  attacca",
    "E  interagisci",
    "ESC  pausa      F11  schermo intero",
]


class Menu:
    def __init__(self, finestra):
        self.finestra = finestra
        self.bottone_start = Bottone("START", (LUNGHEZZA // 2, ALTEZZA // 2 + 20))
        self.bottone_esci = Bottone("ESCI", (LUNGHEZZA // 2, ALTEZZA // 2 + 90))

    def draw(self):
        self.finestra.blit(assets.SFONDO_IMAGES[3], (0, 0))
        testo_con_ombra(self.finestra, assets.FONT["dungeon_70"], "Nemici Miei",
                        (LUNGHEZZA // 2, ALTEZZA // 2 - 110), ROSSO, ancora="center")
        self.bottone_start.draw(self.finestra)
        self.bottone_esci.draw(self.finestra)
        y = ALTEZZA - 140
        for riga in COMANDI:
            testo_con_ombra(self.finestra, assets.FONT["arcade_22"], riga, (LUNGHEZZA // 2, y), ancora="center")
            y += 28

    def handle_events(self, events):
        """Restituisce 'start', 'esci' oppure None."""
        for e in events:
            if e.type == pygame.KEYDOWN and e.key in (pygame.K_RETURN, pygame.K_KP_ENTER, pygame.K_SPACE):
                return "start"
        if self.bottone_start.cliccato(events):
            return "start"
        if self.bottone_esci.cliccato(events):
            return "esci"
        return None
