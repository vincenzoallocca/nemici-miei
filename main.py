import asyncio
import random
import pygame
import assets
import math
from settings import LUNGHEZZA, ALTEZZA, FPS, TITOLO, WEB, TASTO_PAUSA, percorso, dispositivo_touch
from world import Mondo
from player import Giocatore
from hud import HUD, testo_con_ombra
from enemy import Enemy
from boss import Boss
from npc import Npc
from portal import Portale
from menu import Menu
from win import Win
from riprova import Try, Pausa
from effetti import Effetti
from touch import ControlliTouch

# posizioni sulla griglia (colonna, riga)
SPAWN_GIOCATORE = (0, 0)
POSIZIONE_NPC = (8, 8)
POSIZIONE_PORTALE = (8, 0)
SPAWN_BOSS = (0, 0)
FRAME_PRIMA_DEI_CREDITI = 120
FRAME_DI_TREGUA = 180   # all'inizio i pipistrelli aspettano 3 secondi: il tempo di leggere il suggerimento


class Gioco:
    """Gestisce gli stati del gioco: menu, gioco, pausa, morto, vittoria."""

    def __init__(self, finestra):
        self.finestra = finestra
        self.menu = Menu(finestra)
        self.schermata_morte = Try(finestra)
        self.schermata_pausa = Pausa(finestra)
        self.win = None
        self.canzone = None
        self.touch = ControlliTouch()
        self.volume_musica = 0.3
        if dispositivo_touch():
            self.touch.attivo = self.menu.touch = True
        self.vai_al_menu()

    # ---------- utilità ----------
    def cambia_musica(self, nome, volume):
        if nome != self.canzone:
            self.canzone = nome
            self.volume_musica = volume
            assets.suona_musica(nome, volume)

    def cambia_audio(self):
        assets.imposta_audio(not assets.audio_attivo, self.volume_musica)
        self.schermata_pausa.aggiorna_audio(assets.audio_attivo)

    @property
    def comando_attacco(self):
        return "ATK" if self.touch.attivo else "Click o Spazio"

    def vai_al_menu(self):
        self.stato = "menu"
        self.cambia_musica("soundtrack.ogg", 0.2)

    # crea da zero tutti gli oggetti della partita (usato anche per "riprova")
    def nuova_partita(self):
        f = self.finestra
        self.effetti = Effetti()
        self.mondo = Mondo(1, f, assets.livelli)
        self.player = Giocatore(*SPAWN_GIOCATORE, self.mondo, f, self.effetti)
        self.npc = Npc(*POSIZIONE_NPC, self.mondo, f, assets.NPC_IMAGES[1])
        self.portale = Portale(*POSIZIONE_PORTALE, self.mondo, f)
        self.bats = self.genera_pipistrelli(random.randint(3, 5))
        self.boss = None
        self.hud = HUD(f, self.player, self.npc, self.boss, self.mondo)
        self.hud.touch = self.touch.attivo
        self.touch.reset()
        if self.touch.attivo:
            self.hud.mostra_messaggio("Trascina il dito a sinistra per muoverti. Raggiungi Ugo, il mago con il punto esclamativo.", 7)
        else:
            self.hud.mostra_messaggio("Muoviti con WASD o le frecce. Raggiungi Ugo, il mago con il punto esclamativo.", 7)
        self.timer_vittoria = 0
        self.tregua = FRAME_DI_TREGUA
        self.stato = "gioco"
        self.cambia_musica("soundtrack.ogg", 0.2)

    def genera_pipistrelli(self, quanti):
        # niente pipistrelli addosso al giocatore o sul portale
        px, py = SPAWN_GIOCATORE
        liberi = self.mondo.tiles_liberi(
            lambda tx, ty: max(abs(tx - px), abs(ty - py)) >= 4 and self.mondo.tipo_tile(tx, ty) != 20)
        posizioni = random.sample(liberi, min(quanti, len(liberi)))
        return [Enemy(tx, ty, self.mondo, self.finestra, self.player, self.effetti) for tx, ty in posizioni]

    def entra_livello_2(self):
        self.mondo.nuovo_livello(2)
        self.player.level = 2
        self.player.piazza(self.player.tile_x, self.player.tile_y)
        self.bats = []
        self.boss = Boss(*SPAWN_BOSS, self.mondo, self.finestra, self.player, self.effetti)
        self.hud.boss = self.boss
        self.tregua = FRAME_DI_TREGUA
        self.effetti.testo("LIVELLO 2", LUNGHEZZA // 2, ALTEZZA // 2 - 120, (255, 80, 80))
        self.hud.mostra_messaggio(f"Sconfiggi il boss! Attacca con {self.comando_attacco} e allontanati quando trema.", 7)
        self.cambia_musica("boss_soundtrack.ogg", 1.0)

    def nemici_vivi(self):
        nemici = self.bats if self.mondo.livello_id == 1 else [self.boss]
        return [n for n in nemici if n is not None and n.vivo]

    # ---------- logica ----------
    def step(self, events):
        """Esegue un frame. Restituisce False quando il gioco deve chiudersi."""
        for e in events:
            if e.type == pygame.QUIT:
                return False
            if e.type == pygame.FINGERDOWN:
                # primo tocco: mostra i comandi touch anche nel menu
                self.touch.attivo = self.menu.touch = True
            if e.type == pygame.KEYDOWN and e.key == pygame.K_m and self.stato in ("gioco", "pausa", "menu"):
                self.cambia_audio()
            if e.type == pygame.KEYDOWN and e.key == pygame.K_F11 and not WEB:
                pygame.display.toggle_fullscreen()

        if self.stato == "menu":
            scelta = self.menu.handle_events(events)
            if scelta == "esci":
                return False
            if scelta == "start":
                self.nuova_partita()
            else:
                self.menu.draw()

        elif self.stato == "gioco":
            self.aggiorna_gioco(events)
            self.disegna_gioco()

        elif self.stato == "pausa":
            self.schermata_pausa.sottotitolo = "" if self.touch.attivo else f"{TASTO_PAUSA} per riprendere, M per l'audio"
            self.disegna_gioco(aggiorna_animazioni=False)
            self.schermata_pausa.draw()
            scelta = self.schermata_pausa.handle_events(events)
            esc = any(e.type == pygame.KEYDOWN and e.key in (pygame.K_ESCAPE, pygame.K_p) for e in events)
            if scelta == "audio":
                self.cambia_audio()
            elif scelta == "riprendi" or esc:
                self.stato = "gioco"
            elif scelta == "menu":
                self.vai_al_menu()

        elif self.stato == "morto":
            self.schermata_morte.sottotitolo = "" if self.touch.attivo else "Premi R per riprovare"
            self.disegna_gioco(aggiorna_animazioni=False)
            self.schermata_morte.draw()
            scelta = self.schermata_morte.handle_events(events)
            if scelta == "riprova":
                self.nuova_partita()
            elif scelta == "menu":
                self.vai_al_menu()

        elif self.stato == "vittoria":
            self.win.update()
            if self.win.handle_events(events) == "menu":
                self.vai_al_menu()

        return True

    def aggiorna_gioco(self, events):
        azioni_touch = self.touch.handle_events(events)
        self.hud.touch = self.touch.attivo
        if "pausa" in azioni_touch:
            self.touch.reset()
            self.stato = "pausa"
            return
        if "interagisci" in azioni_touch:
            self.interagisci()
        if "attacca" in azioni_touch:
            self.player.attacca(self.nemici_vivi())

        for e in events:
            if e.type == pygame.KEYDOWN:
                if e.key in (pygame.K_ESCAPE, pygame.K_p):
                    self.stato = "pausa"
                    return
                if e.key == pygame.K_e:
                    self.interagisci()
                if e.key == pygame.K_SPACE:
                    self.player.attacca(self.nemici_vivi())
            elif e.type == pygame.MOUSEBUTTONDOWN and e.button == 1 and not getattr(e, "touch", False):
                # i tocchi sullo schermo generano anche click "finti": li gestiscono i comandi touch
                self.player.attacca(self.nemici_vivi())
            elif e.type == pygame.WINDOWFOCUSLOST:
                self.stato = "pausa"
                return

        nemici = self.nemici_vivi()
        # il giocatore non può attraversare i nemici, e i nemici non si sovrappongono
        tiles_nemici = {(n.tile_x, n.tile_y) for n in nemici}
        if self.player.vivo:
            self.player.update(tiles_nemici, self.touch.passo())
        if self.tregua > 0:
            self.tregua -= 1
        else:
            for n in nemici:
                occupati = {(self.player.tile_x, self.player.tile_y)} | {
                    (m.tile_x, m.tile_y) for m in nemici if m is not n}
                n.update(occupati)

        if self.mondo.livello_id == 1:
            self.portale.update()
            if self.player.sul_portale() and not self.player.thirsty:
                self.entra_livello_2()

        if not self.player.vivo:
            self.schermata_morte.nuovo_consiglio()
            self.stato = "morto"
        elif self.boss is not None and not self.boss.vivo:
            # piccola pausa per godersi la vittoria prima dei crediti
            self.timer_vittoria += 1
            if self.timer_vittoria >= FRAME_PRIMA_DEI_CREDITI:
                self.win = Win(self.finestra, self.player.monete)
                self.stato = "vittoria"

    def interagisci(self):
        if self.mondo.livello_id != 1:
            return
        if self.player.is_near_npc(self.npc) and not self.hud.giaparlato:
            self.hud.interagisci_npc()
            if self.hud.giaparlato:
                self.hud.mostra_messaggio("Vai alla fontanella e premi E per bere.", 5)
        elif self.player.puo_bere(self.hud.giaparlato):
            self.player.bevi()
            self.hud.mostra_messaggio("Il portale si e' aperto! Raggiungilo seguendo la freccia.", 6)

    # ---------- disegno ----------
    def disegna_gioco(self, aggiorna_animazioni=True):
        self.mondo.disegna()

        # tutti gli oggetti "in piedi" vengono ordinati per profondità:
        # così chi sta davanti copre chi sta dietro
        disegnabili = self.mondo.oggetti_alti()
        if self.mondo.livello_id == 1:
            disegnabili.append((self.npc.profondita, self.npc.disegna))
            attivo = not self.player.thirsty
            disegnabili.append((self.portale.profondita, lambda: self.portale.disegna(attivo)))
            nemici = self.bats
        else:
            nemici = [self.boss]
        for n in nemici:
            if n.vivo:
                disegnabili.append((n.profondita, n.disegna))
        disegnabili.append((self.player.profondita, self.player.disegna))

        for _, disegna in sorted(disegnabili, key=lambda d: d[0]):
            disegna()

        self.disegna_freccia_obiettivo()
        if aggiorna_animazioni:
            self.effetti.update(self.finestra)
        if self.boss is not None and not self.boss.vivo:
            testo_con_ombra(self.finestra, assets.FONT["dungeon_70"], "VITTORIA!",
                            (LUNGHEZZA // 2, ALTEZZA // 2 - 60), (250, 210, 60), ancora="center")
        self.hud.draw()
        if self.stato == "gioco":
            self.touch.draw(self.finestra)

    def disegna_freccia_obiettivo(self):
        """Freccia gialla che rimbalza sopra il prossimo obiettivo (fontanella o portale)."""
        if self.mondo.livello_id != 1 or not self.hud.giaparlato:
            return
        if self.player.thirsty:
            _, rect = self.mondo.lista_fontanelle[0]
            x, y = rect.centerx, rect.top + 6
        else:
            x, y = self.portale.rect.centerx, self.portale.rect.top
        y -= 14 + int(abs(math.sin(pygame.time.get_ticks() / 250)) * 10)
        punta = [(x, y), (x - 13, y - 16), (x - 5, y - 16), (x - 5, y - 30), (x + 5, y - 30), (x + 5, y - 16), (x + 13, y - 16)]
        pygame.draw.polygon(self.finestra, (250, 210, 60), punta)
        pygame.draw.polygon(self.finestra, (0, 0, 0), punta, 2)


async def main():
    pygame.init()
    try:
        pygame.display.set_icon(pygame.image.load(percorso("logo.ico")))
    except pygame.error:
        pass
    # nel browser è la pagina a ridimensionare il canvas, su pc SCALED permette lo schermo intero
    finestra = pygame.display.set_mode((LUNGHEZZA, ALTEZZA), 0 if WEB else pygame.SCALED)
    pygame.display.set_caption(TITOLO)

    # carica gli assets (dopo aver creato la finestra, per poter usare convert_alpha)
    assets.load_assets()

    clock = pygame.time.Clock()
    gioco = Gioco(finestra)

    # loop principale del gioco
    run = True
    while run:
        run = gioco.step(pygame.event.get())
        pygame.display.flip()
        clock.tick(FPS)
        await asyncio.sleep(0)  # necessario per girare nel browser (pygbag)

    pygame.quit()


if __name__ == "__main__":
    asyncio.run(main())
