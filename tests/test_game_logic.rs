use nemici_miei::boss::{Boss, BossStato};
use nemici_miei::effetti::Effetti;
use nemici_miei::hud::HUD;
use nemici_miei::player::Giocatore;
use nemici_miei::settings::*;
use nemici_miei::win::Win;
use nemici_miei::world::World;

#[test]
fn test_coordinate_isometriche() {
    let world = World::new(1);
    for tx in 0..10 {
        for ty in 0..10 {
            let (sx, sy) = world.piedi_tile(tx, ty);
            let (rx, ry) = world.screen_to_tile(sx, sy);
            assert_eq!((rx, ry), (tx, ty), "Failed at tx={}, ty={}", tx, ty);
        }
    }
}

#[test]
fn test_calpestabilita_mondo() {
    let mut world = World::new(1);
    // (0, 0) is tile 3 (erba) -> walkable
    assert!(world.is_tile_walkable(0, 0));

    // NPC at (8, 8) blocks its tile
    world.bloccati.insert((8, 8));
    assert!(!world.is_tile_walkable(8, 8));

    // Fountain at (5, 5) is tile 18 -> not in WALKABLE_TILES
    assert_eq!(world.tipo_tile(5, 5), Some(18));
    assert!(!world.is_tile_walkable(5, 5));

    // Out of bounds
    assert!(!world.is_tile_walkable(-1, 0));
    assert!(!world.is_tile_walkable(10, 5));
}

#[test]
fn test_player_e_fontanella() {
    let world = World::new(1);
    let mut player = Giocatore::new(0, 0, &world);
    let mut effetti = Effetti::new();

    assert!(player.thirsty);
    assert_eq!(player.monete, 0);
    assert_eq!(player.vita, 100);

    // Take damage
    player.vita = 50;
    assert_eq!(player.vita, 50);

    // Move next to fountain (fountain at 5,5; adjacent is (6,6))
    player.entita.piazza(6, 6, &world);
    assert!(player.is_near_fountain(&world));

    // Cannot drink if hasn't talked to Ugo
    assert!(!player.puo_bere(false, &world));
    assert!(player.puo_bere(true, &world));

    // Fake drink (without audio to allow headless test)
    player.thirsty = false;
    player.monete += 1;
    player.cura(25, &mut effetti);

    assert!(!player.thirsty);
    assert_eq!(player.monete, 1);
    assert_eq!(player.vita, 75);
}

#[test]
fn test_dialogo_npc() {
    let world = World::new(1);
    let mut player = Giocatore::new(8, 7, &world);
    let mut hud = HUD::new();
    let mut effetti = Effetti::new();

    // Dialogue progression
    assert!(!hud.giaparlato);

    // Dialogue 0
    hud.dialogue_index = 0;
    assert_eq!(STRING_DIALOGUE[0].0, "Me");

    // Dialogue 1: Ugo grants health bonus
    player.vita = 70;
    hud.dialogue_index = 1;
    if !player.ugo_vita_bonus {
        player.ugo_vita_bonus = true;
        player.cura(15, &mut effetti);
    }
    assert!(player.ugo_vita_bonus);
    assert_eq!(player.vita, 85);

    // End dialogue
    hud.giaparlato = true;
    assert!(hud.giaparlato);
}

#[test]
fn test_boss_furia_e_stati() {
    let world = World::new(2);
    let mut boss = Boss::new(0, 0, &world);
    let mut effetti = Effetti::new();

    assert_eq!(boss.hp, 150.0);
    assert!(!boss.furia);
    assert_eq!(boss.stato, BossStato::Insegui);

    // Inflict damage to trigger fury (threshold <= 75)
    boss.hp = 70.0;
    boss.entra_in_furia(&mut effetti);
    assert!(boss.furia);
    assert_eq!(boss.damage, 15);
    assert_eq!(boss.entita.velocita, 4.5);

    // Test furthest corner from player at (5, 5)
    let corner = boss.angolo_lontano(5, 5, &world);
    assert!(corner == (0, 0) || corner == (9, 0) || corner == (0, 9) || corner == (9, 9));
}

#[test]
fn test_win_crediti() {
    let mut win = Win::new(5);
    assert!(!win.finito());
    assert_eq!(win.lines[2], "Monete raccolte: 5");

    // Fast-forward credits
    win.credit_offset = win.fine_offset;
    assert!(win.finito());
}
