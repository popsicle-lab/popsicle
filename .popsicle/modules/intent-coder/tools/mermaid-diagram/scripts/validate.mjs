#!/usr/bin/env node
/**
 * Validate ```mermaid blocks in Markdown files under path.
 * Uses ui/node_modules/mermaid when available (same renderer family as Popsicle UI).
 */
import { readFileSync, existsSync, statSync, readdirSync } from "fs";
import { join, resolve, isAbsolute } from "path";
import { pathToFileURL } from "url";

function parseArgs(argv) {
  const out = { path: "", format: "text" };
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === "--path" && argv[i + 1]) out.path = argv[++i];
    else if (argv[i] === "--format" && argv[i + 1]) out.format = argv[++i];
  }
  return out;
}

function collectMdFiles(target) {
  const st = statSync(target);
  if (st.isFile()) return target.endsWith(".md") ? [target] : [];
  const out = [];
  for (const name of readdirSync(target)) {
    const p = join(target, name);
    try {
      const s = statSync(p);
      if (s.isDirectory()) out.push(...collectMdFiles(p));
      else if (s.isFile() && name.endsWith(".md")) out.push(p);
    } catch {
      /* skip */
    }
  }
  return out.sort();
}

function extractBlocks(content) {
  const re = /```mermaid\s*\n([\s\S]*?)```/g;
  const blocks = [];
  let m;
  while ((m = re.exec(content)) !== null) {
    const line =
      content.slice(0, m.index).split("\n").length;
    blocks.push({ line, code: m[1].trim() });
  }
  return blocks;
}

function lintConservative(code) {
  const issues = [];
  if (/[\u{1F300}-\u{1FAFF}]/u.test(code)) {
    issues.push("contains emoji in diagram text");
  }
  if (/\\n/.test(code)) {
    issues.push("contains literal \\n in labels (use <br/> instead)");
  }
  if (/<[a-z][\s\S]*>/i.test(code)) {
    issues.push("contains HTML-like tags in diagram");
  }
  return issues;
}

async function loadMermaid(root) {
  const candidates = [
    join(root, "ui/node_modules/mermaid/dist/mermaid.core.mjs"),
    join(root, "ui/node_modules/mermaid/dist/mermaid.esm.mjs"),
    join(root, "ui/node_modules/mermaid/dist/mermaid.esm.min.mjs"),
  ];
  for (const p of candidates) {
    if (existsSync(p)) {
      const mod = await import(pathToFileURL(p).href);
      return mod.default ?? mod;
    }
  }
  return null;
}

async function validateBlock(mermaid, code) {
  const conservative = lintConservative(code);
  if (conservative.length) {
    return { ok: false, errors: conservative };
  }
  if (!mermaid) {
    return { ok: true, warnings: ["mermaid package not installed; skipped parse (run: cd ui && npm install)"] };
  }
  try {
    await mermaid.parse(code);
    return { ok: true, warnings: [] };
  } catch (e) {
    return { ok: false, errors: [String(e.message ?? e)] };
  }
}

const args = parseArgs(process.argv.slice(2));
const root = process.env.ROOT || process.cwd();
if (!args.path) {
  console.error("error: --path required");
  process.exit(1);
}
let target = args.path;
if (!isAbsolute(target)) target = resolve(root, target);
if (!existsSync(target)) {
  console.error(`error: path not found: ${target}`);
  process.exit(1);
}

const files = collectMdFiles(target);
if (files.length === 0) {
  console.error(`error: no .md files under ${target}`);
  process.exit(1);
}

const mermaid = await loadMermaid(root);
if (mermaid?.initialize) {
  mermaid.initialize({ startOnLoad: false, securityLevel: "loose" });
}

const results = [];
let status = 0;
let blockCount = 0;

for (const file of files) {
  const content = readFileSync(file, "utf8");
  const blocks = extractBlocks(content);
  if (blocks.length === 0) continue;
  for (const b of blocks) {
    blockCount += 1;
    const rel = file.startsWith(root) ? file.slice(root.length + 1) : file;
    const v = await validateBlock(mermaid, b.code);
    if (!v.ok) status = 1;
    results.push({
      file: rel,
      line: b.line,
      ok: v.ok,
      errors: v.errors ?? [],
      warnings: v.warnings ?? [],
    });
  }
}

if (blockCount === 0) {
  console.error(`error: no mermaid blocks found under ${target}`);
  process.exit(1);
}

const summary = {
  ok: status === 0,
  path: args.path,
  files_scanned: files.length,
  blocks: blockCount,
  results,
};

if (args.format === "json") {
  console.log(JSON.stringify(summary, null, 2));
} else {
  for (const r of results) {
    const mark = r.ok ? "OK" : "FAIL";
    console.log(`${mark} ${r.file}:${r.line}`);
    for (const e of r.errors) console.log(`  error: ${e}`);
    for (const w of r.warnings) console.log(`  warn: ${w}`);
  }
  console.log(`---\n${summary.blocks} block(s), ${status === 0 ? "all passed" : "has failures"}`);
}

process.exit(status);
