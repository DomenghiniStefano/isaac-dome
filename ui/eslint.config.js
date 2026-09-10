import pluginVue from 'eslint-plugin-vue'
import { withVueTs, vueTsConfigs } from '@vue/eslint-config-typescript'

export default withVueTs(
  { ignores: ['dist', 'node_modules'] },
  pluginVue.configs['flat/essential'],
  vueTsConfigs.recommended,
  {
    // shadcn-vue primitives are named for the element they stand for — Button.vue,
    // Badge.vue. The rule exists to keep app components from clashing with HTML elements;
    // a folder of primitives imported by name is exactly where single words belong.
    files: ['src/components/ui/**/*.vue'],
    rules: { 'vue/multi-word-component-names': 'off' },
  },
)
