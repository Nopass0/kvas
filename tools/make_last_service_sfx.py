"""Synthesizes the extra sound effects of LAST SERVICE (doors, the kitchen fan, footsteps on
grass) into games/last_service/assets/audio/sfx. Deterministic: the same files every run.

    python tools/make_last_service_sfx.py
"""
import os
import wave

import numpy as np

RATE = 44100
OUT = os.path.join(os.path.dirname(__file__), '..', 'games', 'last_service', 'assets', 'audio', 'sfx')
rng = np.random.default_rng(17)


def save(name, x, stereo=False):
    x = np.clip(x / max(1e-6, np.abs(x).max()) * 0.85, -1, 1)
    data = (x * 32767).astype(np.int16)
    with wave.open(os.path.join(OUT, name), 'wb') as w:
        w.setnchannels(2 if stereo else 1)
        w.setsampwidth(2)
        w.setframerate(RATE)
        w.writeframes(data.tobytes())


def t(seconds):
    return np.arange(int(RATE * seconds)) / RATE


def lowpass(x, cutoff):
    a = np.exp(-2 * np.pi * cutoff / RATE)
    y = np.zeros_like(x)
    acc = 0.0
    for i, v in enumerate(x):
        acc = (1 - a) * v + a * acc
        y[i] = acc
    return y


def creak(seconds, base, wobble):
    """A hinge: stick-slip pulses whose rate wanders, through a wooden resonance."""
    tt = t(seconds)
    rate = base * (1 + wobble * np.sin(2 * np.pi * 1.3 * tt) + 0.15 * np.sin(2 * np.pi * 5.1 * tt))
    phase = np.cumsum(rate) / RATE
    pulses = (np.diff(np.floor(phase), prepend=0) > 0).astype(float)
    body = np.zeros_like(tt)
    for f, d in ((420, 0.0025), (980, 0.0012), (1730, 0.0007)):
        k = np.exp(-np.arange(int(RATE * 0.02)) / (RATE * d)) * np.sin(2 * np.pi * f * np.arange(int(RATE * 0.02)) / RATE)
        body += np.convolve(pulses, k)[:len(tt)]
    env = np.minimum(1, tt / 0.05) * np.minimum(1, (seconds - tt) / 0.2)
    return body * env * (0.6 + 0.4 * np.sin(2 * np.pi * 0.7 * tt) ** 2)


def thud(seconds, low):
    tt = t(seconds)
    noise = lowpass(rng.standard_normal(len(tt)), 600)
    tone = np.sin(2 * np.pi * low * tt) * np.exp(-tt * 18)
    return (noise * 0.6 + tone * 1.2) * np.exp(-tt * 14)


def main():
    os.makedirs(OUT, exist_ok=True)
    open_ = creak(0.9, 38, 0.45)
    save('sfx_door_open.wav', open_ + np.concatenate([np.zeros(int(RATE * 0.05)), thud(0.85, 90)[: len(open_) - int(RATE * 0.05)] * 0.25]))
    close = np.concatenate([creak(0.35, 55, 0.3), thud(0.5, 70)])
    save('sfx_door_close.wav', close)
    # Kitchen extractor: a motor hum, blade rate and a loose rattle; loops seamlessly (whole cycles).
    tt = t(4.0)
    hum = np.sin(2 * np.pi * 50 * tt) * 0.5 + np.sin(2 * np.pi * 100 * tt) * 0.25 + np.sin(2 * np.pi * 150 * tt) * 0.08
    blades = lowpass(rng.standard_normal(len(tt)), 900) * (0.6 + 0.4 * np.sin(2 * np.pi * 12.5 * tt))
    rattle = (np.sin(2 * np.pi * 6.25 * tt) > 0.96) * lowpass(rng.standard_normal(len(tt)), 3000) * 0.6
    save('amb_fan.wav', hum + blades * 0.7 + rattle)
    # Steps on wet grass and gravel.
    step = np.zeros(int(RATE * 0.35))
    crunch = lowpass(rng.standard_normal(len(step)), 2500) * np.exp(-t(0.35) * 22)
    step += crunch + thud(0.35, 60) * 0.3
    save('sfx_step_grass.wav', step)
    print('sfx written to', os.path.normpath(OUT))


if __name__ == '__main__':
    main()
