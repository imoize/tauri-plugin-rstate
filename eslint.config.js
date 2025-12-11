import createConfig from './packages/eslint-config/index.js';

export default createConfig({
  rules: {
    'pnpm/yaml-enforce-settings': ['off'],
  },
});
