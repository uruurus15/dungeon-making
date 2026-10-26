# qwen-image(ComfyUI)で、ゲームの絵を一通り作る。
#   ComfyUI の venv の python で実行する:
#   C:\Users\hamuh\ComfyUI\.venv\Scripts\python.exe tools\gen_art.py [--only mon_bat,scene_town] [--redo] [--post-only]
# 1) 生成した元画像を art_work/raw/<name>.png に置く(あれば再生成しない)
# 2) 後処理(マゼンタ背景を透過・余白を切って正方形・リサイズ)して assets/img/<name>.png に書く
# assets/img/<name>.png があれば、ゲームは SVG より優先して読み込む。
import argparse
import json
import os
import subprocess
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.normpath(os.path.join(HERE, '..'))
RAW = os.path.join(ROOT, 'art_work', 'raw')
OUT = os.path.join(ROOT, 'assets', 'img')
QWEN = os.path.expanduser(r'~\.claude\skills\qwen-image\scripts\qwen_image.py')
os.makedirs(RAW, exist_ok=True)
os.makedirs(OUT, exist_ok=True)

STYLE_CHAR = ("Chibi fantasy RPG sprite, full body, centered, front-facing, standing pose. {d} "
              "Clean dark-brown lineart, soft cel shading with painterly texture detail, muted earthy medieval fantasy palette. "
              "Small simple eyes with a single tiny highlight, no big sparkling anime eyes. Isekai medieval fantasy storybook feeling. "
              "Solid flat pure magenta #FF00FF background, no ground shadow, no text.")
STYLE_OBJ = ("Game icon of a single object, three-quarter top-down view, centered. {d} "
             "Clean dark-brown lineart, soft cel shading with painterly texture detail, muted earthy medieval fantasy palette. "
             "Solid flat pure magenta #FF00FF background, no ground shadow, no text.")
STYLE_TILE = ("Seamless square game tile texture, strictly top-down view, filling the entire frame edge to edge, no border, no objects outside the tile. {d} "
              "Painterly hand-drawn detail, muted earthy medieval fantasy palette, even lighting.")
STYLE_SCENE = ("Illustrated in the same style as chibi RPG character sprites: clean dark-brown lineart, soft cel shading with light painterly texture, "
               "simplified cute shapes, muted earthy medieval fantasy palette, cozy storybook JRPG scene. Characters are chibi, 2.5 heads tall, with small plain eyes. "
               "No text, no UI, no frame. {d}")

STYLE_EMPTY = ("Illustrated with clean dark-brown lineart, soft cel shading with light painterly texture, simplified cute shapes, muted earthy medieval fantasy palette, cozy storybook JRPG background art. "
               "An uninhabited scene, background only. No text, no UI, no frame. {d}")

# name: (kind, prompt, aspect, out_size, seed)
#  kind: char / obj / tile / scene
ITEMS = {}


def add(name, kind, desc, size, aspect='1:1', seed=100):
    style = {'char': STYLE_CHAR, 'obj': STYLE_OBJ, 'tile': STYLE_TILE, 'scene': STYLE_SCENE, 'empty': STYLE_EMPTY}[kind]
    ITEMS[name] = (kind, style.format(d=desc), aspect, size, seed)


# --- 主人公たち(portrait_a / hero_map は char_hero から作る) ---
add('char_hero', 'char', "A young village boy adventurer named Allen with short messy auburn hair, a green hooded tunic with a brown leather belt, leather bracers and boots, a small satchel, carrying a short magic shovel with a golden blade over his shoulder. Calm friendly expression.", 320, seed=11)
add('portrait_j', 'char', "Jelly, a small round olive-green slime companion with a gentle smile and simple dot eyes, glossy highlights, wearing a tiny brown leather scarf, translucent jelly body.", 256, seed=21)
add('portrait_m', 'char', "Meimi, a young girl mage and childhood friend, long blonde braid, a simple linen dress with a brown hooded cloak, leather belt with small pouches, holding a wooden staff topped with a small blue crystal. Gentle calm expression.", 320, seed=31)

