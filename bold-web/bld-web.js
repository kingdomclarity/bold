#!/usr/bin/env node
/*
  bld web, the BOLD app layer compiler
  Version 1.0.0
  Built in-house by BOLD Studios. Compiles .bold source into a finished web app.

  Usage:
    node bld-web.js build <file.bold> [outDir]
    node bld-web.js check <file.bold>

  The BOLD app layer is declarative, readable, and built on a simple conviction:
  code should read like intent. You describe the app; bld web builds it
  with the BOLD design system, excellence by default.
*/

"use strict";

const fs = require("fs");
const path = require("path");

/* ------------------------------------------------------------------ */
/* Lexer: turn source lines into indented nodes                        */
/* ------------------------------------------------------------------ */

function lex(source) {
  const lines = source.split(/\r?\n/);
  const nodes = [];
  for (let i = 0; i < lines.length; i++) {
    const raw = lines[i];
    const noComment = stripComment(raw);
    if (!noComment.trim()) continue;
    const indent = noComment.match(/^\s*/)[0].replace(/\t/g, "  ").length;
    nodes.push({ line: i + 1, indent, text: noComment.trim() });
  }
  return nodes;
}

function stripComment(line) {
  // strip // comments that are not inside quotes
  let out = "";
  let inStr = false;
  for (let i = 0; i < line.length; i++) {
    const ch = line[i];
    if (ch === '"') inStr = !inStr;
    if (!inStr && ch === "/" && line[i + 1] === "/") break;
    out += ch;
  }
  return out;
}

/* Tokenize one statement: keywords, "strings", numbers, identifiers */
function tokens(text) {
  const re = /"([^"]*)"|(\d+(?:\.\d+)?)|([A-Za-z_][\w-]*)/g;
  const out = [];
  let m;
  while ((m = re.exec(text)) !== null) {
    if (m[1] !== undefined) out.push({ t: "str", v: m[1] });
    else if (m[2] !== undefined) out.push({ t: "num", v: parseFloat(m[2]) });
    else out.push({ t: "word", v: m[3] });
  }
  return out;
}

/* ------------------------------------------------------------------ */
/* Parser: nodes -> tree                                               */
/* ------------------------------------------------------------------ */

function parse(nodes) {
  const root = { kind: "root", children: [], indent: -1 };
  const stack = [root];
  const errors = [];

  for (const n of nodes) {
    const tk = tokens(n.text);
    if (!tk.length) continue;
    const node = { kind: tk[0].v, tk, line: n.line, indent: n.indent, children: [] };
    while (stack.length > 1 && stack[stack.length - 1].indent >= n.indent) stack.pop();
    stack[stack.length - 1].children.push(node);
    stack.push(node);
  }

  return { root, errors };
}

function str(node, i) {
  let count = 0;
  for (const t of node.tk) {
    if (t.t === "str") { if (count === i) return t.v; count++; }
  }
  return null;
}
function num(node) { const t = node.tk.find(t => t.t === "num"); return t ? t.v : null; }
function wordAfter(node, kw) {
  for (let i = 0; i < node.tk.length - 1; i++) {
    if (node.tk[i].t === "word" && node.tk[i].v === kw) return node.tk[i + 1].v;
  }
  return null;
}

/* ------------------------------------------------------------------ */
/* Themes: the BOLD design system                                      */
/* ------------------------------------------------------------------ */

const THEMES = {
  gold:     { bg: "#0a0a0f", panel: "#12121a", text: "#eceaf2", dim: "#9a97a8", accent: "#d9a94b", accent2: "#f0c975", onAccent: "#0b0b10" },
  midnight: { bg: "#070b14", panel: "#0e1424", text: "#e8ecf6", dim: "#8d97ad", accent: "#60a5fa", accent2: "#93c5fd", onAccent: "#06080f" },
  ember:    { bg: "#0e0806", panel: "#1a100c", text: "#f4ece8", dim: "#a89a93", accent: "#f97316", accent2: "#fdba74", onAccent: "#140a06" },
  light:    { bg: "#faf9f7", panel: "#ffffff", text: "#1a1820", dim: "#6f6b7d", accent: "#b98a2e", accent2: "#8a6620", onAccent: "#ffffff" }
};

/* ------------------------------------------------------------------ */
/* Code generator                                                      */
/* ------------------------------------------------------------------ */

