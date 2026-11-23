#!/usr/bin/env bash
# Compila la versione web (WebAssembly) e prepara una cartella pronta da pubblicare
# con solo i file che il gioco carica davvero.
#
# Uso: scripts/build-web.sh [cartella di uscita]   (predefinita: dist)
set -euo pipefail

cd "$(dirname "$0")/.."
OUT="${1:-dist}"

cargo build --locked --profile web --target wasm32-unknown-unknown

rm -rf "$OUT"
mkdir -p "$OUT/assets/audio/web" "$OUT/assets/[SFONDI]"
cp web/index.html web/robustezza.js "$OUT/"
cp target/wasm32-unknown-unknown/web/nemici-miei.wasm "$OUT/"
cp assets/Arcade.ttf assets/DungeonFont.ttf assets/PixelifySans.ttf "$OUT/assets/"
for dir in "[BLOCCHI]" "[EROE]" "[ITEMS]" "[MOBS]" "[NPC]" "[PORTALE]"; do
    cp -r "assets/$dir" "$OUT/assets/"
done
cp "assets/[SFONDI]/bg3.png" "$OUT/assets/[SFONDI]/"
for suono in acqua bat_mobile boss_morte boss_soundtrack coin grass-001 player_dmg soundtrack spada-004 stone; do
    cp "assets/audio/web/$suono.mp3" "$OUT/assets/audio/web/"
done

# loader JS di miniquad/macroquad (versione 0.4)
if [ ! -f "$OUT/mq_js_bundle.js" ]; then
    curl -fsSL -o "$OUT/mq_js_bundle.js" https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js
fi

# impronta di versione: dopo un aggiornamento il browser non può riusare dalla cache un
# .wasm vecchio (su GitHub Pages i file restano in cache 10 minuti)
impronta() { sha256sum "$1" | cut -c1-12; }
V_WASM=$(impronta "$OUT/nemici-miei.wasm")
V_JS=$(impronta "$OUT/mq_js_bundle.js")
V_ROB=$(impronta "$OUT/robustezza.js")
sed -i -e "s|mq_js_bundle.js\"|mq_js_bundle.js?v=$V_JS\"|" -e "s|robustezza.js\"|robustezza.js?v=$V_ROB\"|" -e "s|nemici-miei.wasm\"|nemici-miei.wasm?v=$V_WASM\"|g" "$OUT/index.html"
grep -q "nemici-miei.wasm?v=" "$OUT/index.html" || { echo "impronta di versione non applicata a index.html" >&2; exit 1; }

echo "Versione web pronta in $OUT ($(du -sh "$OUT" | cut -f1))"