# --- 魔物 ---
add('mon_bat', 'char', "Crow-bat monster: a plump black-feathered crow with leathery bat wings, tattered feathers, sharp yellow beak, menacing yet cute.", 256, seed=41)
add('mon_pig', 'char', "Boar-pig monster: a stout wild boar with bristly russet fur, a pig snout and two small tusks, muscular and angry.", 256, seed=42)
add('mon_cat', 'char', "Werecat monster: a bipedal tabby cat-folk in a patched purple tunic holding a small rusty dagger, mischievous grin.", 256, seed=43)
add('mon_gob', 'char', "Small goblin monster (Koburin) with pointed ears and green skin wearing a crude leather helmet, dragging an oversized wooden club.", 256, seed=44)
add('mon_slime', 'char', "Slime monster: a translucent olive-green jelly blob in a droplet shape, glossy highlights, simple dot eyes, cheerful.", 256, seed=45)
add('mon_boss', 'char', "Boss monster Hebiringo: a giant sinister red apple with a venomous green snake coiled around it and rising from its top, fangs, a leaf, fearsome and bulky.", 320, seed=46)
add('mon_wan', 'char', "Dog-man soldier monster (Wanwan): a bipedal brown dog in leather armor holding a short sword and a small wooden shield.", 256, seed=47)
add('mon_gobl', 'char', "Goblin warrior monster: a large green-skinned goblin in an iron helmet and patched leather armor holding a spiked club, tusks.", 256, seed=48)
add('mon_ghost', 'char', "Laughing ghost monster: a pale ragged-sheet ghost with hollow dark eyes, a wide laughing mouth and a faint wispy tail.", 256, seed=49)
add('mon_potato', 'char', "Bomb potato monster: a living brown potato with green sprouts on top and a lit fuse stuck in it, angry eyes, like a bomb.", 256, seed=50)
add('mon_lion', 'char', "Boss monster Rion: a majestic anthropomorphic lion with a thick fiery orange mane, a scarred face, steel shoulder armor and massive claws, imposing.", 320, seed=51)


# --- 3F〜5F の魔物 ---
add('mon_hibuta', 'char', "Fire-boar monster (Hibuta): a stocky orange pig-beast with smoldering glowing embers on its back, snorting smoke, small tusks.", 256, seed=201)
add('mon_ningen', 'char', "Human-faced dinosaur monster (Ningen-saurus): a bipedal green lizard-dinosaur with a strangely human face, holding a stone hand-axe, wearing a rough leather chestplate.", 256, seed=202)
add('mon_firelime', 'char', "Fire slime monster: an orange-red glowing jelly blob with small flames flickering on top, simple dot eyes.", 256, seed=203)
add('mon_hotghost', 'char', "Hot ghost monster: a laughing ragged-sheet ghost glowing orange-red with floating ember wisps.", 256, seed=204)
add('mon_nekomata', 'char', "Nekomata monster: a two-tailed cat sorceress in a purple robe holding a small bow-shaped staff, mystical and sly.", 256, seed=205)
add('mon_waterlime', 'char', "Water slime monster: a translucent blue jelly blob with water droplets and ripples, simple dot eyes.", 256, seed=206)
add('mon_coldghost', 'char', "Cold ghost monster: a pale icy-blue laughing ghost with frost crystals on its sheet.", 256, seed=207)
add('mon_otter', 'char', "Otterman monster: a bipedal river otter warrior in blue scale armor holding a curved sword and a seashell shield.", 256, seed=208)
add('mon_carrot', 'char', "Missile carrot monster: a living orange carrot with a leafy green top and a rocket-like pointed tip, angry face.", 256, seed=209)
add('mon_crowcat', 'char', "Crow-cat monster: a black cat with crow wings and black feathers, yellow eyes, sly grin.", 256, seed=210)
add('mon_blackcat', 'char', "Black werecat monster: a bipedal black cat assassin in a dark leather vest holding twin daggers, wearing a silver ring.", 256, seed=211)
add('mon_wizdrill', 'char', "Drill wizard monster: a goblin-like mage with a spiral drill-shaped pointed hat and a tattered robe, holding a short bow.", 256, seed=212)
add('mon_blackwar', 'char', "Black warrior monster: a tall knight in black iron armor with a horned helmet and a heavy black longsword.", 256, seed=213)
add('mon_bone', 'char', "Bone fighter monster: a skeleton warrior with a rusty round shield and bronze boots, holding a curved sword.", 256, seed=214)
add('mon_dino', 'char', "Boss monster Honoo-saurus: a large red fire dinosaur with glowing lava cracks across its hide and small horns, smoke from its nostrils, imposing and bulky.", 320, seed=215)
add('mon_sorcerer', 'char', "Boss monster Otter Sorcerer: an otter sorcerer in deep blue robes holding a tall staff topped with frost crystals, white-tipped fur, imposing.", 320, seed=216)
add('mon_tetsu', 'char', "Boss monster Tetsujin: a huge iron golem knight made of dark steel plates with glowing yellow eye slits and heavy fists, imposing and bulky.", 320, seed=217)

