# Web 版(GitHub Pages)の 配布ファイルを docs/ に まとめる。
#   1) cargo build --release --target wasm32-unknown-unknown
#   2) python tools/pack_web.py
# docs/ に index.html・wasm・JS・assets.pack(画像・音・フォントを 1つに)が できる。
import os, shutil, struct
ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
OUT = os.path.join(ROOT, 'docs')
os.makedirs(OUT, exist_ok=True)
for f in ['index.html', 'mq_js_bundle.js', 'sapp_jsutils.js', 'quad-storage.js']:
    shutil.copy(os.path.join(ROOT, 'web', f), os.path.join(OUT, f))
shutil.copy(os.path.join(ROOT, 'target', 'wasm32-unknown-unknown', 'release', 'dungeon_making.wasm'), os.path.join(OUT, 'dungeon_making.wasm'))
files = []
for sub in ['img', 'sfx', 'bgm', 'font']:
    d = os.path.join(ROOT, 'assets', sub)
    for n in sorted(os.listdir(d)):
        if n.lower().endswith(('.png', '.ogg', '.ttf')):
            files.append(('assets/%s/%s' % (sub, n), os.path.join(d, n)))
with open(os.path.join(OUT, 'assets.pack'), 'wb') as w:
    w.write(struct.pack('<I', len(files)))
    for name, path in files:
        data = open(path, 'rb').read()
        nb = name.encode('utf-8')
        w.write(struct.pack('<I', len(nb))); w.write(nb)
        w.write(struct.pack('<I', len(data))); w.write(data)
print('files', len(files), 'pack MB', round(os.path.getsize(os.path.join(OUT, 'assets.pack')) / 1e6, 1))
for n in os.listdir(OUT):
    print(n, round(os.path.getsize(os.path.join(OUT, n)) / 1e6, 2), 'MB')
