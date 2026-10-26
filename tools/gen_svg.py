# 仮のSVG素材を assets/svg/ に書き出す。差し替えるときは同名の assets/img/<name>.png を置けばそちらが優先される。
import os, random
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'assets', 'svg')
os.makedirs(OUT, exist_ok=True)
OL = '#3a2616'
W, H = 576, 432


def save(name, w, h, body, defs=''):
    s = f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}"><defs>{defs}</defs>{body}</svg>'
    with open(os.path.join(OUT, name + '.svg'), 'w', encoding='utf-8') as f:
        f.write(s)


def rect(x, y, w, h, fill, stroke=None, sw=1.5, rx=0):
    st = f' stroke="{stroke}" stroke-width="{sw}"' if stroke else ''
    return f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{rx}" fill="{fill}"{st}/>'


def circ(cx, cy, r, fill, stroke=None, sw=1.5):
    st = f' stroke="{stroke}" stroke-width="{sw}"' if stroke else ''
    return f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="{fill}"{st}/>'


def ell(cx, cy, rx, ry, fill, stroke=None, sw=1.5):
    st = f' stroke="{stroke}" stroke-width="{sw}"' if stroke else ''
    return f'<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{fill}"{st}/>'


def poly(pts, fill, stroke=None, sw=1.5):
    st = f' stroke="{stroke}" stroke-width="{sw}" stroke-linejoin="round"' if stroke else ''
    p = ' '.join(f'{x},{y}' for x, y in pts)
    return f'<polygon points="{p}" fill="{fill}"{st}/>'


def path(d, fill='none', stroke=None, sw=1.5):
    st = f' stroke="{stroke}" stroke-width="{sw}" stroke-linecap="round" stroke-linejoin="round"' if stroke else ''
    return f'<path d="{d}" fill="{fill}"{st}/>'


def person(cx, by, scale=1.0, skin='#f6d2ae', hair='#5a3a22', cloth='#3b6ea5', cloth2='#2a4f7c', hairstyle='short', extra=''):
    """by = 足元のy。chibi 立ち絵(高さ約 90*scale)"""
    g = f'<g transform="translate({cx},{by}) scale({scale})">'
    g += ell(0, 2, 20, 5, 'rgba(0,0,0,0.25)')
    g += rect(-9, -16, 7, 16, '#4a3322', OL, 1.5, 2) + rect(2, -16, 7, 16, '#4a3322', OL, 1.5, 2)
    g += path('M-16,-18 L-12,-48 Q0,-54 12,-48 L16,-18 Z', cloth, OL, 2)
    g += rect(-16, -24, 32, 6, cloth2, OL, 1.5, 2)
    g += rect(-22, -46, 8, 22, cloth, OL, 1.5, 3) + rect(14, -46, 8, 22, cloth, OL, 1.5, 3)
    g += circ(-18, -22, 4.5, skin, OL, 1.2) + circ(18, -22, 4.5, skin, OL, 1.2)
    if hairstyle == 'long':
        g += path('M-20,-58 Q-24,-30 -14,-30 L14,-30 Q24,-30 20,-58 Z', hair, OL, 2)
    g += circ(0, -64, 20, skin, OL, 2)
    if hairstyle != 'bald':
        g += path('M-21,-66 Q-22,-88 0,-86 Q22,-88 21,-66 Q14,-76 0,-74 Q-14,-76 -21,-66 Z', hair, OL, 2)
    else:
        g += path('M-19,-70 Q-20,-72 -14,-74 M19,-70 Q20,-72 14,-74', 'none', OL, 2)
    g += circ(-7, -62, 2.6, '#2a1a10') + circ(7, -62, 2.6, '#2a1a10')
    g += path('M-4,-53 Q0,-50 4,-53', 'none', OL, 1.5)
    g += circ(-12, -57, 3, 'rgba(255,120,120,0.35)') + circ(12, -57, 3, 'rgba(255,120,120,0.35)')
    g += extra
    g += '</g>'
    return g


# ---------------- タイル(36x36) ----------------
def tile_floor():
    b = rect(0, 0, 36, 36, '#b79a6c')
    rnd = random.Random(1)
    for _ in range(7):
        b += ell(rnd.randint(2, 30), rnd.randint(2, 30), rnd.randint(2, 4), rnd.randint(1, 3), '#a5885c')
    b += rect(0, 0, 36, 36, 'none', '#8f7448', 1)
    save('tile_floor', 36, 36, b)


def tile_rock():
    b = rect(0, 0, 36, 36, '#5a4634')
    b += path('M0,12 H36 M0,24 H36 M12,0 V12 M24,12 V24 M8,24 V36 M28,24 V36 M30,0 V12', 'none', '#3f3125', 2)
    b += rect(2, 2, 32, 8, 'rgba(255,255,255,0.07)')
    b += rect(0, 0, 36, 36, 'none', '#2e231a', 1.5)
    save('tile_rock', 36, 36, b)


def tile_bedrock():
    b = rect(0, 0, 36, 36, '#2b2530')
    b += path('M0,0 L36,36 M36,0 L0,36', 'none', '#3b3340', 2)
    save('tile_bedrock', 36, 36, b)


