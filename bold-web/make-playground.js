// Builds bold-playground.html by embedding the real BOLD app layer compiler.
"use strict";
const fs = require("fs");

const compilerSrc = fs.readFileSync(__dirname + "/bld-web.js", "utf8").replace(/^#![^\n]*\n/, "");

const DEFAULT_PROGRAM = fs.readFileSync(__dirname + "/examples/his-glory.bold", "utf8");

const html = `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>BOLD Playground | Write BOLD, see it live</title>
<meta name="description" content="The official BOLD playground by BOLD Studios. Write BOLD on the left, watch your app build live on the right.">
<style>
:root{--bg:#08080c;--bg2:#0c0c12;--panel:#101018;--line:rgba(255,255,255,.07);--text:#eceaf2;--dim:#9a97a8;--faint:#605d70;--gold:#d9a94b;--gold2:#f0c975;--mono:"SF Mono",ui-monospace,Menlo,Consolas,monospace;--sans:-apple-system,BlinkMacSystemFont,"SF Pro Display","Segoe UI",Inter,sans-serif}
*{margin:0;padding:0;box-sizing:border-box}
html,body{height:100%}
body{font-family:var(--sans);background:var(--bg);color:var(--text);overflow:hidden;-webkit-font-smoothing:antialiased}
.top{display:flex;align-items:center;gap:16px;height:52px;padding:0 20px;border-bottom:1px solid var(--line);background:linear-gradient(180deg,#101017,#0b0b10)}
.wordmark{font-weight:800;letter-spacing:.14em;font-size:13px}
.wordmark b{background:linear-gradient(120deg,var(--gold2),var(--gold));-webkit-background-clip:text;background-clip:text;color:transparent}
.wordmark span{font-weight:300;color:var(--dim)}
.tabs{display:flex;gap:4px;margin-left:14px}
.tab{font-size:12px;color:var(--dim);padding:7px 16px;border-radius:9px;cursor:pointer;border:1px solid transparent}
.tab.on{color:var(--gold2);background:rgba(217,169,75,.12);border-color:rgba(217,169,75,.3)}
.top .grow{flex:1}
.status{font-size:11px;color:var(--faint)}
.status .ok{color:#4ade80}.status .err{color:#fb7185}
.wrap{display:grid;grid-template-columns:1fr 1fr;height:calc(100vh - 52px - 30px)}
.pane{display:flex;flex-direction:column;min-width:0;min-height:0}
.pane-head{font-size:10px;letter-spacing:.2em;font-weight:700;color:var(--faint);padding:12px 18px;border-bottom:1px solid var(--line);background:var(--bg2)}
textarea#src{flex:1;background:var(--bg);color:var(--text);border:0;outline:0;resize:none;padding:18px 22px;font-family:var(--mono);font-size:13px;line-height:1.8;tab-size:2;caret-color:var(--gold2)}
.preview{flex:1;border-left:1px solid var(--line);background:#000}
.preview iframe{width:100%;height:100%;border:0;background:#0a0a0f}
.docs{display:none;overflow-y:auto;height:calc(100vh - 52px - 30px);padding:44px 8vw 80px;line-height:1.75}
.docs.on{display:block}
.wrap.off{display:none}
.docs h1{font-size:34px;letter-spacing:-.01em;margin-bottom:6px}
.docs h1 b{background:linear-gradient(120deg,var(--gold2),var(--gold));-webkit-background-clip:text;background-clip:text;color:transparent}
.docs .lead{color:var(--dim);font-size:16px;max-width:680px}
.docs h2{font-size:20px;margin:44px 0 12px;color:var(--gold2)}
.docs p{color:var(--dim);max-width:720px;margin:10px 0;font-size:14.5px}
.docs pre{background:var(--panel);border:1px solid var(--line);border-radius:14px;padding:18px 22px;font-family:var(--mono);font-size:12.5px;line-height:1.8;margin:16px 0;overflow-x:auto;max-width:720px;color:#cfcbe0}
.docs code{color:var(--gold2)}
.docs table{border-collapse:collapse;margin:16px 0;font-size:13px;max-width:720px}
.docs td,.docs th{border:1px solid var(--line);padding:9px 14px;text-align:left}
.docs th{color:var(--gold2);font-weight:600;background:var(--panel)}
.docs td{color:var(--dim)}
.docs td code{font-family:var(--mono);font-size:12px}
.foot{height:30px;display:flex;align-items:center;justify-content:center;gap:8px;font-size:10.5px;color:var(--faint);border-top:1px solid var(--line)}
.foot a{color:var(--gold);text-decoration:none}
@media(max-width:860px){.wrap{grid-template-columns:1fr;grid-template-rows:1fr 1fr}.preview{border-left:0;border-top:1px solid var(--line)}}
</style>
</head>
<body>
<div class="top">
  <div class="wordmark"><b>BOLD</b> <span>PLAYGROUND</span></div>
  <div class="tabs">
    <div class="tab on" data-view="play">Playground</div>
    <div class="tab" data-view="learn">Learn BOLD</div>
  </div>
  <div class="grow"></div>
  <div class="status" id="status"><span class="ok">&#10003;</span> compiled clean</div>
</div>

<div class="wrap" id="playView">
  <div class="pane">
    <div class="pane-head">SOURCE &#183; app.bold</div>
    <textarea id="src" spellcheck="false"></textarea>
  </div>
  <div class="pane preview">
    <div class="pane-head">LIVE PREVIEW &#183; built by bld in your browser</div>
    <iframe id="out" title="BOLD live preview"></iframe>
  </div>
</div>

<div class="docs" id="learnView">
  <h1>Learn <b>BOLD</b> in ten minutes</h1>
  <p class="lead">BOLD is the in-house programming language of BOLD Ventures. This playground runs the BOLD app layer, the part of the language that describes user interfaces. It is built on one conviction: code should read like intent. You describe the app in plain, ordered words, and the compiler builds it with the BOLD design system, excellence by default.</p>

  <h2>1. Every app starts with a name</h2>
  <pre><code>app</code> "My First App"
  <code>theme</code> gold</pre>
  <p>Themes available today: gold, midnight, ember, and light. Each one ships with a complete, matched design system so your app looks world class before you write a second line.</p>

  <h2>2. Pages are the rooms of your house</h2>
  <pre><code>page</code> Home <code>at</code> "/"
  <code>hero</code>
    <code>title</code> "Welcome"
    <code>subtitle</code> "It is good that you are here."
    <code>button</code> "Get Started" <code>goto</code> About

<code>page</code> About <code>at</code> "/about"
  <code>text</code> "Tell your story here."</pre>
  <p>Indentation is structure. Children sit two spaces inside their parent, the same way an outline works on paper. Buttons navigate with goto followed by a page name, and the compiler wires the routing for you.</p>

  <h2>3. The elements</h2>
  <table>
    <tr><th>Element</th><th>What it does</th></tr>
    <tr><td><code>hero</code></td><td>The opening statement of a page. Holds title, subtitle, button.</td></tr>
    <tr><td><code>title / subtitle / heading / text</code></td><td>Typography, largest to smallest.</td></tr>
    <tr><td><code>button "Label" goto Page</code></td><td>A call to action that navigates.</td></tr>
    <tr><td><code>section "Label"</code></td><td>A titled block of content.</td></tr>
    <tr><td><code>grid 3</code></td><td>A responsive grid. Put cards inside, or pull from data.</td></tr>
    <tr><td><code>card "Title" "Text"</code></td><td>A beautiful content card.</td></tr>
    <tr><td><code>row</code> + <code>stat "Label" "Value"</code></td><td>A row of headline numbers.</td></tr>
    <tr><td><code>list</code> + <code>item "Text"</code></td><td>A clean list.</td></tr>
    <tr><td><code>cta "Label" goto Page</code></td><td>A centered closing call to action.</td></tr>
    <tr><td><code>verse "Words" "Reference"</code></td><td>Scripture, honored with its own design.</td></tr>
    <tr><td><code>image "url" "alt text"</code></td><td>A rounded, responsive image.</td></tr>
    <tr><td><code>spacer 40</code></td><td>Breathing room, in pixels.</td></tr>
  </table>

  <h2>4. Data lives in one place</h2>
  <pre><code>data</code> ventures
  <code>item</code> "BOLD Studios" "Creative and platform builds"
  <code>item</code> "BOLD Social"  "The flagship consumer app"

<code>page</code> Home <code>at</code> "/"
  <code>grid</code> 3 <code>from</code> ventures</pre>
  <p>Declare data once, render it anywhere with from. Change the data, and every grid and list that uses it updates on the next build.</p>

  <h2>5. Comments and craft</h2>
  <pre>// Anything after two slashes is a note to your future self.
// Write them generously. Clarity is kindness.</pre>

  <h2>6. Build it</h2>
  <pre>bld check app.bold   // confirm it compiles clean
bld build app.bold   // outputs dist/index.html, ready to deploy</pre>
  <p>One file in, one finished app out. Deploy the dist folder to Netlify and you are live.</p>

  <h2>The BOLD standard</h2>
  <p>Every app compiled by bld ships with responsive layout, SEO metadata, smooth page transitions, and the BOLD Studios footer credit. The design excellence is not optional and not extra work. It is the floor.</p>
</div>

<div class="foot">Designed and Developed by <a href="https://BOLDStudios.io">BOLD Studios</a>. &nbsp;BOLD and the bld toolchain are in-house technology of BOLD Ventures.</div>

<script>
// ---- BOLD app layer compiler, embedded (same code that runs in the CLI) ----
var module = { exports: {} };
var require = function(){ return {}; };
var process = { argv: [], exit: function(){} };
${compilerSrc}
var boldc = module.exports;
</script>
<script>
(function(){
  "use strict";
  var DEFAULT = ${JSON.stringify(DEFAULT_PROGRAM)};
  var src = document.getElementById("src");
  var out = document.getElementById("out");
  var status = document.getElementById("status");
  src.value = DEFAULT;

  var t = null;
  function build(){
    try {
      var r = boldc.compile(src.value);
      if (r.ok) {
        out.srcdoc = r.html;
        status.innerHTML = '<span class="ok">&#10003;</span> compiled clean &#183; ' + r.pages.length + ' page' + (r.pages.length===1?'':'s') + (r.warnings.length ? ' &#183; ' + r.warnings.length + ' warning' + (r.warnings.length===1?'':'s') : '');
      } else {
        status.innerHTML = '<span class="err">&#10007;</span> ' + r.errors[0];
      }
    } catch(e) {
      status.innerHTML = '<span class="err">&#10007;</span> ' + e.message;
    }
  }
  src.addEventListener("input", function(){ clearTimeout(t); t = setTimeout(build, 250); });
  src.addEventListener("keydown", function(e){
    if (e.key === "Tab") {
      e.preventDefault();
      var s = this.selectionStart;
      this.value = this.value.slice(0, s) + "  " + this.value.slice(this.selectionEnd);
      this.selectionStart = this.selectionEnd = s + 2;
      clearTimeout(t); t = setTimeout(build, 250);
    }
  });
  build();

  // tabs
  var playView = document.getElementById("playView");
  var learnView = document.getElementById("learnView");
  document.querySelectorAll(".tab").forEach(function(tab){
    tab.onclick = function(){
      document.querySelectorAll(".tab").forEach(function(x){ x.classList.remove("on"); });
      tab.classList.add("on");
      var learn = tab.getAttribute("data-view") === "learn";
      playView.classList.toggle("off", learn);
      learnView.classList.toggle("on", learn);
    };
  });
})();
</script>
</body>
</html>`;

fs.writeFileSync(__dirname + "/../bold-playground.html", html);
console.log("playground built: " + (html.length / 1024).toFixed(1) + " KB");
