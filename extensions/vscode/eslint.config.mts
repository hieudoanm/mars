import js from '@eslint/js';
import globals from 'globals';
import tseslint from 'typescript-eslint';
import { defineConfig } from 'eslint/config';

export default defineConfig([
  {
    ignores: ['**/dist/**', '**/build/**', '**/out/**', '**/coverage/**', '**/node_modules/**'],
  },
  {
    files: ['**/*.{js,mjs,cjs,ts,mts,cts,jsx,tsx}'],
    plugins: { js },
    extends: ['js/recommended'],
    languageOptions: { globals: { ...globals.browser, ...globals.node } },
  },
  tseslint.configs.recommended,
  {
    files: ['**/*.{js,mjs,cjs,ts,mts,cts,jsx,tsx}'],
    rules: {
      // Arrow functions only: no `function` declarations; function expressions
      // remain allowed for constructor mocks (e.g. `vi.fn(function () {})`).
      'func-style': ['error', 'expression', { allowArrowFunctions: true }],
    },
  },
]);