def tile_barrier():
    b = rect(0, 0, 36, 36, '#5a4634')
    b += path('M0,12 H36 M0,24 H36 M12,0 V12 M24,12 V24', 'none', '#3f3125', 2)
    for x in (6, 18, 30):
        b += circ(x, 18, 3.2, '#e0453a', '#7a1a14', 1)
    b += rect(0, 0, 36, 36, 'none', '#2e231a', 1.5)
    save('tile_barrier', 36, 36, b)


def tile_stairs_up():
    b = rect(0, 0, 36, 36, '#b79a6c')
    cols = ['#e8dcc0', '#d6c9a8', '#c4b793', '#b2a47e', '#a09169']
    for i in range(5):
        b += rect(4, 4 + i * 6, 28, 5, cols[i], '#6d5b3a', 1)
    b += poly([(18, 3), (28, 12), (8, 12)], '#6fbf4a', '#2f6a1c', 1.2)
    save('tile_stairs_up', 36, 36, b)


def tile_stairs_down():
    b = rect(0, 0, 36, 36, '#3b2f45')
    cols = ['#d9d2e6', '#c0b8d2', '#a79fbc', '#8e86a4', '#756d8b']
    for i in range(5):
        b += rect(4 + i * 2, 4 + i * 6, 28 - i * 4, 5, cols[i], '#2a2238', 1)
    b += poly([(18, 33), (28, 24), (8, 24)], '#ffd24a', '#8a6a10', 1.2)
    save('tile_stairs_down', 36, 36, b)


def tile_portal():
    b = rect(0, 0, 36, 36, '#b79a6c')
    d = '<radialGradient id="pg"><stop offset="0" stop-color="#2a0a3a"/><stop offset="0.6" stop-color="#7a2aa8"/><stop offset="1" stop-color="#d98cff"/></radialGradient>'
    b += ell(18, 18, 16, 16, 'url(#pg)', '#3b0a52', 2)
    b += path('M18,8 Q26,14 18,18 Q10,22 18,28', 'none', '#f3d8ff', 2)
    save('tile_portal', 36, 36, b, d)


# ---------------- 部屋 ----------------
def room_base():
    return rect(0, 0, 36, 36, '#b79a6c')


def room_trough():
    b = room_base()
    b += rect(3, 9, 30, 20, '#8a5a2b', OL, 2, 3)
    b += rect(6, 12, 24, 14, '#d9b35a', OL, 1.5, 2)
    for x in (10, 17, 24):
        b += ell(x, 19, 3, 2, '#9a7a2a')
    b += rect(2, 7, 4, 24, '#6a421c', OL, 1.2) + rect(30, 7, 4, 24, '#6a421c', OL, 1.2)
    save('room_エサおけ', 36, 36, b)


def room_dump():
    b = room_base()
    b += ell(18, 22, 14, 9, '#4f5a3a', OL, 2)
    b += circ(12, 18, 5, '#9aa05a', OL, 1.2) + circ(22, 16, 4, '#b8b36a', OL, 1.2)
    b += rect(15, 6, 8, 10, '#8a8f98', OL, 1.5, 2) + rect(14, 4, 10, 4, '#a5aab2', OL, 1.5, 2)
    b += path('M24,26 L30,20 M10,26 L6,22', 'none', '#e8e0cc', 2.5)
    save('room_ゴミすてば', 36, 36, b)


def room_bed():
    b = room_base()
    b += rect(5, 6, 26, 26, '#b2813a', OL, 2, 4)
    b += path('M7,10 H29 M7,15 H29 M7,20 H29 M7,25 H29', 'none', '#d9b35a', 2)
    b += rect(9, 7, 18, 7, '#f1ead8', OL, 1.5, 3)
    b += rect(5, 17, 26, 15, '#c85a4a', OL, 1.5, 3)
    save('room_わらのベッド', 36, 36, b)


def room_chest(open_=False):
    b = room_base()
    b += rect(4, 14, 28, 17, '#8a8f98', OL, 2, 3)
    if open_:
        b += rect(4, 6, 28, 9, '#6f747d', OL, 2, 3)
        b += rect(6, 15, 24, 6, '#ffe27a', OL, 1)
    else:
        b += rect(4, 8, 28, 9, '#a5aab2', OL, 2, 3)
        b += rect(15, 14, 6, 7, '#ffd24a', OL, 1.2, 1)
    b += path('M12,8 V31 M24,8 V31', 'none', '#5a5f68', 2)
    save('room_てつのたからばこ' + ('_open' if open_ else ''), 36, 36, b)


def hero_map():
    b = person(18, 34, 0.40, hair='#b5522b', cloth='#3b8f57', cloth2='#2a6a3f')
    save('hero_map', 36, 36, b)


def portraits():
    b = circ(32, 32, 30, '#cfe3b0', OL, 2) + person(32, 62, 0.62, hair='#b5522b', cloth='#3b8f57', cloth2='#2a6a3f')
    save('portrait_a', 64, 64, b)
    s = circ(32, 32, 30, '#c9d8f0', OL, 2)
    s += ell(32, 40, 20, 17, '#6fd16f', '#2a7a2a', 2) + path('M14,40 Q32,10 50,40 Z', '#8bea8b', '#2a7a2a', 2)
    s += circ(25, 34, 3.2, '#13301a') + circ(39, 34, 3.2, '#13301a') + path('M28,42 Q32,46 36,42', 'none', '#13301a', 2)
    s += ell(24, 28, 4, 2.5, 'rgba(255,255,255,0.6)')
    save('portrait_j', 64, 64, s)
    b = circ(32, 32, 30, '#f0d3e6', OL, 2) + person(32, 62, 0.62, hair='#e0a830', cloth='#c8508a', cloth2='#9a3868', hairstyle='long')
    save('portrait_m', 64, 64, b)


