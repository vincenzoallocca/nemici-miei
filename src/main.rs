use macroquad::prelude::*;
use nemici_miei::assets::Assets;
use nemici_miei::game::Gioco;
use nemici_miei::settings::{ALTEZZA, LUNGHEZZA, TITOLO};

fn window_conf() -> Conf {
    Conf {
        window_title: TITOLO.to_string(),
        window_width: LUNGHEZZA as i32,
        window_height: ALTEZZA as i32,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut assets = Assets::load().await;
    let mut gioco = Gioco::new(&assets);
    gioco.vai_al_menu(&mut assets);

    loop {
        clear_background(BLACK);

        let sw = screen_width();
        let sh = screen_height();
        let scale = (sw / LUNGHEZZA).min(sh / ALTEZZA);
        let vp_w = (LUNGHEZZA * scale).round();
        let vp_h = (ALTEZZA * scale).round();
        let ox = ((sw - vp_w) / 2.0).round();
        let oy = ((sh - vp_h) / 2.0).round();

        // Positive Y zoom ensures top-left is (0, 0) and bottom-right is (960, 720) (right-side up!)
        let camera = Camera2D {
            zoom: vec2(2.0 / LUNGHEZZA, 2.0 / ALTEZZA),
            target: vec2(LUNGHEZZA / 2.0, ALTEZZA / 2.0),
            viewport: Some((ox as i32, oy as i32, vp_w as i32, vp_h as i32)),
            ..Default::default()
        };
        set_camera(&camera);

        let (mx, my) = mouse_position();
        let mouse_world = camera.screen_to_world(vec2(mx, my));
        let mouse_tuple = (mouse_world.x, mouse_world.y);

        let keep_running = gioco.update(&mut assets, mouse_tuple);
        if !keep_running {
            break;
        }

        gioco.draw(&assets, mouse_tuple);

        set_default_camera();

        next_frame().await;
    }
}
