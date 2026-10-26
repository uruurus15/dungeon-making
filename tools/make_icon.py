# art_work/icon_raw.png(qwen-image で作ったシャベルの絵)から、ウィンドウ/exe 用のアイコンを作る。
#   C:\Users\hamuh\ComfyUI\.venv\Scripts\python.exe tools\make_icon.py
import os
from PIL import Image

ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
src = Image.open(os.path.join(ROOT, 'art_work', 'icon_raw.png')).convert('RGBA')
# シャベルが大きく見えるよう、中央を正方形に切り出す
cx, cy, side = 500, 520, 700
im = src.crop((cx - side // 2, cy - side // 2, cx + side // 2, cy + side // 2))
out = os.path.join(ROOT, 'assets', 'icon')
os.makedirs(out, exist_ok=True)
for n in (16, 32, 64):
    with open(os.path.join(out, f'icon_{n}.rgba'), 'wb') as f:
        f.write(im.resize((n, n), Image.LANCZOS).tobytes())
im.resize((256, 256), Image.LANCZOS).save(os.path.join(out, 'icon.png'))
im.resize((256, 256), Image.LANCZOS).save(os.path.join(out, 'icon.ico'), sizes=[(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
print('ok')