# ---------------- モンスター(192x192) ----------------
def eyes(cx, cy, gap, r=6, col='#2a1a10'):
    return (circ(cx - gap, cy, r, 'white', OL, 2) + circ(cx + gap, cy, r, 'white', OL, 2)
            + circ(cx - gap + 1, cy + 1, r * 0.55, col) + circ(cx + gap + 1, cy + 1, r * 0.55, col))


def mon_bat():
    b = ell(96, 176, 40, 8, 'rgba(0,0,0,0.25)')
    b += path('M96,90 Q40,40 10,80 Q36,76 44,100 Q58,84 74,112 Z', '#3b2a5a', OL, 3)
    b += path('M96,90 Q152,40 182,80 Q156,76 148,100 Q134,84 118,112 Z', '#3b2a5a', OL, 3)
    b += ell(96, 112, 36, 40, '#2a2040', OL, 3) + ell(96, 124, 24, 26, '#4a3a6a')
    b += poly([(72, 80), (66, 52), (88, 72)], '#2a2040', OL, 3) + poly([(120, 80), (126, 52), (104, 72)], '#2a2040', OL, 3)
    b += eyes(96, 100, 14, 8, '#c02020')
    b += poly([(86, 118), (96, 140), (106, 118)], '#f0a830', OL, 2.5)
    b += path('M86,150 L82,168 M106,150 L110,168', 'none', '#f0a830', 4)
    save('mon_bat', 192, 192, b)


def mon_pig():
    b = ell(96, 176, 62, 9, 'rgba(0,0,0,0.25)')
    b += ell(96, 120, 64, 48, '#b86a4a', OL, 3)
    b += ell(96, 100, 52, 40, '#c97d58')
    for x in (50, 142):
        b += poly([(x - 8, 154), (x + 8, 154), (x + 6, 176), (x - 6, 176)], '#8a4a30', OL, 2.5)
    b += ell(96, 92, 40, 34, '#d99a74', OL, 3)
    b += ell(96, 108, 20, 15, '#f0b8a0', OL, 2.5) + circ(89, 108, 3.5, '#6a2a20') + circ(103, 108, 3.5, '#6a2a20')
    b += poly([(66, 70), (58, 44), (82, 62)], '#b86a4a', OL, 3) + poly([(126, 70), (134, 44), (110, 62)], '#b86a4a', OL, 3)
    b += eyes(96, 86, 18, 6)
    b += path('M76,120 Q70,134 76,138 M116,120 Q122,134 116,138', 'none', '#fff', 5)
    b += path('M70,76 Q96,66 122,76', 'none', '#6a2a20', 4)
    save('mon_pig', 192, 192, b)


def mon_cat():
    b = ell(96, 178, 44, 8, 'rgba(0,0,0,0.25)')
    b += rect(70, 108, 52, 62, '#7a5a8a', OL, 3, 10)
    b += rect(60, 110, 16, 44, '#c9a27a', OL, 3, 7) + rect(116, 110, 16, 44, '#c9a27a', OL, 3, 7)
    b += rect(74, 160, 16, 18, '#4a3a2a', OL, 3, 4) + rect(102, 160, 16, 18, '#4a3a2a', OL, 3, 4)
    b += circ(96, 82, 36, '#c9a27a', OL, 3)
    b += poly([(66, 62), (62, 30), (86, 52)], '#c9a27a', OL, 3) + poly([(126, 62), (130, 30), (106, 52)], '#c9a27a', OL, 3)
    b += poly([(68, 56), (66, 40), (80, 52)], '#f0a8b0') + poly([(124, 56), (126, 40), (112, 52)], '#f0a8b0')
    b += eyes(96, 80, 15, 7, '#2a7a2a')
    b += poly([(92, 94), (100, 94), (96, 100)], '#e07a8a', OL, 1.5)
    b += path('M96,100 V106 M86,106 Q96,112 106,106', 'none', OL, 2)
    b += path('M60,92 H78 M60,100 H78 M132,92 H114 M132,100 H114', 'none', OL, 1.5)
    b += path('M120,150 Q160,140 150,100', 'none', '#c9a27a', 10) + path('M120,150 Q160,140 150,100', 'none', OL, 2)
    b += rect(122, 128, 8, 26, '#e8dcc0', OL, 2, 3)
    save('mon_cat', 192, 192, b)


