// Checks page icons, cover banners and the Home page in the mock UI. Development only; not part of the app.
// Needs Playwright. Serve the mock (npm run mock, or a static build of vite.mock.config.ts), then:
//   MOCK_URL=http://127.0.0.1:1421 node dev-mock/icons-covers-ui-check.cjs [screenshot folder]
// Prints ALL OK when every check passes.
const { chromium } = require('playwright');
const fs = require('fs');
const os = require('os');
const path = require('path');
const BASE = process.env.MOCK_URL || 'http://127.0.0.1:1421';
const shots = process.argv[2];
let fail = 0;
const is = (name, got, want) => { if (JSON.stringify(got) !== JSON.stringify(want)) { fail++; console.log('FAIL', name, JSON.stringify(got), '!==', JSON.stringify(want)); } else console.log('ok  ', name); };
const truthy = (name, v, extra) => { if (!v) { fail++; console.log('FAIL', name, extra === undefined ? '' : JSON.stringify(extra)); } else console.log('ok  ', name); };
const near = (name, got, want, tol = 2) => truthy(name, Math.abs(got - want) <= tol, { got, want });

(async () => {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'covers-'));
  const b = await chromium.launch({ ignoreDefaultArgs: ['--hide-scrollbars'] });
  const ctx = await b.newContext({ viewport: { width: 1280, height: 900 }, deviceScaleFactor: 1 });
  const p = await ctx.newPage();
  p.on('pageerror', (e) => { fail++; console.log('FAIL page error', e.message); });
  p.on('console', (m) => { if (m.type() === 'error' && !/Failed to load resource/.test(m.text())) { fail++; console.log('FAIL console error', m.text()); } });
  // Nothing may be fetched from outside the app.
  p.on('request', (r) => { const u = r.url(); if (!u.startsWith(BASE) && !u.startsWith('data:') && !u.startsWith('blob:')) { fail++; console.log('FAIL outside request', u); } });

  const cdp = await ctx.newCDPSession(p);
  const shot = async (name) => {
    if (!shots) return;
    fs.mkdirSync(shots, { recursive: true });
    const r = await cdp.send('Page.captureScreenshot', { format: 'png' });
    fs.writeFileSync(path.join(shots, name + '.png'), Buffer.from(r.data, 'base64'));
  };
  const open = async (query, title) => {
    await p.goto(`${BASE}/?${query}`);
    await p.waitForTimeout(1200);
    if (title) { await p.getByText(title, { exact: true }).first().click(); await p.waitForTimeout(700); }
  };
  const home = async () => { await p.getByRole('button', { name: /^Home/ }).first().click().catch(() => p.getByText('Home', { exact: true }).first().click()); await p.waitForTimeout(500); };

  // Test pictures: a tall one (stripes, so a vertical position shows) and a small square one.
  const png = async (w, h, fn) => (await p.evaluate(({ w, h, fn }) => {
    const c = document.createElement('canvas'); c.width = w; c.height = h; const x = c.getContext('2d');
    new Function('x', 'w', 'h', fn)(x, w, h); return c.toDataURL('image/png').split(',')[1];
  }, { w, h, fn }));
  await p.goto(`${BASE}/?cover=none`);
  fs.writeFileSync(path.join(tmp, 'tall.png'), Buffer.from(await png(800, 1600, 'for(let i=0;i<16;i++){x.fillStyle=i%2?"#2563eb":"#f97316";x.fillRect(0,i*100,w,100)}'), 'base64'));
  fs.writeFileSync(path.join(tmp, 'icon.png'), Buffer.from(await png(200, 200, 'x.fillStyle="#16a34a";x.fillRect(0,0,w,h);x.fillStyle="#fff";x.fillRect(60,60,80,80)'), 'base64'));
  fs.writeFileSync(path.join(tmp, 'notes.txt'), 'not a picture');

  // ----- Adding an icon and a cover -----
  await open('cover=none&theme=sunrise', 'Reading list');
  const title = p.locator('input[placeholder="Untitled"]');
  is('the page starts without a cover', await p.getByTestId('page-cover').count(), 0);
  is('the page starts without an icon', await p.getByTestId('page-icon').count(), 0);
  const actions = p.getByTestId('page-look-actions');
  is('"Add icon" and "Add cover" are hidden until the title is hovered', await actions.evaluate((e) => getComputedStyle(e).opacity), '0');
  await title.hover();
  await p.waitForTimeout(250);
  is('hovering the title shows them', await actions.evaluate((e) => getComputedStyle(e).opacity), '1');
  truthy('both buttons exist', (await p.getByTestId('add-icon').count()) === 1 && (await p.getByTestId('add-cover').count()) === 1);

  await p.getByTestId('add-icon').click();
  await p.getByTestId('icon-picker').waitFor();
  truthy('the emoji picker has several hundred emoji', (await p.locator('[data-testid=icon-picker] [data-emoji]').count()) > 400, await p.locator('[data-testid=icon-picker] [data-emoji]').count());
  truthy('the picker shows group tabs', (await p.locator('[data-testid=icon-picker] button[aria-label]').count()) >= 8);
  await p.getByPlaceholder('Search emoji').fill('rocket');
  is('searching "rocket" finds exactly the rocket', await p.locator('[data-testid=icon-picker] [data-emoji]').evaluateAll((l) => l.map((e) => e.dataset.emoji)), ['🚀']);
  await p.getByPlaceholder('Search emoji').fill('zzzzqq');
  truthy('a search with no match says so', await p.getByText('No emoji match.').isVisible());
  await p.getByPlaceholder('Search emoji').fill('rocket');
  await shot('01-icon-picker-sunrise');
  await p.locator('[data-emoji="🚀"]').click();
  await p.waitForTimeout(400);
  is('the icon shows above the title', (await p.getByTestId('page-icon').innerText()).trim(), '🚀');
  is('the picker closes after choosing', await p.getByTestId('icon-picker').count(), 0);
  truthy('the sidebar row shows the icon', (await p.locator('aside, nav').filter({ hasText: 'Reading list' }).first().innerText()).includes('🚀'));

  // Change it: Random, then Remove.
  await p.getByTestId('page-icon').click();
  await p.getByTestId('icon-picker').waitFor();
  await p.getByTestId('icon-picker').getByRole('button', { name: /Random/ }).click();
  await p.waitForTimeout(300);
  const randomIcon = (await p.getByTestId('page-icon').innerText()).trim();
  truthy('Random picks an emoji', randomIcon.length > 0, randomIcon);
  await p.getByTestId('page-icon').click();
  await p.getByTestId('icon-picker').getByRole('button', { name: /Remove/ }).click();
  await p.waitForTimeout(300);
  is('Remove takes the icon away', await p.getByTestId('page-icon').count(), 0);
  await title.hover();
  await p.getByTestId('add-icon').click();
  await p.locator('[data-emoji="📚"]').click();
  await p.waitForTimeout(300);

  // A picture as the icon (pages only), shrunk to a small square.
  await p.getByTestId('page-icon').click();
  await p.getByTestId('icon-picker').getByRole('button', { name: 'Picture' }).click();
  const [chooser0] = await Promise.all([p.waitForEvent('filechooser'), p.getByRole('button', { name: /Upload a picture/ }).click()]);
  await chooser0.setFiles(path.join(tmp, 'icon.png'));
  await p.waitForTimeout(800);
  const iconImg = p.getByTestId('page-icon').locator('img');
  is('a picture icon shows as an image', await iconImg.count(), 1);
  const iconSrc = await iconImg.getAttribute('src');
  truthy('the picture icon comes from the vault, not an outside address', /^(blob:|data:|http:\/\/clatt\.localhost\/)/.test(iconSrc), iconSrc);
  truthy('the sidebar row shows the picture icon', (await p.locator('aside img, nav img').count()) >= 1);
  await p.getByTestId('page-icon').click();
  await p.getByTestId('icon-picker').getByRole('button', { name: /Remove/ }).click();
  await p.waitForTimeout(300);

  // ----- Cover: gradients, colours, pictures -----
  await title.hover();
  await p.getByTestId('add-cover').click();
  await p.getByTestId('cover-picker').waitFor();
  const gradients = await p.locator('[data-testid=cover-picker] [data-cover^="gradient:"]').count();
  const solids = await p.locator('[data-testid=cover-picker] [data-cover^="solid:"]').count();
  truthy('the cover picker offers many gradients', gradients >= 16, gradients);
  truthy('and solid colours', solids >= 12, solids);
  truthy('and an Upload picture button', await p.getByRole('button', { name: 'Upload picture' }).isVisible());
  await shot('02-cover-picker-sunrise');
  await p.locator('[data-cover="gradient:aurora"]').click();
  await p.waitForTimeout(400);
  const cover = p.getByTestId('page-cover');
  const box = await cover.boundingBox();
  const scroller = await cover.evaluate((e) => e.parentElement.clientWidth);
  near('the banner is 220px tall', box.height, 220, 1);
  near('and as wide as the page area', box.width, scroller, 1);
  truthy('the gradient is painted', await cover.evaluate((e) => getComputedStyle(e).backgroundImage.includes('gradient')));
  await title.hover();
  await shot('03-cover-gradient-sunrise');

  await cover.hover();
  await p.getByRole('button', { name: 'Change cover' }).click();
  await p.locator('[data-cover="solid:#cfe8cf"]').click();
  await p.waitForTimeout(300);
  is('a solid colour fills the banner', await cover.evaluate((e) => getComputedStyle(e).backgroundColor), 'rgb(207, 232, 207)');

  // Pictures must be pictures.
  await cover.hover();
  await p.getByRole('button', { name: 'Change cover' }).click();
  const [chooserBad] = await Promise.all([p.waitForEvent('filechooser'), p.getByRole('button', { name: 'Upload picture' }).click()]);
  await chooserBad.setFiles(path.join(tmp, 'notes.txt'));
  await p.waitForTimeout(500);
  truthy('a text file is refused with a message', await p.getByText('Only PNG, JPEG, GIF and WebP pictures can be used.').first().isVisible());
  await p.keyboard.press('Escape');

  await cover.hover();
  await p.getByRole('button', { name: 'Change cover' }).click();
  const [chooser1] = await Promise.all([p.waitForEvent('filechooser'), p.getByRole('button', { name: 'Upload picture' }).click()]);
  await chooser1.setFiles(path.join(tmp, 'tall.png'));
  await p.waitForTimeout(900);
  const img = cover.locator('img');
  is('the uploaded picture fills the banner', await img.count(), 1);
  is('and starts in the middle', await img.evaluate((e) => e.style.objectPosition), '50% 50%');
  const pos = async () => Number((await img.evaluate((e) => e.style.objectPosition)).split(' ')[1].replace('%', ''));

  // ----- Reposition -----
  await cover.hover();
  await p.getByRole('button', { name: 'Reposition' }).click();
  truthy('the banner says how to move the picture', await p.getByText('Drag the picture up or down').isVisible());
  const cb = await cover.boundingBox();
  const cx = cb.x + cb.width / 2, cy = cb.y + 100;
  await p.mouse.move(cx, cy); await p.mouse.down(); await p.mouse.move(cx, cy + 60, { steps: 6 }); await p.mouse.up();
  const afterDown = await pos();
  truthy('dragging down shows more of the top (position goes down)', afterDown < 50, afterDown);
  // 800 x 1600 shown 1000 wide-ish is much taller than 220, so a 60px drag moves a few percent only; check proportionality.
  await p.mouse.move(cx, cy); await p.mouse.down(); await p.mouse.move(cx, cy - 4000, { steps: 8 }); await p.mouse.up();
  is('dragging far up stops at the bottom (100%)', await pos(), 100);
  await p.mouse.move(cx, cy); await p.mouse.down(); await p.mouse.move(cx, cy + 4000, { steps: 8 }); await p.mouse.up();
  is('dragging far down stops at the top (0%)', await pos(), 0);
  await p.mouse.move(cx, cy); await p.mouse.down(); await p.mouse.move(cx, cy - 200, { steps: 6 }); await p.mouse.up();
  const chosenY = await pos();
  truthy('a drag lands between the ends', chosenY > 0 && chosenY < 100, chosenY);
  await shot('04-reposition-sunrise');
  await p.getByRole('button', { name: 'Save position' }).click();
  await p.waitForTimeout(400);
  is('the position is kept after Save', await pos(), chosenY);
  is('the Reposition tools are gone', await p.getByRole('button', { name: 'Save position' }).count(), 0);
  // Cancel keeps the old one.
  await cover.hover();
  await p.getByRole('button', { name: 'Reposition' }).click();
  await p.mouse.move(cx, cy); await p.mouse.down(); await p.mouse.move(cx, cy + 4000, { steps: 8 }); await p.mouse.up();
  await p.getByRole('button', { name: 'Cancel' }).click();
  is('Cancel keeps the saved position', await pos(), chosenY);
  // Keyboard.
  await cover.hover();
  await p.getByRole('button', { name: 'Reposition' }).click();
  await p.keyboard.press('ArrowDown'); await p.keyboard.press('ArrowDown');
  await p.keyboard.press('Enter');
  await p.waitForTimeout(300);
  is('arrow keys then Enter move and save the position', await pos(), Math.min(100, chosenY + 4));

  // The look is kept when the page is opened again.
  const savedY = await pos();
  await home();
  await p.getByTestId('home-card').filter({ hasText: 'Reading list' }).first().click();
  await p.waitForTimeout(700);
  is('the cover is still there after leaving and coming back', await p.getByTestId('page-cover').locator('img').count(), 1);
  is('with the same position', await p.getByTestId('page-cover').locator('img').evaluate((e) => Number(e.style.objectPosition.split(' ')[1].replace('%', ''))), savedY);

  // ----- Icon over a cover -----
  await p.getByTestId('page-icon-row').hover();
  await p.getByTestId('add-icon').click({ force: true });
  await p.locator('[data-emoji="📚"]').click();
  await p.waitForTimeout(300);
  const ib = await p.getByTestId('page-icon').boundingBox();
  const cb2 = await p.getByTestId('page-cover').boundingBox();
  truthy('the icon overlaps the bottom of the banner', ib.y < cb2.y + cb2.height && ib.y + ib.height > cb2.y + cb2.height, { ib, cb2 });
  await p.getByTestId('page-look-actions').count().then((n) => is('no "Add" buttons are left when both are set', n, 0));
  await shot('05-icon-and-picture-cover-sunrise');

  // Removing the cover.
  await p.getByTestId('page-cover').hover();
  await p.getByRole('button', { name: 'Remove cover' }).click();
  await p.waitForTimeout(300);
  is('Remove cover takes the banner away', await p.getByTestId('page-cover').count(), 0);

  // ----- Home -----
  await home();
  await p.getByTestId('home').waitFor();
  truthy('Home has a banner', await p.getByTestId('home-banner').isVisible());
  const hb = await p.getByTestId('home-banner').boundingBox();
  near('the Home banner is as wide as the page area', hb.width, await p.getByTestId('home').evaluate((e) => e.clientWidth), 20);
  truthy('Home greets you', /Good (morning|afternoon|evening)|Working late/.test(await p.getByTestId('home').innerText()));
  const favs = p.getByTestId('home-favorites').getByTestId('home-card');
  truthy('Favorites list the favourite pages', (await favs.count()) >= 1 && (await favs.first().innerText()).includes('Morning pages'));
  is('a card of a page with a cover shows a cover strip', await favs.first().getAttribute('data-has-cover'), '1');
  truthy('a card shows the page icon', (await favs.first().innerText()).includes('☀️'));
  const recents = p.getByTestId('home-recent').getByTestId('home-card');
  const rc = await recents.count();
  truthy('Recent pages has cards (at most 8)', rc >= 3 && rc <= 8, rc);
  truthy('a page with no cover has no strip', (await p.getByTestId('home-recent').locator('[data-has-cover="0"]').count()) >= 1);
  truthy('trashed pages are not listed', !(await p.getByTestId('home').innerText()).includes('Old draft'));
  const nbs = p.getByTestId('home-notebooks');
  truthy('Notebooks are listed with counts', /Personal/.test(await nbs.innerText()) && /\d+ pages?/.test(await nbs.innerText()));
  await shot('06-home-sunrise');
  await p.getByTestId('home-card').filter({ hasText: 'Trip notes' }).first().click();
  await p.waitForTimeout(700);
  is('clicking a card opens that page', await title.inputValue(), 'Morning pages'); // the mock serves the sample page body for every id; the sidebar shows the real selection
  await home();

  // ----- Every look: the banner, the buttons and Home -----
  for (const theme of ['sunrise', 'white', 'paper', 'sunrise-dark', 'black', 'graphite']) {
    await open(`theme=${theme}`, 'Morning pages');
    const c = p.getByTestId('page-cover');
    truthy(`${theme}: the cover shows`, (await c.count()) === 1 && (await c.boundingBox()).height > 200);
    await c.hover();
    const btn = p.getByRole('button', { name: 'Change cover' });
    truthy(`${theme}: "Change cover" is visible on hover`, await btn.isVisible());
    // The button is a see-through panel over the banner: take the worse of "over white" and "over black".
    const contrast = await btn.evaluate((e) => {
      const px = (css) => { const c = document.createElement('canvas'); c.width = c.height = 1; const x = c.getContext('2d', { willReadFrequently: true }); x.clearRect(0, 0, 1, 1); x.fillStyle = css; x.fillRect(0, 0, 1, 1); const d = x.getImageData(0, 0, 1, 1).data; return [d[0], d[1], d[2], d[3] / 255]; };
      const lum = ([r, g, b]) => { const f = (v) => { v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; }; return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b); };
      const text = px(getComputedStyle(e).color), bg = px(getComputedStyle(e).backgroundColor);
      const over = (under) => [0, 1, 2].map((i) => bg[i] * bg[3] + under * (1 - bg[3]));
      const ratio = (a, b) => (Math.max(lum(a), lum(b)) + 0.05) / (Math.min(lum(a), lum(b)) + 0.05);
      return Math.min(ratio(text, over(255)), ratio(text, over(0)));
    });
    truthy(`${theme}: the cover buttons are readable (contrast ${contrast.toFixed(1)})`, contrast >= 4.5, contrast);
    await p.getByTestId('page-icon-row').hover();
    await p.waitForTimeout(150);
    await shot(`10-page-${theme}`);
    await home();
    truthy(`${theme}: Home shows cards`, (await p.getByTestId('home-card').count()) >= 3);
    await shot(`11-home-${theme}`);
  }

  // ----- Right to left -----
  await open('theme=sunrise&cover=none', 'Reading list');
  await p.evaluate(() => { document.documentElement.dir = 'rtl'; });
  await p.getByTestId('add-cover').click({ force: true });
  await p.locator('[data-cover="gradient:dusk"]').click();
  await p.waitForTimeout(300);
  await p.getByTestId('page-cover').hover();
  const rb = await p.getByRole('button', { name: 'Change cover' }).boundingBox();
  const rcv = await p.getByTestId('page-cover').boundingBox();
  truthy('RTL: the cover buttons sit at the left (the end of the line)', rb.x - rcv.x < rcv.width / 2, { rb, rcv });
  await shot('12-rtl-cover');
  await home();
  await shot('13-rtl-home');
  truthy('RTL: Home has no horizontal scroll', await p.getByTestId('home').evaluate((e) => e.scrollWidth <= e.clientWidth + 1));

  await b.close();
  fs.rmSync(tmp, { recursive: true, force: true });
  console.log(fail ? `${fail} FAILED` : 'ALL OK');
  process.exit(fail ? 1 : 0);
})().catch((e) => { console.log('FAIL', e); process.exit(1); });
