import pygame
import assets
from entita import Entita
from effetti import Fendente
from world import TILES_ERBA, TILES_ACQUA, TILES_PIETRA

VELOCITA_NORMALE = 4
VELOCITA_ACQUA = 2


class Giocatore(Entita):
    def __init__(self, tile_x, tile_y, mondo, finestra, effetti):
        super().__init__(tile_x, tile_y, mondo, finestra, VELOCITA_NORMALE)
        self.effetti = effetti
        self.thirsty = True       # deve bere alla fontanella prima di poter usare il portale
        self.ugo_vita_bonus = False

        # statistiche del giocatore
        self.vita = 100              # salute
        self.vita_max = 100          # salute massima
        self.monete = 0              # numero di monete raccolte
        self.level = 1               # livello
        self.danno = 20              # danno inflitto per colpo
        self.last_attack_time = -10_000
        self.attack_cooldown = 600   # millisecondi tra un attacco e l'altro
        self.attaccato_timer = 0     # frame dell'effetto rosso dopo aver subito danni

    @property
    def vivo(self):
        return self.vita > 0

    def update(self, occupati=(), passo_touch=None):
        if self.in_movimento:
            self.aggiorna_movimento()
        else:
            self.gestisci_input(occupati, passo_touch)
        if self.attaccato_timer > 0:
            self.attaccato_timer -= 1

    def gestisci_input(self, occupati=(), passo_touch=None):
        keys = pygame.key.get_pressed()
        dx = dy = 0
        direzione = None
        if keys[pygame.K_a] or keys[pygame.K_LEFT]:
            dx -= 1
            direzione = "left"
        if keys[pygame.K_d] or keys[pygame.K_RIGHT]:
            dx += 1
            direzione = "right"
        if keys[pygame.K_w] or keys[pygame.K_UP]:
            dy -= 1
            direzione = "up"
        if keys[pygame.K_s] or keys[pygame.K_DOWN]:
            dy += 1
            direzione = "down"
        if not (dx or dy) and passo_touch:
            # joystick del telefono
            dx, dy = passo_touch
            direzione = self.direzione_passo(dx, dy)
        if direzione:
            self.direction = direzione
        if (dx or dy) and self.muovi_verso(self.tile_x + dx, self.tile_y + dy, occupati):
            self.suono_passo()

    @staticmethod
    def direzione_passo(dx, dy):
        """Sprite da usare per un passo: come la tastiera, e per le diagonali la direzione sullo schermo."""
        if dx == -dy:            # destra/sinistra sullo schermo
            return "right" if dx > 0 else "left"
        if dx == dy:             # su/giù sullo schermo
            return "down" if dx > 0 else "up"
        if dy:
            return "down" if dy > 0 else "up"
        return "right" if dx > 0 else "left"

    # riproduce suoni e imposta la velocità in base al tipo di terreno
    def suono_passo(self):
        tile = self.mondo.tipo_tile(self.tile_x, self.tile_y)
        self.velocita = VELOCITA_NORMALE
        if tile in TILES_ERBA:
            assets.suona("grass", maxtime=300, fade_ms=50)
        elif tile in TILES_ACQUA:
            assets.suona("acqua", maxtime=300, fade_ms=50)
            self.velocita = VELOCITA_ACQUA
        elif tile in TILES_PIETRA:
            assets.suona("stone", maxtime=300, fade_ms=50)

    def sul_portale(self):
        return self.mondo.tipo_tile(self.tile_x, self.tile_y) == 20 and not self.in_movimento

    def disegna(self):
        if self.in_movimento:
            img = self.anima(assets.PLAYER_FRAMES, 8)
        else:
            img = self.anima(assets.PLAYER_IDLE, 20)
        self.disegna_ombra(30)
        rect = self.disegna_sprite(img, alzo=4)

        # sovrapposizione rossa quando viene colpito
        if self.attaccato_timer > 0:
            maschera = pygame.mask.from_surface(img)
            rosso = maschera.to_surface(setcolor=(255, 0, 0, 110), unsetcolor=(0, 0, 0, 0))
            self.finestra.blit(rosso, rect)

    # cura il giocatore senza superare la vita massima
    def cura(self, quantita):
        prima = self.vita
        self.vita = min(self.vita_max, self.vita + quantita)
        if self.vita > prima:
            self.effetti.testo(f"+{self.vita - prima}", self.x, self.y - 60, (90, 230, 90))

    # controlla se il giocatore è vicino alla fontanella (anche in diagonale)
    def is_near_fountain(self):
        fx, fy = self.mondo.trova_fontanella()
        if fx is None:
            return False
        # la fontanella occupa il suo tile e quelli "19" attorno
        occupati = [(x, y) for x in range(fx - 1, fx + 1) for y in range(fy - 1, fy + 1)
                    if self.mondo.tipo_tile(x, y) in (18, 19)]
        return any(max(abs(self.tile_x - x), abs(self.tile_y - y)) == 1 for x, y in occupati)

    def puo_bere(self, ha_parlato):
        return self.is_near_fountain() and self.thirsty and ha_parlato

    # bere alla fontanella: sblocca il portale, dà una moneta e cura
    def bevi(self):
        self.thirsty = False
        self.monete += 1
        assets.suona("coin")
        self.effetti.testo("+1 moneta", self.x, self.y - 80, (250, 210, 60))
        self.cura(25)

    # prende danno
    def take_damage(self, amount):
        if not self.vivo:
            return
        self.vita = max(0, self.vita - amount)
        assets.suona("player_dmg", maxtime=300, fade_ms=50)
        self.attaccato_timer = 15
        self.effetti.testo(f"-{amount}", self.x, self.y - 60, (255, 80, 80))

    # controlla se il giocatore è vicino all'npc (anche in diagonale)
    def is_near_npc(self, npc):
        return self.distanza(npc) == 1

    # attacca i nemici vivi vicini; restituisce True se l'attacco è partito
    def attacca(self, nemici):
        ora = pygame.time.get_ticks()
        if ora - self.last_attack_time < self.attack_cooldown:
            return False
        self.last_attack_time = ora
        self.effetti.aggiungi(Fendente(self.x, self.y - 25, self.direction))
        assets.suona("attack")
        for nemico in nemici:
            if nemico.vivo and self.distanza(nemico) <= 1:
                nemico.take_damage(self.danno)
        return True

    # percentuale di ricarica dell'attacco (per la barra nell'hud)
    def ricarica_attacco(self):
        trascorso = pygame.time.get_ticks() - self.last_attack_time
        return min(1.0, trascorso / self.attack_cooldown)
