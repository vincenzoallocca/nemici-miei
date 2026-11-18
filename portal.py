import assets
from settings import GRANDEZZA_TILES


class Portale:
    """Portale animato che porta al livello successivo."""

    def __init__(self, tile_x, tile_y, mondo, finestra):
        self.mondo = mondo
        self.finestra = finestra
        self.tile_x = tile_x
        self.tile_y = tile_y
        self.frame_index = 0
        self.frame_counter = 0
        sx, sy = self.mondo.tile_to_screen(tile_x, tile_y)
        bottom_center = (sx + GRANDEZZA_TILES // 2, sy + GRANDEZZA_TILES // 2)
        self.rect = assets.PORTAL_FRAMES[0].get_rect(midbottom=bottom_center)

    @property
    def profondita(self):
        return self.rect.bottom

    def update(self):
        self.frame_counter += 1
        if self.frame_counter >= 10:
            self.frame_counter = 0
            self.frame_index = (self.frame_index + 1) % len(assets.PORTAL_FRAMES)

    def disegna(self, attivo=True):
        img = assets.PORTAL_FRAMES[self.frame_index]
        if not attivo:
            # finché il giocatore non ha bevuto il portale resta "spento"
            img = img.copy()
            img.set_alpha(110)
        self.finestra.blit(img, self.rect)
