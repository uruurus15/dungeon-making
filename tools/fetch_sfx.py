# Openverse(https://api.openverse.org/v1/audio/)から 効果音を集めて assets/sfx/*.ogg にする。
#   PYTHONUTF8=1 python tools/fetch_sfx.py [--only イベント名,…] [--redo]
# - ライセンスは cc0 / pdm だけ(クレジット不要)。足りないイベントだけ cc-by も許し、CREDITS に書く
# - 検索結果は art_work/sfx/cache に保存(再実行で API を叩かない)。1秒あけて問い合わせる
# - assets/sfx_pick.json に {イベント名: openverse の id} を書くと その音を使う(差しかえ用)
# - ffmpeg で モノラル・先頭の無音を削る・長さを切る・フェードアウト・音量をそろえる
import json
import os
import subprocess
import sys
import time
import urllib.parse
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from sfx_table import EVENTS  # noqa: E402

ROOT = os.path.join(HERE, '..')
CACHE = os.path.join(ROOT, 'art_work', 'sfx', 'cache')
OUT = os.path.join(ROOT, 'assets', 'sfx')
PICK = os.path.join(ROOT, 'assets', 'sfx_pick.json')
UA = {'User-Agent': 'dungeon-making-sfx/1.0 (personal game project)'}
os.makedirs(CACHE, exist_ok=True)
os.makedirs(OUT, exist_ok=True)
FFMPEG = 'ffmpeg'


def api(query, lic):
    key = os.path.join(CACHE, 'q_%s_%s.json' % (lic.replace(',', '+'), urllib.parse.quote(query, safe='')))
    if os.path.exists(key):
        return json.load(open(key, encoding='utf-8'))
    url = 'https://api.openverse.org/v1/audio/?' + urllib.parse.urlencode({'q': query, 'license': lic, 'page_size': 20})
    for k in range(3):
        try:
            d = json.load(urllib.request.urlopen(urllib.request.Request(url, headers=UA), timeout=40))
            break
        except Exception as e:  # noqa: BLE001
            print('  api retry', e)
            time.sleep(3 + 3 * k)
    else:
        return {'results': []}
    json.dump(d, open(key, 'w', encoding='utf-8'), ensure_ascii=False)
    time.sleep(1.0)
    return d


def download(r):
    path = os.path.join(CACHE, 'a_%s.%s' % (r['id'], r.get('filetype') or 'mp3'))
    if os.path.exists(path) and os.path.getsize(path) > 500:
        return path
    try:
        b = urllib.request.urlopen(urllib.request.Request(r['url'], headers=UA), timeout=60).read()
    except Exception as e:  # noqa: BLE001
        print('  download failed', r['id'], e)
        return None
    if len(b) < 500:
        return None
    open(path, 'wb').write(b)
    return path


def process(src, dst, secs, gain):
    # 先頭の無音を削る → 長さを切る → フェードアウト → モノ・22.05kHz → 音量をそろえる(loudnorm でなく 最大値基準)
    tmp = dst + '.tmp.wav'
    for th in (30, 45, 60):
        af = 'silenceremove=start_periods=1:start_threshold=-%ddB,atrim=0:%s,afade=t=out:st=%s:d=0.08,aresample=22050' % (th, secs, max(secs - 0.08, 0.05))
        subprocess.run([FFMPEG, '-y', '-loglevel', 'error', '-i', src, '-ac', '1', '-af', af, tmp], check=True)
        if os.path.getsize(tmp) > 22050 * 2 * 0.08:  # 0.08秒より長ければ採用
            break
    # 最大値を測って gain に合わせる
    p = subprocess.run([FFMPEG, '-i', tmp, '-af', 'volumedetect', '-f', 'null', '-'], capture_output=True, text=True)
    mx = -6.0
    for line in p.stderr.splitlines():
        if 'max_volume' in line:
            mx = float(line.split('max_volume:')[1].split('dB')[0])
    target = -1.0 + 20 * __import__('math').log10(max(gain, 0.05))
    subprocess.run([FFMPEG, '-y', '-loglevel', 'error', '-i', tmp, '-af', 'volume=%sdB' % (target - mx), '-c:a', 'libvorbis', '-q:a', '3', dst], check=True)
    os.remove(tmp)


