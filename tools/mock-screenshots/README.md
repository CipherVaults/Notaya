# Mock screenshots

`shot.mjs` saves a PNG of the Notaya **mock UI** (fake data in a plain browser, `npm run mock`). `act.mjs` does the same with a list of scripted clicks, keys and screenshots. They never touch the real app or a vault. Development use only.

## Run

From the project root, in PowerShell:

```powershell
# 1. Start the mock (port 1421) in its own window or as a background job
npm run mock

# 2. Start headless Edge on port 9333 with a throwaway profile folder
$prof = Join-Path $env:TEMP "edge-shot-$(Get-Random)"
$edge = Start-Process "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe" `
  -ArgumentList "--headless=new","--remote-debugging-port=9333","--user-data-dir=$prof","about:blank" -PassThru

# 3. Take a screenshot (out.png, url, optional JS to run first, wait ms, width, height)
node tools\mock-screenshots\shot.mjs out.png http://127.0.0.1:1421/ "" 900 1360 860

# 4. Stop both and confirm the ports are free
Stop-Process -Id $edge.Id          # the Edge you started; its child processes end with it
# stop the mock with Ctrl+C in its window (or stop that job)
Remove-Item $prof -Recurse -Force
@(Get-NetTCPConnection -State Listen -LocalPort 9333,1421 -ErrorAction SilentlyContinue).Count   # must be 0
```

Needs Node 24 and Microsoft Edge. Port 9333 is only for this headless Edge; check that nothing else listens there before you start. Put the PNGs outside the repository (or in an ignored folder).
