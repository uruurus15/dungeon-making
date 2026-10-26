# 11F〜20F のデータ(アイテム・魔物・出現表)を wiki から作って Rust の断片にする。
#   PYTHONUTF8=1 python tools/gen_data1120.py  →  art_work/data1120/*.txt (UTF-8)
# 知らない名前が出たら その場で止まる(黙って落とさない)。
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import parse_wiki as P  # noqa: E402
import parse1120 as Q  # noqa: E402

ROOT = os.path.join(HERE, '..')
OUT = os.path.join(ROOT, 'art_work', 'data1120')
os.makedirs(OUT, exist_ok=True)

# ---- 名前 → id ----
ID = {'カラスコウモリ': 'bat', 'ブタイノシシ': 'pig', 'ワーネコ': 'cat', 'ワーワン': 'wan', 'ゴブリン': 'gobl', 'ワライオバケ': 'ghost', 'ヒブタ': 'hibuta', 'ニンゲンザウルス': 'ningen',
      '火ライム': 'firelime', 'アツイオバケ': 'hotghost', 'ネコマタ': 'nekomata', '水ライム': 'waterlime', 'サムイオバケ': 'coldghost', 'カワウソードマン': 'otter', 'ミサイルニンジン': 'carrot',
      'カラスネコ': 'crowcat', 'ブラックワーネコ': 'blackcat', 'ウイザードリル': 'wizdrill', 'くろのウオリアー': 'blackwar', 'ホネファイター': 'bone', 'スライム': 'slime', 'バクダンイモ': 'potato',
      'チカラウシ': 'chikaraushi', 'パワーワン': 'powerwan', '炎のウオリアー': 'flamewar', '氷のウオリアー': 'icewar', 'マスターウオリア': 'masterwar', 'ホノオジゴク': 'hotjigoku',
      'フブキジゴク': 'icejigoku', 'バッファローブル': 'buffalo', 'ワーペン': 'penguin', 'ノロイピーマン': 'pepper', 'ぎりぎりネコマタ': 'nekomata2', 'いただきネコマタ': 'nekomata3',
      'マダシンデネー': 'zombie', 'カモノハシーフ': 'platypus', 'ワルキリー': 'valkyrie', 'ワーネコクイーン': 'catqueen', 'テイオウワーワン': 'emperorwan', 'ドクライム': 'poisonlime',
      'アクマノヨロイ': 'demonarmor',
      'アナゲリラ': 'anagerilla', 'ランバーダック': 'lumberduck', 'ダルマ': 'daruma', 'サムライウルフ': 'samuraiwolf', 'ショーグンウルフ': 'shogun', 'キラーコック': 'killercock',
      'ヒダルマ': 'hidaruma', 'メイドール': 'maidoll', 'ユキダルマ': 'yukidaruma', 'キノコトカゲ': 'mushroomlizard', 'ワービートル': 'warbeetle', '炎のマスター': 'flamemaster',
      'ハイマスター': 'highmaster', 'デタラメ': 'detarame', 'イモムシ': 'caterpillar', 'キラーフラワー': 'killerflower', 'ホネウイザード': 'bonewizard', 'マジックドンキー': 'magicdonkey',
      'アクマノサイコロ': 'devildice', 'ジゴクノサケビ': 'hellscream', 'マホウノタテ': 'magicshield', 'マホウノユミ': 'magicbow', 'マホウノケン': 'magicsword',
      'ワルキリー＋': 'valkyrie2', 'カワウソーサラー': 'sorcerer_n'}
SPRITE = {'shogun': 'mon_shogunwolf', 'valkyrie2': 'mon_valkyrie', 'sorcerer_n': 'mon_sorcerer'}
NORMAL_NEW = ['anagerilla', 'lumberduck', 'daruma', 'samuraiwolf', 'shogun', 'killercock', 'hidaruma', 'maidoll', 'yukidaruma', 'mushroomlizard', 'warbeetle', 'flamemaster', 'highmaster',
              'detarame', 'caterpillar', 'killerflower', 'bonewizard', 'magicdonkey', 'devildice', 'hellscream', 'magicshield', 'magicbow', 'magicsword', 'valkyrie2', 'sorcerer_n']
JP = {v: k for k, v in ID.items()}

