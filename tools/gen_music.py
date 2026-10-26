# 音楽を numpy で合成して assets/bgm/{town,battle,boss}.ogg にする(ループ用)。
#   PYTHONUTF8=1 python tools/gen_music.py
# 音源は サイン波・三角波・パルス波・ノイズだけ(外部の素材は使わない)。小節数ぴったりで 終わりの余韻は 頭にたたみ込むので、ループのつなぎ目で切れない。
import os
import subprocess

import numpy as np

SR = 32000
HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, '..', 'assets', 'bgm')
os.makedirs(OUT, exist_ok=True)
NAMES = {'C': 0, 'D': 2, 'E': 4, 'F': 5, 'G': 7, 'A': 9, 'B': 11}


def freq(n):
    """'A4' 'F#5' 'Bb3' -> Hz"""
    base = NAMES[n[0]]
    i = 1
    if n[i] == '#':
        base += 1
        i += 1
    elif n[i] == 'b':
        base -= 1
        i += 1
    octave = int(n[i:])
    midi = 12 * (octave + 1) + base
    return 440.0 * 2 ** ((midi - 69) / 12)


class Song:
    def __init__(self, bpm, bars, beats_per_bar=4):
        self.bpm = bpm
        self.beat = 60.0 / bpm
        self.bars = bars
        self.n = int(round(bars * beats_per_bar * self.beat * SR))
        self.tail = int(SR * 2.0)
        self.buf = np.zeros(self.n + self.tail)

    def add(self, t_beats, sig, gain=1.0):
        s = int(round(t_beats * self.beat * SR))
        e = min(s + len(sig), len(self.buf))
        if s < len(self.buf):
            self.buf[s:e] += sig[: e - s] * gain

    def loop(self):
        """余韻を頭にたたみ込んで ちょうど n サンプルにする"""
        out = self.buf[: self.n].copy()
        tail = self.buf[self.n:]
        out[: len(tail)] += tail[: self.n]
        return out


def env(n, a, d, s, r):
    """ADSR(秒)。n サンプルの長さ(release を含む)"""
    e = np.ones(n) * s
    na, nd, nr = int(a * SR), int(d * SR), int(r * SR)
    na = max(na, 1)
    e[:na] = np.linspace(0, 1, na)
    if nd > 0 and na + nd < n:
        e[na:na + nd] = np.linspace(1, s, nd)
    if nr > 0 and nr < n:
        e[-nr:] *= np.linspace(1, 0, nr)
    return e


def osc(kind, f, n, vib=0.0, duty=0.5):
    t = np.arange(n) / SR
    ph = 2 * np.pi * f * t
    if vib:
        ph = ph + vib * np.sin(2 * np.pi * 5.2 * t) * np.minimum(t * 3, 1.0)
    if kind == 'sine':
        return np.sin(ph)
    if kind == 'tri':
        return 2 / np.pi * np.arcsin(np.sin(ph))
    if kind == 'pulse':
        return np.where((ph / (2 * np.pi)) % 1.0 < duty, 1.0, -1.0)
    if kind == 'saw':
        return 2 * ((ph / (2 * np.pi)) % 1.0) - 1
    raise ValueError(kind)


def lowpass(x, k):
    """単純な移動平均ぎみの ローパス(k=0..1 大きいほど暗い)"""
    a = 1 - k
    y = np.zeros_like(x)
    acc = 0.0
    for i in range(len(x)):
        acc += a * (x[i] - acc)
        y[i] = acc
    return y


def lp_fast(x, k):
    from scipy.signal import lfilter
    a = 1 - k
    return lfilter([a], [1, -(1 - a)], x)


def note(kind, f, beats, song, gain=0.3, a=0.01, d=0.1, s=0.7, r=0.12, vib=0.0, duty=0.5, dark=0.0, hold=0.92):
    dur = max(beats * song.beat * hold, 0.04)
    n = int(dur * SR) + int(r * SR)
    x = osc(kind, f, n, vib, duty) * env(n, a, d, s, r)
    if dark:
        x = lp_fast(x, dark)
    return x * gain


