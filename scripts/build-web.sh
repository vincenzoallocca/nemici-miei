#!/usr/bin/env bash
# Compila la versione web (WebAssembly) e prepara una cartella pronta da pubblicare
# con solo i file che il gioco carica davvero.
#
# Uso: scripts/build-web.sh [cartella di uscita]   (predefinita: dist)
set -euo pipefail

cd "$(dirname "$0")/.."
OUT="${1:-dist}"

cargo build --locked --release --target wasm32-unknown-unknown

rm -rf "$OUT"
mkdir -p "$OUT/assets/audio" "$OUT/assets/[SFONDI]"
cp web/index.html "$OUT/"
cp target/wasm32-unknown-unknown/release/nemici-miei.wasm "$OUT/"
cp assets/Arcade.ttf assets/DungeonFont.ttf "$OUT/assets/"
for dir in "[BLOCCHI]" "[EROE]" "[ITEMS]" "[MOBS]" "[NPC]" "[PORTALE]"; do
    cp -r "assets/$dir" "$OUT/assets/"
done
cp "assets/[SFONDI]/bg3.png" "$OUT/assets/[SFONDI]/"
for suono in acqua bat_mobile boss_morte boss_soundtrack coin grass-001 player_dmg soundtrack spada-004 stone; do
    cp "assets/audio/$suono.ogg" "$OUT/assets/audio/"
done

# loader JS di miniquad/macroquad (versione 0.4)
if [ ! -f "$OUT/mq_js_bundle.js" ]; then
    curl -fsSL -o "$OUT/mq_js_bundle.js" https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js
fi

echo "Versione web pronta in $OUT ($(du -sh "$OUT" | cut -f1))"
