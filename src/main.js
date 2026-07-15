const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { open: openDialog, confirm: confirmDialog, message } = window.__TAURI__.dialog;

let rootPath = "";
const expanded = new Set();
const $ = (id) => document.getElementById(id);

// theme: saved preference wins, otherwise follow the OS
const savedTheme = localStorage.getItem("theme");
const osDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
document.documentElement.dataset.theme = savedTheme || (osDark ? "dark" : "light");
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
  for (const s of ["drives", "progress", "main"]) $(s).hidden = s !== id;
}

async function init() {
  const drives = await invoke("list_drives");
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
