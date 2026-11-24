import js from '@eslint/js';
import tseslint from 'typescript-eslint';
import hooks from 'eslint-plugin-react-hooks';

export default tseslint.config(
  { ignores: ['**/dist/**', '**/node_modules/**', 'e2e/screenshots/**', 'e2e/test-results/**', 'e2e/playwright-report/**'] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  { plugins: { 'react-hooks': hooks }, rules: { ...hooks.configs.recommended.rules } },
);