def pluck(f, beats, song, gain=0.3):
    dur = max(beats * song.beat, 0.1) + 0.5
    n = int(dur * SR)
    t = np.arange(n) / SR
    x = np.zeros(n)
    for h, amp in ((1, 1.0), (2, 0.5), (3, 0.25), (4, 0.12), (5, 0.06)):
        x += amp * np.sin(2 * np.pi * f * h * t) * np.exp(-t * (3.0 + 1.6 * h))
    x *= np.minimum(t * 400, 1.0)
    return x * gain


def kick(song, gain=0.6):
    n = int(0.22 * SR)
    t = np.arange(n) / SR
    f = 120 * np.exp(-t * 22) + 42
    x = np.sin(2 * np.pi * np.cumsum(f) / SR) * np.exp(-t * 14)
    return x * gain


def snare(song, gain=0.4):
    n = int(0.2 * SR)
    t = np.arange(n) / SR
    rng = np.random.RandomState(7)
    x = rng.randn(n) * np.exp(-t * 20) * 0.7 + np.sin(2 * np.pi * 190 * t) * np.exp(-t * 28) * 0.5
    return x * gain


def hat(song, gain=0.18, open_=False):
    n = int((0.16 if open_ else 0.05) * SR)
    t = np.arange(n) / SR
    rng = np.random.RandomState(11 if open_ else 3)
    x = rng.randn(n)
    x = x - lp_fast(x, 0.55)  # 高域だけ
    return x * np.exp(-t * (18 if open_ else 70)) * gain


CHORDS = {
    'C': ['C', 'E', 'G'], 'G': ['G', 'B', 'D'], 'Am': ['A', 'C', 'E'], 'F': ['F', 'A', 'C'], 'Em': ['E', 'G', 'B'], 'Dm': ['D', 'F', 'A'], 'Bb': ['Bb', 'D', 'F'],
    'A': ['A', 'C#', 'E'], 'Gm': ['G', 'Bb', 'D'], 'E': ['E', 'G#', 'B'],
}


def tones(ch, octave):
    """コードの音(根音から上へ並べる)"""
    out = []
    o = octave
    prev = -1
    for nme in CHORDS[ch]:
        base = NAMES[nme[0]] + (1 if '#' in nme else 0) - (1 if 'b' in nme[1:] else 0)
        if base < prev:
            o += 1
        prev = base
        out.append('%s%d' % (nme, o))
    return out


def melody(song, track, start_beat, kind='sine', gain=0.3, **kw):
    t = start_beat
    for tok in track.split():
        nme, b = tok.split('/')
        b = float(b)
        if nme != 'R':
            song.add(t, note(kind, freq(nme), b, song, gain=gain, **kw))
        t += b
    return t


