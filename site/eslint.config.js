import tseslint from 'typescript-eslint';
import sveltePlugin from 'eslint-plugin-svelte';
import astroPlugin from 'eslint-plugin-astro';
import svelteParser from 'svelte-eslint-parser';

export default tseslint.config(
  {
    ignores: ['dist', '.astro', 'node_modules', 'pagefind', 'public/fonts'],
  },
  ...tseslint.configs.recommended,
  ...sveltePlugin.configs['flat/recommended'],
  ...astroPlugin.configs.recommended,
  {
    files: ['**/*.svelte'],
    languageOptions: {
      parser: svelteParser,
      parserOptions: {
        parser: tseslint.parser,
        extraFileExtensions: ['.svelte'],
      },
    },
  },
  {
    files: ['src/env.d.ts'],
    rules: {
      '@typescript-eslint/triple-slash-reference': 'off',
    },
  },
  {
    rules: {
      '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
    },
  },
);
