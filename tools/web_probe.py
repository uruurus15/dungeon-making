# Edge(ヘッドレス)で Web 版を 開き、コンソールの ログ・画面・フレーム時間を 調べる(確認用)。
#   python tools/web_probe.py [URL] [秒] [出力png] [キー列: 例 F2,wait:5,KeyS]
import json, subprocess, sys, time, base64, urllib.request
import websocket
url = sys.argv[1] if len(sys.argv) > 1 else 'http://localhost:8765/'
secs = float(sys.argv[2]) if len(sys.argv) > 2 else 30
out = sys.argv[3] if len(sys.argv) > 3 else 'shots/web_probe.png'
keys = sys.argv[4].split(',') if len(sys.argv) > 4 else []
EDGE = r'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
port = 9333
p = subprocess.Popen([EDGE, '--headless=new', '--remote-debugging-port=%d' % port, '--remote-allow-origins=*', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist', '--window-size=' + __import__('os').environ.get('WS', '1280,720'), '--user-data-dir=C:/Users/hamuh/AppData/Local/Temp/edge_probe', 'about:blank'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
time.sleep(3)
tabs = json.loads(urllib.request.urlopen('http://localhost:%d/json' % port).read())
tab = [t for t in tabs if t['type'] == 'page'][0]
ws = websocket.create_connection(tab['webSocketDebuggerUrl'], max_size=None)
mid = [0]
def send(method, params=None):
    mid[0] += 1
    ws.send(json.dumps({'id': mid[0], 'method': method, 'params': params or {}}))
    return mid[0]
logs = []
def pump(t):
    end = time.time() + t
    ws.settimeout(0.2)
    while time.time() < end:
        try:
            m = json.loads(ws.recv())
        except Exception:
            continue
        if m.get('method') == 'Runtime.consoleAPICalled':
            logs.append(' '.join(str(a.get('value', a.get('description', ''))) for a in m['params']['args'])[:300])
        elif m.get('method') == 'Runtime.exceptionThrown':
            logs.append('EXC ' + json.dumps(m['params']['exceptionDetails'])[:300])
        elif m.get('method') == 'Log.entryAdded':
            logs.append('LOG ' + m['params']['entry'].get('text', '')[:300])
send('Runtime.enable'); send('Log.enable'); send('Page.enable')
send('Page.navigate', {'url': url})
# フレーム時間の 計測(requestAnimationFrame)
send('Runtime.evaluate', {'expression': "window.__f=[];(function t(a){window.__f.push(a);requestAnimationFrame(t)})(performance.now())"})
pump(secs)
for k in keys:
    if k.startswith('wait:'):
        pump(float(k[5:])); continue
    code = k
    send('Input.dispatchKeyEvent', {'type': 'keyDown', 'code': code, 'key': code, 'windowsVirtualKeyCode': {'F2': 113, 'KeyS': 83, 'KeyW': 87, 'KeyA': 65, 'KeyZ': 90, 'Enter': 13, 'Tab': 9}.get(code, 0)})
    send('Input.dispatchKeyEvent', {'type': 'keyUp', 'code': code, 'key': code, 'windowsVirtualKeyCode': {'F2': 113, 'KeyS': 83, 'KeyW': 87, 'KeyA': 65, 'KeyZ': 90, 'Enter': 13, 'Tab': 9}.get(code, 0)})
    pump(0.4)
i = send('Page.captureScreenshot', {'format': 'png'})
end = time.time() + 20
while time.time() < end:
    try:
        m = json.loads(ws.recv())
    except Exception:
        continue
    if m.get('id') == i:
        open(out, 'wb').write(base64.b64decode(m['result']['data'])); break
j = send('Runtime.evaluate', {'expression': "JSON.stringify((function(){var f=window.__f;var d=[];for(var i=1;i<f.length;i++)d.push(f[i]-f[i-1]);d=d.slice(-120);d.sort(function(a,b){return a-b});return {frames:f.length,median_ms:d[d.length>>1],p90_ms:d[Math.floor(d.length*0.9)],max_ms:d[d.length-1]}})())"})
end = time.time() + 10
while time.time() < end:
    try:
        m = json.loads(ws.recv())
    except Exception:
        continue
    if m.get('id') == j:
        print('FRAMES', m['result']['result'].get('value')); break
print('\n'.join(logs[-30:]))
p.terminate()
