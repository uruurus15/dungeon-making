# wiki(research フォルダの atwiki-p*.md)から、6F〜10F の魔物のドロップと、アイテムの値を取り出して
# Rust の data.rs に貼る断片を art_work/data610.rs.txt に書く。
#   python tools/parse_wiki.py
import os
import re
import sys

R = os.path.expanduser(r'~\research\dungeon-maker-shovel\sources')
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'art_work', 'data610.rs.txt')


def lines(name):
    return [l.strip() for l in open(os.path.join(R, name), encoding='utf-8').read().split('\n')]


def sections(L, heads):
    """見出し(### xxx)ごとに行を分ける"""
    cur = None
    out = {}
    for l in L:
        if l.startswith('### '):
            cur = l[4:].strip()
            out[cur] = []
        elif cur is not None:
            out.setdefault(cur, []).append(l)
    return out


def rows(Lsec, minnum=3):
    """ 名前 数 数 数 … の行を拾う(空行は無視) """
    L = [l for l in Lsec if l]
    res = {}
    i = 0
    while i < len(L) - 3:
        if re.fullmatch(r'-?\d+', L[i + 1] or '') and re.fullmatch(r'\d+', L[i + 2] or '') and re.fullmatch(r'\d+', L[i + 3] or '') and not re.fullmatch(r'-?\d+', L[i]):
            res[L[i]] = (int(L[i + 1]), int(L[i + 2]), int(L[i + 3]), L[i + 4] if i + 4 < len(L) else '')
            i += 4
        else:
            i += 1
    return res


w = sections(lines('atwiki-p15.md'), None)
a = sections(lines('atwiki-p16.md'), None)
weapons = {}
for k, v in w.items():
    if k in ('更新履歴',):
        continue
    for n, t in rows(v).items():
        weapons[n] = t
armor = {}
for k, kind in (('盾', 'Shield'), ('頭', 'Head'), ('体', 'Body'), ('足', 'Feet'), ('指輪', 'Ring')):
    for n, t in rows(a.get(k, [])).items():
        armor[n] = (kind,) + t

# p17: アイテム / たべもの
p17 = sections(lines('atwiki-p17.md'), None)


def rows2(Lsec):
    """ 名前 買値 売値 … (買値が空のこともある) """
    L = [l for l in Lsec if l]
    res = {}
    for i in range(len(L) - 2):
        if re.fullmatch(r'\d+', L[i + 1]) and not re.fullmatch(r'\d+', L[i]):
            buy = int(L[i + 1])
            sell = int(L[i + 2]) if re.fullmatch(r'\d+', L[i + 2]) else 0
            res[L[i]] = (buy, sell, L[i + 3] if i + 3 < len(L) else '')
    return res


items17 = rows2(p17.get('アイテムずかん', []))
foods17 = rows2(p17.get('たべものずかん', []))

# p25: モンスター表
L = [l for l in lines('atwiki-dmakerds-25.md') if l]
a0 = L.index('モンスター') if 'モンスター' in L else 0
MON = ['チカラウシ', 'パワーワン', '炎のウオリアー', '氷のウオリアー', 'マスターウオリア', 'ホノオジゴク', 'フブキジゴク', 'バッファローブル', 'ワーペン', 'ノロイピーマン',
       'ぎりぎりネコマタ', 'いただきネコマタ', 'マダシンデネー', 'カモノハシーフ', 'ワルキリー', 'ワーネコクイーン', 'テイオウワーワン', 'ドクライム', 'アクマノヨロイ',
       'デブリン', 'アクトパス', 'キングリオン', 'アンクルドラゴン', 'デビルプリンス']
ROOMS = {'通路', 'エサおけ', 'ゴミすてば', 'わらのベッド', 'りっぱなベッド', 'ごうかなベッド', 'いろり', 'だんろ', '水のみば', 'ふんすい', 'さかば', 'けっかいのへや', 'さいだん', 'れいはいどう', 'はたけ', 'はなぞの',
         'どうじょう', 'じっけんしつ', 'コロシアム', 'まほうじん', 'れいぞうこ'}
starts = {}
for m in MON:
    for i, t in enumerate(L):
        if t == m and i + 1 < len(L) and (L[i + 1] in ROOMS or re.fullmatch(r'\d+F魔穴', L[i + 1])):
            starts[m] = i
            break
order = sorted(starts.items(), key=lambda kv: kv[1])
blocks = {}
for idx, (m, st) in enumerate(order):
    en = order[idx + 1][1] if idx + 1 < len(order) else len(L)
    # 次のブロックの前にある別のモンスター名(今回の対象外)は 図鑑の他の魔物。ROOMS の直後にある名前を境にする
    seg = L[st + 1:en]
    spawn = []
    j = 0
    while j < len(seg) and (seg[j] in ROOMS or re.fullmatch(r'\d+F(魔穴|ボス)', seg[j])):
        spawn.append(seg[j])
        j += 1
    rest = seg[j:]
    drops = []
    for t in rest:
        # 対象外の別の魔物の見出し(次の魔物名)で打ち切る: ROOMS が直後に来る語
        drops.append(t)
    blocks[m] = (spawn, drops)

known = {}
known.update({n: ('Weapon',) + t for n, t in weapons.items()})
known.update(armor)

out = []
missing = set()
for m, (spawn, drops) in blocks.items():
    out.append(f'// {m}: 出現 {spawn}')
    got = []
    for d in drops:
        if d in known or d in items17 or d in foods17:
            got.append(d)
        else:
            missing.add(d)
    # 次の魔物名が混ざらないよう、既知アイテムだけ残す
    out.append(f'//   ドロップ候補: {got}')
with open(OUT, 'w', encoding='utf-8') as f:
    f.write('\n'.join(out) + '\n\n// ---- 武器 ----\n')
    for n, t in sorted(weapons.items()):
        f.write(f'W {n} power={t[0]} buy={t[1]} sell={t[2]} {t[3][:30]}\n')
    f.write('// ---- 防具 ----\n')
    for n, t in sorted(armor.items()):
        f.write(f'{t[0][0]} {n} power={t[1]} buy={t[2]} sell={t[3]} {t[4][:30]}\n')
    f.write('// ---- アイテム ----\n')
    for n, t in sorted(items17.items()):
        f.write(f'I {n} buy={t[0]} sell={t[1]} {t[2][:30]}\n')
    f.write('// ---- 食べ物 ----\n')
    for n, t in sorted(foods17.items()):
        f.write(f'F {n} buy={t[0]} sell={t[1]} {t[2][:30]}\n')
print('blocks', len(blocks), 'weapons', len(weapons), 'armor', len(armor), 'items', len(items17), 'foods', len(foods17))
print('no block for:', [m for m in MON if m not in blocks])