function esc(s) {
  return String(s == null ? "" : s)
    .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function slug(s) {
  return String(s).toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "page";
}

function compile(source, opts) {
  opts = opts || {};
  const { root } = parse(lex(source));
  const errors = [];
  const warnings = [];

  const appNode = root.children.find(c => c.kind === "app");
  if (!appNode) errors.push("Every BOLD app starts with: app \"Name\"");
  const appName = appNode ? (str(appNode, 0) || "BOLD App") : "BOLD App";

  let themeName = "gold";
  const dataSets = {};
  const pages = [];

  const scan = (node) => {
    for (const c of node.children) {
      if (c.kind === "theme") {
        const w = c.tk.find(t => t.t === "word" && t.v !== "theme" && t.v !== "at");
        if (w && THEMES[w.v.toLowerCase()]) themeName = w.v.toLowerCase();
        else if (w) warnings.push("Line " + c.line + ": unknown theme \"" + w.v + "\", using gold");
      }
      if (c.kind === "data") {
        const name = wordAfter(c, "data") || "data" + c.line;
        dataSets[name] = c.children
          .filter(x => x.kind === "item")
          .map(x => ({ title: str(x, 0) || "", text: str(x, 1) || "", extra: str(x, 2) || "" }));
      }
      if (c.kind === "page") {
        const pname = wordAfter(c, "page") || "Page" + pages.length;
        const route = str(c, 0) || "/" + slug(pname);
        pages.push({ name: pname, route, node: c });
      }
    }
  };
  scan(root);
  if (appNode) scan(appNode);

  if (!pages.length) errors.push("No pages found. Add: page Home at \"/\"");
  if (errors.length) return { ok: false, errors, warnings };

  const theme = THEMES[themeName];
  const routeOf = {};
  pages.forEach(p => { routeOf[p.name] = p.route; });

  const linkFor = (node) => {
    const target = wordAfter(node, "goto");
    if (target && routeOf[target]) return "#" + routeOf[target];
    if (target) { warnings.push("Line " + node.line + ": goto " + target + " does not match a page"); return "#/"; }
    const href = str(node, 1);
    return href || "#/";
  };

  /* ----- element renderers ----- */
  function renderChildren(node) {
    return node.children.map(renderEl).join("\n");
  }

  function renderEl(el) {
    switch (el.kind) {
      case "hero": {
        return '<header class="b-hero">' + renderChildren(el) + "</header>";
      }
      case "title":
        return '<h1 class="b-title">' + esc(str(el, 0)) + "</h1>";
      case "subtitle":
        return '<p class="b-subtitle">' + esc(str(el, 0)) + "</p>";
      case "heading":
        return '<h2 class="b-heading">' + esc(str(el, 0)) + "</h2>";
      case "text":
        return '<p class="b-text">' + esc(str(el, 0)) + "</p>";
      case "button":
        return '<a class="b-btn" href="' + esc(linkFor(el)) + '">' + esc(str(el, 0)) + "</a>";
      case "link":
        return '<a class="b-link" href="' + esc(linkFor(el)) + '">' + esc(str(el, 0)) + "</a>";
      case "section": {
        const label = str(el, 0);
        return '<section class="b-section">' +
          (label ? '<h2 class="b-heading">' + esc(label) + "</h2>" : "") +
          renderChildren(el) + "</section>";
      }
      case "grid": {
        const cols = num(el) || 3;
        const from = wordAfter(el, "from");
        let inner = "";
        if (from && dataSets[from]) {
          inner = dataSets[from].map(d =>
            '<div class="b-card"><h3>' + esc(d.title) + "</h3><p>" + esc(d.text) + "</p></div>").join("");
        } else {
          inner = renderChildren(el);
        }
        return '<div class="b-grid" style="grid-template-columns:repeat(' + cols + ',1fr)">' + inner + "</div>";
      }
      case "card": {
        return '<div class="b-card"><h3>' + esc(str(el, 0)) + "</h3><p>" + esc(str(el, 1) || "") + "</p>" +
          renderChildren(el) + "</div>";
      }
      case "stat": {
        return '<div class="b-stat"><div class="b-stat-num">' + esc(str(el, 1) || "0") + '</div><div class="b-stat-label">' + esc(str(el, 0)) + "</div></div>";
      }
      case "row":
        return '<div class="b-row">' + renderChildren(el) + "</div>";
      case "image": {
        const src = str(el, 0) || "";
        return '<img class="b-img" src="' + esc(src) + '" alt="' + esc(str(el, 1) || appName) + '">';
      }
      case "list": {
        const from = wordAfter(el, "from");
        if (from && dataSets[from]) {
          return '<ul class="b-list">' + dataSets[from].map(d => "<li><b>" + esc(d.title) + "</b> " + esc(d.text) + "</li>").join("") + "</ul>";
        }
        return '<ul class="b-list">' + el.children.map(c => "<li>" + esc(str(c, 0)) + "</li>").join("") + "</ul>";
      }
      case "cta": {
        return '<div class="b-cta"><a class="b-btn" href="' + esc(linkFor(el)) + '">' + esc(str(el, 0)) + "</a></div>";
      }
      case "verse": {
        const v = str(el, 0), ref = str(el, 1);
        return '<blockquote class="b-verse">&ldquo;' + esc(v) + "&rdquo;" + (ref ? '<cite>' + esc(ref) + "</cite>" : "") + "</blockquote>";
      }
      case "spacer":
        return '<div style="height:' + (num(el) || 40) + 'px"></div>';
      case "item":
        return ""; // handled by parent
      default:
        warnings.push("Line " + el.line + ": unknown element \"" + el.kind + "\" was skipped");
        return "";
    }
  }

  /* ----- pages ----- */
  const pageHtml = pages.map(p =>
    '<main class="b-page" data-route="' + esc(p.route) + '" id="page-' + slug(p.name) + '" hidden>' +
    renderChildren(p.node) +
    "</main>"
  ).join("\n");

  const nav = pages.length > 1
    ? '<nav class="b-nav">' + pages.map(p => '<a href="#' + esc(p.route) + '" data-nav="' + esc(p.route) + '">' + esc(p.name) + "</a>").join("") + "</nav>"
    : "";

  const html = `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>${esc(appName)}</title>
<meta name="description" content="${esc(appName)}, built with BOLD by BOLD Studios.">
<meta property="og:title" content="${esc(appName)}">
<meta property="og:type" content="website">
<style>
:root{--bg:${theme.bg};--panel:${theme.panel};--text:${theme.text};--dim:${theme.dim};--accent:${theme.accent};--accent2:${theme.accent2};--on-accent:${theme.onAccent}}
*{margin:0;padding:0;box-sizing:border-box}
body{font-family:-apple-system,BlinkMacSystemFont,"SF Pro Display","Segoe UI",Inter,sans-serif;background:var(--bg);color:var(--text);-webkit-font-smoothing:antialiased;min-height:100vh;display:flex;flex-direction:column}
.b-brand{display:flex;align-items:center;justify-content:space-between;padding:20px 6vw;border-bottom:1px solid color-mix(in srgb,var(--text) 8%,transparent)}
.b-brand b{letter-spacing:.12em;font-size:14px;background:linear-gradient(120deg,var(--accent2),var(--accent));-webkit-background-clip:text;background-clip:text;color:transparent}
.b-nav{display:flex;gap:22px}
.b-nav a{color:var(--dim);text-decoration:none;font-size:13px;letter-spacing:.04em;transition:color .2s}
.b-nav a:hover,.b-nav a.on{color:var(--accent2)}
.b-page{flex:1;width:min(1060px,92vw);margin:0 auto;padding:48px 0 80px;animation:fade .5s ease}
@keyframes fade{from{opacity:0;transform:translateY(10px)}to{opacity:1;transform:none}}
.b-hero{text-align:center;padding:72px 0 56px}
.b-title{font-size:clamp(34px,6vw,64px);font-weight:800;letter-spacing:-.02em;line-height:1.08;background:linear-gradient(150deg,var(--text),color-mix(in srgb,var(--text) 55%,var(--accent)));-webkit-background-clip:text;background-clip:text;color:transparent}
.b-subtitle{margin-top:18px;font-size:clamp(15px,2vw,19px);color:var(--dim);max-width:620px;margin-left:auto;margin-right:auto;line-height:1.6}
.b-hero .b-btn{margin-top:32px}
.b-heading{font-size:clamp(22px,3vw,30px);font-weight:700;letter-spacing:-.01em;margin:48px 0 20px}
.b-text{color:var(--dim);line-height:1.75;max-width:680px;margin:12px 0}
.b-btn{display:inline-block;background:linear-gradient(120deg,var(--accent2),var(--accent));color:var(--on-accent);font-weight:700;font-size:14px;padding:13px 30px;border-radius:12px;text-decoration:none;letter-spacing:.02em;transition:transform .18s ease,box-shadow .18s ease;box-shadow:0 8px 28px color-mix(in srgb,var(--accent) 30%,transparent)}
.b-btn:hover{transform:translateY(-2px);box-shadow:0 14px 36px color-mix(in srgb,var(--accent) 42%,transparent)}
.b-link{color:var(--accent2);text-decoration:none}
.b-grid{display:grid;gap:18px;margin:22px 0}
.b-card{background:var(--panel);border:1px solid color-mix(in srgb,var(--text) 8%,transparent);border-radius:18px;padding:26px;transition:transform .2s ease,border-color .2s ease}
.b-card:hover{transform:translateY(-3px);border-color:color-mix(in srgb,var(--accent) 45%,transparent)}
.b-card h3{font-size:17px;margin-bottom:8px}
.b-card p{color:var(--dim);font-size:13.5px;line-height:1.6}
.b-row{display:flex;gap:18px;flex-wrap:wrap;margin:22px 0}
.b-stat{background:var(--panel);border:1px solid color-mix(in srgb,var(--text) 8%,transparent);border-radius:18px;padding:24px 30px;min-width:170px}
.b-stat-num{font-size:34px;font-weight:800;color:var(--accent2)}
.b-stat-label{color:var(--dim);font-size:12px;letter-spacing:.08em;margin-top:6px;text-transform:uppercase}
.b-list{margin:16px 0 16px 20px;color:var(--dim);line-height:2}
.b-cta{text-align:center;padding:44px 0}
.b-verse{margin:44px auto;max-width:640px;text-align:center;font-size:17px;line-height:1.7;color:var(--dim);font-style:italic;border-top:1px solid color-mix(in srgb,var(--accent) 35%,transparent);border-bottom:1px solid color-mix(in srgb,var(--accent) 35%,transparent);padding:26px 10px}
.b-verse cite{display:block;margin-top:10px;font-size:12px;letter-spacing:.14em;color:var(--accent2);font-style:normal;text-transform:uppercase}
.b-img{max-width:100%;border-radius:18px;margin:20px 0}
.b-footer{border-top:1px solid color-mix(in srgb,var(--text) 8%,transparent);padding:26px 6vw;text-align:center;font-size:12px;color:var(--dim)}
.b-footer a{color:var(--accent2);text-decoration:none}
@media(max-width:760px){.b-grid{grid-template-columns:1fr!important}.b-brand{flex-direction:column;gap:14px}}
</style>
</head>
<body>
<div class="b-brand"><b>${esc(appName).toUpperCase()}</b>${nav}</div>
${pageHtml}
<footer class="b-footer">Designed and Developed by <a href="https://BOLDStudios.io">BOLD Studios</a>. Built with BOLD.</footer>
<script>
(function(){
  var pages=document.querySelectorAll(".b-page");
  function route(){
    var h=location.hash.replace(/^#/,"")||"${esc(pages[0].route)}";
    var found=false;
    pages.forEach(function(p){var on=p.getAttribute("data-route")===h;p.hidden=!on;if(on)found=true;});
    if(!found){pages.forEach(function(p,i){p.hidden=i!==0;});}
    document.querySelectorAll("[data-nav]").forEach(function(a){a.classList.toggle("on",a.getAttribute("data-nav")===h);});
    window.scrollTo(0,0);
  }
  window.addEventListener("hashchange",route);
  route();
})();
<\/script>
</body>
</html>`;

  return { ok: true, html, appName, theme: themeName, pages: pages.map(p => p.name), warnings };
}

/* ------------------------------------------------------------------ */
/* CLI                                                                 */
/* ------------------------------------------------------------------ */

function main() {
  const [, , cmd, file, outDir] = process.argv;
  if (!cmd || !file || ["build", "check"].indexOf(cmd) === -1) {
    console.log("bld web 1.0.0, the BOLD app layer compiler");
    console.log("Usage: node bld-web.js build <file.bold> [outDir]");
    console.log("       node bld-web.js check <file.bold>");
    process.exit(1);
  }
  const source = fs.readFileSync(file, "utf8");
  const t0 = Date.now();
  const result = compile(source);

  result.warnings.forEach(w => console.log("  warn  " + w));
  if (!result.ok) {
    result.errors.forEach(e => console.error("  error " + e));
    process.exit(1);
  }
  if (cmd === "check") {
    console.log("  ok    " + result.appName + " compiles clean. Pages: " + result.pages.join(", "));
    return;
  }
  const dir = outDir || path.join(path.dirname(file), "dist");
  fs.mkdirSync(dir, { recursive: true });
  const out = path.join(dir, "index.html");
  fs.writeFileSync(out, result.html);
  console.log("  ok    Built \"" + result.appName + "\" (" + result.pages.length + " pages, theme " + result.theme + ") in " + (Date.now() - t0) + "ms");
  console.log("  out   " + out);
}

if (require.main === module) main();
module.exports = { compile, lex, parse, THEMES };