# --- 3F〜5F の部屋 ---
add('room_いろり', 'obj', "A sunken hearth fire pit with an iron pot hanging over burning logs.", 128, seed=221)
add('room_水のみば', 'obj', "A stone water trough with clear water and a wooden bucket beside it.", 128, seed=222)
add('room_りっぱなベッド', 'obj', "A fine four-poster bed with a red velvet canopy, white linen and golden trim.", 128, seed=223)
add('room_はなぞの', 'tile', "Top-down flower garden of colorful flowers in raised beds with a small stone path, a short wooden fence around the edge.", 108, seed=224)
add('room_どうじょう', 'tile', "Top-down wooden dojo training floor with straw training dummies and weapon racks along the edges, a short wooden fence around the edge.", 108, seed=225)


# --- 6F〜10F の魔物 ---
add('mon_chikaraushi', 'char', "Strong ox monster (Chikara-ushi): a muscular bipedal bull with a nose ring and brown fur, carrying a huge boulder club on its shoulder.", 256, seed=301)
add('mon_powerwan', 'char', "Power dog-man monster (Power-wan): a muscular bipedal brown dog in heavy plate armor wielding a big broadsword and a tower shield.", 256, seed=302)
add('mon_flamewar', 'char', "Flame warrior monster: a knight in red-orange armor with ember glow wielding a wavy flaming flamberge sword.", 256, seed=303)
add('mon_icewar', 'char', "Ice warrior monster: a knight in pale frosty blue armor with icicles wielding an ice-crystal sword.", 256, seed=304)
add('mon_masterwar', 'char', "Master warrior monster: an elite knight in black armor with gold trim and a red plume wielding a black blade, imposing.", 256, seed=305)
add('mon_hotjigoku', 'char', "Fire hell monster: a small red imp-demon engulfed in flames with burning eyes.", 256, seed=306)
add('mon_icejigoku', 'char', "Blizzard hell monster: an icy blue imp-demon surrounded by swirling snow and frost.", 256, seed=307)
add('mon_buffalo', 'char', "Buffalo bull monster: a huge angry buffalo with long curved horns and a dark shaggy mane.", 256, seed=308)
add('mon_penguin', 'char', "Were-penguin monster (Wa-pen): a bipedal penguin warrior holding a trident-like spear and a round shield.", 256, seed=309)
add('mon_pepper', 'char', "Cursed pepper monster: a creepy green bell pepper with a dark purple aura and a hollow grinning face.", 256, seed=310)
add('mon_nekomata2', 'char', "Nekomata sorceress monster: a two-tailed cat in a silver-trimmed dark dress holding a staff, bold and sly.", 256, seed=311)
add('mon_nekomata3', 'char', "Nekomata queen sorceress monster: a two-tailed cat wearing a golden tiara and a regal gown, holding a jeweled staff, haughty.", 256, seed=312)
add('mon_zombie', 'char', "Comedian zombie monster: a pale zombie in a ragged tuxedo and a silly crooked hat, grinning goofily.", 256, seed=313)
add('mon_platypus', 'char', "Platypus thief monster: a bipedal platypus in a bandana holding a knife, sneaky.", 256, seed=314)
add('mon_valkyrie', 'char', "Valkyrie monster: a maiden warrior with a winged helm, light armor and a long spear, noble and stern.", 256, seed=315)
add('mon_catqueen', 'char', "Queen of werecats monster: a bipedal cat in a royal red cloak and small crown, holding a jeweled dagger, haughty.", 256, seed=316)
add('mon_emperorwan', 'char', "Emperor dog-warrior monster: a bipedal dog in golden royal armor with a cape holding a kaiser blade, majestic.", 256, seed=317)
add('mon_poisonlime', 'char', "Poison slime monster: a purple jelly blob with bubbling toxic drips and a skull-like face.", 256, seed=318)
add('mon_demonarmor', 'char', "Demon armor monster: a haunted empty suit of dark spiked armor with glowing red eyes and a cruel spear.", 256, seed=319)
add('mon_debrin', 'char', "Boss monster Debrin: a fat red horned devil with a cone-shaped horn hat and a fiery staff, grinning menacingly.", 320, seed=320)
add('mon_octopus', 'char', "Boss monster Octopus king (Actopus): a giant blue octopus wearing a small crown and a sea-green cloak, tentacles raised.", 320, seed=321)
add('mon_kinglion', 'char', "Boss monster King Lion: a majestic lion king in a royal mantle and crown, holding a great spear, mane glowing gold.", 320, seed=322)
add('mon_dragon', 'char', "Boss monster Uncle Dragon: a large wise green-brown dragon wearing a long scarf, breathing smoke.", 320, seed=323)
add('mon_devilprince', 'char', "Boss monster Devil Prince: a sinister prince in ornate dark armor with curved horns, a cape and a glowing red blade.", 320, seed=324)

