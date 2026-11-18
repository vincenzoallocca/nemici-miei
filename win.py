import pygame
import assets
from hud import testo_con_ombra
from settings import BIANCO, ROSSO, GIALLO, LUNGHEZZA, ALTEZZA


class Win:
    """Titoli di coda che scorrono dal basso verso l'alto dopo la vittoria."""

    def __init__(self, finestra, monete=0):
        self.finestra = finestra
        self.font2 = assets.FONT["dungeon_50"]
        self.font3 = assets.FONT["arcade_30"]
        self.line_spacing = 44
        self.credit_speed = 1
        self.credit_offset = ALTEZZA  # parte da sotto lo schermo

        self.credits_lines = [
            "HAI VINTO!",
            "",
            f"Monete raccolte: {monete}",
            "",
            "Sviluppato da: Rock A' Raso GAME LLC",
            "",
            "Allocca Vincenzo",
            "Cerciello Antonio",
            "Cestie' Augusto",
            "Pellegrino Andrea",
            "Prisco Carmine",
            "Sdino Raffaele",
            "",
            "Grazie per aver giocato!",
        ]
        # i crediti si fermano quando l'ultima riga arriva a metà schermo
        self.fine_offset = ALTEZZA // 2 - (len(self.credits_lines) - 1) * self.line_spacing

    @property
    def finito(self):
        return self.credit_offset <= self.fine_offset

    def update(self):
        if not self.finito:
            self.credit_offset -= self.credit_speed
        self.draw()

    def draw(self):
        self.finestra.fill((0, 0, 0))
        for i, line in enumerate(self.credits_lines):
            y = self.credit_offset + i * self.line_spacing
            if -50 < y < ALTEZZA + 50:
                colore = GIALLO if line == "HAI VINTO!" else BIANCO
                img = self.font3.render(line, True, colore)
                self.finestra.blit(img, img.get_rect(center=(LUNGHEZZA // 2, y)))

        # titolo fisso sopra i crediti che scorrono
        pygame.draw.rect(self.finestra, (0, 0, 0), (0, 0, LUNGHEZZA, 90))
        testo_con_ombra(self.finestra, self.font2, "Crediti", (LUNGHEZZA // 2, 45), ROSSO, ancora="center")
        if self.finito:
            testo_con_ombra(self.finestra, assets.FONT["arcade_22"], "Premi INVIO per tornare al menu",
                            (LUNGHEZZA // 2, ALTEZZA - 40), ancora="center")

    def handle_events(self, events):
        """Restituisce 'menu' quando il giocatore vuole tornare al menu."""
        for e in events:
            if e.type == pygame.KEYDOWN and e.key in (pygame.K_RETURN, pygame.K_KP_ENTER, pygame.K_ESCAPE):
                return "menu"
            if e.type == pygame.MOUSEBUTTONDOWN and self.finito:
                return "menu"
        return None
