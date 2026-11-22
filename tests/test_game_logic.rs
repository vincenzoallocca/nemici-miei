//! Test senza finestra: la logica del gioco non usa né grafica né audio.

use nemici_miei::boss::{Boss, BossStato, VELOCITA_BOSS_FURIA};
use nemici_miei::effetti::Effetti;
use nemici_miei::enemy::{Enemy, PRIMO_MORSO};
use nemici_miei::game::{GameState, Gioco};
use nemici_miei::hud::HUD;
use nemici_miei::input::{Input, Tasti};
use nemici_miei::livelli::{Obiettivo, LIVELLI};
use nemici_miei::pickup::TipoOggetto;
use nemici_miei::player::Giocatore;
use nemici_miei::settings::*;
use nemici_miei::win::Win;
use nemici_miei::world::{distanza, World};

// ---------- aiuti ----------

fn premi(tasti: Tasti) -> Input {
    Input {
        premuti: tasti,
        ..Default::default()
    }
}

fn invio() -> Input {
    premi(Tasti {
        invio: true,
        ..Default::default()
    })
}

fn tasto_e() -> Input {
    premi(Tasti {
        e: true,
        ..Default::default()
    })
}

fn spazio() -> Input {
    premi(Tasti {
        spazio: true,
        ..Default::default()
    })
}

fn frames(g: &mut Gioco, n: usize) {
    for _ in 0..n {
        assert!(g.update(&Input::default()));
    }
}

/// Cammina fino a distanza `raggio` da `meta`, un passo di logica alla volta.
/// Si ferma anche se si cambia livello (es. entrando nel portale).
fn cammina_verso(g: &mut Gioco, meta: (i32, i32), raggio: i32) {
    let livello = g.livello;
    for _ in 0..3000 {
        if g.livello != livello || g.stato != GameState::Gioco {
            return;
        }
        let pos = g.player.entita.tile();
        if distanza(pos, meta) <= raggio && !g.player.entita.in_movimento() {
            return;
        }
        let mut input = Input::default();
        if !g.player.entita.in_movimento() {
            let occupati: Vec<_> = g.bats.iter().filter(|b| b.vivo()).map(|b| b.entita.tile()).collect();
            if let Some((x, y)) = g.mondo.passo_verso(pos, meta, raggio, &occupati) {
                input.passo = (x - pos.0, y - pos.1);
            }
        }
        assert!(g.update(&input));
        // i pipistrelli non devono fermare il test
        g.player.vita = g.player.vita_max;
    }
    panic!("non sono riuscito a raggiungere {:?}", meta);
}

fn nuova_partita() -> Gioco {
    let mut g = Gioco::new();
    assert_eq!(g.stato, GameState::Menu);
    assert!(g.update(&invio()));
    assert_eq!(g.stato, GameState::Gioco);
    g
}

// ---------- mondo ----------

#[test]
fn coordinate_isometriche_andata_e_ritorno() {
    for id in 1..=LIVELLI.len() as u32 {
        let world = World::new(id);
        for tx in 0..world.num_colonne as i32 {
            for ty in 0..world.num_righe as i32 {
                let (sx, sy) = world.piedi_tile(tx, ty);
                assert_eq!(world.screen_to_tile(sx, sy), (tx, ty), "livello {} tile {},{}", id, tx, ty);
            }
        }
    }
}

#[test]
fn mappa_centrata_e_su_pixel_interi() {
    for id in 1..=LIVELLI.len() as u32 {
        let world = World::new(id);
        assert_eq!(world.offset_x.fract(), 0.0);
        assert_eq!(world.offset_y.fract(), 0.0);
        for t in &world.tiles {
            assert!(t.x >= 0.0 && t.x + GRANDEZZA_TILES <= LUNGHEZZA, "livello {} esce di lato", id);
            assert!(t.y >= 0.0 && t.y + GRANDEZZA_TILES <= ALTEZZA - HUD_HEIGHT, "livello {} esce sotto", id);
        }
    }
}

#[test]
fn calpestabilita_mondo() {
    let mut world = World::new(1);
    assert!(world.is_tile_walkable(0, 0));

    world.blocca(8, 8);
    assert!(!world.is_tile_walkable(8, 8));

    assert_eq!(world.tipo_tile(5, 5), Some(TILE_FONTANELLA));
    assert!(!world.is_tile_walkable(5, 5));
    assert_eq!(world.trova_fontanella(), Some((5, 5)));

    assert!(!world.is_tile_walkable(-1, 0));
    assert!(!world.is_tile_walkable(10, 5));
}