# ---- 出現表(floors.txt = 各階ページの「出現するモンスター」) ----
ROOMS = {'通路', 'エサおけ', 'ゴミすてば', 'わらのベッド', 'りっぱなベッド', 'ごうかなベッド', 'いろり', 'だんろ', '水のみば', 'ふんすい', 'さかば', 'けっかいのへや', 'さいだん', 'れいはいどう',
         'はたけ', 'はなぞの', 'どうじょう', 'じっけんしつ', 'コロシアム', 'れいぞうこ', 'まほうじん'}
txt = open(os.path.join(ROOT, 'art_work', 'w20', 'floors.txt'), encoding='utf-8').read()
SP = {}
for sec in txt.split('========='):
    m = re.search(r'^## (\d+)F', sec, re.M)
    if not m:
        continue
    f = int(m.group(1))
    a = sec.index('### 出現するモンスター')
    b = sec.index('### ボス')
    lines = [l.strip() for l in sec[a:b].split('\n')[1:] if l.strip()]
    tbl = {}
    cur = None
    for l in lines:
        if l in ROOMS:
            cur = l
            tbl[cur] = []
        else:
            assert cur, (f, l)
            for n in l.split('、'):
                n = n.strip().replace('！', '')
                if n not in ID:
                    sys.exit('知らない魔物名: %dF %s %r' % (f, cur, n))
                tbl[cur].append(ID[n])
    SP[f] = tbl

arms = []
cor = []
for f in sorted(SP):
    for room, ids in SP[f].items():
        if room == '通路':
            cor.append('        %d => &[%s],' % (f, ', '.join('"%s"' % i for i in ids)))
        elif ids:
            arms.append('        (%d, "%s") => &[%s],' % (f, room, ', '.join('"%s"' % i for i in ids)))
open(os.path.join(OUT, 'spawns.txt'), 'w', encoding='utf-8').write('\n'.join(arms) + '\n')
open(os.path.join(OUT, 'corridor.txt'), 'w', encoding='utf-8').write('\n'.join(cor) + '\n')

# ---- アイテム ----
RING_CODE = [('つよさ', 1), ('こうげき', 1), ('じょうぶ', 2), ('ぼうぎょ', 2), ('かしこ', 3)]
KIND_ARMOR = {'Shield': 'Shield', 'Head': 'Head', 'Body': 'Body', 'Feet': 'Feet', 'Ring': 'Ring'}


def esc(s):
    return s.replace('"', '\\"')


def good_desc(n, d):
    """wiki の備考欄は 入手元の魔物名などが混ざる。装備の説明らしい物だけ残す"""
    if not d or re.search(r'B\d+F', d) or d in JP.values() or d in P.items17 or d in P.foods17 or d in P.weapons or d in P.armor or d in ID:
        return n + '。'
    return d[:24]


def item_line(n):
    if n in P.weapons:
        t = P.weapons[n]
        return 'it("%s", Kind::Weapon, %d, %d, %d, 0, "%s"),' % (n, t[1], t[2], t[0], esc(good_desc(n, t[3])))
    if n in P.armor:
        t = P.armor[n]
        kind = KIND_ARMOR[t[0]]
        code = 0
        pw = t[1]
        if kind == 'Ring':
            # 効果が つよさ/じょうぶさ/かしこさ の数値でない指輪(耐性など)は 効果なし(0)にして resist 表にまかせる
            m = re.search(r'(攻撃力|つよさ)\+(\d+)', t[4])
            code, pw = (1, int(m.group(2))) if m else (0, 0)
        return 'it("%s", Kind::%s, %d, %d, %d, %d, "%s"),' % (n, kind, t[2], t[3], pw, code, esc(good_desc(n, t[4])))
    if n in P.foods17 or n in P.items17:
        t = P.foods17.get(n) or P.items17[n]
        if t[1] > 0:
            buy, sell = t[0], t[1]
        else:
            buy, sell = 0, t[0]
        if n in P.foods17:
            src = t[2]
            desc = (src + 'が落とす。') if src in ID else n + '。'
            return 'it("%s", Kind::%s, %d, %d, 0, 0, "%s"),' % (n, 'Veg' if buy > 0 else 'Food', buy, sell, esc(desc))
        return 'it("%s", Kind::Loot, 0, %d, 0, 0, "%s"),' % (n, sell or 1500, esc(good_desc(n, t[2]) if t[2] else '高く売れる。'))
    sys.exit('知らないアイテム: ' + n)


