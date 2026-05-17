#!/usr/bin/env node
/**
 * Synchronise docs/ → site/src/content/.
 * - docs/*.md (sauf docs/adr/ et docs/superpowers/) → site/src/content/docs/<path>
 * - docs/adr/*.md → site/src/content/adrs/<filename>
 * Injecte un front-matter `sourcePath` (chemin relatif au repo root) si
 * le fichier n'a pas déjà de front-matter, pour générer "Edit on GitHub".
 */
import {
  mkdirSync,
  readdirSync,
  readFileSync,
  writeFileSync,
  rmSync,
  existsSync,
} from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const REPO_ROOT = resolve(__dirname, '..');
const DOCS_DIR = resolve(REPO_ROOT, 'docs');
const TARGET_DOCS = resolve(REPO_ROOT, 'site/src/content/docs');
const TARGET_ADRS = resolve(REPO_ROOT, 'site/src/content/adrs');

const HAS_FRONTMATTER = /^---\r?\n/;

function walk(dir) {
  const entries = readdirSync(dir, { withFileTypes: true });
  const out = [];
  for (const e of entries) {
    const full = join(dir, e.name);
    if (e.isDirectory()) {
      out.push(...walk(full));
    } else if (e.isFile() && e.name.endsWith('.md')) {
      out.push(full);
    }
  }
  return out;
}

function syncFile(src, dest) {
  mkdirSync(dirname(dest), { recursive: true });
  const raw = readFileSync(src, 'utf8');
  const sourcePath = relative(REPO_ROOT, src).replace(/\\/g, '/');
  let out;
  if (HAS_FRONTMATTER.test(raw)) {
    out = raw.replace(/^---\r?\n/, `---\nsourcePath: ${sourcePath}\n`);
  } else {
    out = `---\nsourcePath: ${sourcePath}\n---\n\n${raw}`;
  }
  writeFileSync(dest, out, 'utf8');
}

function clean(dir) {
  if (existsSync(dir)) {
    rmSync(dir, { recursive: true, force: true });
  }
  mkdirSync(dir, { recursive: true });
}

function main() {
  if (!existsSync(DOCS_DIR)) {
    console.error(`Source docs/ not found at ${DOCS_DIR}`);
    process.exit(1);
  }

  clean(TARGET_DOCS);
  clean(TARGET_ADRS);

  let count = 0;
  const all = walk(DOCS_DIR);
  for (const file of all) {
    const relPath = relative(DOCS_DIR, file).replace(/\\/g, '/');
    if (relPath.startsWith('adr/')) {
      const flat = relPath.replace(/^adr\//, '');
      syncFile(file, join(TARGET_ADRS, flat));
    } else if (relPath.startsWith('superpowers/')) {
      continue;
    } else {
      syncFile(file, join(TARGET_DOCS, relPath));
    }
    count++;
  }

  console.log(`✓ Synced ${count} markdown files to site/src/content/`);
}

main();