#[test]
fn percorso_aggira_gli_ostacoli() {
    let world = World::new(1);
    // dalla sinistra della fontanella alla destra: (3,5) -> (6,5) va aggirata
    let mut pos = (3, 5);
    let mut passi = 0;
    while distanza(pos, (6, 5)) > 0 {
        let nuovo = world.passo_verso(pos, (6, 5), 0, &[]).expect("c'è una strada");
        assert_eq!(distanza(pos, nuovo), 1);
        assert!(world.is_tile_walkable(nuovo.0, nuovo.1));
        pos = nuovo;
        passi += 1;
        assert!(passi < 20);
    }
    // nessuna strada se la meta è circondata
    let occupati = [(1, 0), (0, 1)];
    assert_eq!(world.passo_verso((5, 5), (0, 0), 0, &occupati), None);
}

#[test]
fn ogni_livello_e_giocabile() {
    for (i, def) in LIVELLI.iter().enumerate() {
        let mut world = World::da_definizione(i as u32 + 1, def);
        if let Some((x, y)) = def.npc {
            world.blocca(x, y);
        }
        assert!(world.is_tile_walkable(def.spawn.0, def.spawn.1), "{}: spawn", def.nome);
        let raggiungibile = |meta: (i32, i32), raggio: i32| {
            distanza(def.spawn, meta) <= raggio || world.passo_verso(def.spawn, meta, raggio, &[]).is_some()
        };
        if let Some(p) = def.portale {
            assert!(world.is_tile_walkable(p.0, p.1), "{}: portale", def.nome);
            assert!(raggiungibile(p, 0), "{}: portale irraggiungibile", def.nome);
        } else {
            assert_eq!(def.obiettivo, Obiettivo::SconfiggiBoss, "{}: senza portale serve un boss", def.nome);
        }
        if let Some(n) = def.npc {
            assert!(raggiungibile(n, 1), "{}: Ugo irraggiungibile", def.nome);
        }
        if let Some(b) = def.boss {
            assert!(world.is_tile_walkable(b.0, b.1), "{}: boss", def.nome);
        }
        assert_eq!(def.portale.is_none(), i + 1 == LIVELLI.len(), "{}: solo l'ultimo livello è senza portale", def.nome);
    }
}

// ---------- personaggi ----------

#[test]
fn player_e_fontanella() {
    let world = World::new(1);
    let mut player = Giocatore::new(0, 0, &world);
    let mut effetti = Effetti::new();

    assert!(player.thirsty);
    assert_eq!(player.monete, 0);
    assert_eq!(player.vita, VITA_GIOCATORE);

    player.vita = 50;
    player.entita.piazza(6, 6, &world);
    assert!(player.is_near_fountain(&world));

    // senza aver parlato con Ugo non si beve
    assert!(!player.puo_bere(false, &world));
    assert!(player.puo_bere(true, &world));

    player.bevi(&mut effetti);
    assert!(!player.thirsty);
    assert_eq!(player.monete, 1);
    assert_eq!(player.vita, 75);
    assert!(!effetti.suoni.is_empty(), "bere fa suonare la moneta");
}

#[test]
fn attacco_ha_un_tempo_di_ricarica() {
    let world = World::new(1);
    let mut player = Giocatore::new(0, 0, &world);
    let mut effetti = Effetti::new();
    assert!(player.esegui_attacco(&mut effetti));
    assert!(!player.esegui_attacco(&mut effetti));
    assert_eq!(player.ricarica_attacco(), 0.0);
    for _ in 0..COOLDOWN_ATTACCO {
        player.update(&world, &[], (0, 0), None, &mut effetti);
    }
    assert_eq!(player.ricarica_attacco(), 1.0);
    assert!(player.esegui_attacco(&mut effetti));
}

#[test]
fn pipistrello_morde_dopo_un_attimo() {
    let world = World::new(1);
    let mut player = Giocatore::new(3, 3, &world);
    let mut bat = Enemy::new(4, 3, &world);
    let mut effetti = Effetti::new();
    for _ in 0..PRIMO_MORSO - 1 {
        bat.update(&world, &mut player, &mut effetti, &[(3, 3)]);
    }
    assert_eq!(player.vita, VITA_GIOCATORE);
    bat.update(&world, &mut player, &mut effetti, &[(3, 3)]);
    assert_eq!(player.vita, VITA_GIOCATORE - bat.dmg);

    // ucciso dà una moneta
    bat.take_damage(100, &mut player, &mut effetti);
    assert!(!bat.vivo());
    assert_eq!(player.monete, 1);
}