def main():
    only = None
    if '--only' in sys.argv:
        only = set(sys.argv[sys.argv.index('--only') + 1].split(','))
    redo = '--redo' in sys.argv
    pick = json.load(open(PICK, encoding='utf-8')) if os.path.exists(PICK) else {}
    credits = {}
    cpath = os.path.join(CACHE, 'credits.json')
    if os.path.exists(cpath):
        credits = json.load(open(cpath, encoding='utf-8'))
    for ev, (query, maxms, secs, gain) in EVENTS.items():
        if only and ev not in only:
            continue
        dst = os.path.join(OUT, ev + '.ogg')
        if os.path.exists(dst) and not redo and ev in credits:
            continue
        print('==', ev, query)
        cands = []
        queries = query.split('|')
        for qi, q in enumerate(queries):
            for lic in ('cc0,pdm', 'by'):
                d = api(q, lic)
                for r in d.get('results', []):
                    if r.get('source') != 'wikimedia_audio' and not r['title'].startswith(('LL-', 'Nl-', 'En-', 'De-', 'Fr-')) and r.get('license') in ('cc0', 'pdm', 'by') and 80 <= (r.get('duration') or 0) <= maxms and r['id'] not in [c['id'] for c in cands]:
                        r['_q'] = qi
                        r['_words'] = q.split()
                        cands.append(r)
                if len([c for c in cands if c['license'] in ('cc0', 'pdm')]) >= 3:
                    break
            if len([c for c in cands if c['license'] in ('cc0', 'pdm')]) >= 2:
                break

        def score(r):
            text = (r['title'] + ' ' + ' '.join(t.get('name', '') for t in (r.get('tags') or []))).lower()
            return sum(1 for w in r['_words'] if w.lower() in text)

        cands.sort(key=lambda r: (r['id'] != pick.get(ev), r['license'] == 'by', r['_q'], -score(r), abs((r.get('duration') or 0) - 1200)))
        done = False
        for r in cands[:8]:
            src = download(r)
            if not src:
                continue
            try:
                process(src, dst, secs, gain)
            except Exception as e:  # noqa: BLE001
                print('  ffmpeg failed', r['id'], e)
                continue
            credits[ev] = {'id': r['id'], 'title': r['title'], 'creator': r.get('creator'), 'license': r['license'], 'version': r.get('license_version'),
                           'url': r.get('foreign_landing_url'), 'source': r.get('source'), 'duration_ms': r.get('duration')}
            print('  ->', r['title'][:40], r['license'], r.get('duration'), 'ms', r.get('creator'))
            done = True
            break
        if not done:
            print('  !! 見つからない', ev)
        json.dump(credits, open(cpath, 'w', encoding='utf-8'), ensure_ascii=False, indent=1)
    # CREDITS
    lines = ['# 効果音の出典(Openverse 経由)', '', '`tools/fetch_sfx.py` が集めた。ライセンスは CC0 / パブリックドメイン(クレジット不要)と CC BY(クレジット必要)。', '',
             '| イベント | 題名 | 作者 | ライセンス | 出典 | Openverse id |', '|---|---|---|---|---|---|']
    for ev in EVENTS:
        c = credits.get(ev)
        if c:
            lic = c['license'].upper() + (' ' + c['version'] if c.get('version') else '')
            lines.append('| %s | %s | %s | %s | %s | %s |' % (ev, (c['title'] or '').replace('|', '/'), (c['creator'] or '').replace('|', '/'), lic, c['url'], c['id']))
    open(os.path.join(OUT, 'CREDITS.md'), 'w', encoding='utf-8').write('\n'.join(lines) + '\n')
    missing = [e for e in EVENTS if e not in credits]
    print('missing:', missing)


main()