# --- 6F〜10F の部屋 ---
add('room_だんろ', 'obj', "A stone fireplace with a roaring fire and a wooden mantel.", 128, seed=331)
add('room_けっかいのへや', 'obj', "A glowing purple magic circle on stone floor with a faint barrier dome of runes.", 128, seed=332)
add('room_ふんすい', 'obj', "An ornate stone fountain with flowing water.", 128, seed=333)
add('room_ごうかなベッド', 'obj', "A luxurious gold-trimmed bed with silk sheets and large plump pillows.", 128, seed=334)
add('room_れいはいどう', 'obj', "A small chapel altar with a stained-glass window and candles.", 128, seed=335)
add('room_じっけんしつ', 'tile', "Top-down alchemy laboratory with bubbling cauldrons, tables of flasks and a glowing magic circle, a short wooden fence around the edge.", 108, seed=336)


# --- 11F〜20F ---
add('mon_anagerilla', 'char', 'Burly mole monster (Anagerilla): a bipedal mole with a pink nose and huge digging claws, wearing a miner helmet.', 256, seed=401)
add('mon_lumberduck', 'char', 'Lumber duck monster: a bipedal duck lumberjack in a plaid shirt carrying a big axe.', 256, seed=402)
add('mon_daruma', 'char', 'Daruma doll monster: a round red daruma with a stern face and thick eyebrows, hopping.', 256, seed=403)
add('mon_samuraiwolf', 'char', 'Samurai wolf monster: a bipedal grey wolf in samurai armor with an eboshi hat and a katana.', 256, seed=404)
add('mon_shogunwolf', 'char', 'Shogun wolf monster: a majestic bipedal white wolf in ornate shogun armor with a crested helmet and cape, holding a katana.', 320, seed=405)
add('mon_killercock', 'char', 'Killer rooster monster: an aggressive bipedal rooster with sharp spurs, a red comb and a feather headband.', 256, seed=406)
add('mon_hidaruma', 'char', 'Fire daruma monster: an orange-red daruma doll engulfed in flames.', 256, seed=407)
add('mon_maidoll', 'char', 'Maid doll monster: a stiff living porcelain doll in a maid outfit with an apron, holding a feather duster.', 256, seed=408)
add('mon_yukidaruma', 'char', 'Snow daruma monster: a round icy-blue snow daruma with a carrot nose.', 256, seed=409)
add('mon_mushroomlizard', 'char', 'Mushroom lizard monster: a green lizard with a big spotted mushroom growing on its back.', 256, seed=410)
add('mon_warbeetle', 'char', 'War beetle monster: a bipedal rhinoceros beetle warrior with a long horn and shell armor.', 256, seed=411)
add('mon_flamemaster', 'char', 'Flame master monster: a veteran swordsman in crimson hooded robes and armor wielding a flaming sword.', 256, seed=412)
add('mon_highmaster', 'char', 'High master monster: an elite dark knight master with white hair in layered black armor holding a great sword.', 256, seed=413)
add('mon_detarame', 'char', 'Random chaotic monster (Detarame): a bizarre jester-like creature stitched from mismatched parts with a question mark.', 256, seed=414)
add('mon_caterpillar', 'char', 'Giant caterpillar monster: a fat green caterpillar with many tiny legs.', 256, seed=415)
add('mon_killerflower', 'char', 'Killer flower monster: a carnivorous plant with a big toothy flower head and leafy arms.', 256, seed=416)
add('mon_bonewizard', 'char', 'Bone wizard monster: a skeleton mage in a tattered robe holding a staff topped with a skull.', 256, seed=417)
add('mon_magicdonkey', 'char', 'Magic donkey monster: a donkey wearing a pointy wizard hat carrying bubbling potion flasks.', 256, seed=418)
add('mon_devildice', 'char', 'Devil dice monster: a big red dice with a demon face, tiny horns and bat wings.', 256, seed=419)
add('mon_hellscream', 'char', 'Hell scream monster: a floating screaming ghostly mask with dark flames around it.', 256, seed=420)
add('mon_magicshield', 'char', 'Living magic shield monster: a floating ornate round shield with glowing runes.', 256, seed=421)
add('mon_magicbow', 'char', 'Living magic bow monster: a floating elven bow glowing with a light arrow.', 256, seed=422)
add('mon_magicsword', 'char', 'Living magic sword monster: a floating enchanted sword wreathed in blue light.', 256, seed=423)
add('mon_dorunuma', 'char', 'Boss Dorunuma: a huge mud swamp monster, a bubbling dripping mound of brown mud with glowing yellow eyes.', 320, seed=424)
add('mon_devilprincess', 'char', 'Boss Devil Princess: a sinister princess with small horns and bat wings in a black and purple gown holding a scepter.', 320, seed=425)
add('mon_hitotsuoni', 'char', 'Boss one-eyed ogre: a huge muscular ogre with a single big eye and a horn, holding a spiked club.', 320, seed=426)
add('mon_flameflower', 'char', 'Boss Flame Flower: a giant red flower monster with petals of fire and thick vines.', 320, seed=427)
add('mon_kage', 'char', 'Boss Kage: a cold dark humanoid shadow with pale glowing eyes surrounded by icy mist.', 320, seed=428)
add('mon_mitsuoni', 'char', 'Boss three-eyed ogre: a huge ogre with three eyes, curved horns and a spiked iron club.', 320, seed=429)
add('mon_elephantom', 'char', 'Boss Elephantom: a ghostly giant elephant with long tusks and a trunk of green flame.', 320, seed=430)
add('mon_devillord', 'char', 'Boss Devil Lord: a towering demon king in black and gold armor with a crown of horns holding a giant cursed sword.', 320, seed=431)
add('mon_kingsnake', 'char', 'Boss King Snake Apple: a giant crowned red apple wrapped in green snake coils with poison drips.', 320, seed=432)
add('room_コロシアム', 'tile', 'Top-down colosseum arena with sand floor, stone seating walls and crossed swords banners.', 108, seed=433)
add('room_れいぞうこ', 'obj', 'A big old-fashioned icebox refrigerator, white metal with a latch handle, frosty.', 128, seed=434)
add('room_瘴気発生装置', 'obj', 'A sinister miasma generator machine: a squat dark iron cauldron-like engine with pipes, a glass tank full of swirling purple toxic gas, and purple smoke leaking from vents.', 128, seed=436)
add('room_まほうじん', 'obj', 'A glowing violet magic circle with arcane runes on stone floor, rising sparkles.', 128, seed=435)

