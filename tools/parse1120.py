# p25(モンスター表) から 11F〜20F の魔物のドロップを取り出す。
import os, re, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import parse_wiki as P

R = os.path.expanduser(r'~\research\dungeon-maker-shovel\sources')
NEW_NAMES = ['アナゲリラ', 'ランバーダック', 'ダルマ', 'サムライウルフ', 'ショーグンウルフ', 'キラーコック', 'ヒダルマ', 'メイドール', 'ユキダルマ', 'キノコトカゲ', 'ワービートル',
             '炎のマスター', 'ハイマスター', 'デタラメ', 'イモムシ', 'キラーフラワー', 'ホネウイザード', 'マジックドンキー', 'アクマノサイコロ', 'ジゴクノサケビ', 'マホウノタテ', 'マホウノユミ', 'マホウノケン',
             'ワルキリー＋', 'カワウソーサラー',
             'ドロヌマ', 'ジョーグンウルフ', 'デビルプリンセス', 'ヒトツメオニ', 'フレイムフラワー', 'カゲ', 'ミツメオニ', 'エレファントム', 'デビルロード', 'ヘビリンゴ']
ROOMS = P.ROOMS | {'ごうかなへや'}
L = [l.strip() for l in open(os.path.join(R, 'atwiki-dmakerds-25.md'), encoding='utf-8').read().split('\n')]
L = [l for l in L if l]
start = L.index('ダルマ') if 'ダルマ' in L else 0
# アナゲリラの行から
start = next(i for i, l in enumerate(L) if l == 'アナゲリラ' and L[i + 1] == 'はたけ')
L = L[start:]
names = set(NEW_NAMES) | {'(' + n + ')' for n in NEW_NAMES} | {'(ハイマスター)', '(炎のマスター)', '(ワルキリー＋)', '(カワウソーサラー)', '(ショーグンウルフ)', '(アツイオバケ！)', '(サムイオバケ！)'}
heads = []
for i, l in enumerate(L):
    if l in names and i + 1 < len(L) and (L[i + 1] in ROOMS or re.fullmatch(r'\d+F(魔穴|ボス)', L[i + 1]) or L[i + 1] in ('元4Fボス', '元12Fボス')):
        heads.append(i)
blocks = {}
for k, i in enumerate(heads):
    j = heads[k + 1] if k + 1 < len(heads) else len(L)
    seg = L[i + 1:j]
    spawn, drops = [], []
    n = 0
    while n < len(seg) and (seg[n] in ROOMS or re.fullmatch(r'\d+F(魔穴|ボス)', seg[n]) or seg[n].startswith('元')):
        spawn.append(seg[n]); n += 1
    drops = seg[n:]
    nm = L[i].strip('()')
    blocks[nm] = (spawn, drops)
known = set(P.weapons) | set(P.armor) | set(P.items17) | set(P.foods17)
if __name__ == '__main__':
    for nm, (sp, dr) in blocks.items():
        ok = [d for d in dr if d in known]
        bad = [d for d in dr if d not in known]
        print(nm, sp, '\n   OK', ok, '\n   ??', bad)
