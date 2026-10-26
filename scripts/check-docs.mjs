#!/usr/bin/env node
// 校验 Markdown 中的相对链接都指向仓库内已存在的文件。
// 用法：node scripts/check-docs.mjs
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const skipDirs = new Set([".git", "node_modules", "dist", "target"]);

function walk(dir, acc = []) {
  for (const name of readdirSync(dir)) {
    if (skipDirs.has(name)) continue;
    const path = join(dir, name);
    if (statSync(path).isDirectory()) walk(path, acc);
    else if (name.endsWith(".md")) acc.push(path);
  }
  return acc;
}

const linkPattern = /\[[^\]]*\]\(([^)\s]+)\)/g;
let checked = 0;
const broken = [];

for (const file of walk(root)) {
  const body = readFileSync(file, "utf8").replace(/```[\s\S]*?```/g, "");
  for (const [, target] of body.matchAll(linkPattern)) {
    if (/^[a-z][a-z0-9+.-]*:/i.test(target) || target.startsWith("#")) continue;
    checked += 1;
    const resolved = resolve(dirname(file), target.split("#")[0]);
    if (!existsSync(resolved)) broken.push(`${relative(root, file)} -> ${target}`);
  }
}

console.log(`checked ${checked} relative links in Markdown`);
if (broken.length > 0) {
  console.error(`broken links (${broken.length}):\n${broken.join("\n")}`);
  process.exit(1);
}
console.log("OK: all relative links resolve");
