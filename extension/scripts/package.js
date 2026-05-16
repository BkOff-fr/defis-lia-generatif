// Sobr.ia extension — packaging des bundles (C27.1).
//
// Produit :
//   - dist/sobria-extension-chrome-v<version>.zip
//   - dist-firefox/sobria-extension-firefox-v<version>.xpi
//
// Imprime le SHA-256 de chaque archive (pour copier-coller dans les release notes).
// Utilise uniquement les modules Node stdlib (zlib + crypto + fs/promises). Pas de dep externe.

import { readFileSync, writeFileSync, statSync, readdirSync, existsSync, rmSync } from 'node:fs';
import { resolve, join, relative, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { deflateRawSync } from 'node:zlib';

const __dirname = dirname(fileURLToPath(import.meta.url));
const extensionRoot = resolve(__dirname, '..');

const pkg = JSON.parse(readFileSync(resolve(extensionRoot, 'package.json'), 'utf8'));
const version = pkg.version;

/** Walks a directory recursively and returns absolute paths to all files. */
function walk(dir) {
  const out = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const fullPath = join(dir, entry.name);
    if (entry.isDirectory()) out.push(...walk(fullPath));
    else if (entry.isFile()) out.push(fullPath);
  }
  return out;
}

const CRC_TABLE = (() => {
  const table = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    table[n] = c >>> 0;
  }
  return table;
})();

function crc32(buf) {
  let crc = 0xffffffff;
  for (let i = 0; i < buf.length; i++) {
    crc = CRC_TABLE[(crc ^ buf[i]) & 0xff] ^ (crc >>> 8);
  }
  return (crc ^ 0xffffffff) >>> 0;
}

/**
 * Construit un ZIP non chiffré, méthode 8 (deflate), conforme aux specs PKZIP / WebExtensions.
 * Pas de support ZIP64 : suffisant pour < 4 GB.
 */
function buildZip(files, baseDir) {
  const localParts = [];
  const centralParts = [];
  let offset = 0;

  for (const filePath of files) {
    const name = relative(baseDir, filePath).split('\\').join('/');
    const data = readFileSync(filePath);
    const compressed = deflateRawSync(data);
    const useDeflate = compressed.length < data.length;
    const finalData = useDeflate ? compressed : data;
    const method = useDeflate ? 8 : 0;
    const crc = crc32(data);
    const nameBytes = Buffer.from(name, 'utf8');

    // DOS time/date — figés à 2026-01-01 00:00:00 pour des builds reproductibles.
    const dosTime = 0;
    const dosDate = ((2026 - 1980) << 9) | (1 << 5) | 1;

    const localHeader = Buffer.alloc(30);
    localHeader.writeUInt32LE(0x04034b50, 0);
    localHeader.writeUInt16LE(20, 4); // version needed
    localHeader.writeUInt16LE(0, 6); // flags
    localHeader.writeUInt16LE(method, 8);
    localHeader.writeUInt16LE(dosTime, 10);
    localHeader.writeUInt16LE(dosDate, 12);
    localHeader.writeUInt32LE(crc, 14);
    localHeader.writeUInt32LE(finalData.length, 18);
    localHeader.writeUInt32LE(data.length, 22);
    localHeader.writeUInt16LE(nameBytes.length, 26);
    localHeader.writeUInt16LE(0, 28);

    localParts.push(localHeader, nameBytes, finalData);

    const centralHeader = Buffer.alloc(46);
    centralHeader.writeUInt32LE(0x02014b50, 0);
    centralHeader.writeUInt16LE(20, 4); // version made by
    centralHeader.writeUInt16LE(20, 6); // version needed
    centralHeader.writeUInt16LE(0, 8); // flags
    centralHeader.writeUInt16LE(method, 10);
    centralHeader.writeUInt16LE(dosTime, 12);
    centralHeader.writeUInt16LE(dosDate, 14);
    centralHeader.writeUInt32LE(crc, 16);
    centralHeader.writeUInt32LE(finalData.length, 20);
    centralHeader.writeUInt32LE(data.length, 24);
    centralHeader.writeUInt16LE(nameBytes.length, 28);
    centralHeader.writeUInt16LE(0, 30);
    centralHeader.writeUInt16LE(0, 32);
    centralHeader.writeUInt16LE(0, 34);
    centralHeader.writeUInt16LE(0, 36);
    centralHeader.writeUInt32LE(0, 38);
    centralHeader.writeUInt32LE(offset, 42);

    centralParts.push(centralHeader, nameBytes);

    offset += localHeader.length + nameBytes.length + finalData.length;
  }

  const centralBuffer = Buffer.concat(centralParts);
  const localBuffer = Buffer.concat(localParts);

  const end = Buffer.alloc(22);
  end.writeUInt32LE(0x06054b50, 0);
  end.writeUInt16LE(0, 4); // disk
  end.writeUInt16LE(0, 6); // disk with cd
  end.writeUInt16LE(files.length, 8);
  end.writeUInt16LE(files.length, 10);
  end.writeUInt32LE(centralBuffer.length, 12);
  end.writeUInt32LE(localBuffer.length, 16);
  end.writeUInt16LE(0, 20);

  return Buffer.concat([localBuffer, centralBuffer, end]);
}

function packBundle({ srcDir, archiveName }) {
  if (!existsSync(srcDir)) {
    console.error(`[package] dossier introuvable : ${srcDir}`);
    return null;
  }
  const files = walk(srcDir);
  const archivePath = resolve(extensionRoot, 'dist', archiveName);
  // Si l'archive existe (re-run), on l'efface.
  if (existsSync(archivePath)) rmSync(archivePath);
  const buf = buildZip(files, srcDir);
  writeFileSync(archivePath, buf);
  const stats = statSync(archivePath);
  const sha = createHash('sha256').update(buf).digest('hex');
  return { archivePath, size: stats.size, sha };
}

function fmtKb(bytes) {
  return `${(bytes / 1024).toFixed(1)} KB`;
}

// dist/ doit exister pour stocker les archives.
const distRoot = resolve(extensionRoot, 'dist');
if (!existsSync(distRoot)) {
  console.error(`[package] dossier dist/ introuvable. Lance d'abord : npm run build`);
  process.exit(1);
}

const results = [];

const chromeResult = packBundle({
  srcDir: resolve(extensionRoot, 'dist'),
  archiveName: `sobria-extension-chrome-v${version}.zip`
});
if (chromeResult) results.push({ label: 'Chrome', ...chromeResult });

const firefoxSrc = resolve(extensionRoot, 'dist-firefox');
if (existsSync(firefoxSrc)) {
  const firefoxResult = packBundle({
    srcDir: firefoxSrc,
    archiveName: `sobria-extension-firefox-v${version}.xpi`
  });
  if (firefoxResult) results.push({ label: 'Firefox', ...firefoxResult });
} else {
  console.warn(
    '[package] dist-firefox/ absent — lance `npm run build:firefox` pour produire le .xpi'
  );
}

console.log('\n[package] archives générées :');
for (const r of results) {
  console.log(`  ${r.label.padEnd(8)} ${relative(extensionRoot, r.archivePath)}`);
  console.log(`           taille : ${fmtKb(r.size)}`);
  console.log(`           sha256 : ${r.sha}`);
  if (r.size > 500 * 1024) {
    console.warn(`           ⚠ taille > 500 KB (cible NF-04 du brief C27)`);
  }
}