def mon_gob():
    b = ell(96, 178, 46, 8, 'rgba(0,0,0,0.25)')
    b += rect(70, 104, 52, 60, '#6a4a2a', OL, 3, 10)
    b += rect(76, 160, 16, 18, '#3a2a1a', OL, 3, 4) + rect(100, 160, 16, 18, '#3a2a1a', OL, 3, 4)
    b += rect(56, 108, 16, 40, '#7aa84a', OL, 3, 7) + rect(120, 108, 16, 40, '#7aa84a', OL, 3, 7)
    b += rect(132, 60, 14, 90, '#8a5a2b', OL, 3, 5) + ell(139, 54, 20, 18, '#9a6a35', OL, 3)
    b += circ(96, 80, 34, '#7aa84a', OL, 3)
    b += poly([(64, 78), (30, 66), (66, 94)], '#7aa84a', OL, 3) + poly([(128, 78), (162, 66), (126, 94)], '#7aa84a', OL, 3)
    b += eyes(96, 76, 14, 7, '#a02020')
    b += path('M82,98 Q96,108 110,98', 'none', OL, 3) + poly([(86, 98), (90, 106), (94, 99)], 'white', OL, 1.5) + poly([(106, 98), (102, 106), (98, 99)], 'white', OL, 1.5)
    b += path('M78,62 Q96,54 114,62', 'none', '#3a5a1a', 4)
    save('mon_gob', 192, 192, b)


def mon_slime():
    b = ell(96, 168, 58, 9, 'rgba(0,0,0,0.25)')
    b += path('M36,160 Q28,100 60,74 Q96,40 132,74 Q164,100 156,160 Q96,176 36,160 Z', '#5ccf6a', '#1f6a2c', 4)
    b += path('M52,150 Q44,106 68,84 Q96,58 124,84 Q148,106 140,150 Q96,162 52,150 Z', '#7fe68a')
    b += ell(76, 92, 14, 8, 'rgba(255,255,255,0.65)')
    b += circ(76, 118, 9, '#10301a') + circ(116, 118, 9, '#10301a') + circ(73, 114, 3, 'white') + circ(113, 114, 3, 'white')
    b += path('M84,138 Q96,148 108,138', 'none', '#10301a', 4)
    save('mon_slime', 192, 192, b)


def mon_boss():
    b = ell(96, 178, 70, 10, 'rgba(0,0,0,0.3)')
    b += path('M30,110 Q20,50 80,36 Q96,30 112,36 Q172,50 162,110 Q158,170 96,176 Q34,170 30,110 Z', '#c8302a', OL, 4)
    b += path('M44,108 Q36,62 84,48 Q96,44 108,48 Q156,62 148,108 Q144,156 96,162 Q48,156 44,108 Z', '#e2554a')
    b += ell(70, 68, 14, 8, 'rgba(255,255,255,0.5)')
    b += path('M96,36 Q96,16 112,8', 'none', '#4a2a0a', 6) + path('M112,8 Q132,2 138,22 Q122,28 112,8 Z', '#5aa63a', OL, 3)
    b += path('M80,38 Q70,22 56,26 Q66,40 80,44 Z', '#5aa63a', OL, 3)
    b += path('M30,150 Q-6,150 8,176 Q30,190 60,172', 'none', '#6aa84a', 12) + path('M30,150 Q-6,150 8,176 Q30,190 60,172', 'none', OL, 2)
    b += eyes(96, 94, 26, 10, '#6a1a1a')
    b += path('M64,128 Q96,150 128,128', 'none', OL, 5)
    for x in (74, 88, 104, 118):
        b += poly([(x - 4, 131), (x + 4, 131), (x, 142)], 'white', OL, 1.5)
    b += path('M62,70 L84,80 M130,80 L152,70', 'none', OL, 5)
    save('mon_boss', 192, 192, b)


# ---------------- 場面(576x432) ----------------
def sky(top='#8fd0f0', bot='#e8f6ff'):
    d = f'<linearGradient id="sk" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{top}"/><stop offset="1" stop-color="{bot}"/></linearGradient>'
    return d, rect(0, 0, W, H, 'url(#sk)')


def tree(x, y, s=1.0):
    return (f'<g transform="translate({x},{y}) scale({s})">' + rect(-5, -30, 10, 34, '#6a4a2a', OL, 2)
            + circ(0, -48, 26, '#4fa84a', OL, 2.5) + circ(-14, -36, 17, '#5cba55', OL, 2) + circ(15, -37, 17, '#45963f', OL, 2) + '</g>')


def house(x, y, w, h, roof, wall, door_col='#7a4a22', sign=''):
    b = rect(x, y, w, h, wall, OL, 3)
    b += poly([(x - 12, y + 2), (x + w / 2, y - 52), (x + w + 12, y + 2)], roof, OL, 3)
    b += rect(x + w / 2 - 14, y + h - 46, 28, 46, door_col, OL, 2.5, 3) + circ(x + w / 2 + 8, y + h - 22, 2.5, '#ffd24a')
    b += rect(x + 12, y + 18, 26, 24, '#cfe9f5', OL, 2) + path(f'M{x+25},{y+18} V{y+42} M{x+12},{y+30} H{x+38}', 'none', OL, 1.5)
    b += rect(x + w - 38, y + 18, 26, 24, '#cfe9f5', OL, 2) + path(f'M{x+w-25},{y+18} V{y+42} M{x+w-38},{y+30} H{x+w-12}', 'none', OL, 1.5)
    b += sign
    return b



# ---------------- 2F の部屋・魔物 ----------------
def room_tavern():
    b = room_base()
    b += rect(3, 14, 30, 18, '#8a5a2b', OL, 2, 3) + rect(3, 12, 30, 5, '#b2813a', OL, 2, 2)
    b += rect(8, 4, 8, 10, '#e8c850', OL, 1.5, 2) + rect(9, 3, 6, 3, '#fff6d8', OL, 1)
    b += rect(20, 6, 8, 8, '#c8502a', OL, 1.5, 3)
    b += path('M4,22 H32 M4,27 H32', 'none', '#6a421c', 1.5)
    save('room_さかば', 36, 36, b)


