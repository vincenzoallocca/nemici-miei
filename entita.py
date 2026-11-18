import math
import pygame


class Entita:
    """Base comune per tutto ciò che si muove sulla griglia isometrica (giocatore, nemici, boss).

    La posizione sullo schermo è il punto dei "piedi" (float), così il movimento è fluido
    e indipendente dalla grandezza dello sprite (che cambia da un frame all'altro).
    """

    def __init__(self, tile_x, tile_y, mondo, finestra, velocita):
        self.mondo = mondo
        self.finestra = finestra
        self.velocita = velocita
        self.direction = "down"
        self.frame_index = 0
        self.frame_counter = 0
        self.flash_timer = 0   # frame in cui lo sprite lampeggia dopo essere stato colpito
        self.piazza(tile_x, tile_y)

    # mette l'entità su un tile senza animazione
    def piazza(self, tile_x, tile_y):
        self.tile_x, self.tile_y = tile_x, tile_y
        self.x, self.y = self.mondo.piedi_tile(tile_x, tile_y)
        self.dest_x, self.dest_y = self.x, self.y

    @property
    def in_movimento(self):
        return (self.x, self.y) != (self.dest_x, self.dest_y)

    @property
    def profondita(self):
        # in isometrico ciò che sta più in basso sullo schermo è più vicino alla telecamera
        return self.y

    def distanza(self, altro):
        """Distanza in tiles (anche le diagonali contano 1)."""
        return max(abs(self.tile_x - altro.tile_x), abs(self.tile_y - altro.tile_y))

    def direzione_verso(self, dx, dy):
        if abs(dx) > abs(dy):
            return "right" if dx > 0 else "left"
        return "down" if dy > 0 else "up"

    # inizia lo spostamento verso un tile adiacente; restituisce False se non è possibile
    def muovi_verso(self, tx, ty, occupati=()):
        if not self.mondo.is_tile_walkable(tx, ty) or (tx, ty) in occupati:
            return False
        self.tile_x, self.tile_y = tx, ty
        self.dest_x, self.dest_y = self.mondo.piedi_tile(tx, ty)
        return True

    # avvicina la posizione alla destinazione
    def aggiorna_movimento(self):
        dx = self.dest_x - self.x
        dy = self.dest_y - self.y
        dist = math.hypot(dx, dy)
        if dist <= self.velocita:
            self.x, self.y = self.dest_x, self.dest_y
        else:
            self.x += dx / dist * self.velocita
            self.y += dy / dist * self.velocita

    # passo verso un bersaglio: prova prima l'asse più lontano, poi l'altro (così non resta bloccato)
    def insegui(self, target_x, target_y, occupati=()):
        dx = target_x - self.tile_x
        dy = target_y - self.tile_y
        if dx == 0 and dy == 0:
            return False
        passo_x = (1 if dx > 0 else -1, 0) if dx else None
        passo_y = (0, 1 if dy > 0 else -1) if dy else None
        tentativi = [passo_x, passo_y] if abs(dx) > abs(dy) else [passo_y, passo_x]
        for passo in tentativi:
            if passo and self.muovi_verso(self.tile_x + passo[0], self.tile_y + passo[1], occupati):
                self.direction = self.direzione_verso(*passo)
                return True
        return False

    def anima(self, frames, velocita_anim):
        lista = frames[self.direction]
        self.frame_counter += 1
        if self.frame_counter >= velocita_anim:
            self.frame_counter = 0
            self.frame_index += 1
        self.frame_index %= len(lista)
        return lista[self.frame_index]

    # ombra ellittica sotto i piedi: aiuta a capire su quale tile si trova l'entità
    def disegna_ombra(self, larghezza):
        altezza = max(4, larghezza // 3)
        ombra = pygame.Surface((larghezza, altezza), pygame.SRCALPHA)
        pygame.draw.ellipse(ombra, (0, 0, 0, 70), ombra.get_rect())
        self.finestra.blit(ombra, ombra.get_rect(center=(round(self.x), round(self.y))))

    def disegna_sprite(self, img, alzo=0):
        rect = img.get_rect(midbottom=(round(self.x), round(self.y) - alzo))
        if self.flash_timer > 0:
            self.flash_timer -= 1
            img = img.copy()
            img.fill((180, 180, 180), special_flags=pygame.BLEND_RGB_ADD)
        self.finestra.blit(img, rect)
        return rect
