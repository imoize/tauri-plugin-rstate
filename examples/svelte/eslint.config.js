import createConfig from '@repo/eslint-config/index.js';

export default createConfig(
  {
    svelte: true,
  },
  {
    ignores: ['dist/**/*', '.svelte-kit/**', 'node_modules/**', 'src-tauri/target/**', 'src-tauri/gen/**'],
  },
  {
    files: ['**/*.svelte'],
    rules: {
      'prefer-const': ['off', {}],
    },
  },
);
