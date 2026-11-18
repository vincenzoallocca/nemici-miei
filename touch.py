import math
import pygame
import assets
from settings import LUNGHEZZA, ALTEZZA, HUD_HEIGHT, GRANDEZZA_TILES

# gli 8 passi possibili sulla griglia e la direzione in cui puntano sullo schermo
PASSI = [(dx, dy) for dx in (-1, 0, 1) for dy in (-1, 0, 1) if dx or dy]


def vettore_schermo(dx, dy):
    """Direzione sullo schermo di un passo (dx, dy) sulla griglia isometrica."""
    return (dx - dy) * GRANDEZZA_TILES / 2, (dx + dy) * GRANDEZZA_TILES / 4


class Pulsante:
    def __init__(self, centro, raggio, etichetta):
        self.centro = centro
        self.raggio = raggio
        self.etichetta = etichetta
        self.dito = None   # id del dito che lo sta premendo

    def contiene(self, pos):
        return math.dist(pos, self.centro) <= self.raggio * 1.15

    def draw(self, finestra, font):
        premuto = self.dito is not None
        superficie = pygame.Surface((self.raggio * 2 + 4, self.raggio * 2 + 4), pygame.SRCALPHA)
        c = (self.raggio + 2, self.raggio + 2)
        pygame.draw.circle(superficie, (255, 255, 255, 150 if premuto else 80), c, self.raggio)
        pygame.draw.circle(superficie, (0, 0, 0, 160), c, self.raggio, 3)
        finestra.blit(superficie, superficie.get_rect(center=self.centro))
        testo = font.render(self.etichetta, True, (20, 20, 20))
        finestra.blit(testo, testo.get_rect(center=self.centro))


class ControlliTouch:
    """Joystick virtuale (metà sinistra dello schermo) e pulsanti per giocare da telefono.

    Si attivano da soli al primo tocco sullo schermo.
    """

    RAGGIO_JOYSTICK = 70
    ZONA_MORTA = 18

    def __init__(self):
        self.attivo = False
        base = ALTEZZA - HUD_HEIGHT
        self.attacco = Pulsante((LUNGHEZZA - 105, base - 95), 62, "ATK")
        self.interagisci = Pulsante((LUNGHEZZA - 235, base - 55), 44, "E")
        self.pausa = Pulsante((LUNGHEZZA - 38, 40), 26, "II")
        self.pulsanti = [self.attacco, self.interagisci, self.pausa]
        self.joy_dito = None
        self.joy_centro = (130, base - 110)
        self.joy_pos = self.joy_centro

    @staticmethod
    def posizione(evento):
        # le coordinate delle dita sono normalizzate tra 0 e 1
        return evento.x * LUNGHEZZA, evento.y * ALTEZZA

    def handle_events(self, events):
        """Aggiorna lo stato dei comandi. Restituisce le azioni del frame: 'attacca', 'interagisci', 'pausa'."""
        azioni = set()
        for e in events:
            if e.type == pygame.FINGERDOWN:
                self.attivo = True
                pos = self.posizione(e)
                for p in self.pulsanti:
                    if p.dito is None and p.contiene(pos):
                        p.dito = e.finger_id
                        if p is self.interagisci:
                            azioni.add("interagisci")
                        elif p is self.pausa:
                            azioni.add("pausa")
                        break
                else:
                    if self.joy_dito is None and pos[0] < LUNGHEZZA * 0.55 and pos[1] < ALTEZZA - HUD_HEIGHT:
                        self.joy_dito = e.finger_id
                        self.joy_centro = self.joy_pos = pos
            elif e.type == pygame.FINGERMOTION and e.finger_id == self.joy_dito:
                self.joy_pos = self.posizione(e)
            elif e.type == pygame.FINGERUP:
                if e.finger_id == self.joy_dito:
                    self.joy_dito = None
                for p in self.pulsanti:
                    if p.dito == e.finger_id:
                        p.dito = None
        # tenendo premuto il pulsante si continua ad attaccare (il cooldown limita la frequenza)
        if self.attacco.dito is not None:
            azioni.add("attacca")
        return azioni

    def reset(self):
        self.joy_dito = None
        for p in self.pulsanti:
            p.dito = None

    def passo(self):
        """Passo sulla griglia (dx, dy) indicato dal joystick, oppure None."""
        if self.joy_dito is None:
            return None
        vx = self.joy_pos[0] - self.joy_centro[0]
        vy = self.joy_pos[1] - self.joy_centro[1]
        lunghezza = math.hypot(vx, vy)
        if lunghezza < self.ZONA_MORTA:
            return None

        # sceglie il passo la cui direzione sullo schermo è più simile a quella del dito
        def somiglianza(passo):
            sx, sy = vettore_schermo(*passo)
            return (sx * vx + sy * vy) / math.hypot(sx, sy)

        return max(PASSI, key=somiglianza)

    def draw(self, finestra):
        if not self.attivo:
            return
        # joystick: base e pomello
        centro = self.joy_centro if self.joy_dito is not None else (130, ALTEZZA - HUD_HEIGHT - 110)
        base = pygame.Surface((self.RAGGIO_JOYSTICK * 2 + 4,) * 2, pygame.SRCALPHA)
        c = (self.RAGGIO_JOYSTICK + 2,) * 2
        pygame.draw.circle(base, (255, 255, 255, 60), c, self.RAGGIO_JOYSTICK)
        pygame.draw.circle(base, (0, 0, 0, 120), c, self.RAGGIO_JOYSTICK, 3)
        finestra.blit(base, base.get_rect(center=centro))
        pomello = centro
        if self.joy_dito is not None:
            vx, vy = self.joy_pos[0] - centro[0], self.joy_pos[1] - centro[1]
            d = math.hypot(vx, vy)
            if d > self.RAGGIO_JOYSTICK:
                vx, vy = vx / d * self.RAGGIO_JOYSTICK, vy / d * self.RAGGIO_JOYSTICK
            pomello = (centro[0] + vx, centro[1] + vy)
        pygame.draw.circle(finestra, (255, 255, 255), pomello, 28)
        pygame.draw.circle(finestra, (0, 0, 0), pomello, 28, 3)

        font = assets.FONT["arcade_30"]
        for p in self.pulsanti:
            p.draw(finestra, font)
