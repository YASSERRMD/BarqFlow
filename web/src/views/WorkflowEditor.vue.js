/// <reference types="../../node_modules/.vue-global-types/vue_3.5_0_0_0.d.ts" />
import { ref } from 'vue';
import { VueFlow, useVueFlow } from '@vue-flow/core';
import { Background } from '@vue-flow/background';
import { Controls } from '@vue-flow/controls';
import { MiniMap } from '@vue-flow/minimap';
import { Plus, Play, Save, Settings2, Loader2 } from 'lucide-vue-next';
import CustomNode from '../components/CustomNode.vue';
import NodePanel from '../components/NodePanel.vue';
import { useWorkflowStore } from '../stores/workflows';
import { useRoute } from 'vue-router';
const route = useRoute();
const workflowStore = useWorkflowStore();
const { onConnect, addEdges, toObject } = useVueFlow();
const nodes = ref([
    {
        id: '1',
        type: 'custom',
        label: 'Manual Trigger',
        position: { x: 100, y: 150 },
        data: { type: 'trigger', label: 'Manual Trigger', description: 'Click execute to start' }
    },
    {
        id: '2',
        type: 'custom',
        label: 'HTTP Request',
        position: { x: 400, y: 150 },
        data: { type: 'action', label: 'HTTP Request', description: 'GET https://api.example.com' }
    }
]);
const edges = ref([
    { id: 'e1-2', source: '1', target: '2', animated: true, style: { stroke: '#0ea5e9', strokeWidth: 2 } }
]);
const selectedNode = ref(null);
onConnect((params) => {
    addEdges([params]);
});
function onNodeClick({ node }) {
    selectedNode.value = node;
}
async function handleExecute() {
    if (workflowStore.loading)
        return;
    // Set running status on all nodes for visual effect
    nodes.value.forEach(n => n.data.status = 'running');
    try {
        // In a real app, we'd use route.params.id
        const mockWorkflowId = '00000000-0000-0000-0000-000000000000';
        const result = await workflowStore.executeWorkflow(mockWorkflowId);
        // Update node statuses based on result
        nodes.value.forEach(n => {
            const nodeName = n.data.label;
            if (result.data && result.data[nodeName]) {
                n.data.status = result.data[nodeName].success ? 'success' : 'error';
            }
            else {
                n.data.status = 'success'; // Default for triggers
            }
        });
    }
    catch (err) {
        nodes.value.forEach(n => n.data.status = 'error');
    }
}
async function handleSave() {
    const flow = toObject();
    console.log('Saving flow:', flow);
    // await workflowStore.saveWorkflow({ ...flow, name: 'My First Workflow' })
}
debugger; /* PartiallyEnd: #3632/scriptSetup.vue */
const __VLS_ctx = {};
let __VLS_components;
let __VLS_directives;
// CSS variable injection 
// CSS variable injection end 
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "h-full w-full flex overflow-hidden bg-canvas" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex-1 relative overflow-hidden" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "absolute top-6 left-6 right-6 flex justify-between items-center z-10 pointer-events-none" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "bg-white/90 backdrop-blur-xl shadow-lg shadow-slate-200/50 border border-slate-200 rounded-2xl px-6 py-3 flex items-center gap-4 pointer-events-auto" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "w-10 h-10 bg-brand-50 rounded-xl flex items-center justify-center text-brand-600" },
});
const __VLS_0 = {}.Settings2;
/** @type {[typeof __VLS_components.Settings2, ]} */ ;
// @ts-ignore
const __VLS_1 = __VLS_asFunctionalComponent(__VLS_0, new __VLS_0({
    ...{ class: "w-6 h-6" },
}));
const __VLS_2 = __VLS_1({
    ...{ class: "w-6 h-6" },
}, ...__VLS_functionalComponentArgsRest(__VLS_1));
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({});
__VLS_asFunctionalElement(__VLS_intrinsicElements.h1, __VLS_intrinsicElements.h1)({
    ...{ class: "font-bold text-slate-800 text-lg leading-tight" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.p, __VLS_intrinsicElements.p)({
    ...{ class: "text-xs text-slate-400 font-medium" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "h-8 w-px bg-slate-100 mx-2" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
    ...{ class: "px-2.5 py-1 bg-green-100 text-green-700 text-[10px] font-bold uppercase tracking-wider rounded-lg" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex gap-3 pointer-events-auto" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.button, __VLS_intrinsicElements.button)({
    ...{ onClick: (__VLS_ctx.handleSave) },
    ...{ class: "bg-white/90 backdrop-blur-xl shadow-lg shadow-slate-200/50 border border-slate-200 hover:bg-slate-50 text-slate-700 px-5 py-3 rounded-2xl flex items-center gap-2 transition-all hover:-translate-y-0.5 active:translate-y-0 font-bold text-sm" },
});
const __VLS_4 = {}.Save;
/** @type {[typeof __VLS_components.Save, ]} */ ;
// @ts-ignore
const __VLS_5 = __VLS_asFunctionalComponent(__VLS_4, new __VLS_4({
    ...{ class: "w-4 h-4" },
}));
const __VLS_6 = __VLS_5({
    ...{ class: "w-4 h-4" },
}, ...__VLS_functionalComponentArgsRest(__VLS_5));
__VLS_asFunctionalElement(__VLS_intrinsicElements.button, __VLS_intrinsicElements.button)({
    ...{ onClick: (__VLS_ctx.handleExecute) },
    disabled: (__VLS_ctx.workflowStore.loading),
    ...{ class: "bg-brand-500 hover:bg-brand-600 shadow-xl shadow-brand-500/30 text-white px-6 py-3 rounded-2xl flex items-center gap-2 transition-all hover:-translate-y-1 active:translate-y-0 font-bold text-sm disabled:opacity-70" },
});
if (__VLS_ctx.workflowStore.loading) {
    const __VLS_8 = {}.Loader2;
    /** @type {[typeof __VLS_components.Loader2, ]} */ ;
    // @ts-ignore
    const __VLS_9 = __VLS_asFunctionalComponent(__VLS_8, new __VLS_8({
        ...{ class: "w-4 h-4 animate-spin" },
    }));
    const __VLS_10 = __VLS_9({
        ...{ class: "w-4 h-4 animate-spin" },
    }, ...__VLS_functionalComponentArgsRest(__VLS_9));
}
else {
    const __VLS_12 = {}.Play;
    /** @type {[typeof __VLS_components.Play, ]} */ ;
    // @ts-ignore
    const __VLS_13 = __VLS_asFunctionalComponent(__VLS_12, new __VLS_12({
        ...{ class: "w-4 h-4 fill-current" },
    }));
    const __VLS_14 = __VLS_13({
        ...{ class: "w-4 h-4 fill-current" },
    }, ...__VLS_functionalComponentArgsRest(__VLS_13));
}
(__VLS_ctx.workflowStore.loading ? 'Executing...' : 'Execute');
__VLS_asFunctionalElement(__VLS_intrinsicElements.button, __VLS_intrinsicElements.button)({
    ...{ class: "absolute bottom-8 left-8 w-14 h-14 bg-white shadow-2xl shadow-slate-300 border border-slate-200 rounded-2xl flex items-center justify-center text-slate-600 hover:text-brand-600 hover:border-brand-300 transition-all hover:-translate-y-1 z-10 group" },
});
const __VLS_16 = {}.Plus;
/** @type {[typeof __VLS_components.Plus, ]} */ ;
// @ts-ignore
const __VLS_17 = __VLS_asFunctionalComponent(__VLS_16, new __VLS_16({
    ...{ class: "w-7 h-7 group-hover:rotate-90 transition-transform duration-300" },
}));
const __VLS_18 = __VLS_17({
    ...{ class: "w-7 h-7 group-hover:rotate-90 transition-transform duration-300" },
}, ...__VLS_functionalComponentArgsRest(__VLS_17));
const __VLS_20 = {}.VueFlow;
/** @type {[typeof __VLS_components.VueFlow, typeof __VLS_components.VueFlow, ]} */ ;
// @ts-ignore
const __VLS_21 = __VLS_asFunctionalComponent(__VLS_20, new __VLS_20({
    ...{ 'onNodeClick': {} },
    nodes: (__VLS_ctx.nodes),
    edges: (__VLS_ctx.edges),
    nodeTypes: ({ custom: __VLS_ctx.CustomNode }),
    ...{ class: "bg-graph-pattern" },
    defaultViewport: ({ zoom: 1.2, x: 0, y: 0 }),
    minZoom: (0.2),
    maxZoom: (4),
}));
const __VLS_22 = __VLS_21({
    ...{ 'onNodeClick': {} },
    nodes: (__VLS_ctx.nodes),
    edges: (__VLS_ctx.edges),
    nodeTypes: ({ custom: __VLS_ctx.CustomNode }),
    ...{ class: "bg-graph-pattern" },
    defaultViewport: ({ zoom: 1.2, x: 0, y: 0 }),
    minZoom: (0.2),
    maxZoom: (4),
}, ...__VLS_functionalComponentArgsRest(__VLS_21));
let __VLS_24;
let __VLS_25;
let __VLS_26;
const __VLS_27 = {
    onNodeClick: (__VLS_ctx.onNodeClick)
};
__VLS_23.slots.default;
const __VLS_28 = {}.Background;
/** @type {[typeof __VLS_components.Background, ]} */ ;
// @ts-ignore
const __VLS_29 = __VLS_asFunctionalComponent(__VLS_28, new __VLS_28({
    patternColor: "#e2e8f0",
    gap: (24),
}));
const __VLS_30 = __VLS_29({
    patternColor: "#e2e8f0",
    gap: (24),
}, ...__VLS_functionalComponentArgsRest(__VLS_29));
const __VLS_32 = {}.Controls;
/** @type {[typeof __VLS_components.Controls, ]} */ ;
// @ts-ignore
const __VLS_33 = __VLS_asFunctionalComponent(__VLS_32, new __VLS_32({
    position: "bottom-right",
    ...{ class: "!bg-white !border-slate-200 !shadow-lg !rounded-xl overflow-hidden" },
}));
const __VLS_34 = __VLS_33({
    position: "bottom-right",
    ...{ class: "!bg-white !border-slate-200 !shadow-lg !rounded-xl overflow-hidden" },
}, ...__VLS_functionalComponentArgsRest(__VLS_33));
const __VLS_36 = {}.MiniMap;
/** @type {[typeof __VLS_components.MiniMap, ]} */ ;
// @ts-ignore
const __VLS_37 = __VLS_asFunctionalComponent(__VLS_36, new __VLS_36({
    ...{ class: "!bg-white/80 !backdrop-blur-md !border-slate-200 !shadow-xl !rounded-2xl" },
}));
const __VLS_38 = __VLS_37({
    ...{ class: "!bg-white/80 !backdrop-blur-md !border-slate-200 !shadow-xl !rounded-2xl" },
}, ...__VLS_functionalComponentArgsRest(__VLS_37));
var __VLS_23;
/** @type {[typeof NodePanel, ]} */ ;
// @ts-ignore
const __VLS_40 = __VLS_asFunctionalComponent(NodePanel, new NodePanel({
    node: (__VLS_ctx.selectedNode),
}));
const __VLS_41 = __VLS_40({
    node: (__VLS_ctx.selectedNode),
}, ...__VLS_functionalComponentArgsRest(__VLS_40));
/** @type {__VLS_StyleScopedClasses['h-full']} */ ;
/** @type {__VLS_StyleScopedClasses['w-full']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['overflow-hidden']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-canvas']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-1']} */ ;
/** @type {__VLS_StyleScopedClasses['relative']} */ ;
/** @type {__VLS_StyleScopedClasses['overflow-hidden']} */ ;
/** @type {__VLS_StyleScopedClasses['absolute']} */ ;
/** @type {__VLS_StyleScopedClasses['top-6']} */ ;
/** @type {__VLS_StyleScopedClasses['left-6']} */ ;
/** @type {__VLS_StyleScopedClasses['right-6']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-between']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['z-10']} */ ;
/** @type {__VLS_StyleScopedClasses['pointer-events-none']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-white/90']} */ ;
/** @type {__VLS_StyleScopedClasses['backdrop-blur-xl']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-lg']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-slate-200/50']} */ ;
/** @type {__VLS_StyleScopedClasses['border']} */ ;
/** @type {__VLS_StyleScopedClasses['border-slate-200']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-2xl']} */ ;
/** @type {__VLS_StyleScopedClasses['px-6']} */ ;
/** @type {__VLS_StyleScopedClasses['py-3']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-4']} */ ;
/** @type {__VLS_StyleScopedClasses['pointer-events-auto']} */ ;
/** @type {__VLS_StyleScopedClasses['w-10']} */ ;
/** @type {__VLS_StyleScopedClasses['h-10']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-brand-50']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-xl']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-center']} */ ;
/** @type {__VLS_StyleScopedClasses['text-brand-600']} */ ;
/** @type {__VLS_StyleScopedClasses['w-6']} */ ;
/** @type {__VLS_StyleScopedClasses['h-6']} */ ;
/** @type {__VLS_StyleScopedClasses['font-bold']} */ ;
/** @type {__VLS_StyleScopedClasses['text-slate-800']} */ ;
/** @type {__VLS_StyleScopedClasses['text-lg']} */ ;
/** @type {__VLS_StyleScopedClasses['leading-tight']} */ ;
/** @type {__VLS_StyleScopedClasses['text-xs']} */ ;
/** @type {__VLS_StyleScopedClasses['text-slate-400']} */ ;
/** @type {__VLS_StyleScopedClasses['font-medium']} */ ;
/** @type {__VLS_StyleScopedClasses['h-8']} */ ;
/** @type {__VLS_StyleScopedClasses['w-px']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-slate-100']} */ ;
/** @type {__VLS_StyleScopedClasses['mx-2']} */ ;
/** @type {__VLS_StyleScopedClasses['px-2.5']} */ ;
/** @type {__VLS_StyleScopedClasses['py-1']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-green-100']} */ ;
/** @type {__VLS_StyleScopedClasses['text-green-700']} */ ;
/** @type {__VLS_StyleScopedClasses['text-[10px]']} */ ;
/** @type {__VLS_StyleScopedClasses['font-bold']} */ ;
/** @type {__VLS_StyleScopedClasses['uppercase']} */ ;
/** @type {__VLS_StyleScopedClasses['tracking-wider']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-lg']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-3']} */ ;
/** @type {__VLS_StyleScopedClasses['pointer-events-auto']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-white/90']} */ ;
/** @type {__VLS_StyleScopedClasses['backdrop-blur-xl']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-lg']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-slate-200/50']} */ ;
/** @type {__VLS_StyleScopedClasses['border']} */ ;
/** @type {__VLS_StyleScopedClasses['border-slate-200']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:bg-slate-50']} */ ;
/** @type {__VLS_StyleScopedClasses['text-slate-700']} */ ;
/** @type {__VLS_StyleScopedClasses['px-5']} */ ;
/** @type {__VLS_StyleScopedClasses['py-3']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-2xl']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-2']} */ ;
/** @type {__VLS_StyleScopedClasses['transition-all']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:-translate-y-0.5']} */ ;
/** @type {__VLS_StyleScopedClasses['active:translate-y-0']} */ ;
/** @type {__VLS_StyleScopedClasses['font-bold']} */ ;
/** @type {__VLS_StyleScopedClasses['text-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['w-4']} */ ;
/** @type {__VLS_StyleScopedClasses['h-4']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-brand-500']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:bg-brand-600']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-xl']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-brand-500/30']} */ ;
/** @type {__VLS_StyleScopedClasses['text-white']} */ ;
/** @type {__VLS_StyleScopedClasses['px-6']} */ ;
/** @type {__VLS_StyleScopedClasses['py-3']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-2xl']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-2']} */ ;
/** @type {__VLS_StyleScopedClasses['transition-all']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:-translate-y-1']} */ ;
/** @type {__VLS_StyleScopedClasses['active:translate-y-0']} */ ;
/** @type {__VLS_StyleScopedClasses['font-bold']} */ ;
/** @type {__VLS_StyleScopedClasses['text-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['disabled:opacity-70']} */ ;
/** @type {__VLS_StyleScopedClasses['w-4']} */ ;
/** @type {__VLS_StyleScopedClasses['h-4']} */ ;
/** @type {__VLS_StyleScopedClasses['animate-spin']} */ ;
/** @type {__VLS_StyleScopedClasses['w-4']} */ ;
/** @type {__VLS_StyleScopedClasses['h-4']} */ ;
/** @type {__VLS_StyleScopedClasses['fill-current']} */ ;
/** @type {__VLS_StyleScopedClasses['absolute']} */ ;
/** @type {__VLS_StyleScopedClasses['bottom-8']} */ ;
/** @type {__VLS_StyleScopedClasses['left-8']} */ ;
/** @type {__VLS_StyleScopedClasses['w-14']} */ ;
/** @type {__VLS_StyleScopedClasses['h-14']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-white']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-2xl']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-slate-300']} */ ;
/** @type {__VLS_StyleScopedClasses['border']} */ ;
/** @type {__VLS_StyleScopedClasses['border-slate-200']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-2xl']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-center']} */ ;
/** @type {__VLS_StyleScopedClasses['text-slate-600']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:text-brand-600']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:border-brand-300']} */ ;
/** @type {__VLS_StyleScopedClasses['transition-all']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:-translate-y-1']} */ ;
/** @type {__VLS_StyleScopedClasses['z-10']} */ ;
/** @type {__VLS_StyleScopedClasses['group']} */ ;
/** @type {__VLS_StyleScopedClasses['w-7']} */ ;
/** @type {__VLS_StyleScopedClasses['h-7']} */ ;
/** @type {__VLS_StyleScopedClasses['group-hover:rotate-90']} */ ;
/** @type {__VLS_StyleScopedClasses['transition-transform']} */ ;
/** @type {__VLS_StyleScopedClasses['duration-300']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-graph-pattern']} */ ;
/** @type {__VLS_StyleScopedClasses['!bg-white']} */ ;
/** @type {__VLS_StyleScopedClasses['!border-slate-200']} */ ;
/** @type {__VLS_StyleScopedClasses['!shadow-lg']} */ ;
/** @type {__VLS_StyleScopedClasses['!rounded-xl']} */ ;
/** @type {__VLS_StyleScopedClasses['overflow-hidden']} */ ;
/** @type {__VLS_StyleScopedClasses['!bg-white/80']} */ ;
/** @type {__VLS_StyleScopedClasses['!backdrop-blur-md']} */ ;
/** @type {__VLS_StyleScopedClasses['!border-slate-200']} */ ;
/** @type {__VLS_StyleScopedClasses['!shadow-xl']} */ ;
/** @type {__VLS_StyleScopedClasses['!rounded-2xl']} */ ;
var __VLS_dollars;
const __VLS_self = (await import('vue')).defineComponent({
    setup() {
        return {
            VueFlow: VueFlow,
            Background: Background,
            Controls: Controls,
            MiniMap: MiniMap,
            Plus: Plus,
            Play: Play,
            Save: Save,
            Settings2: Settings2,
            Loader2: Loader2,
            CustomNode: CustomNode,
            NodePanel: NodePanel,
            workflowStore: workflowStore,
            nodes: nodes,
            edges: edges,
            selectedNode: selectedNode,
            onNodeClick: onNodeClick,
            handleExecute: handleExecute,
            handleSave: handleSave,
        };
    },
});
export default (await import('vue')).defineComponent({
    setup() {
        return {};
    },
});
; /* PartiallyEnd: #4569/main.vue */
//# sourceMappingURL=WorkflowEditor.vue.js.map