# Openverse(音声)の検索。  python tools/ov.py "クエリ" [件数] [license_type]
import json, sys, urllib.parse, urllib.request
q = sys.argv[1]
n = int(sys.argv[2]) if len(sys.argv) > 2 else 8
params = {'q': q, 'page_size': n, 'license': 'cc0,pdm,by', 'category': 'sound_effect'}
if len(sys.argv) > 3:
    params['category'] = sys.argv[3]
url = 'https://api.openverse.org/v1/audio/?' + urllib.parse.urlencode(params)
req = urllib.request.Request(url, headers={'User-Agent': 'dungeon-making-sfx/1.0'})
d = json.load(urllib.request.urlopen(req, timeout=30))
print('count', d.get('result_count'))
for r in d['results']:
    print(r['id'], '|', r['title'][:40], '|', r['license'], r.get('license_version'), '|', r.get('duration'), 'ms |', r.get('filetype'), '|', r.get('creator'), '|', r.get('source'), '|', r['url'][:90])
