<script setup lang="ts">
const props = defineProps<{
  node: any
  nodeSchema: any
  missingRequiredParameters: string[]
  localNotice: string | null
  propertyType: (prop: any) => string
  isPropertyVisible: (prop: any) => boolean
  collectionDrafts: Record<string, string>
  collectionErrors: Record<string, string>
  resolvedPropertyOptions: (prop: any) => any[]
  dynamicPropertyLoading: Record<string, boolean>
  dynamicPropertyErrors: Record<string, string | null>
  dynamicPropertyNotes: Record<string, string | null>
  onCollectionInput: (propName: string, event: Event) => void
  formatCollectionDraft: (propName: string) => void
  refreshDynamicOptions: (propName: string) => void
}>()
</script>

<template>
  <section class="space-y-5 bg-white border border-slate-200 rounded-lg p-5">
    <div class="flex items-start justify-between gap-3">
      <div>
        <h3 class="font-semibold text-slate-800">Parameters</h3>
        <p class="mt-1 text-xs text-slate-500">
          Configure the inputs that drive this node at runtime.
        </p>
      </div>
      <span class="rounded-full bg-slate-100 px-2 py-1 text-[10px] font-bold uppercase tracking-wide text-slate-500">
        {{ nodeSchema?.properties?.length || 0 }} fields
      </span>
    </div>

    <div
      v-if="props.localNotice"
      class="rounded-lg border border-amber-200 bg-amber-50 px-3 py-2 text-xs font-medium text-amber-700"
    >
      {{ props.localNotice }}
    </div>

    <div
      v-if="props.missingRequiredParameters.length > 0"
      class="rounded-lg border border-amber-200 bg-amber-50 px-3 py-2 text-xs text-amber-700"
    >
      Missing required parameters: {{ props.missingRequiredParameters.join(', ') }}
    </div>

    <div v-if="props.nodeSchema && props.nodeSchema.properties" class="space-y-4">
      <template v-for="(prop, pIdx) in props.nodeSchema.properties" :key="pIdx">
        <div v-if="props.isPropertyVisible(prop)">
          <label class="mb-1.5 block text-sm font-medium text-slate-700">{{ prop.displayName }}</label>

          <div v-if="props.propertyType(prop) === 'string'" class="relative">
            <input
              v-model="props.node.data.properties[prop.name]"
              type="text"
              :placeholder="prop.placeholder || ''"
              class="w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-sm text-slate-900 shadow-sm focus:border-brand-500 focus:ring-1 focus:ring-brand-500"
            />
          </div>

          <div v-else-if="props.propertyType(prop) === 'text'" class="relative">
            <textarea
              v-model="props.node.data.properties[prop.name]"
              rows="5"
              :placeholder="prop.placeholder || ''"
              class="w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-sm text-slate-900 shadow-sm focus:border-brand-500 focus:ring-1 focus:ring-brand-500 font-mono"
            />
          </div>

          <div v-else-if="props.propertyType(prop) === 'number'" class="relative">
            <input
              v-model.number="props.node.data.properties[prop.name]"
              type="number"
              class="w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-sm text-slate-900 shadow-sm focus:border-brand-500 focus:ring-1 focus:ring-brand-500"
            />
          </div>

          <div v-else-if="props.propertyType(prop) === 'options'">
            <select
              v-model="props.node.data.properties[prop.name]"
              class="w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-sm text-slate-900 shadow-sm focus:border-brand-500 focus:ring-1 focus:ring-brand-500"
            >
              <option v-for="opt in props.resolvedPropertyOptions(prop)" :key="opt.value" :value="opt.value">
                {{ opt.name }}
              </option>
            </select>
          </div>

          <div v-else-if="props.propertyType(prop) === 'loadOptions'" class="space-y-2">
            <div class="flex items-center gap-2">
              <select
                v-model="props.node.data.properties[prop.name]"
                class="w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-sm text-slate-900 shadow-sm focus:border-brand-500 focus:ring-1 focus:ring-brand-500"
              >
                <option
                  v-for="opt in props.resolvedPropertyOptions(prop)"
                  :key="opt.value"
                  :value="opt.value"
                >
                  {{ opt.name }}
                </option>
              </select>
              <button
                type="button"
                class="rounded-md border border-slate-300 bg-white px-3 py-2 text-xs font-semibold text-slate-600 transition hover:bg-slate-50"
                @click="props.refreshDynamicOptions(prop.name)"
              >
                {{ props.dynamicPropertyLoading[prop.name] ? 'Loading…' : 'Reload' }}
              </button>
            </div>
            <p
              v-if="props.dynamicPropertyErrors[prop.name]"
              class="text-xs text-amber-700"
            >
              {{ props.dynamicPropertyErrors[prop.name] }}
            </p>
            <p
              v-else-if="props.dynamicPropertyNotes[prop.name]"
              class="text-xs text-slate-500"
            >
              {{ props.dynamicPropertyNotes[prop.name] }}
            </p>
            <input
              v-model="props.node.data.properties[prop.name]"
              type="text"
              :placeholder="prop.placeholder || 'Enter a custom value if your model is not listed'"
              class="w-full rounded-md border border-dashed border-slate-300 bg-slate-50 px-3 py-2 text-sm text-slate-900 shadow-sm focus:border-brand-500 focus:bg-white focus:ring-1 focus:ring-brand-500"
            />
          </div>

          <div v-else-if="props.propertyType(prop) === 'boolean'" class="mt-2 flex items-center">
            <input
              v-model="props.node.data.properties[prop.name]"
              type="checkbox"
              class="h-4 w-4 rounded border-slate-300 bg-white text-brand-600 focus:ring-brand-500"
            />
            <label class="ml-2 text-sm text-slate-700">{{ prop.description || prop.displayName }}</label>
          </div>

          <div
            v-else-if="props.propertyType(prop) === 'collection' || props.propertyType(prop) === 'fixedCollection'"
            class="space-y-2"
          >
            <textarea
              :value="props.collectionDrafts[prop.name] || '[]'"
              rows="6"
              class="w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-sm text-slate-900 shadow-sm focus:border-brand-500 focus:ring-1 focus:ring-brand-500 font-mono"
              @input="props.onCollectionInput(prop.name, $event)"
              @blur="props.formatCollectionDraft(prop.name)"
            />
            <p class="text-xs text-slate-500">Provide valid JSON (array or object).</p>
            <p v-if="props.collectionErrors[prop.name]" class="text-xs text-red-600">
              {{ props.collectionErrors[prop.name] }}
            </p>
          </div>

          <p v-if="prop.description && props.propertyType(prop) !== 'boolean'" class="mt-1 text-xs text-slate-500">
            {{ prop.description }}
          </p>
          <p v-if="prop.hint" class="mt-1 text-xs text-slate-400">
            Hint: {{ prop.hint }}
          </p>
        </div>
      </template>
    </div>

    <div
      v-else-if="(props.node.data.kind || props.node.data.type) === 'action'"
      class="rounded-lg border border-amber-200 bg-amber-50 p-3 text-xs text-amber-700"
    >
      No dynamic schema available from backend. Using fallback rendering.
    </div>
  </section>
</template>