# --- 部屋(アイコン) ---
add('room_エサおけ', 'obj', "A wooden animal feeding trough filled with hay and grain.", 128, seed=61)
add('room_ゴミすてば', 'obj', "A pile of garbage: a broken barrel, fish bones, rotten vegetables and rags.", 128, seed=62)
add('room_わらのベッド', 'obj', "A simple medieval straw bed with a rough wool blanket and a small pillow.", 128, seed=63)
add('room_てつのたからばこ', 'obj', "An iron-banded wooden treasure chest, closed, with a padlock.", 128, seed=64)
add('room_てつのたからばこ_open', 'obj', "An iron-banded wooden treasure chest, open, full of gold coins.", 128, seed=64)  # 実際はチェストを edit で開けた(qwen --image)
add('room_ぎんのたからばこ', 'obj', "A silver treasure chest, closed.", 128, seed=64)  # 実際は鉄の宝箱を edit で銀色にした(qwen --image)
add('room_ぎんのたからばこ_open', 'obj', "A silver treasure chest, open.", 128, seed=64)
add('room_おうごんチェスト', 'obj', "A golden treasure chest, closed.", 128, seed=64)
add('room_おうごんチェスト_open', 'obj', "A golden treasure chest, open.", 128, seed=64)
add('room_さかば', 'obj', "A small tavern counter with wooden barrels and pewter ale mugs.", 128, seed=65)
add('room_さいだん', 'obj', "A weathered stone altar with melted candles and a small skull, eerie.", 128, seed=66)
add('room_はたけ', 'tile', "Top-down vegetable farm plot with tilled soil rows of cabbages, carrots and pumpkins, with a short rough wooden fence around the edge.", 108, seed=67)