def room_altar():
    b = room_base()
    b += rect(6, 18, 24, 12, '#8a8f98', OL, 2, 2) + rect(9, 12, 18, 7, '#a5aab2', OL, 2, 2)
    b += rect(16, 2, 4, 12, '#e8e8f0', OL, 1.5) + rect(12, 6, 12, 3, '#e8e8f0', OL, 1.5)
    b += circ(10, 28, 2, '#ffb84a') + circ(26, 28, 2, '#ffb84a')
    save('room_さいだん', 36, 36, b)


def room_field():
    b = rect(0, 0, 36, 36, '#8a6a3a')
    for i in range(4):
        b += path(f'M2,{6+i*8} H34', 'none', '#6a4a22', 3)
    for i in range(3):
        for j in range(3):
            b += circ(7 + i * 11, 8 + j * 10, 3.2, ['#4aa84a', '#e08a2a', '#6ac05a'][(i + j) % 3], OL, 0.8)
    b += rect(0, 0, 36, 36, 'none', '#4a3216', 1.5)
    save('room_はたけ', 36, 36, b)


def mon_wan():
    b = ell(96, 176, 56, 8, 'rgba(0,0,0,0.25)')
    b += rect(60, 100, 72, 64, '#8a6a4a', OL, 3, 14)
    b += rect(52, 118, 18, 46, '#b08a5a', OL, 3, 7) + rect(122, 118, 18, 46, '#b08a5a', OL, 3, 7)
    b += rect(66, 156, 20, 20, '#4a3a2a', OL, 3, 4) + rect(106, 156, 20, 20, '#4a3a2a', OL, 3, 4)
    b += circ(96, 78, 40, '#b08a5a', OL, 3)
    b += poly([(58, 70), (46, 36), (78, 58)], '#8a6a4a', OL, 3) + poly([(134, 70), (146, 36), (114, 58)], '#8a6a4a', OL, 3)
    b += ell(96, 92, 22, 16, '#f0d8b0', OL, 2.5) + ell(96, 86, 8, 6, '#3a2a1a')
    b += eyes(96, 68, 17, 7, '#2a1a10')
    b += path('M88,98 Q96,106 104,98', 'none', OL, 2.5) + ell(96, 108, 6, 5, '#e07a8a', OL, 1.5)
    b += rect(136, 60, 10, 80, '#cfd6df', OL, 3, 3) + rect(132, 136, 18, 8, '#8a5a2b', OL, 2.5)
    save('mon_wan', 192, 192, b)


def mon_gobl():
    b = ell(96, 178, 56, 9, 'rgba(0,0,0,0.3)')
    b += rect(62, 96, 68, 70, '#7a4a2a', OL, 3, 12)
    b += rect(76, 160, 18, 18, '#3a2a1a', OL, 3, 4) + rect(100, 160, 18, 18, '#3a2a1a', OL, 3, 4)
    b += rect(44, 100, 20, 52, '#5a9a3a', OL, 3, 8) + rect(128, 100, 20, 52, '#5a9a3a', OL, 3, 8)
    b += circ(96, 72, 38, '#5a9a3a', OL, 3)
    b += path('M62,56 Q96,24 130,56 L126,64 Q96,44 66,64 Z', '#8a8f98', OL, 3) + poly([(96, 30), (90, 14), (102, 14)], '#8a8f98', OL, 2)
    b += poly([(60, 78), (24, 64), (62, 94)], '#5a9a3a', OL, 3) + poly([(132, 78), (168, 64), (130, 94)], '#5a9a3a', OL, 3)
    b += eyes(96, 74, 15, 7, '#a02020')
    b += path('M80,96 Q96,108 112,96', 'none', OL, 3) + poly([(84, 96), (88, 106), (92, 97)], 'white', OL, 1.5) + poly([(108, 96), (104, 106), (100, 97)], 'white', OL, 1.5)
    b += rect(134, 76, 14, 86, '#8a5a2b', OL, 3, 5) + ell(141, 70, 22, 18, '#9a6a35', OL, 3)
    save('mon_gobl', 192, 192, b)


def mon_ghost():
    b = ell(96, 176, 44, 7, 'rgba(0,0,0,0.15)')
    b += path('M40,150 Q30,60 96,40 Q162,60 152,150 Q140,134 128,152 Q112,134 96,154 Q80,134 64,152 Q52,134 40,150 Z', '#e8eefc', '#6a7aa8', 4)
    b += path('M40,100 Q10,110 24,134 Q36,128 44,116 Z', '#e8eefc', '#6a7aa8', 3) + path('M152,100 Q182,110 168,134 Q156,128 148,116 Z', '#e8eefc', '#6a7aa8', 3)
    b += ell(70, 92, 10, 14, '#2a3050') + ell(122, 92, 10, 14, '#2a3050')
    b += path('M66,120 Q96,150 126,120 Q96,132 66,120 Z', '#2a3050', OL, 2)
    b += circ(58, 112, 6, 'rgba(255,140,160,0.5)') + circ(134, 112, 6, 'rgba(255,140,160,0.5)')
    save('mon_ghost', 192, 192, b)


