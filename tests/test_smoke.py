"""Test automatici senza finestra: simulano una partita completa.

Esecuzione:  python -m unittest discover -s tests
"""
import os
import sys
import unittest

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
os.environ.setdefault("SDL_AUDIODRIVER", "dummy")
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import pygame  # noqa: E402

pygame.init()
pygame.display.set_mode((960, 720))

import assets  # noqa: E402

assets.load_assets()

from main import Gioco  # noqa: E402


def tasto(key):
    return pygame.event.Event(pygame.KEYDOWN, key=key, mod=0, unicode="", scancode=0)


def click(pos=(0, 0)):
    return pygame.event.Event(pygame.MOUSEBUTTONDOWN, button=1, pos=pos)


class TestPartita(unittest.TestCase):
    def setUp(self):
        self.finestra = pygame.display.get_surface()
        self.gioco = Gioco(self.finestra)

    def frames(self, n, events=()):
        for _ in range(n):
            self.assertTrue(self.gioco.step(list(events)))

    def inizia(self):
        self.gioco.step([tasto(pygame.K_RETURN)])
        self.assertEqual(self.gioco.stato, "gioco")

    def test_menu_e_avvio(self):
        self.frames(5)
        self.assertEqual(self.gioco.stato, "menu")
        self.inizia()
        g = self.gioco
        self.assertTrue(3 <= len(g.bats) <= 5)
        # nessun pipistrello nasce vicino al giocatore o su un tile non camminabile
        for b in g.bats:
            self.assertGreaterEqual(max(b.tile_x, b.tile_y), 4)
            self.assertTrue(g.mondo.is_tile_walkable(b.tile_x, b.tile_y))
        self.frames(120)

    def test_coordinate_isometriche(self):
        self.inizia()
        m = self.gioco.mondo
        for tx in range(m.num_colonne):
            for ty in range(m.num_righe):
                self.assertEqual(m.screen_to_tile(*m.piedi_tile(tx, ty)), (tx, ty))

    def test_partita_completa(self):
        self.inizia()
        g = self.gioco
        g.bats = []  # isola la logica della storia dai pipistrelli

        # il portale non funziona finché non si è bevuto
        g.player.piazza(8, 0)
        self.frames(2)
        self.assertEqual(g.mondo.livello_id, 1)

        # parla con Ugo fino alla fine del dialogo
        g.player.piazza(8, 7)
        self.assertTrue(g.player.is_near_npc(g.npc))
        self.assertFalse(g.mondo.is_tile_walkable(8, 8), "l'npc deve bloccare il suo tile")
        for _ in range(len(assets.STRING_DIALOGUE) + 1):
            self.frames(1, [tasto(pygame.K_e)])
        self.assertTrue(g.hud.giaparlato)

        # beve alla fontanella (anche da un tile in diagonale)
        g.player.piazza(6, 6)
        self.assertTrue(g.player.is_near_fountain())
        self.frames(1, [tasto(pygame.K_e)])
        self.assertFalse(g.player.thirsty)
        self.assertEqual(g.player.monete, 1)

        # entra nel portale
        g.player.piazza(8, 0)
        self.frames(2)
        self.assertEqual(g.mondo.livello_id, 2)
        self.assertIsNotNone(g.boss)

        # combatte col boss
        boss = g.boss
        for _ in range(20):
            if not boss.vivo:
                break
            g.player.piazza(boss.tile_x + 1 if boss.tile_x < 9 else boss.tile_x - 1, boss.tile_y)
            g.player.last_attack_time = -10_000
            g.player.vita = 100
            self.frames(1, [click()])
        self.assertFalse(boss.vivo)
        self.assertTrue(boss.furia)

        self.frames(130)
        self.assertEqual(g.stato, "vittoria")
        self.frames(1, [tasto(pygame.K_RETURN)])
        self.assertEqual(g.stato, "menu")

    def test_morte_e_riprova(self):
        self.inizia()
        g = self.gioco
        g.player.take_damage(1000)
        self.frames(1)
        self.assertEqual(g.stato, "morto")
        self.frames(1, [tasto(pygame.K_r)])
        self.assertEqual(g.stato, "gioco")
        self.assertEqual(g.player.vita, 100)
        self.assertEqual(g.mondo.livello_id, 1)

    def test_comandi_touch(self):
        self.inizia()
        g = self.gioco
        g.bats = []

        def dito(tipo, fid, x, y):
            return pygame.event.Event(tipo, finger_id=fid, touch_id=0, x=x / 960, y=y / 720, dx=0, dy=0, pressure=1)

        # trascinando il joystick verso destra il giocatore si sposta a destra sullo schermo
        g.player.piazza(2, 7)
        x0 = g.player.x
        self.frames(1, [dito(pygame.FINGERDOWN, 1, 200, 400)])
        self.assertTrue(g.touch.attivo)
        self.frames(1, [dito(pygame.FINGERMOTION, 1, 300, 400)])
        self.assertEqual(g.touch.passo(), (1, -1))
        self.frames(30)
        self.assertGreater(g.player.x, x0)
        self.assertEqual(g.player.direction, "right")
        self.frames(1, [dito(pygame.FINGERUP, 1, 300, 400)])
        self.assertIsNone(g.touch.passo())

        # il pulsante di attacco fa partire l'attacco
        g.player.last_attack_time = -10_000
        cx, cy = g.touch.attacco.centro
        self.frames(1, [dito(pygame.FINGERDOWN, 2, cx, cy)])
        self.assertGreater(g.player.last_attack_time, -10_000)
        self.frames(1, [dito(pygame.FINGERUP, 2, cx, cy)])

        # il pulsante pausa mette in pausa
        px, py = g.touch.pausa.centro
        self.frames(1, [dito(pygame.FINGERDOWN, 3, px, py)])
        self.assertEqual(g.stato, "pausa")

    def test_pausa(self):
        self.inizia()
        self.frames(1, [tasto(pygame.K_ESCAPE)])
        self.assertEqual(self.gioco.stato, "pausa")
        self.frames(1, [tasto(pygame.K_ESCAPE)])
        self.assertEqual(self.gioco.stato, "gioco")

    def test_pipistrelli_attaccano_e_muoiono(self):
        self.inizia()
        g = self.gioco
        # lascia che i pipistrelli raggiungano il giocatore fermo
        self.frames(60 * 15)
        self.assertLess(g.player.vita, 100)
        vicini = [b for b in g.bats if b.distanza(g.player) <= 1]
        self.assertTrue(vicini)
        # i pipistrelli non si sovrappongono
        posizioni = [(b.tile_x, b.tile_y) for b in g.bats]
        self.assertEqual(len(posizioni), len(set(posizioni)))
        monete = g.player.monete
        bat = vicini[0]
        while bat.vivo:
            g.player.last_attack_time = -10_000
            g.player.vita = 100
            self.frames(1, [tasto(pygame.K_SPACE)])
        self.assertGreater(g.player.monete, monete)


if __name__ == "__main__":
    unittest.main()