# --- タイル ---
add('tile_floor', 'tile', "Packed dirt dungeon floor with flat flagstones, warm tan brown.", 72, seed=71)
add('tile_rock', 'tile', "Rugged cave rock wall, dark brown stone, rough surface.", 72, seed=72)
add('tile_bedrock', 'tile', "Near-black purple bedrock, indestructible, subtle cracks.", 72, seed=73)
add('tile_barrier', 'tile', "Cave rock wall with glowing red magical rune dots forming a dotted line across the middle.", 72, seed=74)
add('tile_stairs_up', 'tile', "A short stone stairway leading up toward the surface, steps seen from above, bright warm light at the top, dirt floor around it.", 72, seed=75)
add('tile_stairs_down', 'tile', "A stone stairway descending into darkness, steps seen from above, cold bluish dark at the bottom.", 72, seed=76)
add('tile_portal', 'tile', "A swirling purple magic vortex portal glowing on a dirt floor, seen from above.", 72, seed=77)

# --- 場面(案A: キャラと同じセル塗りの絵柄) ---
add('scene_town', 'scene', "Wide view of a small medieval fantasy village in bright daytime, viewed from slightly above. The upper quarter of the image is calm sky and hills. Top-left: a dark cave entrance in a rocky hillside. Bottom-left: a small cottage with a chimney. Center-left: a stone shop with an orange tiled roof and a crossed-swords sign (armor shop), a red-haired woman stands at its door. Center-right: a blue-roofed shop with a purple potion sign (magic shop), an elderly white-bearded man in a purple robe stands at its door. Right: a green-roofed carpenter workshop with a wooden chair sign, a man in a red cap stands by the door. Foreground: a dirt plaza with a stone fountain at the center, a young blonde village girl near the fountain, an old village chief in a brown coat at the lower left, and at the lower right a wooden quest notice board with pinned parchment notices.", 0, '4:3', seed=81)
add('scene_armor', 'scene', "Interior of a medieval fantasy armor and weapon shop. A young woman shopkeeper with an auburn ponytail and a leather apron stands in the exact center behind a wooden counter, upper body visible. Swords, shields and armor hang on wall racks. The lower third of the image is the wooden counter front.", 0, '4:3', seed=82)
add('scene_magic', 'scene', "Interior of a medieval fantasy alchemist potion shop. An elderly white-bearded man in a purple robe stands in the exact center behind a wooden counter, upper body visible. Shelves of colorful potion bottles, hanging herbs, a crystal ball. The lower third of the image is the wooden counter front.", 0, '4:3', seed=83)
add('scene_furn', 'scene', "Interior of a medieval carpenter's furniture workshop. A stocky bearded man in a red cap stands in the exact center behind a wooden counter, upper body visible. Beds, chairs, barrels and wood planks around. The lower third of the image is the wooden counter front.", 0, '4:3', seed=84)
add('scene_plaza', 'scene', "A medieval fantasy village square with a stone fountain in the center, a vegetable market stall on the left, and a young blonde village girl in a simple dress standing at center-right. No other characters. Bright daytime.", 0, '4:3', seed=85)
add('scene_mayor', 'scene', "Interior of a village chief's study. An elderly white-bearded man in a brown coat stands in the exact center behind a wooden desk, upper body visible. A map on the wall, bookshelves, a quest notice board. The lower third of the image is the desk front.", 0, '4:3', seed=86)
add('scene_home', 'empty', "Interior of a cozy medieval cottage. A bed on the right, a window on the left, a golden magic shovel leaning against the center wall, a wooden table in the lower middle. Warm light. No characters.", 0, '4:3', seed=87)
add('bg_battle', 'empty', "First-person view down a stone dungeon corridor with torches on both walls, a packed dirt floor and a dark archway at the far end, symmetrical perspective. Completely empty corridor: no person, no creature, no character, nobody anywhere, the floor is bare.", 0, '4:3', seed=188)
add('scene_title', 'empty', "A golden magic shovel stuck in a grassy hill at sunrise, rolling green hills, a distant stone castle, a large empty sky area in the upper half. Landscape only: no person, no character, nobody.", 0, '4:3', seed=189)

