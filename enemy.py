import math
import pygame
import assets
from entita import Entita


class Enemy(Entita):
    """Pipistrello: insegue il giocatore e lo morde quando gli è accanto."""

    def __init__(self, tile_x, tile_y, mondo, finestra, player, effetti, hp=50, cooldown=2500, velocita=2, dmg=10):
        super().__init__(tile_x, tile_y, mondo, finestra, velocita)
        self.player = player
        self.effetti = effetti
        self.hp = hp
        self.hp_max = hp
        self.cooldown = cooldown
        self.dmg = dmg
        self.last_attack_time = 0
        self.vicino = False   # True quando è accanto al giocatore
        self.fase_volo = tile_x + tile_y  # sfasa l'oscillazione tra i pipistrelli

    @property
    def vivo(self):
        return self.hp > 0

    def update(self, occupati=()):
        if not self.vivo:
            return
        if self.in_movimento:
            self.aggiorna_movimento()
        elif self.distanza(self.player) <= 1:
            self.attacca()
        else:
            self.vicino = False
            # insegue il giocatore senza entrare nel suo tile o in quello di un altro nemico
            self.insegui(self.player.tile_x, self.player.tile_y, occupati)

    def attacca(self):
        ora = pygame.time.get_ticks()
        if not self.vicino:
            # appena arriva accanto aspetta un attimo prima di mordere: il giocatore può reagire
            self.vicino = True
            self.last_attack_time = ora - self.cooldown + 700
        if ora - self.last_attack_time >= self.cooldown:
            self.player.take_damage(self.dmg)
            self.last_attack_time = ora

    def disegna(self):
        if not self.vivo:
            return
        self.disegna_ombra(26)
        self.fase_volo += 0.08
        alzo = 30 + int(math.sin(self.fase_volo) * 4)  # svolazza sopra il terreno
        self.disegna_sprite(self.anima(assets.ENEMY_FRAMES, 10), alzo)

    def take_damage(self, amount):
        if not self.vivo:
            return
        self.hp = max(0, self.hp - amount)
        self.flash_timer = 8
        self.effetti.testo(f"-{amount}", self.x, self.y - 60, (255, 230, 120))
        if not self.vivo:
            assets.suona("bat_morte")
            # ogni pipistrello sconfitto lascia una moneta
            self.player.monete += 1
            assets.suona("coin")
            self.effetti.testo("+1 moneta", self.x, self.y - 85, (250, 210, 60))
