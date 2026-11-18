import pygame
import assets
from entita import Entita

CARICA_NORMALE = 35   # frame di preparazione prima del colpo
CARICA_FURIA = 22
DURATA_RITIRATA = 180  # frame passati a ritirarsi e rigenerarsi dopo un colpo


class Boss(Entita):
    """Il boss insegue il giocatore, carica un colpo e poi si ritira a rigenerarsi.

    Sotto metà vita entra in furia: diventa più grande, rosso, veloce e colpisce più forte.
    """

    def __init__(self, tile_x, tile_y, mondo, finestra, player, effetti, damage=10, hp=150):
        super().__init__(tile_x, tile_y, mondo, finestra, velocita=3)
        self.player = player
        self.effetti = effetti
        self.damage = damage
        self.hp = hp
        self.hp_max = hp
        self.soglia_furia = hp / 2
        self.furia = False
        self.stato = "insegui"     # insegui | carica | ritirata
        self.timer = 0
        self.rigenerazione = 0.05  # vita recuperata per frame durante la ritirata

    @property
    def vivo(self):
        return self.hp > 0

    def update(self, occupati=()):
        if not self.vivo:
            return
        if not self.furia and self.hp <= self.soglia_furia:
            self.entra_in_furia()

        if self.in_movimento:
            self.aggiorna_movimento()

        if self.stato == "ritirata":
            self.timer += 1
            # rigenera, ma senza uscire dalla furia
            limite = self.soglia_furia if self.furia else self.hp_max
            self.hp = min(limite, self.hp + self.rigenerazione)
            if not self.in_movimento:
                angolo = self.angolo_lontano()
                self.insegui(angolo[0], angolo[1], occupati)
            if self.timer >= DURATA_RITIRATA:
                self.stato = "insegui"

        elif self.stato == "carica":
            self.timer += 1
            if self.timer >= (CARICA_FURIA if self.furia else CARICA_NORMALE):
                # il colpo va a segno solo se il giocatore è ancora vicino
                if self.distanza(self.player) <= 1:
                    self.player.take_damage(self.damage)
                assets.suona("boss_attacco")
                self.stato = "ritirata"
                self.timer = 0

        elif not self.in_movimento:
            if self.distanza(self.player) <= 1:
                self.stato = "carica"
                self.timer = 0
                self.direction = self.direzione_verso(self.player.tile_x - self.tile_x,
                                                      self.player.tile_y - self.tile_y)
            else:
                self.insegui(self.player.tile_x, self.player.tile_y, occupati)

    def entra_in_furia(self):
        self.furia = True
        self.velocita = 4.5
        self.damage = int(self.damage * 1.5)
        self.effetti.testo("FURIA!", self.x, self.y - 120, (255, 60, 60))

    # l'angolo della mappa più lontano dal giocatore (dove andare a rigenerarsi)
    def angolo_lontano(self):
        angoli = [(0, 0), (self.mondo.num_colonne - 1, 0), (0, self.mondo.num_righe - 1),
                  (self.mondo.num_colonne - 1, self.mondo.num_righe - 1)]
        return max(angoli, key=lambda a: abs(a[0] - self.player.tile_x) + abs(a[1] - self.player.tile_y))

    def disegna(self):
        if not self.vivo:
            return
        frames = assets.BOSS_FRAMES_FURIA if self.furia else assets.BOSS_FRAMES
        self.disegna_ombra(48 if self.furia else 38)
        if self.stato == "carica":
            # trema mentre prepara il colpo
            img = frames[self.direction][0]
            scossa = 2 if (self.timer // 3) % 2 else -2
            self.x += scossa
            rect = self.disegna_sprite(img)
            self.x -= scossa
            pygame.draw.circle(self.finestra, (255, 60, 60), (rect.centerx, rect.top - 12), 6)
        else:
            self.disegna_sprite(self.anima(frames, 10))

    def take_damage(self, amount):
        if not self.vivo:
            return
        self.hp = max(0, self.hp - amount)
        self.flash_timer = 8
        self.effetti.testo(f"-{amount}", self.x, self.y - 100, (255, 230, 120))
        if not self.vivo:
            assets.suona("morte")
