import pygame
import assets


class TestoFluttuante:
    """Numero/testo che sale e svanisce (es. danni inflitti, monete raccolte)."""

    def __init__(self, testo, x, y, colore, durata=45):
        self.img = assets.FONT["dungeon_25"].render(testo, True, colore)
        self.ombra = assets.FONT["dungeon_25"].render(testo, True, (0, 0, 0))
        self.x, self.y = x, y
        self.durata = durata
        self.timer = 0

    @property
    def finito(self):
        return self.timer >= self.durata

    def update(self, finestra):
        self.timer += 1
        self.y -= 0.8
        alpha = max(0, 255 - int(255 * self.timer / self.durata))
        for img, dx in ((self.ombra, 2), (self.img, 0)):
            img.set_alpha(alpha)
            finestra.blit(img, img.get_rect(center=(self.x + dx, self.y + dx)))


class Fendente:
    """Arco bianco disegnato davanti al giocatore quando attacca."""

    ANGOLI = {"right": -0.6, "down": 0.9, "left": 2.5, "up": 4.0}

    def __init__(self, x, y, direzione, durata=10):
        self.x, self.y = x, y
        self.direzione = direzione
        self.durata = durata
        self.timer = 0

    @property
    def finito(self):
        return self.timer >= self.durata

    def update(self, finestra):
        self.timer += 1
        inizio = self.ANGOLI.get(self.direzione, 0.9)
        progresso = self.timer / self.durata
        rect = pygame.Rect(0, 0, 70, 46)
        rect.center = (self.x, self.y)
        pygame.draw.arc(finestra, (255, 255, 255), rect, inizio, inizio + 0.6 + 1.6 * progresso, 4)


class Effetti:
    """Contenitore di tutti gli effetti attivi."""

    def __init__(self):
        self.lista = []

    def aggiungi(self, effetto):
        self.lista.append(effetto)

    def testo(self, testo, x, y, colore=(255, 255, 255)):
        self.aggiungi(TestoFluttuante(testo, x, y, colore))

    def update(self, finestra):
        for e in self.lista:
            e.update(finestra)
        self.lista = [e for e in self.lista if not e.finito]
