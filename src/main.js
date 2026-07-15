const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { open: openDialog, confirm: confirmDialog, message } = window.__TAURI__.dialog;

let rootPath = "";
const expanded = new Set();
const $ = (id) => document.getElementById(id);
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
  const div = $("drives");
  div.innerHTML = "<h2>Select a drive to scan</h2>";
  for (const d of drives) {
    const btn = document.createElement("button");
    btn.className = "drive-btn";
    btn.textContent = `${d.letter}  —  ${fmt(d.total - d.free)} used of ${fmt(d.total)}`;
    btn.onclick = () => startScan(d.letter + "\\");
    div.appendChild(btn);
  }
  show("drives");
}

async function startScan(drive) {
  rootPath = drive;
  expanded.clear();
  show("progress");
  await invoke("start_scan", { drive });
}

listen("scan-progress", (e) => {
  $("prog-text").textContent = `${e.payload.files.toLocaleString()} files · ${fmt(e.payload.bytes)}`;
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
      `<span class="bar"><span style="width:${pct}%"></span></span>` +
      `<span class="size">${fmt(c.size)}</span>`;
    const kids = document.createElement("div");
    row.onclick = async () => {
      if (!c.is_dir) return;
      if (expanded.has(path)) {
        expanded.delete(path);
        kids.innerHTML = "";
        row.querySelector(".arrow").textContent = "▸";
      } else {
        expanded.add(path);
        row.querySelector(".arrow").textContent = "▾";
        await renderLevel(kids, path, depth + 1);
      }
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
  input.onkeydown = async (e) => {
    if (e.key === "Enter") {
      try {
        await invoke("rename", { path, newName: input.value });
        await renderTree();
      } catch (err) {
        await message(String(err), { title: "Error", kind: "error" });
        await renderTree();
      }
    } else if (e.key === "Escape") {
      renderTree();
    }
  };
}

init();
