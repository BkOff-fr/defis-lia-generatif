// Sobr.ia extension — Vite config (C27.1).
//
// Build en N passes pour respecter les contraintes MV3 :
//   - Pass `main`   : popup + options (HTML, ES modules).
//   - Pass `sw`     : service-worker en IIFE monolithique (MV3 Chrome n'accepte pas
//                     les `import` hors `"type": "module"` — on bundle tout en un fichier).
//   - Pass `iife`   : 1 content script en format IIFE par passe. Rollup interdit
//                     l'IIFE multi-entry → on enchaîne 3 builds depuis scripts/build.js.
//
// Variables d'environnement :
//   - SOBRIA_TARGET  = 'chrome' (défaut) | 'firefox'  → dist/ vs dist-firefox/ et le
//     manifest copié (manifest.json vs manifest.firefox.json).
//   - SOBRIA_KIND    = 'main' (défaut) | 'sw' | 'iife' → choisit la passe.
//   - SOBRIA_CONTENT = 'chatgpt' | 'claude' | 'le-chat' (requis si SOBRIA_KIND=iife).
//
// Orchestrée par scripts/build.js (cf. package.json).

import { defineConfig, type Plugin } from 'vite';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { copyFileSync, mkdirSync, existsSync, readdirSync } from 'node:fs';

const __dirname = dirname(fileURLToPath(import.meta.url));

type Target = 'chrome' | 'firefox';
type Kind = 'main' | 'sw' | 'iife';
type Content = 'chatgpt' | 'claude' | 'le-chat';

const target: Target = (process.env['SOBRIA_TARGET'] as Target) ?? 'chrome';
const kind: Kind = (process.env['SOBRIA_KIND'] as Kind) ?? 'main';
const outDir = target === 'firefox' ? 'dist-firefox' : 'dist';

const mainEntries = {
  popup: resolve(__dirname, 'src/popup/index.html'),
  options: resolve(__dirname, 'src/options/index.html')
};

const serviceWorkerEntry = resolve(__dirname, 'src/background/service-worker.ts');

function contentEntry(content: Content): { name: string; path: string } {
  const map: Record<Content, string> = {
    chatgpt: 'src/content/chatgpt.ts',
    claude: 'src/content/claude.ts',
    'le-chat': 'src/content/le-chat.ts'
  };
  return { name: `content-${content}`, path: resolve(__dirname, map[content]) };
}

const isIife = kind === 'iife';
const isServiceWorker = kind === 'sw';

let iifeInput: { name: string; path: string } | undefined;
if (isIife) {
  const contentEnv = process.env['SOBRIA_CONTENT'] as Content | undefined;
  if (!contentEnv) {
    throw new Error('SOBRIA_KIND=iife requires SOBRIA_CONTENT (chatgpt | claude | le-chat)');
  }
  iifeInput = contentEntry(contentEnv);
}

export default defineConfig({
  build: {
    outDir,
    // emptyOutDir uniquement sur la passe main : sw + iife ajoutent au dossier.
    emptyOutDir: kind === 'main' && process.env['SOBRIA_WATCH'] !== '1',
    sourcemap: false,
    minify: 'esbuild',
    target: 'esnext',
    chunkSizeWarningLimit: 500,
    rollupOptions: isServiceWorker
      ? {
          input: { 'service-worker': serviceWorkerEntry },
          output: {
            format: 'iife',
            entryFileNames: 'service-worker.js',
            inlineDynamicImports: true,
            extend: true
          }
        }
      : isIife && iifeInput
        ? {
            input: { [iifeInput.name]: iifeInput.path },
            output: {
              format: 'iife',
              entryFileNames: '[name].js',
              inlineDynamicImports: true,
              assetFileNames: 'assets/[name][extname]'
            }
          }
        : {
            input: mainEntries,
            output: {
              entryFileNames: 'assets/[name].js',
              chunkFileNames: 'assets/[name]-[hash].js',
              assetFileNames: 'assets/[name][extname]'
            }
          }
  },
  plugins: kind === 'main' ? [copyAssetsPlugin(target, outDir)] : []
});

function copyAssetsPlugin(activeTarget: Target, finalOutDir: string): Plugin {
  return {
    name: 'sobria-copy-assets',
    writeBundle() {
      const manifestSrc = activeTarget === 'firefox' ? 'manifest.firefox.json' : 'manifest.json';
      copyFileSync(
        resolve(__dirname, manifestSrc),
        resolve(__dirname, finalOutDir, 'manifest.json')
      );

      // Icônes : SVG mutualisés depuis web/static et le design system (cf. scripts/sync-logos.js).
      const iconsSrcDir = resolve(__dirname, 'src/assets/icons');
      const iconsDstDir = resolve(__dirname, finalOutDir, 'icons');
      if (existsSync(iconsSrcDir)) {
        if (!existsSync(iconsDstDir)) mkdirSync(iconsDstDir, { recursive: true });
        for (const file of readdirSync(iconsSrcDir)) {
          if (file.endsWith('.svg')) {
            copyFileSync(resolve(iconsSrcDir, file), resolve(iconsDstDir, file));
          }
        }
      }

      // Fontes (WOFF2 SIL OFL — voir docs/LICENSES-FONTS.md côté web/).
      const fontsSrcDir = resolve(__dirname, 'src/assets/fonts');
      if (existsSync(fontsSrcDir)) {
        const fontsDstDir = resolve(__dirname, finalOutDir, 'fonts');
        if (!existsSync(fontsDstDir)) mkdirSync(fontsDstDir, { recursive: true });
        for (const file of readdirSync(fontsSrcDir)) {
          if (file.endsWith('.woff2')) {
            copyFileSync(resolve(fontsSrcDir, file), resolve(fontsDstDir, file));
          }
        }
      }
    }
  };
}