def mon_potato():
    b = ell(96, 176, 54, 8, 'rgba(0,0,0,0.25)')
    b += ell(96, 114, 56, 52, '#c8a050', OL, 4) + ell(96, 104, 44, 40, '#dcb866')
    b += path('M92,64 Q88,40 70,34 M96,64 Q104,36 124,32', 'none', '#4a8a2a', 6)
    b += circ(72, 32, 8, '#6aa83a', OL, 2) + circ(126, 30, 8, '#6aa83a', OL, 2)
    b += eyes(96, 104, 20, 8, '#2a1a10')
    b += path('M78,130 Q96,146 114,130', 'none', OL, 4)
    b += circ(78, 86, 3, '#a07a30') + circ(120, 122, 3, '#a07a30') + circ(66, 124, 3, '#a07a30')
    b += rect(130, 120, 14, 30, '#4a4a52', OL, 3, 4) + poly([(137, 120), (130, 106), (144, 106)], '#e04a2a', OL, 2)
    save('mon_potato', 192, 192, b)


def mon_lion():
    b = ell(96, 180, 74, 10, 'rgba(0,0,0,0.3)')
    b += rect(52, 100, 88, 74, '#c89a4a', OL, 4, 16)
    b += rect(44, 120, 22, 54, '#d8aa5a', OL, 3.5, 9) + rect(126, 120, 22, 54, '#d8aa5a', OL, 3.5, 9)
    for x in (44, 126):
        b += path(f'M{x+4},170 l4,10 M{x+11},170 l0,10 M{x+18},170 l-4,10', 'none', '#f4f4f4', 3)
    b += circ(96, 74, 62, '#a05a1a', OL, 4)
    for a in range(0, 360, 30):
        import math
        x = 96 + 62 * math.cos(math.radians(a))
        y = 74 + 62 * math.sin(math.radians(a))
        b += circ(round(x, 1), round(y, 1), 14, '#b86a22', OL, 2.5)
    b += circ(96, 78, 42, '#e8b860', OL, 4)
    b += poly([(66, 48), (60, 22), (84, 42)], '#e8b860', OL, 3) + poly([(126, 48), (132, 22), (108, 42)], '#e8b860', OL, 3)
    b += eyes(96, 72, 17, 8, '#a02020')
    b += ell(96, 92, 16, 11, '#f4d8a8', OL, 2.5) + poly([(90, 86), (102, 86), (96, 94)], '#6a2a20', OL, 1.5)
    b += path('M96,94 V102 M82,100 Q96,112 110,100', 'none', OL, 3) + poly([(86, 104), (90, 112), (94, 104)], 'white', OL, 1.5) + poly([(106, 104), (102, 112), (98, 104)], 'white', OL, 1.5)
    b += path('M60,66 L84,74 M132,66 L108,74', 'none', OL, 5)
    save('mon_lion', 192, 192, b)


def scene_town():
    d, b = sky()
    b += poly([(0, 250), (90, 170), (190, 240), (300, 150), (420, 240), (500, 180), (576, 240), (576, 300), (0, 300)], '#9ab88a', OL, 2.5)
    b += rect(0, 270, W, 162, '#7cc15a')
    b += path('M0,372 Q200,320 300,356 Q420,396 576,340 L576,432 L0,432 Z', '#d9c28a', OL, 2.5)
    b += path('M18,232 Q30,176 88,176 Q146,176 158,232 Z', '#5a5048', OL, 3) + path('M40,232 Q48,200 88,200 Q128,200 136,232 Z', '#1a1418', OL, 2.5)
    b += house(180, 186, 110, 90, '#c8502a', '#f3e2bc', sign=rect(214, 168, 42, 14, '#8a5a2b', OL, 2, 2) + path('M226,172 L246,180 M246,172 L226,180', 'none', '#e8e8f0', 3))
    b += house(318, 190, 110, 86, '#5a64b8', '#efe6d0', sign=rect(352, 172, 42, 14, '#8a5a2b', OL, 2, 2) + ell(373, 179, 7, 5, '#c070e0', OL, 1.5))
    b += house(450, 206, 100, 80, '#4a9a5a', '#f1e5c4', sign=rect(480, 188, 40, 14, '#8a5a2b', OL, 2, 2) + rect(493, 190, 14, 10, '#c58a4a', OL, 1.5))
    b += house(40, 262, 120, 90, '#8a5a3a', '#f6ecd0', door_col='#5a3a1a')
    b += rect(70, 244, 10, 24, '#7a6a5a', OL, 2)
    b += ell(300, 372, 44, 14, '#a9b3bd', OL, 2.5) + ell(300, 368, 36, 10, '#7fc8f0', OL, 2) + rect(293, 340, 14, 26, '#a9b3bd', OL, 2)
    b += path('M300,340 Q284,326 280,340 M300,340 Q316,326 320,340', 'none', '#bfe8ff', 3)
    b += tree(560, 300, 0.9) + tree(20, 330, 0.8) + tree(168, 296, 0.7)
    b += person(236, 292, 0.6, hair='#c05a3a', cloth='#c85a6a', cloth2='#9a3a4a', hairstyle='long')
    b += person(373, 292, 0.6, skin='#f0cfae', hair='#e8e8e8', cloth='#6a4aa8', cloth2='#4a3278', hairstyle='bald',
                extra=path('M-12,-48 Q0,-30 12,-48 L8,-22 L-8,-22 Z', '#f4f4f4', OL, 1.5))
    b += person(500, 300, 0.6, hair='#6a4a2a', cloth='#4a8a4a', cloth2='#2f6a2f', extra=rect(-18, -90, 36, 12, '#c8502a', OL, 2, 3))
    b += person(262, 410, 0.55, hair='#e0a830', cloth='#e08a4a', cloth2='#b05a2a', hairstyle='long')
    b += person(150, 400, 0.6, hair='#dcdcdc', cloth='#8a5a2a', cloth2='#5a3a1a', hairstyle='bald')
    save('scene_town', W, H, b, d)


