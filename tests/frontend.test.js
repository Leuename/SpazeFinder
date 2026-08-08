// Frontend smoke test. No framework, no deps — run with `node tests/frontend.test.js`.
// Loads src/main.js in a stubbed DOM so a broken element id, a syntax error, or a
// bad show() state fails here instead of in a silent blank window.
//
// ponytail: stub only what main.js touches. Swap in jsdom if the UI grows real
// layout logic worth asserting on.

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");

const SRC = path.join(__dirname, "..", "src");
const html = fs.readFileSync(path.join(SRC, "index.html"), "utf8");
const js = fs.readFileSync(path.join(SRC, "main.js"), "utf8");

let passed = 0;
const ok = (name) => { passed++; console.log(`ok   ${name}`); };
const bad = (name, e) => { console.error(`FAIL ${name}\n     ${e.message}`); process.exitCode = 1; };

function test(name, fn) {
  try {
    const r = fn();
    // async tests must be awaited, or a rejection reports as a pass
    if (r && typeof r.then === "function") return void r.then(() => ok(name), (e) => bad(name, e));
    ok(name);
  } catch (e) {
    bad(name, e);
  }
}

// ---------- static checks on index.html ----------

const ids = new Set([...html.matchAll(/id="([^"]+)"/g)].map((m) => m[1]));

test("every element main.js looks up exists in index.html", () => {
  const looked = new Set([...js.matchAll(/\$\("([^"]+)"\)/g)].map((m) => m[1]));
  const missing = [...looked].filter((id) => !ids.has(id));
  assert.deepEqual(missing, [], `main.js references missing ids: ${missing}`);
});

test("splash is visible at rest so it paints before main.js runs", () => {
  const tag = html.match(/<section id="splash"[^>]*>/)[0];
  assert.ok(!tag.includes("hidden"), "splash must not be hidden in markup");
});

test("the other sections start hidden", () => {
  for (const id of ["drives", "progress", "main"]) {
    const tag = html.match(new RegExp(`<section id="${id}"[^>]*>`))[0];
    assert.ok(tag.includes("hidden"), `#${id} should start hidden`);
  }
});

test("theme is set in head, before body renders", () => {
  const head = html.slice(0, html.indexOf("</head>"));
  assert.match(head, /dataset\.theme/, "theme must be applied in a head script");
  assert.match(head, /localStorage\.getItem\("theme"\)/, "head must read the saved theme");
  // main.js still writes the theme from the toggle; it must not be the one to
  // restore it at load, because by then the wrong theme has already painted.
  assert.doesNotMatch(js, /localStorage\.getItem\("theme"\)/,
    "main.js must not restore the saved theme at load — that is the flash");
});

test("icon asset exists and every reference points at it", () => {
  const srcs = [...html.matchAll(/<img[^>]*src="([^"]+)"/g)].map((m) => m[1]);
  assert.ok(srcs.length > 0, "no <img> in the markup at all");
  for (const src of new Set(srcs)) {
    assert.ok(fs.existsSync(path.join(SRC, src)), `${src} missing from src/`);
  }
});

test("the icon appears on the splash and on the scan screen", () => {
  for (const id of ["splash", "progress"]) {
    const start = html.indexOf(`<section id="${id}"`);
    const section = html.slice(start, html.indexOf("</section>", start));
    assert.match(section, /<img class="mark"/, `#${id} is missing the icon`);
  }
});

// The window is created hidden so WebView2's ~700ms white boot rectangle never
// shows. That only works if all three pieces stay in place together.
test("window stays hidden until the page paints", () => {
  const conf = JSON.parse(fs.readFileSync(path.join(SRC, "..", "src-tauri", "tauri.conf.json"), "utf8"));
  const win = conf.app.windows.find((w) => w.label === "main");
  assert.equal(win.visible, false, "main window must be created hidden");

  const caps = JSON.parse(fs.readFileSync(
    path.join(SRC, "..", "src-tauri", "capabilities", "default.json"), "utf8"));
  assert.ok(caps.permissions.includes("core:window:allow-show"),
    "hidden window needs core:window:allow-show or it can never appear");

  assert.match(html, /getCurrentWindow\(\)\.show\(\)/,
    "nothing calls show() — the window would stay invisible forever");
  // rAF alone fires before the splash PNG decodes, revealing an empty window.
  assert.match(html, /\.decode\(\)/,
    "show() must wait on image decode, not just a frame");
});

// ---------- run main.js against a stubbed DOM ----------

function makeEl(id = "") {
  const el = {
    id,
    hidden: false,
    textContent: "",
    innerHTML: "",
    value: "",
    style: { cssText: "" },
    dataset: {},
    classList: { contains: () => false },
    children: [],
    scrollHeight: 10,
    offsetHeight: 10,
    querySelector: () => makeEl(),
    querySelectorAll: () => [],
    appendChild(c) { this.children.push(c); return c; },
    addEventListener() {},
    removeEventListener() {},
    setSelectionRange() {},
    select() {},
    focus() {},
  };
  return el;
}

function run(invokeImpl) {
  const els = new Map([...ids].map((id) => [id, makeEl(id)]));
  const listeners = new Map();
  const ctx = {
    console,
    setTimeout,
    clearTimeout,
    performance: { now: () => 0 },
    requestAnimationFrame: () => 0,
    cancelAnimationFrame: () => {},
    localStorage: { getItem: () => null, setItem() {} },
    document: {
      documentElement: { dataset: {} },
      getElementById: (id) => els.get(id) ?? null,
      createElement: () => makeEl(),
      querySelector: () => makeEl(),
      addEventListener() {},
    },
  };
  ctx.window = {
    innerWidth: 1000,
    innerHeight: 700,
    matchMedia: () => ({ matches: false }),
    __TAURI__: {
      core: { invoke: invokeImpl },
      event: { listen: (name, cb) => listeners.set(name, cb) },
      dialog: { open: async () => null, confirm: async () => false, message: async () => {} },
    },
  };
  ctx.globalThis = ctx;
  vm.createContext(ctx);
  vm.runInContext(js, ctx, { filename: "main.js" });
  return { ctx, els, listeners };
}

test("main.js loads and wires up without throwing", () => {
  run(async (cmd) => (cmd === "list_drives" ? [] : undefined));
});

test("show() reveals exactly one section, splash included", () => {
  const { ctx, els } = run(async () => []);
  for (const target of ["splash", "drives", "progress", "main"]) {
    ctx.show(target);
    for (const id of ["splash", "drives", "progress", "main"]) {
      assert.equal(els.get(id).hidden, id !== target,
        `show("${target}") left #${id}.hidden = ${els.get(id).hidden}`);
    }
  }
});

test("splash holds long enough to be seen, then yields to the drive picker", async () => {
  const { els } = run(async (cmd) =>
    cmd === "list_drives" ? [{ letter: "C:", total: 100, free: 40 }, { letter: "D:", total: 50, free: 50 }] : undefined);

  await new Promise((r) => setTimeout(r, 150));
  assert.equal(els.get("splash").hidden, false, "splash vanished before MIN_SPLASH_MS");

  await new Promise((r) => setTimeout(r, 600));
  assert.equal(els.get("splash").hidden, true, "splash should be gone after MIN_SPLASH_MS");
  assert.equal(els.get("drives").hidden, false, "drive picker should be showing");
});

test("fmt() scales units and keeps bytes whole", () => {
  const { ctx } = run(async () => []);
  assert.equal(ctx.fmt(0), "0 B");
  assert.equal(ctx.fmt(999), "999 B");
  assert.equal(ctx.fmt(1024), "1.0 KB");
  assert.equal(ctx.fmt(1536), "1.5 KB");
  assert.equal(ctx.fmt(1024 ** 3), "1.0 GB");
  assert.equal(ctx.fmt(1024 ** 5), "1024.0 TB"); // saturates at TB by design
});

process.on("exit", () => {
  console.log(`\n${passed} passed${process.exitCode ? ", failures above" : ""}`);
});