#[test]
fn dialogo_con_ugo() {
    let mut g = nuova_partita();
    let npc = g.def().npc.expect("Ugo nel primo livello");
    g.player.vita = 70;
    g.player.entita.piazza(npc.0, npc.1 - 1, &g.mondo);

    g.interagisci();
    assert!(g.npc.as_ref().unwrap().parlando);
    for _ in 1..STRING_DIALOGUE.len() {
        g.interagisci();
    }
    assert!(g.player.ugo_vita_bonus);
    assert_eq!(g.player.vita, 85);
    g.interagisci();
    assert!(g.hud.giaparlato);
    assert!(!g.npc.as_ref().unwrap().parlando);

    // allontanarsi interrompe un dialogo
    let mut hud = HUD::new();
    let mut npc2 = nemici_miei::npc::Npc::new(npc.0, npc.1, &mut g.mondo);
    hud.interagisci_npc(&mut npc2, &mut g.player, &mut g.effetti);
    g.player.entita.piazza(0, 0, &g.mondo);
    hud.update(Some(&mut npc2), &g.player);
    assert!(!npc2.parlando);
}

#[test]
fn boss_furia_e_stati() {
    let world = World::new(LIVELLI.len() as u32);
    let mut boss = Boss::new(0, 0, &world);
    let mut effetti = Effetti::new();

    assert_eq!(boss.hp, boss.hp_max);
    assert!(!boss.furia);
    assert_eq!(boss.stato, BossStato::Insegui);

    boss.hp = boss.soglia_furia - 5.0;
    boss.entra_in_furia(&mut effetti);
    assert!(boss.furia);
    assert_eq!(boss.damage, 15);
    assert_eq!(boss.entita.velocita, VELOCITA_BOSS_FURIA);

    // per riprendersi scappa nel punto più lontano
    let rifugio = world.punto_lontano(5, 5);
    assert!(world.is_tile_walkable(rifugio.0, rifugio.1));
    assert!(distanza(rifugio, (5, 5)) >= 4);
}

#[test]
fn crediti() {
    let mut win = Win::new(5);
    assert!(!win.finito());
    assert_eq!(win.lines[2], "Monete raccolte: 5");
    for _ in 0..10_000 {
        win.update();
    }
    assert!(win.finito());
}

// ---------- partita ----------

#[test]
fn menu_e_avvio() {
    let mut g = Gioco::new();
    frames(&mut g, 5);
    assert_eq!(g.stato, GameState::Menu);
    assert!(g.update(&invio()));
    assert_eq!(g.stato, GameState::Gioco);
    let (min, max) = LIVELLI[0].pipistrelli;
    assert!((min..=max).contains(&g.bats.len()));
    for b in &g.bats {
        assert!(distanza(b.entita.tile(), LIVELLI[0].spawn) >= 4, "pipistrello troppo vicino");
        assert!(g.mondo.is_tile_walkable(b.entita.tile_x, b.entita.tile_y));
    }
}

#[test]
fn tregua_iniziale() {
    let mut g = nuova_partita();
    let posizioni: Vec<_> = g.bats.iter().map(|b| b.entita.tile()).collect();
    frames(&mut g, FRAME_DI_TREGUA as usize - 1);
    let dopo: Vec<_> = g.bats.iter().map(|b| b.entita.tile()).collect();
    assert_eq!(posizioni, dopo, "durante la tregua i pipistrelli stanno fermi");
}

#[test]
fn pausa_e_audio() {
    let mut g = nuova_partita();
    g.audio.clear();
    assert!(g.update(&premi(Tasti {
        esc: true,
        ..Default::default()
    })));
    assert_eq!(g.stato, GameState::Pausa);
    assert!(g.update(&premi(Tasti {
        m: true,
        ..Default::default()
    })));
    assert!(!g.audio_attivo, "M spegne l'audio (una volta sola)");
    assert!(g.update(&premi(Tasti {
        p: true,
        ..Default::default()
    })));
    assert_eq!(g.stato, GameState::Gioco);
}

#[test]
fn morte_e_riprova() {
    let mut g = nuova_partita();
    g.player.monete = 3;
    g.monete_inizio_livello = 1;
    g.player.vita = 0;
    frames(&mut g, 1);
    assert_eq!(g.stato, GameState::Morto);
    assert!(g.update(&premi(Tasti {
        r: true,
        ..Default::default()
    })));
    assert_eq!(g.stato, GameState::Gioco);
    assert_eq!(g.player.vita, g.player.vita_max);
    assert_eq!(g.player.monete, 1, "riprova riparte con le monete di inizio livello");
    assert_eq!(g.livello, 0);
}