add('mon_needleworm', 'char', 'Needle worm monster: a plump pale-green worm with a ring of sharp bristly needles and spikes all along its back, small angry eyes, curled upright.', 256, seed=501)
add('mon_hinezumi', 'char', 'Fire mouse monster (Hinezumi): a chubby chibi mouse on all fours like a real animal, side three-quarter view, warm red-orange fur, round ears, long thin tail, and a blazing fire burning on its back like a campfire, flames rising from its back.', 256, seed=512)

add('mon_shieldmaiden', 'char', 'Shield maiden: a young armored girl knight with a silver winged helmet, holding a huge round ornate shield in front of her body and a short sword, sturdy defensive stance.', 256, seed=503)
add('mon_detaramedice', 'char', 'Chaotic dice monster: a cube-shaped die with each face painted a different bright color (red, blue, green, yellow) and mismatched wrong pips, a goofy mischievous face, tiny arms and legs.', 256, seed=504)

add('char_mayor', 'char', "An elderly village chief with a long white beard and bushy white eyebrows, a long brown coat with a green scarf, holding a wooden walking cane, kindly mischievous smile, slightly hunched posture.", 320, seed=600)
add('scene_walk', 'empty', "A dirt country road leading out of a medieval fantasy village on a bright morning: wooden fences, small vegetable fields, a windmill in the distance, wide road in the foreground, side view, no people.", 0, '4:3', seed=601)
add('scene_site', 'empty', "A wide empty grassy open plot at the edge of a village, a flat meadow with a few small rocks, a distant tree line and soft hills, big blue sky, nothing built here at all, no people.", 0, '4:3', seed=602)

COPIES = {'portrait_a': 'char_hero', 'hero_map': 'char_hero'}
SIZES = {'portrait_a': 256, 'hero_map': 96}


def generate(name, force=False):
    kind, prompt, aspect, size, seed = ITEMS[name]
    raw = os.path.join(RAW, name + '.png')
    if os.path.exists(raw) and not force:
        return raw
    pf = os.path.join(RAW, name + '.txt')
    with open(pf, 'w', encoding='utf-8') as f:
        f.write(prompt)
    tmp = os.path.join(RAW, '_tmp')
    os.makedirs(tmp, exist_ok=True)
    cmd = [sys.executable, QWEN, '--prompt-file', pf, '--aspect', aspect, '--seed', str(seed), '--out', tmp, '--name', name]
    env = dict(os.environ, PYTHONIOENCODING='utf-8', PYTHONUTF8='1')
    r = subprocess.run(cmd, capture_output=True, text=True, encoding='utf-8', errors='replace', env=env)
    if r.stdout is None:
        r.stdout = ''
    line = [l for l in r.stdout.strip().splitlines() if l.startswith('{')]
    if not line:
        print('FAILED', name, r.stdout[-300:], r.stderr[-300:])
        return None
    j = json.loads(line[-1])
    if not j.get('ok'):
        print('FAILED', name, j)
        return None
    src = j['images'][0]['path']
    if os.path.exists(raw):
        os.remove(raw)
    os.replace(src, raw)
    print('ok', name, j['images'][0]['seconds'], 's', flush=True)
    return raw


