# 効果音の表: イベント名 → (検索語, 最大の長さ[ms], 切り出す最大秒, 音量)
# 取得は tools/fetch_sfx.py、使う側は src/audio.rs(イベント名はここと同じ)。
# 気に入らない音は、assets/sfx_pick.json の id を書きかえて fetch_sfx.py を再実行すると差しかわる。
EVENTS = {
    # メニュー
    'move': ('menu cursor click', 600, 0.4, 0.5),
    'ok': ('button select confirm', 1200, 0.8, 0.7),
    'cancel': ('menu back cancel', 1200, 0.6, 0.6),
    'blocked': ('error buzzer|denied beep|negative buzz|wrong', 1500, 0.5, 0.6),
    # ダンジョン
    'step': ('footstep|footsteps gravel|walking step|step', 900, 0.35, 0.35),
    'dig': ('dig shovel dirt', 2500, 1.0, 0.8),
    'fill': ('soil dirt drop|gravel pour|earth thud|dirt', 2500, 0.9, 0.7),
    'place': ('hammer wood|wood knock|furniture|wood thunk', 2500, 1.0, 0.8),
    'break': ('wood crate break', 2500, 1.2, 0.8),
    'undo': ('rewind|reverse|whoosh|swoosh', 2500, 0.8, 0.6),
    'encounter': ('monster growl short', 3000, 1.2, 0.8),
    'chest': ('treasure chest|chest opening|creak open', 3500, 1.5, 0.8),
    'stairs': ('stairs footsteps|steps walking down|stairs|footsteps', 4000, 1.6, 0.7),
    'portal': ('magic portal open whoosh', 4000, 2.0, 0.8),
    # 戦闘
    'attack': ('sword slash', 2000, 0.8, 0.8),
    'hit': ('punch hit impact', 1500, 0.6, 0.8),
    'crit': ('critical hit|heavy impact|smash|strong hit', 2500, 1.0, 0.9),
    'down': ('monster death', 3000, 1.2, 0.8),
    'fire': ('fire spell cast', 3000, 1.4, 0.8),
    'heal': ('healing magic|heal spell|sparkle magic|magic chime', 3000, 1.4, 0.7),
    'sleepspell': ('magic sparkle|harp glissando|fairy magic|dreamy chime', 3500, 1.5, 0.7),
    'item': ('potion drink', 3000, 1.2, 0.7),
    'bomb': ('explosion|bomb blast|explode', 3500, 1.4, 0.8),
    'flee': ('running away footsteps', 3500, 1.2, 0.7),
    'boss': ('boss roar', 5000, 2.0, 0.9),
    'win': ('victory fanfare short', 8000, 3.5, 0.8),
    'lose': ('game over jingle|defeat jingle|lose sound|failure jingle', 8000, 3.5, 0.8),
    # 町・生活
    'buy': ('coins|coin|money|purchase', 3000, 1.2, 0.8),
    'sell': ('cash register', 3000, 1.2, 0.8),
    'eat': ('eating crunch bite', 3000, 1.2, 0.7),
    'equip': ('equip armor', 2500, 1.0, 0.7),
    'quest': ('quest complete jingle', 5000, 2.5, 0.8),
    'morning': ('rooster morning', 5000, 2.5, 0.6),
    'sleep': ('night chime|bell|yawn|sleep', 5000, 2.5, 0.6),
}
