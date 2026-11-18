import pygame
import assets
from hud import testo_con_ombra
from settings import BIANCO, ROSSO, ROSSO_SCURO, LUNGHEZZA, ALTEZZA, WEB, TASTO_PAUSA, dispositivo_touch


class Bottone:
    def __init__(self, testo, centro, larghezza=220, altezza=54):
        self.testo = testo
        self.rect = pygame.Rect(0, 0, larghezza, altezza)
        self.rect.center = centro

    def draw(self, finestra):
        sopra = self.rect.collidepoint(pygame.mouse.get_pos()) and pygame.mouse.get_focused()
        pygame.draw.rect(finestra, ROSSO if sopra else ROSSO_SCURO, self.rect, border_radius=8)
        pygame.draw.rect(finestra, BIANCO, self.rect, 3, border_radius=8)
        img = assets.FONT["arcade_40"].render(self.testo, True, BIANCO)
        finestra.blit(img, img.get_rect(center=self.rect.center))

    def cliccato(self, events):
        """True se il pulsante è stato cliccato in questo frame.

        Nel browser a volte si perde il "premi" (il primo click dà il focus alla pagina) e a volte
        il "rilascia": il pulsante scatta con il primo dei due e ignora l'altro, così non scatta mai due volte.
        """
        for e in events:
            if getattr(e, "button", 1) != 1 or not hasattr(e, "pos"):
                continue
            if e.type == pygame.MOUSEBUTTONDOWN and self.rect.collidepoint(e.pos):
                self.premuto = True
                return True
            if e.type == pygame.MOUSEBUTTONUP:
                gia_gestito = getattr(self, "premuto", False)
                self.premuto = False
                if not gia_gestito and self.rect.collidepoint(e.pos):
                    return True
        return False


COMANDI = [
    "WASD / Frecce  muoviti",
    "Click / Spazio  attacca",
    "E  parla e interagisci",
    f"{TASTO_PAUSA}  pausa      M  audio" + ("" if WEB else "      F11  schermo intero"),
]
COMANDI_TOUCH = [
    "Trascina il dito a sinistra per muoverti",
    "ATK  attacca      E  parla e interagisci",
    "II  pausa",
]


class Menu:
    def __init__(self, finestra):
        self.finestra = finestra
        self.bottone_start = Bottone("START", (LUNGHEZZA // 2, ALTEZZA // 2 + 20))
        # nel browser non si può chiudere la finestra: il bottone ESCI serve solo su pc
        self.bottone_esci = None if WEB else Bottone("ESCI", (LUNGHEZZA // 2, ALTEZZA // 2 + 90))
        self.touch = False   # diventa True al primo tocco sullo schermo

    def draw(self):
        self.finestra.blit(assets.SFONDO_IMAGES[3], (0, 0))
        testo_con_ombra(self.finestra, assets.FONT["dungeon_70"], "Nemici Miei",
                        (LUNGHEZZA // 2, ALTEZZA // 2 - 110), ROSSO, ancora="center")
        self.bottone_start.draw(self.finestra)
        if self.bottone_esci:
            self.bottone_esci.draw(self.finestra)
        righe = COMANDI_TOUCH if (self.touch or dispositivo_touch()) else COMANDI
        riquadro = pygame.Surface((560, len(righe) * 28 + 20), pygame.SRCALPHA)
        riquadro.fill((0, 0, 0, 150))
        y = ALTEZZA - 40 - len(righe) * 28
        self.finestra.blit(riquadro, riquadro.get_rect(midtop=(LUNGHEZZA // 2, y - 10)))
        for riga in righe:
            testo_con_ombra(self.finestra, assets.FONT["arcade_22"], riga, (LUNGHEZZA // 2, y + 12), ancora="center")
            y += 28

    def handle_events(self, events):
        """Restituisce 'start', 'esci' oppure None."""
        for e in events:
            if e.type == pygame.KEYDOWN and e.key in (pygame.K_RETURN, pygame.K_KP_ENTER, pygame.K_SPACE):
                return "start"
        if self.bottone_start.cliccato(events):
            return "start"
        if self.bottone_esci and self.bottone_esci.cliccato(events):
            return "esci"
        return None