def interior(name, wall, floor, deco, keeper):
    b = rect(0, 0, W, H, wall) + rect(0, 300, W, 132, floor)
    b += rect(0, 296, W, 8, '#5a3a1a', OL, 2)
    b += path('M0,330 H576 M0,372 H576 M0,408 H576', 'none', 'rgba(0,0,0,0.15)', 2)
    b += deco + keeper
    b += rect(30, 322, W - 60, 76, '#a5703a', OL, 3.5, 4) + rect(30, 322, W - 60, 12, '#c8905a', OL, 3, 4)
    save(name, W, H, b)


def scene_armor():
    deco = ''
    for x in (60, 190, 330, 460):
        deco += rect(x, 70, 70, 120, '#7a5230', OL, 3, 4)
        deco += path(f'M{x+35},84 V160', 'none', '#cfd6df', 8) + path(f'M{x+35},84 V160', 'none', OL, 1.5)
        deco += poly([(x + 28, 84), (x + 35, 66), (x + 42, 84)], '#cfd6df', OL, 2) + rect(x + 22, 138, 26, 6, '#8a5a2b', OL, 2)
    deco += circ(500, 240, 38, '#8a8f98', OL, 3) + circ(500, 240, 24, '#c8502a', OL, 2.5) + rect(150, 216, 60, 70, '#a5aab2', OL, 3, 8)
    keeper = person(288, 330, 1.45, hair='#c05a3a', cloth='#c85a6a', cloth2='#9a3a4a', hairstyle='long')
    interior('scene_armor', '#d9c9a8', '#8a5a3a', deco, keeper)


def scene_magic():
    deco = rect(50, 60, 480, 20, '#5a3a1a', OL, 2)
    cols = ['#e04a6a', '#4a8ae0', '#6ad06a', '#e0c04a', '#b04ae0']
    for i, x in enumerate(range(80, 500, 70)):
        deco += rect(x, 100, 40, 90, '#7a5230', OL, 3, 3)
        deco += ell(x + 20, 156, 14, 18, cols[i % 5], OL, 2.5) + rect(x + 14, 132, 12, 12, '#d9d2c0', OL, 2)
    deco += circ(480, 250, 34, '#3a2a6a', OL, 3)
    deco += path('M480,222 L488,244 L510,246 L492,260 L498,282 L480,270 L462,282 L468,260 L450,246 L472,244 Z', '#ffd24a', OL, 1.5)
    keeper = person(288, 332, 1.45, skin='#f0cfae', hair='#eeeeee', cloth='#6a4aa8', cloth2='#4a3278', hairstyle='bald',
                    extra=path('M-14,-50 Q0,-26 14,-50 L10,-20 L-10,-20 Z', '#f4f4f4', OL, 1.5))
    interior('scene_magic', '#bfb4d9', '#5a3a6a', deco, keeper)


def scene_furn():
    deco = ''
    for x in (50, 200, 350):
        deco += rect(x, 150, 90, 70, '#8a5a2b', OL, 3, 4) + rect(x + 8, 160, 74, 50, '#d9b35a', OL, 2, 3)
    deco += rect(60, 70, 120, 60, '#b2813a', OL, 3, 4) + path('M72,80 H168 M72,100 H168', 'none', '#d9b35a', 3)
    deco += rect(450, 90, 80, 130, '#8a8f98', OL, 3, 5) + rect(462, 104, 56, 24, '#ffe27a', OL, 2)
    keeper = person(288, 332, 1.45, hair='#6a4a2a', cloth='#4a8a4a', cloth2='#2f6a2f', extra=rect(-19, -92, 38, 12, '#c8502a', OL, 2, 3))
    interior('scene_furn', '#e4d4b0', '#7a5230', deco, keeper)


def scene_plaza():
    d, b = sky('#9fdcf8', '#f4fbff')
    b += rect(0, 250, W, 182, '#8ecb62') + rect(0, 330, W, 102, '#d9c28a')
    for x in range(20, W, 90):
        b += tree(x, 256, 0.8)
    b += ell(288, 382, 160, 34, '#a9b3bd', OL, 3) + ell(288, 376, 140, 28, '#7fc8f0', OL, 2.5) + rect(274, 320, 28, 58, '#a9b3bd', OL, 3)
    b += path('M288,322 Q250,290 240,330 M288,322 Q326,290 336,330', 'none', '#bfe8ff', 5)
    b += rect(60, 250, 150, 90, '#e8d4a0', OL, 3) + poly([(50, 252), (135, 214), (220, 252)], '#e04a4a', OL, 3)
    for i, c in enumerate(['#4aa84a', '#e08a2a', '#e0c04a', '#6ac05a']):
        b += circ(84 + i * 34, 300, 13, c, OL, 2)
    b += person(440, 372, 1.15, hair='#e0a830', cloth='#e08a4a', cloth2='#b05a2a', hairstyle='long')
    save('scene_plaza', W, H, b, d)


