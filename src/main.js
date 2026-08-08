const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { open: openDialog, confirm: confirmDialog, message } = window.__TAURI__.dialog;

let rootPath = "";
const expanded = new Set();
const $ = (id) => document.getElementById(id);

// theme is applied in index.html's head script (pre-paint); this only toggles it
$("theme-toggle").onclick = () => {
  const next = document.documentElement.dataset.theme === "dark" ? "light" : "dark";
  document.documentElement.dataset.theme = next;
  localStorage.setItem("theme", next);
};
const esc = (s) => s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));

function fmt(bytes) {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let i = 0, v = bytes;
  while (v >= 1024 && i < units.length - 1) { v /= 1024; i++; }
  return v.toFixed(i ? 1 : 0) + " " + units[i];
}

function show(id) {
  for (const s of ["splash", "drives", "progress", "main"]) $(s).hidden = s !== id;
}

// Drive enumeration takes ~20ms, so without a floor the splash is gone before the
// eye registers it. ponytail: 500ms is a taste value — lower it if launch feels slow.
const MIN_SPLASH_MS = 500;
const bootAt = Date.now();
let firstInit = true;

async function init() {
  const drives = await invoke("list_drives");
  if (firstInit) {
    firstInit = false; // only the launch splash waits; "Change drive" is instant
    const left = MIN_SPLASH_MS - (Date.now() - bootAt);
    if (left > 0) await new Promise((r) => setTimeout(r, left));
  }
  $("switch-drive").hidden = drives.length === 1;
  if (drives.length === 1) return startScan(drives[0].letter + "\\");
  const list = document.querySelector(".drive-list");
  list.innerHTML = "";
  for (const d of drives) {
    const used = d.total - d.free;
    const pct = d.total ? (used / d.total) * 100 : 0;
    const btn = document.createElement("button");
    btn.className = "drive-row";
    btn.innerHTML =
      `<span class="drive-letter">${esc(d.letter)}</span>` +
      `<span class="drive-usage">${fmt(used)} / ${fmt(d.total)}</span>` +
      `<span class="drive-bar"><i style="width:${pct.toFixed(1)}%"></i></span>`;
    btn.onclick = () => startScan(d.letter + "\\");
    list.appendChild(btn);
  }
  show("drives");
}

// smooth number tween (animated-number style: no bounce, ease-out between updates)
function animatedNumber(render) {
  let shown = 0;
  let raf = null;
  function set(target) {
    const from = shown;
    const t0 = performance.now();
    cancelAnimationFrame(raf);
    (function tick(now) {
      const p = Math.min((now - t0) / 300, 1);
      shown = from + (target - from) * (1 - Math.pow(1 - p, 3));
      render(shown);
      if (p < 1) raf = requestAnimationFrame(tick);
    })(t0);
  }
  function reset() {
    cancelAnimationFrame(raf);
    shown = 0;
    render(0);
  }
  return { set, reset };
}

const bytesCounter = animatedNumber((v) => { $("prog-bytes").textContent = fmt(v); });
const filesCounter = animatedNumber((v) => { $("prog-files").textContent = `${Math.round(v).toLocaleString()} files`; });

async function startScan(drive) {
  rootPath = drive;
  expanded.clear();
  $("scan-target").textContent = `Scanning ${drive}`;
  bytesCounter.reset();
  filesCounter.reset();
  show("progress");
  await invoke("start_scan", { drive });
}

listen("scan-progress", (e) => {
  bytesCounter.set(e.payload.bytes);
  filesCounter.set(e.payload.files);
});

listen("scan-done", async (e) => {
  $("root-label").textContent = rootPath;
  $("scan-summary").textContent =
    `${e.payload.files.toLocaleString()} files · ${fmt(e.payload.bytes)}` +
    (e.payload.denied ? ` · ${e.payload.denied} folders unscanned` : "");
  show("main");
  await renderTree();
});

$("rescan").onclick = () => startScan(rootPath);
$("switch-drive").onclick = () => init();

const joinPath = (parent, name) => (parent.endsWith("\\") ? parent + name : parent + "\\" + name);

// ---------- window sizing ----------
//
// The window is not user-resizable (tauri.conf.json "resizable": false); it sizes itself
// to the tree. Width is driven by the longest visible name at its indent depth, because a
// truncated folder name is the one thing you cannot work around in a read-only view.
// Height follows the row count. Both shrink again when folders collapse.

const { getCurrentWindow, PhysicalSize } = window.__TAURI__.window;