def key_magenta(im):
    a = np.asarray(im.convert('RGB')).astype(np.float32)
    r, g, b = a[..., 0], a[..., 1], a[..., 2]
    m = np.minimum(r, b) - g  # マゼンタらしさ
    alpha = 1.0 - np.clip((m - 22.0) / 45.0, 0.0, 1.0)
    # 縁のマゼンタ被りを取る
    spill = np.clip(m, 0, 255) * (1.0 - alpha) * 0.8
    r2 = np.clip(r - spill, 0, 255)
    b2 = np.clip(b - spill, 0, 255)
    out = np.dstack([r2, g, b2, alpha * 255.0]).astype(np.uint8)
    return Image.fromarray(out, 'RGBA')


def crop_square(im, margin=0.05):
    al = np.asarray(im)[..., 3]
    ys, xs = np.where(al > 24)
    if len(xs) == 0:
        return im
    x0, x1, y0, y1 = xs.min(), xs.max() + 1, ys.min(), ys.max() + 1
    w, h = x1 - x0, y1 - y0
    s = int(max(w, h) * (1 + 2 * margin))
    canvas = Image.new('RGBA', (s, s), (0, 0, 0, 0))
    canvas.paste(im.crop((x0, y0, x1, y1)), ((s - w) // 2, s - h - int(max(w, h) * margin)))
    return canvas


def head_crop(im):
    """立ち絵から 頭〜肩の正方形を切り出す(顔グラフィック用)"""
    al = np.asarray(im)[..., 3]
    ys, xs = np.where(al > 24)
    x0, x1, y0, y1 = xs.min(), xs.max() + 1, ys.min(), ys.max() + 1
    h = y1 - y0
    top = (ys < y0 + int(h * 0.35))
    cx = int(xs[top].mean())
    side = int(h * 0.5)
    left = max(0, cx - side // 2)
    box = (left, y0, min(im.width, left + side), min(im.height, y0 + side))
    return im.crop(box)


def post(name):
    base = COPIES.get(name, name)
    raw = os.path.join(RAW, base + '.png')
    if not os.path.exists(raw):
        return
    kind, _, _, size, _ = ITEMS[base]
    size = SIZES.get(name, size)
    im = Image.open(raw)
    if kind in ('char', 'obj'):
        im = key_magenta(im)
        if name in ('portrait_a', 'portrait_m'):
            im = head_crop(im)
        im = crop_square(im, 0.04)
        im = im.resize((size, size), Image.LANCZOS)
    elif kind == 'tile':
        im = im.convert('RGB')
        if name == 'room_はたけ':
            # 畑は絵の縁が白いので、少し内側を使う
            w, h = im.size
            im = im.crop((int(w * 0.06), int(h * 0.06), int(w * 0.94), int(h * 0.94)))
        im = im.resize((size, size), Image.LANCZOS)
        if name == 'tile_rock':
            from PIL import ImageEnhance
            im = ImageEnhance.Brightness(im).enhance(1.45)
    else:
        im = im.convert('RGB').resize((576, 432), Image.LANCZOS)
    im.save(os.path.join(OUT, name + '.png'))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--only', default='')
    ap.add_argument('--redo', action='store_true')
    ap.add_argument('--post-only', action='store_true')
    a = ap.parse_args()
    names = [n for n in ITEMS if not a.only or n in a.only.split(',')]
    if not a.post_only:
        for n in names:
            generate(n, force=a.redo)
    for n in list(names) + [k for k in COPIES if not a.only or COPIES[k] in names or k in a.only.split(',')]:
        post(n)
    print('done')


if __name__ == '__main__':
    main()