def scene_mayor():
    deco = rect(60, 60, 110, 150, '#7a5230', OL, 3, 4) + rect(70, 72, 90, 126, '#cfe9f5', OL, 2)
    deco += rect(420, 80, 110, 100, '#f4ecd0', OL, 3) + path('M432,100 H518 M432,118 H518 M432,136 H500', 'none', '#5a3a1a', 3)
    deco += rect(250, 70, 76, 70, '#c8502a', OL, 3, 4) + circ(288, 105, 20, '#ffd24a', OL, 2.5)
    keeper = person(288, 332, 1.45, skin='#f0cfae', hair='#dcdcdc', cloth='#8a5a2a', cloth2='#5a3a1a', hairstyle='bald',
                    extra=path('M-12,-50 Q0,-20 12,-50 L8,-18 L-8,-18 Z', '#f4f4f4', OL, 1.5))
    interior('scene_mayor', '#d8c8a0', '#6a4a2a', deco, keeper)


def scene_home():
    deco = rect(60, 60, 130, 140, '#7a5230', OL, 3, 4) + rect(72, 72, 106, 116, '#8fc8f0', OL, 2) + circ(150, 100, 14, '#ffe27a', OL, 2)
    deco += rect(380, 150, 150, 120, '#b2813a', OL, 3, 6) + rect(388, 158, 134, 60, '#f1ead8', OL, 2, 6) + rect(388, 190, 134, 76, '#c85a4a', OL, 2, 6)
    deco += rect(250, 90, 60, 50, '#e8d4a0', OL, 3) + circ(280, 115, 14, '#c85a4a', OL, 2)
    deco += path('M230,170 L330,262', 'none', '#8a5a2b', 9) + path('M230,170 L330,262', 'none', OL, 1.5)
    deco += poly([(330, 240), (352, 262), (338, 292), (312, 270)], '#e8c850', OL, 2.5)
    b = rect(0, 0, W, H, '#e4d6b6') + rect(0, 300, W, 132, '#8a5a3a') + rect(0, 296, W, 8, '#5a3a1a', OL, 2) + deco
    b += rect(180, 330, 220, 70, '#a5703a', OL, 3.5, 4) + rect(180, 330, 220, 12, '#c8905a', OL, 3, 4)
    b += ell(290, 346, 40, 12, '#f4ecd0', OL, 2) + rect(270, 332, 40, 10, '#d9b35a', OL, 2, 4)
    save('scene_home', W, H, b)


def bg_battle():
    d = '<linearGradient id="bb" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#2a2218"/><stop offset="1" stop-color="#6a5232"/></linearGradient>'
    b = rect(0, 0, W, H, 'url(#bb)')
    b += poly([(0, 0), (150, 120), (150, 300), (0, H)], '#4a3a28', OL, 3) + poly([(W, 0), (W - 150, 120), (W - 150, 300), (W, H)], '#4a3a28', OL, 3)
    b += rect(150, 120, W - 300, 180, '#1a140e', OL, 3)
    b += poly([(150, 300), (W - 150, 300), (W, H), (0, H)], '#b79a6c', OL, 3)
    for i in range(1, 6):
        t = (i / 6) ** 1.6
        y = 300 + (H - 300) * t
        b += path(f'M{150 - 150*t},{y} H{W - 150 + 150*t}', 'none', '#8f7448', 2)
    for x in (30, 90, W - 30, W - 90):
        b += rect(x - 4, 150, 8, 30, '#6a4a2a', OL, 1.5) + ell(x, 140, 8, 10, '#ffb84a', OL, 1.5)
    save('bg_battle', W, H, b, d)


def scene_title():
    d, b = sky('#6fb8e8', '#fdf3d0')
    b += poly([(0, 330), (120, 230), (260, 320), (400, 210), (576, 330), (576, 432), (0, 432)], '#6aa84f', OL, 3)
    b += path('M0,380 Q288,330 576,380 L576,432 L0,432 Z', '#8a5a2b', OL, 3)
    b += path('M288,392 L340,250', 'none', '#8a5a2b', 14) + path('M288,392 L340,250', 'none', OL, 2)
    b += poly([(340, 250), (372, 262), (352, 300), (322, 288)], '#e8c850', OL, 3)
    save('scene_title', W, H, b, d)


for fn in [tile_floor, tile_rock, tile_bedrock, tile_barrier, tile_stairs_up, tile_stairs_down, tile_portal,
           room_trough, room_dump, room_bed, lambda: room_chest(False), lambda: room_chest(True),
           room_tavern, room_altar, room_field, mon_wan, mon_gobl, mon_ghost, mon_potato, mon_lion,
           hero_map, portraits, mon_bat, mon_pig, mon_cat, mon_gob, mon_slime, mon_boss,
           scene_town, scene_armor, scene_magic, scene_furn, scene_plaza, scene_mayor, scene_home, bg_battle, scene_title]:
    fn()
print('ok', len(os.listdir(OUT)))
