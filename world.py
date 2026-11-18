import pygame
import random
import assets
from settings import GRANDEZZA_TILES, LUNGHEZZA, ALTEZZA, HUD_HEIGHT

# tiles camminabili
WALKABLE_TILES = {1, 2, 3, 4, 5, 6, 7, 8, 11, 12, 13, 14, 15, 16, 17, 20, 21}
TILES_ERBA = {3, 4, 5}
TILES_ACQUA = {7, 8, 9, 10}
TILES_PIETRA = {11, 12, 13}

# distanza verticale tra l'angolo in alto a sinistra del tile e il punto dove "poggiano i piedi"
OFFSET_PIEDI = 22


class Mondo:
    def __init__(self, livello_id, finestra, livelli):
        self.finestra = finestra
        self.livelli = livelli
        self.sfondi = {1: assets.SFONDO_IMAGES[1], 2: assets.SFONDO_IMAGES[2]}
        self.offset_x = 0
        self.offset_y = 0
        self.nuovo_livello(livello_id)

    # calcola offset per centrare la mappa
    def calcola_offset(self):
        # esclude l'hud dall'altezza totale
        altezza_eff = ALTEZZA - HUD_HEIGHT
        tile_w = GRANDEZZA_TILES

        # conversione coordinate angoli mappa in coordinate isometriche
        angoli = []
        for (tx, ty) in [(0, 0), (self.num_colonne - 1, 0), (0, self.num_righe - 1), (self.num_colonne - 1, self.num_righe - 1)]:
            x = (tx - ty) * (tile_w / 2)
            y = (tx + ty) * (tile_w / 4)
            angoli.append((x, y))

        # calcolo centro griglia
        xs = [p[0] for p in angoli]
        ys = [p[1] for p in angoli]
        centro_x = (min(xs) + max(xs)) / 2
        centro_y = (min(ys) + max(ys)) / 2

        # il -GRANDEZZA_TILES/2 centra il tile (le coordinate sono dell'angolo in alto a sinistra)
        self.offset_x = LUNGHEZZA / 2 - centro_x - tile_w / 2
        self.offset_y = altezza_eff / 2 - centro_y - tile_w / 4

    def carica_mondo(self):
        self.matrice = self.livelli[self.livello_id]
        self.num_righe = len(self.matrice)
        self.num_colonne = len(self.matrice[0])
        self.bloccati = set()        # tiles occupati da oggetti solidi (es. npc)
        self.lista_tiles = []        # terreno
        self.decorazioni = []        # fiori, tronchi, ciottoli: disegnati a terra
        self.lista_fontanelle = []   # oggetti alti: ordinati per profondità insieme ai personaggi
        self.calcola_offset()

        decorazioni_casuali = [
            # (id tile, probabilità)
            (6, 0.10),   # fiori
            (17, 0.05),  # tronchi
            (21, 0.03),  # tulipani
            (15, 0.02),  # ciottoli
        ]
        dim_deco = (GRANDEZZA_TILES // 2, GRANDEZZA_TILES // 2)

        # scorre griglia
        for r in range(self.num_righe):
            for c in range(self.num_colonne):
                # estrae coordinata e verifica se c'é un tile associato a essa
                tile_val = self.matrice[r][c]
                if tile_val not in assets.TILE_IMAGES:
                    continue
                screen_x, screen_y = self.tile_to_screen(c, r)

                # alza il mare
                if tile_val in TILES_ACQUA:
                    screen_y -= 10

                if tile_val == 18:
                    # terreno sotto la fontanella
                    img_ground = pygame.transform.scale(assets.TILE_IMAGES[3], (GRANDEZZA_TILES, GRANDEZZA_TILES))
                    self.lista_tiles.append((img_ground, img_ground.get_rect(topleft=(screen_x, screen_y))))

                    # fontanella
                    img_font = pygame.transform.scale(assets.TILE_IMAGES[tile_val], (100, 100))
                    bottom_center = (screen_x + GRANDEZZA_TILES // 2, screen_y + GRANDEZZA_TILES // 2)
                    rect_font = img_font.get_rect(midbottom=bottom_center)
                    self.lista_fontanelle.append((img_font, rect_font))
                    continue

                # scala a grandezza tile
                img = pygame.transform.scale(assets.TILE_IMAGES[tile_val], (GRANDEZZA_TILES, GRANDEZZA_TILES))
                self.lista_tiles.append((img, img.get_rect(topleft=(screen_x, screen_y))))

                # genera decorazioni casualmente (al massimo una per tile, così non si sovrappongono)
                if tile_val in TILES_ERBA:
                    for deco_id, prob in decorazioni_casuali:
                        if random.random() < prob:
                            img_deco = pygame.transform.scale(assets.TILE_IMAGES[deco_id], dim_deco)
                            centro = (screen_x + GRANDEZZA_TILES // 2, screen_y + GRANDEZZA_TILES // 2)
                            self.decorazioni.append((img_deco, img_deco.get_rect(center=centro)))
                            break

    # conversione da coordinate griglia a schermo (angolo in alto a sinistra dell'immagine del tile)
    def tile_to_screen(self, tx, ty):
        screen_x = (tx - ty) * (GRANDEZZA_TILES / 2) + self.offset_x
        screen_y = (tx + ty) * (GRANDEZZA_TILES / 4) + self.offset_y
        return screen_x, screen_y

    # punto sullo schermo dove un personaggio "poggia i piedi" su un tile
    def piedi_tile(self, tx, ty):
        sx, sy = self.tile_to_screen(tx, ty)
        return sx + GRANDEZZA_TILES / 2, sy + OFFSET_PIEDI

    # conversione da coordinate schermo a griglia (inverso di piedi_tile)
    def screen_to_tile(self, sx, sy):
        a = (sx - self.offset_x - GRANDEZZA_TILES / 2) / (GRANDEZZA_TILES / 2)  # tx - ty
        b = (sy - self.offset_y - OFFSET_PIEDI) / (GRANDEZZA_TILES / 4)         # tx + ty
        return int(round((a + b) / 2)), int(round((b - a) / 2))

    def dentro_mappa(self, tx, ty):
        return 0 <= ty < self.num_righe and 0 <= tx < self.num_colonne

    def tipo_tile(self, tx, ty):
        return self.matrice[ty][tx] if self.dentro_mappa(tx, ty) else None

    # controlla gli ostacoli
    def is_tile_walkable(self, tx, ty):
        if not self.dentro_mappa(tx, ty) or (tx, ty) in self.bloccati:
            return False
        return self.matrice[ty][tx] in WALKABLE_TILES

    # tutti i tiles camminabili che rispettano una condizione (usato per lo spawn dei nemici)
    def tiles_liberi(self, condizione=lambda tx, ty: True):
        return [(c, r) for r in range(self.num_righe) for c in range(self.num_colonne)
                if self.is_tile_walkable(c, r) and condizione(c, r)]

    # disegna sfondo, terreno e decorazioni piatte
    def disegna(self):
        self.finestra.blit(self.sfondi.get(self.livello_id, self.sfondi[1]), (0, 0))
        for img, rect in self.lista_tiles:
            self.finestra.blit(img, rect)
        for img, rect in self.decorazioni:
            self.finestra.blit(img, rect)

    # oggetti alti da ordinare per profondità insieme ai personaggi: (profondità, funzione di disegno)
    def oggetti_alti(self):
        return [(rect.bottom, lambda img=img, rect=rect: self.finestra.blit(img, rect))
                for img, rect in self.lista_fontanelle]

    # trova coordinate sulla griglia fontanella
    def trova_fontanella(self):
        for r in range(self.num_righe):
            for c in range(self.num_colonne):
                if self.matrice[r][c] == 18:
                    return c, r
        return None, None

    # cambia livello
    def nuovo_livello(self, nuovo_livello):
        if nuovo_livello in self.livelli:
            self.livello_id = nuovo_livello
            self.carica_mondo()