# 宝箱(p37)で出る物 + ボス・クエストの品
CHEST_NAMES = ('スリーパー こうてつホルター ちょうじんのゆびわ ホリーメイス マジックローブ じょうぶつのゆびわ まもりのゆびわ ロングアクス ひっさつのぼうし ダイヤモンド てばさき せんしのゆびわ '
               'かぜのつえ かぜのローブ でたらめなにく 炎のゆみ ソードオブファイア ちからのティアラ ハルバード かがみのたて 火のはなのね ホリーダッガー あくまのティアラ キノコ タネあぶら '
               'サツマイモ キラーリング まじょのゆびわ エルフのゆみ ダイヤホルター まふうじのゆびわ カオスバスター カオスブリンガー めがみのティアラ めいおうのつえ せいぎのよろい '
               'ひっさつのゆびわ モグラのももにく ホタテ タコのあし ブルのサーロイン ハマグリ イワシ かわざかな ドクライムのひもの カラスネコのしっぽ とくじょうニンジン トカゲのももにく サイコロにく '
               'シルフのゆみ ゴールドヘルム').split()
QUEST_ITEMS = {'めいおうのしょうこ': 'デビルロードの証。村長に見せよう。', 'フレイムフラワーのはな': 'フレイムフラワーの花。村長に見せよう。', 'エレファントムのきば': 'エレファントムの牙。村長に見せよう。', 'キングヘビリンゴのへた': 'キングヘビリンゴのへた。村長に見せよう。', 'ガイアのゆびわ': 'ドロヌマの落とした指輪。村長に見せよう。', 'マサムネ': 'ショーグンウルフの名刀。村長に見せよう。', 'デビルダッガー': 'デビルプリンセスの短剣。村長に見せよう。',
               'ヒトツメクラブ': 'ヒトツメオニのこん棒。村長に見せよう。', 'コテツ': 'カゲの持っていた刀。村長に見せよう。', 'ミツメクラブ': 'ミツメオニのこん棒。村長に見せよう。',
               'わぎりのにく': 'エレファントムの肉。村長に見せよう。', 'どくリンゴ': 'キングヘビリンゴの毒リンゴ。村長に見せよう。'}
want = []
seen = set()
for nm, (sp, dr) in Q.blocks.items():
    for d in dr:
        if d in Q.known and d not in seen and nm not in ('ヘビリンゴ', 'デビルプリンセス', 'ヒトツメオニ', 'カゲ', 'ミツメオニ', 'エレファントム', 'デビルロード', 'フレイムフラワー', 'ドロヌマ', 'ジョーグンウルフ'):
            seen.add(d)
            want.append(d)
for d in CHEST_NAMES + ['かいふくのつえ', 'いにしえのゆびわ']:
    if d not in seen:
        seen.add(d)
        want.append(d)
for q in QUEST_ITEMS:
    seen.discard(q)
want = [w for w in want if w not in QUEST_ITEMS]
items = [item_line(n) for n in want]
for n, d in QUEST_ITEMS.items():
    if n == '火のはなのね':
        continue
    items.append('it("%s", Kind::Quest, 0, 0, 0, 0, "%s"),' % (n, d))
open(os.path.join(OUT, 'items.txt'), 'w', encoding='utf-8').write('\n'.join('    ' + l for l in items) + '\n')

# ---- 魔物 ----
FIRST = {}
for f in sorted(SP):
    for room, ids in SP[f].items():
        for i in ids:
            FIRST.setdefault(i, f)
KF = {'shogun': 1.7, 'highmaster': 1.5, 'flamemaster': 1.4, 'valkyrie2': 1.3, 'sorcerer_n': 1.2, 'detarame': 1.0, 'devildice': 1.1, 'magicsword': 1.2, 'magicshield': 1.2, 'magicbow': 1.1,
      'hellscream': 1.2, 'bonewizard': 1.0, 'magicdonkey': 1.1, 'killercock': 1.0, 'caterpillar': 0.9, 'killerflower': 1.0, 'anagerilla': 0.9, 'lumberduck': 1.1, 'daruma': 1.1,
      'samuraiwolf': 1.2, 'hidaruma': 1.0, 'yukidaruma': 1.0, 'maidoll': 1.1, 'mushroomlizard': 0.9, 'warbeetle': 1.3}
SPD = {'killercock': 9, 'anagerilla': 4, 'samuraiwolf': 8, 'shogun': 9, 'warbeetle': 5, 'caterpillar': 2, 'magicsword': 9, 'magicbow': 8, 'hellscream': 9, 'devildice': 7}
UNDEAD = {'bonewizard', 'hellscream'}
SCALE = float(os.environ.get('MON_SCALE', '1.0'))  # 11F以降の強さの倍率(ボットで調整)
DROPS = {}
for nm, (sp, dr) in Q.blocks.items():
    if nm in ID:
        DROPS[ID[nm]] = [d for d in dr if d in Q.known and d not in QUEST_ITEMS or d == 'どくリンゴ']
