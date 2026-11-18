import pygame
import assets
from hud import testo_con_ombra
from menu import Bottone
from settings import LUNGHEZZA, ALTEZZA, ROSSO, BIANCO


class Overlay:
    """Schermata semitrasparente sopra il gioco (usata per game over e pausa)."""

    def __init__(self, finestra, titolo, bottoni, sottotitolo=""):
        self.finestra = finestra
        self.titolo = titolo
        self.sottotitolo = sottotitolo
        self.bottoni = bottoni
        self.velo = pygame.Surface((LUNGHEZZA, ALTEZZA), pygame.SRCALPHA)
        self.velo.fill((0, 0, 0, 150))

    def draw(self):
        self.finestra.blit(self.velo, (0, 0))
        testo_con_ombra(self.finestra, assets.FONT["dungeon_70"], self.titolo,
                        (LUNGHEZZA // 2, ALTEZZA // 2 - 110), ROSSO, ancora="center")
        if self.sottotitolo:
            testo_con_ombra(self.finestra, assets.FONT["arcade_22"], self.sottotitolo,
                            (LUNGHEZZA // 2, ALTEZZA // 2 - 55), BIANCO, ancora="center")
        for b in self.bottoni.values():
            b.draw(self.finestra)

    def handle_events(self, events):
        """Restituisce la chiave del bottone cliccato, oppure None."""
        for chiave, b in self.bottoni.items():
            if b.cliccato(events):
                return chiave
        return None


CONSIGLI = [
    "Consiglio: colpisci i pipistrelli appena ti arrivano accanto",
    "Consiglio: bere alla fontanella ti cura",
    "Consiglio: quando il boss trema, allontanati di un passo",
    "Consiglio: Ugo ti regala un po' di vita se ci parli",
]


class Try(Overlay):
    """Schermata di game over."""

    def __init__(self, finestra):
        super().__init__(finestra, "Sei Morto", {
            "riprova": Bottone("RIPROVA", (LUNGHEZZA // 2, ALTEZZA // 2 + 20)),
            "menu": Bottone("MENU", (LUNGHEZZA // 2, ALTEZZA // 2 + 90)),
        }, "Premi R per riprovare")
        self.consiglio = 0

    def nuovo_consiglio(self):
        self.consiglio = (self.consiglio + 1) % len(CONSIGLI)

    def draw(self):
        super().draw()
        testo_con_ombra(self.finestra, assets.FONT["arcade_22"], CONSIGLI[self.consiglio],
                        (LUNGHEZZA // 2, ALTEZZA // 2 + 160), (250, 210, 60), ancora="center")

    def handle_events(self, events):
        if any(e.type == pygame.KEYDOWN and e.key in (pygame.K_r, pygame.K_RETURN) for e in events):
            return "riprova"
        return super().handle_events(events)


class Pausa(Overlay):
    def __init__(self, finestra):
        super().__init__(finestra, "Pausa", {
            "riprendi": Bottone("RIPRENDI", (LUNGHEZZA // 2, ALTEZZA // 2 - 10), 260),
            "audio": Bottone("AUDIO: SI", (LUNGHEZZA // 2, ALTEZZA // 2 + 55), 260),
            "menu": Bottone("MENU", (LUNGHEZZA // 2, ALTEZZA // 2 + 120), 260),
        }, "")

    def aggiorna_audio(self, attivo):
        self.bottoni["audio"].testo = "AUDIO: SI" if attivo else "AUDIO: NO"
