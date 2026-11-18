import math
import pygame
import assets


class Npc:
    """Personaggio fermo con cui si può parlare. Il suo tile non è attraversabile."""

    def __init__(self, tile_x, tile_y, mondo, finestra, image):
        self.mondo = mondo
        self.finestra = finestra
        self.tile_x = tile_x
        self.tile_y = tile_y
        self.parlando = False
        self.image = image
        x, y = self.mondo.piedi_tile(tile_x, tile_y)
        self.rect = self.image.get_rect(midbottom=(round(x), round(y) + 2))
        self.mondo.bloccati.add((tile_x, tile_y))

    @property
    def profondita(self):
        return self.rect.bottom

    def disegna(self):
        self.finestra.blit(self.image, self.rect)

    # piccolo "!" sopra la testa finché non ci si è parlato
    def disegna_avviso(self):
        centro = (self.rect.centerx, self.rect.top - 16 + int(math.sin(pygame.time.get_ticks() / 200) * 3))
        pygame.draw.circle(self.finestra, (0, 0, 0), centro, 13)
        pygame.draw.circle(self.finestra, (250, 210, 60), centro, 11)
        testo = assets.FONT["arcade_22"].render("!", True, (0, 0, 0))
        self.finestra.blit(testo, testo.get_rect(center=centro))
