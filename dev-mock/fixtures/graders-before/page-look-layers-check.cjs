// BUG-7 / BUG-8 (V1.5.1 PC round): the cover buttons must be clickable on a page with an icon and a cover, and the Home glyph must sit above the dimmed banner. Run: npm run mock, then MOCK_URL=http://127.0.0.1:1421/ node dev-mock/page-look-layers-check.cjs <shots dir>
const { chromium } = require('playwright');
(async()=>{
  const b = await chromium.launch(); let fails=0;
  const ok=(n,c)=>{console.log((c?'ok   ':'FAIL ')+n); if(!c) fails++;};
  for (const w of [1280,1600]) {
    const p = await b.newPage({ viewport: { width: w, height: 900 } });
    await p.goto(''+(process.env.MOCK_URL||'http://127.0.0.1:1421/')+'?theme=black'); await p.waitForTimeout(1500);
    await p.getByText('Morning pages').first().click(); await p.waitForTimeout(800);
    const cover = p.getByTestId('page-cover');
    ok('page has cover and icon @'+w, await cover.count()===1 && await p.getByTestId('page-icon').count()===1);
    await cover.hover(); await p.waitForTimeout(400);
    for (const name of ['Change cover','Remove cover']) {
      const btn = p.getByRole('button',{name}).first();
      const bb = await btn.boundingBox();
      if(!bb){ ok(name+' visible @'+w,false); continue; }
      const hit = await p.evaluate(({x,y})=>{const e=document.elementFromPoint(x,y); return e && (e.closest('button')?(e.closest('button').textContent||e.closest('button').getAttribute('aria-label')||'BUTTON'):e.tagName);}, {x:bb.x+bb.width/2,y:bb.y+bb.height/2});
      ok(`${name} receives the click @${w} (got "${hit}")`, hit && (hit.includes(name)||(name==='Remove cover'&&hit==='BUTTON')));
    }
    const icon = await p.getByTestId('page-icon').boundingBox();
    const hit2 = await p.evaluate(({x,y})=>{const e=document.elementFromPoint(x,y); return !!e.closest('[data-testid=page-icon]');}, {x:icon.x+icon.width/2,y:icon.y+icon.height/2});
    ok('the icon still gets its click @'+w, hit2);
    await p.close();
  }
  const p = await b.newPage({ viewport: { width: 1280, height: 900 } });
  await p.goto(''+(process.env.MOCK_URL||'http://127.0.0.1:1421/')+'?theme=black'); await p.waitForTimeout(1500);
  await p.getByText('Home',{exact:true}).first().click(); await p.waitForTimeout(800);
  const z = await p.evaluate(()=>{const g=document.querySelector('[data-testid=home] .text-\\[3\\.2rem\\]'); const r=g.getBoundingClientRect(); const e=document.elementFromPoint(r.x+r.width/2,r.y+r.height/2); return {z:getComputedStyle(g).zIndex,pos:getComputedStyle(g).position, hit:e.className.toString().slice(0,60)};});
  console.log(z);
  ok('home glyph has its own layer above the banner', z.z==='10' && z.pos==='relative');
  const cdp = await p.context().newCDPSession(p);
  const {data} = await cdp.send('Page.captureScreenshot',{format:'png'});
  require('fs').writeFileSync((process.argv[2]||'.')+'/home-black.png', Buffer.from(data,'base64'));
  await b.close(); console.log(fails?'FAILED '+fails:'ALL OK');
})();
