# 街・店の絵の「絵柄の案」を作る。 ComfyUI の venv の python で実行:
#   C:\Users\hamuh\ComfyUI\.venv\Scripts\python.exe tools\gen_proposals.py
# 出力: art_work/proposals/<案>_<場面>.png と 一覧 sheet.png
import json
import os
import subprocess
import sys

from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.normpath(os.path.join(HERE, '..'))
OUT = os.path.join(ROOT, 'art_work', 'proposals')
QWEN = os.path.expanduser(r'~\.claude\skills\qwen-image\scripts\qwen_image.py')
os.makedirs(OUT, exist_ok=True)

STYLES = {
    'A_cel': "Illustrated in the same style as chibi RPG character sprites: clean dark-brown lineart, soft cel shading with light painterly texture, simplified cute shapes, muted earthy medieval fantasy palette, cozy storybook JRPG scene. Villagers are chibi, 2.5 heads tall, with small plain eyes.",
    'B_pixel': "16-bit era JRPG pixel art scene, crisp visible pixels, limited warm earthy palette, dithering, villagers are tiny chibi pixel sprites with small plain eyes.",
    'C_watercolor': "Watercolor picture-book illustration with ink outlines, soft warm earthy pastel tones, gentle paper texture, villagers are chibi, 2.5 heads tall, with small plain eyes.",
    'D_diorama': "Cute 3D miniature diorama render, soft clay toy look, warm lighting, shallow depth of field, villagers are chibi figurines, 2.5 heads tall, with small plain eyes.",
}

SCENES = {
    'town': "Wide view of a small medieval fantasy village in bright daytime. Top-left: a dark cave entrance in a rocky hillside. Bottom-left: a small cottage with a chimney. Center-left: a stone shop with an orange tiled roof and a crossed-swords sign (armor shop), a red-haired woman stands at its door. Center-right: a blue-roofed shop with a purple potion sign (magic shop), an elderly white-bearded man in a purple robe stands at its door. Right: a green-roofed carpenter workshop with a wooden chair sign, a man in a red cap stands by the door. Foreground center: a dirt plaza with a stone fountain, a young blonde village girl near the fountain, an old village chief in a brown coat at lower left. The sky is calm and fairly empty. No text, no UI.",
    'armor': "Interior of a medieval fantasy armor and weapon shop. A young woman shopkeeper with an auburn ponytail and a leather apron stands in the exact center behind a wooden counter, upper body visible. Swords, shields and armor hang on wall racks. The lower third of the image is the wooden counter front. No text, no UI.",
}


def main():
    env = dict(os.environ, PYTHONIOENCODING='utf-8', PYTHONUTF8='1')
    seed = 7
    made = []
    for sk, sv in STYLES.items():
        for nk, nv in SCENES.items():
            name = f'{sk}_{nk}'
            dst = os.path.join(OUT, name + '.png')
            if os.path.exists(dst):
                made.append(dst)
                continue
            pf = os.path.join(OUT, name + '.txt')
            with open(pf, 'w', encoding='utf-8') as f:
                f.write(sv + ' ' + nv)
            r = subprocess.run([sys.executable, QWEN, '--prompt-file', pf, '--aspect', '4:3', '--seed', str(seed), '--out', OUT, '--name', name],
                               capture_output=True, text=True, encoding='utf-8', errors='replace', env=env)
            out = (r.stdout or '').strip().splitlines()
            js = [l for l in out if l.startswith('{')]
            if not js or not json.loads(js[-1]).get('ok'):
                print('FAILED', name)
                continue
            src = json.loads(js[-1])['images'][0]['path']
            os.replace(src, dst)
            print('ok', name, flush=True)
            made.append(dst)
    # 一覧(行=案、列=場面)
    cw, ch = 480, 360
    sheet = Image.new('RGB', (cw * 2, ch * len(STYLES)), (30, 30, 30))
    for i, sk in enumerate(STYLES):
        for j, nk in enumerate(SCENES):
            p = os.path.join(OUT, f'{sk}_{nk}.png')
            if os.path.exists(p):
                sheet.paste(Image.open(p).convert('RGB').resize((cw, ch)), (j * cw, i * ch))
    sheet.save(os.path.join(OUT, 'sheet.png'))
    print('done')


if __name__ == '__main__':
    main()