DROPS['sorcerer_n'] = ['ふぶきのつえ', 'かいふくのつえ', 'ウイズコーン', '氷のローブ', 'ホタテ']
DROPS['valkyrie2'] = ['ワルキリースピア', 'ワルキリーヘルム', 'ワルキリーホルター', 'ワルキリーブーツ', 'よみのゆびわ', 'まもりのゆびわ']


def pct(n):
    if n in P.weapons:
        return 5
    if n in P.armor:
        return 4 if P.armor[n][0] == 'Ring' else 6
    if n in P.foods17:
        return 28
    return 12


lines = []
for mid in NORMAL_NEW:
    k = KF.get(mid, 1.0)
    first = FIRST[mid]
    t = first - 5
    hp = int((70 + 22 * t) * k * SCALE)
    atk = int((34 + 7 * t + (4 if k >= 1.2 else 0)) * SCALE)
    df = int((10 + 3 * t + (3 if k >= 1.3 else 0)) * SCALE)
    spd = SPD.get(mid, 4 + (t % 4))
    gold = int(40 + 12 * t)
    dr = DROPS.get(mid, [])
    for d in dr:
        if d not in want and d not in ('どくリンゴ',):
            sys.exit('ドロップの品が items に無い: %s %s' % (mid, d))
    drs = ', '.join('("%s", %d)' % (d, pct(d)) for d in dr)
    lines.append('    MonDef { id: "%s", name: "%s", hp: %d, atk: %d, def: %d, spd: %d, gold: %d, sprite: "%s", boss: false, undead: %s,\n        drops: &[%s] },' % (
        mid, JP[mid], hp, atk, df, spd, gold, SPRITE.get(mid, 'mon_' + mid), 'true' if mid in UNDEAD else 'false', drs))
# ボス: (id, 名前, 階, 品)
BOSS = [('dorunuma', 'ドロヌマ', 11, 'ガイアのゆびわ', 1.00), ('shogunboss', 'ショーグンウルフ', 12, 'マサムネ', 1.04), ('devilprincess', 'デビルプリンセス', 13, 'デビルダッガー', 1.08),
        ('hitotsuoni', 'ヒトツメオニ', 14, 'ヒトツメクラブ', 1.12), ('flameflower', 'フレイムフラワー', 15, '火のはなのね', 1.16), ('kage', 'カゲ', 16, 'コテツ', 1.20),
        ('mitsuoni', 'ミツメオニ', 17, 'ミツメクラブ', 1.24), ('elephantom', 'エレファントム', 18, 'わぎりのにく', 1.28), ('devillord', 'デビルロード', 19, 'カオスブリンガー', 1.32),
        ('kingsnake', 'キングヘビリンゴ', 20, 'どくリンゴ', 1.40)]
REAL_TOKEN = {'カオスブリンガー': 'めいおうのしょうこ', '火のはなのね': 'フレイムフラワーのはな', 'わぎりのにく': 'エレファントムのきば', 'どくリンゴ': 'キングヘビリンゴのへた'}


def boss_drops(drop):
    if drop in REAL_TOKEN:
        return '("%s", 100), ("%s", 100)' % (REAL_TOKEN[drop], drop)
    return '("%s", 100)' % drop


BSPRITE = {'shogunboss': 'mon_shogunwolf'}
for mid, name, f, drop, mult in BOSS:
    hp = int((1900 + 170 * (f - 10)) * mult * SCALE)
    atk = int((72 + 6 * (f - 10)) * SCALE)
    df = int((28 + 3 * (f - 10)) * SCALE)
    gold = 2500 + 400 * (f - 10)
    lines.append('    MonDef { id: "%s", name: "%s", hp: %d, atk: %d, def: %d, spd: %d, gold: %d, sprite: "%s", boss: true, undead: false,\n        drops: &[%s] },' % (
        mid, name, hp, atk, df, 8 + f % 3, gold, BSPRITE.get(mid, 'mon_' + mid), boss_drops(drop)))
open(os.path.join(OUT, 'mons.txt'), 'w', encoding='utf-8').write('\n'.join(lines) + '\n')
print('items', len(items), 'mons', len(lines), 'floors', sorted(SP))
print('first', {k: v for k, v in FIRST.items() if k in NORMAL_NEW})