const MIN_W = 640;
// Height stays put at the original window height from tauri.conf.json — only width
// adapts. Long trees scroll inside #tree rather than growing the window.
const FIXED_H = 700;
const SLACK = 8; // a hair of air so the longest name never sits flush against the size column

// Text measured on a canvas rather than in the DOM: .name is overflow:hidden, so its
// scrollWidth reports the clipped width and can never tell us the window is too wide.
const meter = document.createElement("canvas").getContext("2d");
let meterFont = "";

// Title bar and borders, in CSS px. setSize takes the inner size, so the frame has to be
// subtracted from the work area or the window ends up taller than the screen.
let frame = null;
async function windowFrame() {
  if (!frame) {
    const win = getCurrentWindow();
    const [outer, inner] = [await win.outerSize(), await win.innerSize()];
    const dpr = window.devicePixelRatio || 1;
    frame = { w: (outer.width - inner.width) / dpr, h: (outer.height - inner.height) / dpr };
  }
  return frame;
}

/// Width the widest row needs, in CSS px, or null when there is nothing to measure.
function contentWidth(rows) {
  const first = rows[0].querySelector(".name");
  if (!first) return null;
  if (!meterFont) {
    const cs = getComputedStyle(first);
    meterFont = `${cs.fontWeight} ${cs.fontSize} ${cs.fontFamily}`;
  }
  meter.font = meterFont;
  // Everything that is not the name or the left indent: arrow, gaps, pct, bar, size,
  // right padding. Derived from a real row so it survives any CSS change.
  const pad0 = parseFloat(rows[0].style.paddingLeft) || 0;
  const fixed = rows[0].getBoundingClientRect().width - pad0 - first.getBoundingClientRect().width;

  let widest = 0;
  for (const row of rows) {
    const name = row.querySelector(".name");
    if (!name || name.querySelector("input")) continue; // mid-rename, width is meaningless
    const pad = parseFloat(row.style.paddingLeft) || 0;
    widest = Math.max(widest, pad + meter.measureText(name.textContent).width);
  }
  return widest + fixed + SLACK;
}

async function fitWindowNow() {
  if ($("main").hidden) return;
  const rows = $("tree").querySelectorAll(".row");
  if (!rows.length) return;

  const needW = contentWidth(rows);
  if (needW == null) return;

  // never spill off the work area, never collapse to unusable
  const { w: frameW, h: frameH } = await windowFrame();
  const w = Math.round(Math.min(Math.max(needW, MIN_W), screen.availWidth - frameW));
  const h = Math.round(Math.min(FIXED_H, screen.availHeight - frameH));
  if (Math.abs(w - window.innerWidth) < 2 && Math.abs(h - window.innerHeight) < 2) return;

  // setSize takes the INNER size — Tauri adds the frame itself, so asking for
  // w+chrome overshoots by exactly the title bar and borders.
  const dpr = window.devicePixelRatio || 1;
  await getCurrentWindow().setSize(new PhysicalSize(Math.round(w * dpr), Math.round(h * dpr)));
}

// Expand/collapse animates for 260ms; resizing mid-animation looks like a stutter, and
// row counts churn during a burst of clicks. One trailing fit settles it.
let fitTimer = null;
function fitWindow(delay = 300) {
  clearTimeout(fitTimer);
  fitTimer = setTimeout(() => fitWindowNow().catch(() => {}), delay);
}

// accordion-style height easing for folder expand/collapse.
// Runs even under prefers-reduced-motion by explicit user choice.
const EASE = "height 260ms cubic-bezier(0.25, 1, 0.5, 1)";

function settleAfter(kids, ms, cleanup) {
  // transitionend can be missed (display changes, interrupted transitions);
  // a timer fallback guarantees styles never stay stuck
  let done = false;
  const finish = () => {
    if (done) return;
    done = true;
    cleanup();
  };
  kids.addEventListener("transitionend", (e) => { if (e.target === kids) finish(); }, { once: true });
  setTimeout(finish, ms + 80);
}

function expandKids(kids) {
  kids.style.overflow = "hidden";
  kids.style.height = "0px";
  void kids.offsetHeight; // commit the start state before transitioning
  kids.style.transition = EASE;
  kids.style.height = kids.scrollHeight + "px";
  settleAfter(kids, 260, () => { kids.style.cssText = ""; });
}

function collapseKids(kids) {
  kids.style.overflow = "hidden";
  kids.style.height = kids.scrollHeight + "px";
  void kids.offsetHeight;
  kids.style.transition = EASE;
  kids.style.height = "0px";
  settleAfter(kids, 260, () => { kids.innerHTML = ""; kids.style.cssText = ""; });
}

