import pygame
import assets
from settings import ALTEZZA, LUNGHEZZA, HUD_HEIGHT, GRIGIO, BIANCO, ROSSO, GIALLO, VERDE, ROSSO_SCURO


def dividi_testo(testo, font, larghezza_max):
    """Spezza un testo in righe che non superano larghezza_max pixel."""
    righe, riga = [], ""
    for parola in testo.split():
        prova = f"{riga} {parola}".strip()
        if font.size(prova)[0] <= larghezza_max:
            riga = prova
        else:
            if riga:
                righe.append(riga)
            riga = parola
    if riga:
        righe.append(riga)
    return righe


def testo_con_ombra(finestra, font, testo, pos, colore=BIANCO, ancora="topleft"):
    ombra = font.render(testo, True, (0, 0, 0))
    img = font.render(testo, True, colore)
    rect = img.get_rect(**{ancora: pos})
    finestra.blit(ombra, rect.move(2, 2))
    finestra.blit(img, rect)
    return rect


def barra(finestra, rect, percentuale, colore, sfondo=(30, 30, 30)):
    pygame.draw.rect(finestra, sfondo, rect, border_radius=4)
    piena = rect.copy()
    piena.width = int(rect.width * max(0.0, min(1.0, percentuale)))
    if piena.width > 0:
        pygame.draw.rect(finestra, colore, piena, border_radius=4)
    pygame.draw.rect(finestra, (0, 0, 0), rect, 2, border_radius=4)


class HUD:
    def __init__(self, finestra, player, npc, boss, mondo):
        self.finestra = finestra
        self.player = player
        self.npc = npc
        self.boss = boss
        self.mondo = mondo
        self.font = assets.FONT["arcade_30"]
        self.font_piccolo = assets.FONT["arcade_22"]
        self.font2 = assets.FONT["dungeon_25"]
        self.font3 = assets.FONT["dungeon_20"]
        self.dialogue_index = 0
        self.giaparlato = False

    # ---------- dialogo con l'npc ----------
    def interagisci_npc(self):
        """Avanza il dialogo (chiamato quando si preme E vicino all'npc)."""
        if self.giaparlato:
            return
        if not self.npc.parlando:
            self.npc.parlando = True
            self.dialogue_index = 0
        else:
            self.dialogue_index += 1
            if self.dialogue_index >= len(assets.STRING_DIALOGUE):
                self.npc.parlando = False
                self.giaparlato = True
                return
        # Ugo regala un po' di vita durante la chiacchierata (una volta sola)
        if self.dialogue_index == 1 and not self.player.ugo_vita_bonus:
            self.player.ugo_vita_bonus = True
            self.player.cura(15)

    def aggiorna_dialogo(self):
        # se il giocatore si allontana il dialogo si interrompe
        if self.npc.parlando and not self.player.is_near_npc(self.npc):
            self.npc.parlando = False
            self.dialogue_index = 0

    def obiettivo(self):
        if self.mondo.livello_id == 2:
            return "Sconfiggi il boss!" if self.boss is None or self.boss.vivo else "Boss sconfitto!"
        if not self.giaparlato:
            return "Parla con Ugo"
        if self.player.thirsty:
            return "Bevi alla fontanella"
        return "Entra nel portale"

    # ---------- disegno ----------
    def draw(self):
        self.aggiorna_dialogo()
        top = ALTEZZA - HUD_HEIGHT
        pygame.draw.rect(self.finestra, GRIGIO, (0, top, LUNGHEZZA, HUD_HEIGHT))
        pygame.draw.line(self.finestra, (20, 20, 20), (0, top), (LUNGHEZZA, top), 3)

        # vita
        self.finestra.blit(assets.ICONE["vita"], (20, top + 14))
        barra(self.finestra, pygame.Rect(60, top + 18, 170, 24), self.player.vita / self.player.vita_max,
              VERDE if self.player.vita > 30 else ROSSO)
        testo_con_ombra(self.finestra, self.font3, f"{self.player.vita} / {self.player.vita_max}",
                        (145, top + 30), ancora="center")

        # monete
        self.finestra.blit(assets.ICONE["moneta"], (20, top + 56))
        self.finestra.blit(self.font2.render(f"Monete: {self.player.monete}", True, BIANCO), (60, top + 60))

        # livello e ricarica dell'attacco
        self.finestra.blit(self.font2.render(f"Livello: {self.player.level}", True, BIANCO), (260, top + 16))
        self.finestra.blit(self.font3.render("Attacco", True, BIANCO), (260, top + 52))
        barra(self.finestra, pygame.Rect(340, top + 56, 100, 14), self.player.ricarica_attacco(), GIALLO)

        # parte destra: dialogo, suggerimento o vita del boss
        destra = pygame.Rect(470, top + 10, LUNGHEZZA - 490, HUD_HEIGHT - 20)
        if self.npc.parlando and self.mondo.livello_id == 1:
            self.disegna_dialogo(destra)
        elif self.mondo.livello_id == 2 and self.boss.vivo:
            self.disegna_vita_boss(destra)
        elif self.mostra_suggerimento():
            testo_con_ombra(self.finestra, self.font, "Premi (E) per interagire", destra.center, ancora="center")

        # obiettivo in alto a sinistra e comandi in alto a destra
        testo_con_ombra(self.finestra, self.font2, f"Obiettivo: {self.obiettivo()}", (16, 12), GIALLO)
        testo_con_ombra(self.finestra, self.font_piccolo, "ESC  pausa", (LUNGHEZZA - 16, 14), ancora="topright")

        # "!" sopra l'npc finché non ci si parla
        if self.mondo.livello_id == 1 and not self.giaparlato and not self.npc.parlando:
            self.npc.disegna_avviso()

    def mostra_suggerimento(self):
        if self.mondo.livello_id != 1:
            return False
        vicino_fontana = self.player.puo_bere(self.giaparlato)
        vicino_npc = self.player.is_near_npc(self.npc) and not self.giaparlato
        return vicino_fontana or vicino_npc

    def disegna_dialogo(self, rect):
        nome, testo = assets.STRING_DIALOGUE[self.dialogue_index]
        pygame.draw.rect(self.finestra, (25, 25, 25), rect, border_radius=6)
        pygame.draw.rect(self.finestra, (90, 90, 90), rect, 2, border_radius=6)
        colore_nome = GIALLO if nome == "Me" else ROSSO
        self.finestra.blit(self.font3.render(nome, True, colore_nome), (rect.x + 10, rect.y + 4))
        y = rect.y + 26
        for riga in dividi_testo(testo, self.font3, rect.width - 20)[:2]:
            self.finestra.blit(self.font3.render(riga, True, BIANCO), (rect.x + 10, y))
            y += 22
        altro = self.font_piccolo.render("E >", True, (180, 180, 180))
        self.finestra.blit(altro, altro.get_rect(bottomright=(rect.right - 8, rect.bottom - 2)))

    def disegna_vita_boss(self, rect):
        etichetta = "BOSS - FURIA!" if self.boss.furia else "BOSS"
        self.finestra.blit(self.font2.render(etichetta, True, ROSSO if self.boss.furia else BIANCO), (rect.x, rect.y + 2))
        barra(self.finestra, pygame.Rect(rect.x, rect.y + 36, rect.width, 22), self.boss.hp / self.boss.hp_max,
              ROSSO, ROSSO_SCURO)
        testo_con_ombra(self.finestra, self.font3, f"{int(self.boss.hp)} / {int(self.boss.hp_max)}",
                        (rect.centerx, rect.y + 47), ancora="center")