#[test]
fn partita_completa() {
    let mut g = nuova_partita();
    frames(&mut g, FRAME_DI_TREGUA as usize);

    let mut livelli_visti = vec![g.livello];
    while g.stato == GameState::Gioco {
        let def = g.def();
        match def.obiettivo {
            Obiettivo::ParlaEBevi => {
                let npc = def.npc.unwrap();
                cammina_verso(&mut g, npc, 1);
                for _ in 0..=STRING_DIALOGUE.len() {
                    assert!(g.update(&tasto_e()));
                }
                assert!(g.hud.giaparlato);
                let fontana = g.mondo.trova_fontanella().unwrap();
                cammina_verso(&mut g, fontana, 1);
                assert!(g.player.puo_bere(true, &g.mondo), "vicino alla fontanella");
                assert!(g.update(&tasto_e()));
                assert!(!g.player.thirsty);
            }
            Obiettivo::SconfiggiPipistrelli => {
                // scorciatoia: il test controlla il passaggio di livello, non il combattimento
                for b in &mut g.bats {
                    b.hp = 0;
                }
                frames(&mut g, 1);
            }
            Obiettivo::RaccogliMonete => {
                for b in &mut g.bats {
                    b.hp = 0;
                }
                assert_eq!(g.monete_rimaste(), def.monete);
                let monete_prima = g.player.monete;
                while let Some(m) = g
                    .oggetti
                    .iter()
                    .find(|o| o.tipo == TipoOggetto::Moneta && !o.preso)
                    .map(|o| o.tile())
                {
                    cammina_verso(&mut g, m, 0);
                }
                assert_eq!(g.player.monete, monete_prima + def.monete as i32);
            }
            Obiettivo::SconfiggiBoss => {
                let boss_pos = g.boss.as_ref().unwrap().entita.tile();
                g.boss.as_mut().unwrap().hp = DANNO_GIOCATORE as f32;
                cammina_verso(&mut g, boss_pos, 1);
                let mut colpi = 0;
                while g.boss.as_ref().unwrap().vivo() {
                    let pos = g.boss.as_ref().unwrap().entita.tile();
                    cammina_verso(&mut g, pos, 1);
                    assert!(g.update(&spazio()));
                    colpi += 1;
                    assert!(colpi < 500, "il boss non muore");
                }
                frames(&mut g, FRAME_PRIMA_DEI_CREDITI as usize);
                break;
            }
        }

        let portale = g.portale.as_ref().expect("portale").tile();
        assert!(g.obiettivo_completato());
        frames(&mut g, 1);
        assert!(g.portale.as_ref().unwrap().aperto, "{}: portale aperto", g.def().nome);
        let prima = g.livello;
        cammina_verso(&mut g, portale, 0);
        assert_eq!(g.livello, prima + 1, "il portale porta al livello dopo");
        livelli_visti.push(g.livello);
        frames(&mut g, FRAME_DI_TREGUA as usize);
    }

    assert_eq!(livelli_visti, (0..LIVELLI.len()).collect::<Vec<_>>());
    assert_eq!(g.stato, GameState::Vittoria);
    assert!(g.win.is_some());
    assert!(g.update(&invio()));
    assert_eq!(g.stato, GameState::Menu);
}

#[test]
fn oggetti_raggiungibili_e_cuori_solo_se_feriti() {
    let mut g = nuova_partita();
    for (i, def) in LIVELLI.iter().enumerate() {
        g.carica_livello(i);
        let monete = g.oggetti.iter().filter(|o| o.tipo == TipoOggetto::Moneta).count();
        let cuori = g.oggetti.iter().filter(|o| o.tipo == TipoOggetto::Cuore).count();
        assert_eq!((monete, cuori), (def.monete, def.cuori), "{}", def.nome);
        for o in &g.oggetti {
            assert!(g.mondo.is_tile_walkable(o.tile_x, o.tile_y));
            assert!(g.mondo.passo_verso(def.spawn, o.tile(), 0, &[]).is_some(), "{}: oggetto irraggiungibile", def.nome);
        }
    }

    let indice = LIVELLI.iter().position(|d| d.cuori > 0).expect("un livello con cuori");
    g.carica_livello(indice);
    for b in &mut g.bats {
        b.hp = 0;
    }
    let cuore = g.oggetti.iter().find(|o| o.tipo == TipoOggetto::Cuore).unwrap().tile();
    // a vita piena il cuore resta a terra
    cammina_verso(&mut g, cuore, 0);
    assert!(!g.oggetti.iter().find(|o| o.tile() == cuore).unwrap().preso);
    g.player.vita = 50;
    frames(&mut g, 1);
    assert!(g.oggetti.iter().find(|o| o.tile() == cuore).unwrap().preso);
    assert_eq!(g.player.vita, 75);
}

#[test]
fn boss_in_furia_chiama_rinforzi() {
    let mut g = nuova_partita();
    g.carica_livello(LIVELLI.len() - 1);
    frames(&mut g, FRAME_DI_TREGUA as usize);
    assert_eq!(g.pipistrelli_vivi(), 0);
    let boss = g.boss.as_mut().unwrap();
    boss.hp = boss.soglia_furia - 1.0;
    frames(&mut g, 1);
    assert!(g.boss.as_ref().unwrap().furia);
    assert_eq!(g.pipistrelli_vivi(), 2);
    frames(&mut g, 1);
    assert_eq!(g.pipistrelli_vivi(), 2, "i rinforzi arrivano una volta sola");
}