def finish(song, name, peak=0.75):
    x = song.loop()
    x = x / max(np.abs(x).max(), 1e-6) * peak
    wav = os.path.join(OUT, name + '.wav')
    pcm = (x * 32767).astype(np.int16)
    import wave
    with wave.open(wav, 'wb') as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(SR)
        w.writeframes(pcm.tobytes())
    ogg = os.path.join(OUT, name + '.ogg')
    subprocess.run(['ffmpeg', '-y', '-loglevel', 'error', '-i', wav, '-c:a', 'libvorbis', '-q:a', '4', ogg], check=True)
    os.remove(wav)
    print(name, '%.1f s' % (len(x) / SR), 'seam |first-last| = %.4f' % abs(x[0] - x[-1]), os.path.getsize(ogg) // 1024, 'KB')


# ---------------- のどかな街 ----------------
def town():
    s = Song(88, 16)
    chords = ['C', 'G', 'Am', 'F', 'C', 'G', 'F', 'C', 'Am', 'Em', 'F', 'C', 'F', 'C', 'G', 'G']
    mel = [
        'E5/1 G5/1 E5/.5 D5/.5 C5/1', 'D5/1.5 B4/.5 D5/1 G5/1', 'C5/1 E5/1 A5/1 G5/.5 E5/.5', 'F5/1.5 E5/.5 D5/1 C5/1',
        'E5/1 G5/1 C6/1 B5/.5 G5/.5', 'A5/1 G5/1 D5/1 B4/1', 'A4/1 C5/1 F5/1 E5/.5 D5/.5', 'C5/2 R/1 G4/1',
        'A4/1 C5/1 E5/1 D5/.5 C5/.5', 'B4/1 E5/1 G5/1 F#5/.5 E5/.5', 'F5/1 A5/1 G5/.5 F5/.5 E5/1', 'D5/1 C5/1 E5/1 G5/1',
        'A5/1.5 G5/.5 F5/1 E5/1', 'E5/1 D5/1 C5/2', 'D5/1 G5/1 B5/1 A5/.5 G5/.5', 'G5/2 F5/1 D5/1',
    ]
    for i, (ch, m) in enumerate(zip(chords, mel)):
        b0 = i * 4
        t = tones(ch, 3)
        # ギターふうのアルペジオ(8分)
        pat = [0, 1, 2, 1, 0, 1, 2, 1]
        for k, p in enumerate(pat):
            s.add(b0 + k * 0.5, pluck(freq(t[p].replace('3', '4') if p else t[p]), 0.5, s), 0.20)
        # ベース(根音と5度)
        root = t[0].replace('3', '2')
        fifth = t[2].replace('3', '2')
        s.add(b0, note('tri', freq(root), 1.8, s, gain=0.38, a=0.02, r=0.2))
        s.add(b0 + 2, note('tri', freq(fifth), 1.8, s, gain=0.30, a=0.02, r=0.2))
        # 旋律(フルートふう)
        melody(s, m, b0, 'sine', gain=0.30, a=0.04, d=0.15, s=0.8, r=0.18, vib=0.6)
        # 2拍めに やわらかいシェイカー
        for k in range(4):
            s.add(b0 + k, hat(s, 0.05), 1.0)
    finish(s, 'town')


# ---------------- 戦闘 ----------------
def battle():
    s = Song(152, 16)
    chords = ['Am', 'F', 'G', 'Am', 'Am', 'F', 'G', 'E', 'Dm', 'F', 'G', 'Am', 'F', 'G', 'E', 'E']
    for i, ch in enumerate(chords):
        b0 = i * 4
        t = tones(ch, 4)
        root = t[0].replace('4', '2')
        # ベース 8分
        for k in range(8):
            f = freq(root if k % 4 != 3 else t[2].replace('4', '2'))
            s.add(b0 + k * 0.5, note('saw', f, 0.5, s, gain=0.2, a=0.005, r=0.04, dark=0.6, hold=0.8))
        # リード(パルス)アルペジオ
        pats = [[0, 1, 2, 1, 2, 1, 0, 1], [2, 1, 0, 1, 2, 3, 2, 1]]
        pat = pats[(i // 4) % 2]
        for k, p in enumerate(pat):
            tn = (t + [t[0].replace('4', '5')])[p]
            tn = tn if p < 3 else t[0].replace('4', '5')
            s.add(b0 + k * 0.5, note('pulse', freq(tn.replace('4', '5') if p < 3 and i % 2 else tn), 0.5, s, gain=0.11, a=0.004, r=0.05, duty=0.25, hold=0.75))
        # ドラム
        s.add(b0, kick(s))
        s.add(b0 + 2, kick(s))
        s.add(b0 + 2.5, kick(s), 0.6)
        s.add(b0 + 1, snare(s))
        s.add(b0 + 3, snare(s))
        for k in range(8):
            s.add(b0 + k * 0.5, hat(s), 1.0 if k % 2 == 0 else 0.6)
    # 旋律(後半に 長い音で)
    lead = ('A5/1.5 G5/.5 E5/1 C5/1 ' 'A5/1.5 C6/.5 A5/1 F5/1 ' 'B5/1.5 A5/.5 G5/1 D5/1 ' 'E5/2 R/1 E5/.5 G5/.5 ')
    melody(s, lead, 0, 'pulse', gain=0.13, a=0.01, d=0.1, s=0.7, r=0.1, duty=0.5, vib=0.3)
    lead2 = ('C6/1 B5/1 A5/1 G5/1 ' 'A5/1 C6/1 E6/1 C6/1 ' 'D6/1 B5/1 G5/1 B5/1 ' 'A5/2 G#5/2 ')
    melody(s, lead2, 16, 'pulse', gain=0.13, a=0.01, d=0.1, s=0.7, r=0.1, duty=0.5, vib=0.3)
    lead3 = ('A5/1.5 G5/.5 E5/1 C5/1 ' 'F5/1.5 A5/.5 C6/1 A5/1 ' 'G5/1.5 B5/.5 D6/1 B5/1 ' 'E6/2 D6/1 B5/1 ')
    melody(s, lead3, 32, 'pulse', gain=0.13, a=0.01, d=0.1, s=0.7, r=0.1, duty=0.5, vib=0.3)
    lead4 = ('C6/1 A5/1 F5/1 A5/1 ' 'D6/1 B5/1 G5/1 B5/1 ' 'G#5/1 B5/1 E6/1 B5/1 ' 'E6/3 R/1 ')
    melody(s, lead4, 48, 'pulse', gain=0.13, a=0.01, d=0.1, s=0.7, r=0.1, duty=0.5, vib=0.3)
    finish(s, 'battle')


# ---------------- ボス ----------------
def boss():
    s = Song(172, 16)
    chords = ['Dm', 'Bb', 'C', 'Dm', 'Dm', 'Bb', 'C', 'A', 'Gm', 'Bb', 'C', 'Dm', 'Bb', 'C', 'A', 'A']
    for i, ch in enumerate(chords):
        b0 = i * 4
        t = tones(ch, 4)
        root = t[0].replace('4', '2')
        # 低い刻みベース(8分)
        for k in range(8):
            s.add(b0 + k * 0.5, note('saw', freq(root), 0.5, s, gain=0.22, a=0.004, r=0.03, dark=0.7, hold=0.7))
        # 低いパッド(長い和音)
        for tn in t:
            s.add(b0, note('saw', freq(tn.replace('4', '3')), 3.9, s, gain=0.05, a=0.15, d=0.3, s=0.8, r=0.3, dark=0.8))
        # 16分のリードアルペジオ
        seq = [0, 2, 1, 2] * 4
        for k, p in enumerate(seq):
            tn = t[p].replace('4', '5')
            s.add(b0 + k * 0.25, note('pulse', freq(tn), 0.25, s, gain=0.085, a=0.003, r=0.03, duty=0.125 if i % 2 else 0.25, hold=0.7))
        # ドラム: 2連キック
        for k in (0, 1.5, 2, 3.5):
            s.add(b0 + k, kick(s), 1.0)
        s.add(b0 + 1, snare(s))
        s.add(b0 + 3, snare(s))
        for k in range(8):
            s.add(b0 + k * 0.5, hat(s, 0.2, open_=(k % 4 == 3)), 1.0)
    m1 = ('D5/1 F5/1 A5/1 D6/1 ' 'D6/1.5 C6/.5 Bb5/1 A5/1 ' 'G5/1 Bb5/1 C6/1 E6/1 ' 'D6/2 A5/1 F5/1 ')
    melody(s, m1, 0, 'saw', gain=0.10, a=0.01, d=0.1, s=0.7, r=0.1, dark=0.45, vib=0.4)
    m2 = ('F6/1 E6/.5 D6/.5 C6/1 D6/1 ' 'D6/1 Bb5/1 F5/1 Bb5/1 ' 'C6/1 E6/1 G6/1 E6/1 ' 'E6/2 C#6/1 A5/1 ')
    melody(s, m2, 16, 'saw', gain=0.10, a=0.01, d=0.1, s=0.7, r=0.1, dark=0.45, vib=0.4)
    m3 = ('D6/1 Bb5/1 G5/1 Bb5/1 ' 'D6/1 F6/1 D6/1 Bb5/1 ' 'C6/1 E6/1 G6/1 C7/1 ' 'A6/2 F6/1 D6/1 ')
    melody(s, m3, 32, 'saw', gain=0.10, a=0.01, d=0.1, s=0.7, r=0.1, dark=0.45, vib=0.4)
    m4 = ('Bb5/1 D6/1 F6/1 D6/1 ' 'C6/1 E6/1 G6/1 E6/1 ' 'A5/1 C#6/1 E6/1 A6/1 ' 'A6/3 R/1 ')
    melody(s, m4, 48, 'saw', gain=0.10, a=0.01, d=0.1, s=0.7, r=0.1, dark=0.45, vib=0.4)
    finish(s, 'boss', 0.78)


if __name__ == '__main__':
    town()
    battle()
    boss()