async function renderTree() {
  const tree = $("tree");
  tree.innerHTML = "";
  await renderLevel(tree, rootPath, 0);
  fitWindow(0); // nothing is animating on a full render
}

async function renderLevel(container, parentPath, depth) {
  let children;
  try {
    children = await invoke("get_children", { path: parentPath });
  } catch {
    return;
  }
  const total = children.reduce((s, c) => s + c.size, 0) || 1;
  let idx = 0;
  for (const c of children) {
    const path = joinPath(parentPath, c.name);
    const pct = (c.size / total) * 100;
    const row = document.createElement("div");
    row.className = "row" + (c.is_dir ? " dir" : "");
    row.style.paddingLeft = depth * 20 + 8 + "px";
    row.innerHTML =
      `<span class="arrow">${c.is_dir ? (expanded.has(path) ? "▾" : "▸") : ""}</span>` +
      `<span class="name" title="${esc(path)}">${esc(c.name)}</span>` +
      `<span class="pct">${pct.toFixed(1)}%</span>` +
      `<span class="bar"><i style="width:${pct}%"></i></span>` +
      `<span class="size">${fmt(c.size)}</span>`;
    const kids = document.createElement("div");
    row.onclick = async () => {
      if (!c.is_dir) return;
      if (expanded.has(path)) {
        expanded.delete(path);
        row.querySelector(".arrow").textContent = "▸";
        collapseKids(kids);
      } else {
        expanded.add(path);
        row.querySelector(".arrow").textContent = "▾";
        await renderLevel(kids, path, depth + 1);
        expandKids(kids);
      }
      fitWindow(); // after the 260ms accordion settles
    };
    row.ondblclick = () => {
      if (c.is_dir) return; // folders expand on single click
      invoke("open_file", { path }).catch(async (err) => {
        await message(String(err), { title: "Error", kind: "error" });
      });
    };
    row.oncontextmenu = (ev) => {
      ev.preventDefault();
      ev.stopPropagation();
      showMenu(ev, path, row);
    };
    container.appendChild(row);
    container.appendChild(kids);
    if (c.is_dir && expanded.has(path)) await renderLevel(kids, path, depth + 1);
  }
}

function showMenu(ev, path, row) {
  const menu = $("menu");
  menu.style.left = Math.min(ev.pageX, window.innerWidth - 200) + "px";
  menu.style.top = Math.min(ev.pageY, window.innerHeight - 180) + "px";
  menu.hidden = false;
  menu.onclick = async (e2) => {
    menu.hidden = true;
    const act = e2.target.dataset.act;
    if (!act) return;
    try {
      if (act === "open") {
        await invoke("open_file", { path });
      } else if (act === "reveal") {
        await invoke("reveal", { path });
      } else if (act === "delete") {
        const ok = await confirmDialog(`Move to Recycle Bin?\n\n${path}`, { title: "Delete", kind: "warning" });
        if (ok) {
          await invoke("delete", { path });
          await renderTree();
        }
      } else if (act === "rename") {
        startRename(row, path);
      } else if (act === "move") {
        const dest = await openDialog({
          directory: true,
          title: "Move to folder",
          defaultPath: path.slice(0, path.lastIndexOf("\\")) || rootPath,
        });
        if (dest) {
          await invoke("move_item", { path, destDir: dest });
          await renderTree();
        }
      }
    } catch (err) {
      await message(String(err), { title: "Error", kind: "error" });
    }
  };
}

document.addEventListener("click", () => { $("menu").hidden = true; });

function startRename(row, path) {
  const span = row.querySelector(".name");
  const input = document.createElement("input");
  input.value = path.split("\\").pop();
  span.textContent = "";
  span.appendChild(input);
  input.focus();
  const dot = input.value.lastIndexOf(".");
  if (!row.classList.contains("dir") && dot > 0) {
    input.setSelectionRange(0, dot); // highlight name only, keep extension
  } else {
    input.select();
  }
  input.onclick = (e) => e.stopPropagation();
  let finished = false; // guards blur firing after Enter/Escape already handled it
  input.onblur = () => {
    if (!finished) {
      finished = true;
      renderTree();
    }
  };
  input.onkeydown = async (e) => {
    if (e.key === "Enter") {
      finished = true;
      try {
        await invoke("rename", { path, newName: input.value });
        await renderTree();
      } catch (err) {
        await message(String(err), { title: "Error", kind: "error" });
        await renderTree();
      }
    } else if (e.key === "Escape") {
      finished = true;
      renderTree();
    }
  };
}

init();
