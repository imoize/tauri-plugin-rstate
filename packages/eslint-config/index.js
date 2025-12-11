import antfu from '@antfu/eslint-config';
import turbo from 'eslint-plugin-turbo';

export default function createConfig(options, ...userConfigs) {
  return antfu({
    type: 'app',
    typescript: true,
    formatters: true,
    stylistic: {
      indent: 2,
      semi: true,
      quotes: 'single',
    },
    ...options,
  }, {
    rules: {
      'ts/consistent-type-definitions': ['error', 'type'],
      'no-console': ['warn'],
      'antfu/no-top-level-await': ['off'],
      'node/prefer-global/process': ['off'],
      'node/no-process-env': ['error'],
      'perfectionist/sort-imports': ['error', {
        tsconfigRootDir: '.',
      }],
      'toml/padding-line-between-pairs': ['off'],
    },
  }, {
    plugins: {
      turbo,
    },
    rules: {
      ...turbo.configs['flat/recommended'].rules,
    },
  }, ...userConfigs);
}
