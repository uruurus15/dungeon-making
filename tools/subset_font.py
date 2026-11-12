# 同梱フォント(assets/font/NotoSansJP-Bold-subset.ttf)を、ソースに 出てくる 文字だけに 絞って 作り直す。
#   python tools/subset_font.py          足りない 文字が あれば 作り直す
#   python tools/subset_font.py --check  足りない 文字を 出すだけ(あれば 終了コード 1)
#   python tools/subset_font.py --force  かならず 作り直す
# Web 版は この フォントしか 使えないので、ここに ない 文字は 画面に 出ない(デスクトップ版は Windows の フォントを 使うので 気づけない)。
# tools/pack_web.py が 毎回 よぶ。もとの フォントは Windows の Noto Sans JP(可変フォント)の 太さ 700。fonttools が 要る。
import glob, os, sys
from fontTools.ttLib import TTFont

ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
OUT = os.path.join(ROOT, 'assets', 'font', 'NotoSansJP-Bold-subset.ttf')
SRC_FONTS = ['C:/Windows/Fonts/NotoSansJP-VF.ttf', os.path.join(ROOT, 'tools', 'NotoSansJP-VF.ttf')]
# ソースに なくても 入れておく 文字(英数字・かな・よく使う 記号)
ALWAYS = (
    ''.join(chr(c) for c in range(0x20, 0x7f))
    + ''.join(chr(c) for c in range(0x3041, 0x3097))
    + ''.join(chr(c) for c in range(0x30a1, 0x30fd))
    + ''.join(chr(c) for c in range(0xff01, 0xff5f))
    + '、。・「」『』【】《》〈〉…‥ー〜～※×÷←→↑↓★☆◆◇○●◎□■△▲▽▼♪♥♡％　'
)


def wanted():
    chars = set(ALWAYS)
    for p in glob.glob(os.path.join(ROOT, 'src', '*.rs')):
        if os.path.basename(p) == 'tests.rs':
            continue
        chars.update(c for c in open(p, encoding='utf-8').read() if not c.isspace() and c.isprintable())
    return chars


def missing():
    if not os.path.exists(OUT):
        return wanted()
    cmap = TTFont(OUT).getBestCmap()
    return {c for c in wanted() if ord(c) not in cmap}


def build():
    from fontTools.varLib import instancer
    from fontTools import subset
    src = next((p for p in SRC_FONTS if os.path.exists(p)), None)
    if src is None:
        sys.exit('もとの フォントが ない: ' + ' / '.join(SRC_FONTS))
    font = instancer.instantiateVariableFont(TTFont(src), {'wght': 700})
    cmap = font.getBestCmap()
    want = wanted()
    absent = sorted(c for c in want if ord(c) not in cmap)
    opt = subset.Options()
    opt.layout_features = ['*']
    opt.name_IDs = ['*']
    opt.notdef_outline = True
    sub = subset.Subsetter(opt)
    sub.populate(unicodes=[ord(c) for c in want if ord(c) in cmap])
    sub.subset(font)
    font.save(OUT)
    print('font: %d 文字 / %.2f MB' % (len(font.getBestCmap()), os.path.getsize(OUT) / 1e6))
    if absent:
        print('font: もとの フォントにも ない 文字: ' + ''.join(absent))


def ensure():
    m = missing()
    if m:
        print('font: 足りない 文字 %d: %s → 作り直す' % (len(m), ''.join(sorted(m))))
        build()
        return True
    return False


if __name__ == '__main__':
    sys.stdout.reconfigure(encoding='utf-8')
    if '--check' in sys.argv:
        m = missing()
        print('font: 足りない 文字 %d: %s' % (len(m), ''.join(sorted(m))))
        sys.exit(1 if m else 0)
    if '--force' in sys.argv:
        build()
    elif not ensure():
        print('font: 足りない 文字は ない')
