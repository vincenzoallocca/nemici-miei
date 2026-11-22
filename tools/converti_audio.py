"""Converte i suoni del gioco in MP3 leggeri per la versione web.

Uso:  pip install soundfile numpy lameenc
      python tools/converti_audio.py

Perché: nel browser i suoni vengono decodificati da `decodeAudioData`, e non tutti i
browser (Safari su iPhone in particolare) decodificano l'Ogg Vorbis; l'MP3 sì, ovunque.
In più la colonna sonora originale pesa 4,4 MB: ricompressa ne pesa meno di 1 e
arriva subito. La versione desktop continua a usare gli .ogg originali.
"""
import os

import lameenc
import numpy as np
import soundfile as sf

BASE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ORIGINE = os.path.join(BASE, "assets", "audio")
USCITA = os.path.join(BASE, "assets", "audio", "web")

# nome: (kbps, mono)
SUONI = {
    "soundtrack": (48, True),
    "boss_soundtrack": (64, False),
    "coin": (64, True),
    "grass-001": (48, True),
    "spada-004": (64, True),
    "player_dmg": (48, True),
    "bat_mobile": (64, True),
    "boss_morte": (64, True),
    "stone": (48, True),
    "acqua": (48, True),
}


def converti(nome, kbps, mono):
    dati, frequenza = sf.read(os.path.join(ORIGINE, nome + ".ogg"), dtype="float32", always_2d=True)
    if mono:
        dati = dati.mean(axis=1, keepdims=True)
    pcm = (np.clip(dati, -1.0, 1.0) * 32767).astype(np.int16)

    enc = lameenc.Encoder()
    enc.set_bit_rate(kbps)
    enc.set_in_sample_rate(frequenza)
    enc.set_channels(pcm.shape[1])
    enc.set_quality(2)  # 2 = alta qualità (lento ma lo si fa una volta sola)
    mp3 = enc.encode(pcm.tobytes()) + enc.flush()

    with open(os.path.join(USCITA, nome + ".mp3"), "wb") as f:
        f.write(mp3)
    return len(mp3)


def main():
    os.makedirs(USCITA, exist_ok=True)
    totale = 0
    for nome, (kbps, mono) in SUONI.items():
        dim = converti(nome, kbps, mono)
        totale += dim
        print(f"{nome:16} {dim // 1024:5} KB  ({kbps} kbps, {'mono' if mono else 'stereo'})")
    print(f"{'totale':16} {totale // 1024:5} KB")


if __name__ == "__main__":
    main()
