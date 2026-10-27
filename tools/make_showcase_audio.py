"""Generates the small sound effects of the showcase game (no external assets needed).

    python tools/make_showcase_audio.py games/showcase/assets/sounds
"""
import math
import random
import struct
import sys
import wave

RATE = 44100


def write(path, samples, stereo=False):
    with wave.open(path, "wb") as w:
        w.setnchannels(2 if stereo else 1)
        w.setsampwidth(2)
        w.setframerate(RATE)
        frames = bytearray()
        for s in samples:
            v = int(max(-1.0, min(1.0, s)) * 32000)
            frames += struct.pack("<h", v) * (2 if stereo else 1)
        w.writeframes(bytes(frames))


def tone(freq, length, volume=0.5, decay=6.0):
    n = int(RATE * length)
    return [volume * math.sin(2 * math.pi * freq * i / RATE) * math.exp(-decay * i / RATE) for i in range(n)]


def mix(*tracks):
    n = max(len(t) for t in tracks)
    out = [0.0] * n
    for t in tracks:
        for i, s in enumerate(t):
            out[i] += s
    return out


def delayed(track, seconds):
    return [0.0] * int(RATE * seconds) + track


def main(out):
    random.seed(4)
    # Gem pickup: bright two-note chime.
    write(f"{out}/gem.wav", mix(tone(880, 0.5, 0.4), delayed(tone(1318, 0.6, 0.4), 0.09), tone(1760, 0.4, 0.15, 9)))
    # Win jingle: rising arpeggio.
    notes = [523, 659, 784, 1046, 1318]
    write(f"{out}/win.wav", mix(*[delayed(tone(f, 0.9, 0.3, 3.5), i * 0.14) for i, f in enumerate(notes)]))
    # Splash: low-passed noise burst.
    n = int(RATE * 0.7)
    low = 0.0
    splash = []
    for i in range(n):
        low += 0.12 * (random.uniform(-1, 1) - low)
        splash.append(low * 2.4 * math.exp(-5.0 * i / RATE))
    write(f"{out}/splash.wav", splash)
    # Ocean waves: loopable swelling noise (8 s, seamless because the envelope is periodic).
    n = RATE * 8
    low = 0.0
    waves = []
    for i in range(n):
        low += 0.03 * (random.uniform(-1, 1) - low)
        swell = 0.55 + 0.45 * math.sin(2 * math.pi * i / n * 2 + 1.0)
        waves.append(low * 4.0 * swell)
    write(f"{out}/waves.wav", waves, stereo=True)
    # Footstep-ish thud for landing.
    write(f"{out}/land.wav", mix(tone(110, 0.25, 0.7, 18), [random.uniform(-1, 1) * 0.15 * math.exp(-40 * i / RATE) for i in range(int(RATE * 0.25))]))


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else ".")
