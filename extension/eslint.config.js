// Sobr.ia extension — ESLint flat config (aligné sur web/eslint.config.js).
// Strict TypeScript + prettier compatibility + globals browser/webextension.

import js from '@eslint/js';
import ts from 'typescript-eslint';
import prettier from 'eslint-config-prettier';
import globals from 'globals';

export default [
  js.configs.recommended,
  ...ts.configs.strict,
  prettier,
  {
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.webextensions,
        ...globals.node
      },
      parserOptions: {
        ecmaVersion: 2022,
        sourceType: 'module'
      }
    }
  },
  {
    files: ['tests/**/*.ts'],
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.webextensions,
        ...globals.node
      }
    },
    rules: {
      // Les tests utilisent `!` après querySelector pour rester concis : la
      // sélection cible des éléments connus de la fixture HTML, et l'absence
      // d'élément fait *intentionnellement* échouer le test avec un crash
      // explicite (plus simple à debugger qu'un `expect(el).toBeTruthy()`).
      '@typescript-eslint/no-non-null-assertion': 'off'
    }
  },
  {
    files: ['scripts/**/*.js'],
    languageOptions: {
      globals: {
        ...globals.node
      },
      sourceType: 'module'
    },
    rules: {
      // Scripts Node tolèrent any : ce sont des outils internes, pas du runtime extension.
      '@typescript-eslint/no-unused-vars': 'off'
    }
  },
  {
    ignores: [
      'dist/**',
      'dist-firefox/**',
      'node_modules/**',
      'coverage/**',
      'test-results/**',
      'playwright-report/**',
      'src/assets/**'
    ]
  }
];
