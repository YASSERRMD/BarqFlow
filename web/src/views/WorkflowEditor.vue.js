/// <reference types="../../node_modules/.vue-global-types/vue_3.5_0_0_0.d.ts" />
import { ref, onMounted } from 'vue';
import { VueFlow, useVueFlow } from '@vue-flow/core';
import { Background } from '@vue-flow/background';
import { Controls } from '@vue-flow/controls';
import { MiniMap } from '@vue-flow/minimap';
import { Plus, Play, Save, Loader2 } from 'lucide-vue-next';
import CustomNode from '../components/CustomNode.vue';
import NodeCreator from '../components/NodeCreator.vue';
import NodePanel from '../components/NodePanel.vue';
import { useWorkflowStore } from '../stores/workflows';
import { useNodeStore } from '../stores/nodes';
import { useRoute } from 'vue-router';
import { v4 as uuidv4 } from 'uuid';
const route = useRoute();
const workflowStore = useWorkflowStore();
const nodeStore = useNodeStore();
const { onConnect, addEdges, toObject, setNodes, setEdges } = useVueFlow();
const nodes = ref([]);
const edges = ref([]);
const selectedNode = ref(null);
const showNodeCreator = ref(false);
// Load Nodes and Workflow on Mount
onMounted(async () => {
    await nodeStore.fetchNodeTypes();
    if (route.params.id && route.params.id !== 'new') {
        await workflowStore.fetchWorkflow(route.params.id);
        const activeWf = workflowStore.activeWorkflow;
        if (activeWf && activeWf.nodes) {
            // Reconstitute nodes mapping backend IWorkflow structure to VueFlow structure
            const loadedNodes = [];
            const loadedEdges = [];
            // Backend IWorkflow nodes are an array of node objects, connections are a map Object
            if (Array.isArray(activeWf.nodes)) {
                activeWf.nodes.forEach((n) => loadedNodes.push(n));
            }
            if (activeWf.connections) {
                Object.keys(activeWf.connections).forEach(sourceNodeName => {
                    const targets = activeWf.connections[sourceNodeName].main[0] || [];
                    targets.forEach((t) => {
                        const sourceNode = loadedNodes.find(n => n.data.label === sourceNodeName);
                        const targetNode = loadedNodes.find(n => n.data.label === t.node);
                        if (sourceNode && targetNode) {
                            loadedEdges.push({ id: `e-${sourceNode.id}-${targetNode.id}`, source: sourceNode.id, target: targetNode.id, animated: true, style: { stroke: '#0ea5e9', strokeWidth: 2 } });
                        }
                    });
                });
            }
            setNodes(loadedNodes);
            setEdges(loadedEdges);
            nodes.value = loadedNodes;
            edges.value = loadedEdges;
        }
    }
});
onConnect((params) => {
    addEdges([{
            ...params,
            animated: true,
            style: { stroke: '#0ea5e9', strokeWidth: 2 }
        }]);
});
function onNodeClick({ node }) {
    selectedNode.value = node;
}
async function handleExecute() {
    if (workflowStore.loading)
        return;
    nodes.value.forEach(n => n.data.status = 'running');
    try {
        const wfId = route.params.id !== 'new' ? route.params.id : '00000000-0000-0000-0000-000000000000';
        const result = await workflowStore.executeWorkflow(wfId);
        nodes.value.forEach(n => {
            const nodeName = n.data.label;
            if (result.data && result.data[nodeName]) {
                n.data.status = result.data[nodeName].success ? 'success' : 'error';
            }
            else {
                n.data.status = 'success';
            }
        });
    }
    catch (err) {
        nodes.value.forEach(n => n.data.status = 'error');
    }
}
async function handleSave() {
    const flow = toObject();
    // Translate VueFlow topology to internal BarqFlow JSON schemas
    const payloadConnections = {};
    flow.edges.forEach(edge => {
        const sourceNode = flow.nodes.find(n => n.id === edge.source);
        const targetNode = flow.nodes.find(n => n.id === edge.target);
        if (sourceNode && targetNode) {
            if (!payloadConnections[sourceNode.data.label]) {
                payloadConnections[sourceNode.data.label] = { main: [[]] };
            }
            payloadConnections[sourceNode.data.label].main[0].push({
                node: targetNode.data.label,
                type: "main",
                index: 0
            });
        }
    });
    const payloadStr = {
        id: route.params.id !== 'new' ? route.params.id : undefined,
        name: workflowStore.activeWorkflow?.name || 'My New Workflow',
        nodes: flow.nodes,
        connections: payloadConnections,
        settings: {}
    };
    await workflowStore.saveWorkflow(payloadStr);
}
function onDragStart(event, nodeTypeObj) {
    if (event.dataTransfer) {
        event.dataTransfer.setData('application/vueflow', JSON.stringify(nodeTypeObj));
        event.dataTransfer.effectAllowed = 'move';
    }
}
function onDrop(event) {
    const nodeDataStr = event.dataTransfer?.getData('application/vueflow');
    if (!nodeDataStr)
        return;
    const nodeSchema = JSON.parse(nodeDataStr);
    // Calculate drag position taking into account the canvas bounding box and scale
    const position = { x: event.offsetX, y: event.offsetY };
    // Set default properties based on schema if not exist
    const propertiesObj = {};
    if (nodeSchema.schema && nodeSchema.schema.properties) {
        nodeSchema.schema.properties.forEach((p) => {
            if (p.default !== undefined)
                propertiesObj[p.name] = p.default;
        });
    }
    const newNode = {
        id: uuidv4(),
        type: 'custom',
        position,
        data: {
            type: nodeSchema.type,
            label: nodeSchema.name, // Usually needs to be unique per node class
            description: nodeSchema.description,
            status: null,
            schema: nodeSchema.schema,
            properties: propertiesObj
        }
    };
    nodes.value.push(newNode);
}
debugger; /* PartiallyEnd: #3632/scriptSetup.vue */
const __VLS_ctx = {};
let __VLS_components;
let __VLS_directives;
// CSS variable injection 
// CSS variable injection end 
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "h-full w-full flex overflow-hidden bg-transparent" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex-1 relative overflow-hidden" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "absolute top-4 left-4 right-4 flex justify-between items-center z-10 pointer-events-none" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "bg-white rounded-lg shadow-sm border border-slate-200 px-4 py-2 flex items-center gap-3 pointer-events-auto" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({});
__VLS_asFunctionalElement(__VLS_intrinsicElements.h1, __VLS_intrinsicElements.h1)({
    ...{ class: "font-bold text-slate-800 text-base leading-tight" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.p, __VLS_intrinsicElements.p)({
    ...{ class: "text-xs text-slate-500" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "h-6 w-px bg-slate-200 mx-1" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
    ...{ class: "px-2 py-1 bg-green-100 border border-green-200 text-green-700 text-[10px] font-bold uppercase tracking-wider rounded" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex gap-2 pointer-events-auto" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.button, __VLS_intrinsicElements.button)({
    ...{ onClick: (__VLS_ctx.handleSave) },
    ...{ class: "bg-white hover:bg-slate-50 border border-slate-200 text-slate-700 px-4 py-2 rounded-lg flex items-center gap-2 transition-colors font-semibold text-sm shadow-sm" },
});
const __VLS_0 = {}.Save;
/** @type {[typeof __VLS_components.Save, ]} */ ;
// @ts-ignore
const __VLS_1 = __VLS_asFunctionalComponent(__VLS_0, new __VLS_0({
    ...{ class: "w-4 h-4" },
}));
const __VLS_2 = __VLS_1({
    ...{ class: "w-4 h-4" },
}, ...__VLS_functionalComponentArgsRest(__VLS_1));
__VLS_asFunctionalElement(__VLS_intrinsicElements.button, __VLS_intrinsicElements.button)({
    ...{ onClick: (__VLS_ctx.handleExecute) },
    disabled: (__VLS_ctx.workflowStore.loading),
    ...{ class: "bg-brand-500 hover:bg-brand-600 text-white px-4 py-2 rounded-lg flex items-center gap-2 transition-colors font-semibold text-sm disabled:opacity-70 shadow-sm" },
});
if (__VLS_ctx.workflowStore.loading) {
    const __VLS_4 = {}.Loader2;
    /** @type {[typeof __VLS_components.Loader2, ]} */ ;
    // @ts-ignore
    const __VLS_5 = __VLS_asFunctionalComponent(__VLS_4, new __VLS_4({
        ...{ class: "w-4 h-4 animate-spin" },
    }));
    const __VLS_6 = __VLS_5({
        ...{ class: "w-4 h-4 animate-spin" },
    }, ...__VLS_functionalComponentArgsRest(__VLS_5));
}
else {
    const __VLS_8 = {}.Play;
    /** @type {[typeof __VLS_components.Play, ]} */ ;
    // @ts-ignore
    const __VLS_9 = __VLS_asFunctionalComponent(__VLS_8, new __VLS_8({
        ...{ class: "w-4 h-4 fill-current" },
    }));
    const __VLS_10 = __VLS_9({
        ...{ class: "w-4 h-4 fill-current" },
    }, ...__VLS_functionalComponentArgsRest(__VLS_9));
}
(__VLS_ctx.workflowStore.loading ? 'Executing...' : 'Execute Workflow');
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "absolute bottom-6 right-6 z-10 pointer-events-auto" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.button, __VLS_intrinsicElements.button)({
    ...{ onClick: (...[$event]) => {
            __VLS_ctx.showNodeCreator = true;
        } },
    ...{ class: "w-12 h-12 bg-brand-500 shadow-lg text-white rounded-full flex items-center justify-center hover:bg-brand-600 hover:scale-105 transition-all" },
});
const __VLS_12 = {}.Plus;
/** @type {[typeof __VLS_components.Plus, ]} */ ;
// @ts-ignore
const __VLS_13 = __VLS_asFunctionalComponent(__VLS_12, new __VLS_12({
    ...{ class: "w-6 h-6" },
}));
const __VLS_14 = __VLS_13({
    ...{ class: "w-6 h-6" },
}, ...__VLS_functionalComponentArgsRest(__VLS_13));
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ onDrop: (__VLS_ctx.onDrop) },
    ...{ onDragover: () => { } },
    ...{ class: "h-full w-full bg-[#f8f9fa]" },
});
const __VLS_16 = {}.VueFlow;
/** @type {[typeof __VLS_components.VueFlow, typeof __VLS_components.VueFlow, ]} */ ;
// @ts-ignore
const __VLS_17 = __VLS_asFunctionalComponent(__VLS_16, new __VLS_16({
    ...{ 'onNodeClick': {} },
    nodes: (__VLS_ctx.nodes),
    edges: (__VLS_ctx.edges),
    nodeTypes: ({ custom: __VLS_ctx.CustomNode }),
    ...{ class: "n8n-canvas" },
    defaultViewport: ({ zoom: 1, x: 0, y: 0 }),
    minZoom: (0.2),
    maxZoom: (2),
}));
const __VLS_18 = __VLS_17({
    ...{ 'onNodeClick': {} },
    nodes: (__VLS_ctx.nodes),
    edges: (__VLS_ctx.edges),
    nodeTypes: ({ custom: __VLS_ctx.CustomNode }),
    ...{ class: "n8n-canvas" },
    defaultViewport: ({ zoom: 1, x: 0, y: 0 }),
    minZoom: (0.2),
    maxZoom: (2),
}, ...__VLS_functionalComponentArgsRest(__VLS_17));
let __VLS_20;
let __VLS_21;
let __VLS_22;
const __VLS_23 = {
    onNodeClick: (__VLS_ctx.onNodeClick)
};
__VLS_19.slots.default;
const __VLS_24 = {}.Background;
/** @type {[typeof __VLS_components.Background, ]} */ ;
// @ts-ignore
const __VLS_25 = __VLS_asFunctionalComponent(__VLS_24, new __VLS_24({
    patternColor: "#ccc",
    gap: (20),
}));
const __VLS_26 = __VLS_25({
    patternColor: "#ccc",
    gap: (20),
}, ...__VLS_functionalComponentArgsRest(__VLS_25));
const __VLS_28 = {}.Controls;
/** @type {[typeof __VLS_components.Controls, ]} */ ;
// @ts-ignore
const __VLS_29 = __VLS_asFunctionalComponent(__VLS_28, new __VLS_28({
    position: "bottom-left",
    ...{ class: "!bg-white !border-slate-200 !shadow-sm !rounded-md overflow-hidden mb-6 ml-6" },
}));
const __VLS_30 = __VLS_29({
    position: "bottom-left",
    ...{ class: "!bg-white !border-slate-200 !shadow-sm !rounded-md overflow-hidden mb-6 ml-6" },
}, ...__VLS_functionalComponentArgsRest(__VLS_29));
const __VLS_32 = {}.MiniMap;
/** @type {[typeof __VLS_components.MiniMap, ]} */ ;
// @ts-ignore
const __VLS_33 = __VLS_asFunctionalComponent(__VLS_32, new __VLS_32({
    ...{ class: "!bg-white !border-slate-200 !shadow-sm !rounded-md mr-20 mb-6" },
}));
const __VLS_34 = __VLS_33({
    ...{ class: "!bg-white !border-slate-200 !shadow-sm !rounded-md mr-20 mb-6" },
}, ...__VLS_functionalComponentArgsRest(__VLS_33));
var __VLS_19;
/** @type {[typeof NodeCreator, ]} */ ;
// @ts-ignore
const __VLS_36 = __VLS_asFunctionalComponent(NodeCreator, new NodeCreator({
    ...{ 'onClose': {} },
    ...{ 'onDragstart': {} },
    show: (__VLS_ctx.showNodeCreator),
}));
const __VLS_37 = __VLS_36({
    ...{ 'onClose': {} },
    ...{ 'onDragstart': {} },
    show: (__VLS_ctx.showNodeCreator),
}, ...__VLS_functionalComponentArgsRest(__VLS_36));
let __VLS_39;
let __VLS_40;
let __VLS_41;
const __VLS_42 = {
    onClose: (...[$event]) => {
        __VLS_ctx.showNodeCreator = false;
    }
};
const __VLS_43 = {
    onDragstart: (__VLS_ctx.onDragStart)
};
var __VLS_38;
/** @type {[typeof NodePanel, ]} */ ;
// @ts-ignore
const __VLS_44 = __VLS_asFunctionalComponent(NodePanel, new NodePanel({
    ...{ 'onClose': {} },
    node: (__VLS_ctx.selectedNode),
}));
const __VLS_45 = __VLS_44({
    ...{ 'onClose': {} },
    node: (__VLS_ctx.selectedNode),
}, ...__VLS_functionalComponentArgsRest(__VLS_44));
let __VLS_47;
let __VLS_48;
let __VLS_49;
const __VLS_50 = {
    onClose: (...[$event]) => {
        __VLS_ctx.selectedNode = null;
    }
};
var __VLS_46;
/** @type {__VLS_StyleScopedClasses['h-full']} */ ;
/** @type {__VLS_StyleScopedClasses['w-full']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['overflow-hidden']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-transparent']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-1']} */ ;
/** @type {__VLS_StyleScopedClasses['relative']} */ ;
/** @type {__VLS_StyleScopedClasses['overflow-hidden']} */ ;
/** @type {__VLS_StyleScopedClasses['absolute']} */ ;
/** @type {__VLS_StyleScopedClasses['top-4']} */ ;
/** @type {__VLS_StyleScopedClasses['left-4']} */ ;
/** @type {__VLS_StyleScopedClasses['right-4']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-between']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['z-10']} */ ;
/** @type {__VLS_StyleScopedClasses['pointer-events-none']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-white']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-lg']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['border']} */ ;
/** @type {__VLS_StyleScopedClasses['border-slate-200']} */ ;
/** @type {__VLS_StyleScopedClasses['px-4']} */ ;
/** @type {__VLS_StyleScopedClasses['py-2']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-3']} */ ;
/** @type {__VLS_StyleScopedClasses['pointer-events-auto']} */ ;
/** @type {__VLS_StyleScopedClasses['font-bold']} */ ;
/** @type {__VLS_StyleScopedClasses['text-slate-800']} */ ;
/** @type {__VLS_StyleScopedClasses['text-base']} */ ;
/** @type {__VLS_StyleScopedClasses['leading-tight']} */ ;
/** @type {__VLS_StyleScopedClasses['text-xs']} */ ;
/** @type {__VLS_StyleScopedClasses['text-slate-500']} */ ;
/** @type {__VLS_StyleScopedClasses['h-6']} */ ;
/** @type {__VLS_StyleScopedClasses['w-px']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-slate-200']} */ ;
/** @type {__VLS_StyleScopedClasses['mx-1']} */ ;
/** @type {__VLS_StyleScopedClasses['px-2']} */ ;
/** @type {__VLS_StyleScopedClasses['py-1']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-green-100']} */ ;
/** @type {__VLS_StyleScopedClasses['border']} */ ;
/** @type {__VLS_StyleScopedClasses['border-green-200']} */ ;
/** @type {__VLS_StyleScopedClasses['text-green-700']} */ ;
/** @type {__VLS_StyleScopedClasses['text-[10px]']} */ ;
/** @type {__VLS_StyleScopedClasses['font-bold']} */ ;
/** @type {__VLS_StyleScopedClasses['uppercase']} */ ;
/** @type {__VLS_StyleScopedClasses['tracking-wider']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-2']} */ ;
/** @type {__VLS_StyleScopedClasses['pointer-events-auto']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-white']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:bg-slate-50']} */ ;
/** @type {__VLS_StyleScopedClasses['border']} */ ;
/** @type {__VLS_StyleScopedClasses['border-slate-200']} */ ;
/** @type {__VLS_StyleScopedClasses['text-slate-700']} */ ;
/** @type {__VLS_StyleScopedClasses['px-4']} */ ;
/** @type {__VLS_StyleScopedClasses['py-2']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-lg']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-2']} */ ;
/** @type {__VLS_StyleScopedClasses['transition-colors']} */ ;
/** @type {__VLS_StyleScopedClasses['font-semibold']} */ ;
/** @type {__VLS_StyleScopedClasses['text-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['w-4']} */ ;
/** @type {__VLS_StyleScopedClasses['h-4']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-brand-500']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:bg-brand-600']} */ ;
/** @type {__VLS_StyleScopedClasses['text-white']} */ ;
/** @type {__VLS_StyleScopedClasses['px-4']} */ ;
/** @type {__VLS_StyleScopedClasses['py-2']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-lg']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-2']} */ ;
/** @type {__VLS_StyleScopedClasses['transition-colors']} */ ;
/** @type {__VLS_StyleScopedClasses['font-semibold']} */ ;
/** @type {__VLS_StyleScopedClasses['text-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['disabled:opacity-70']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['w-4']} */ ;
/** @type {__VLS_StyleScopedClasses['h-4']} */ ;
/** @type {__VLS_StyleScopedClasses['animate-spin']} */ ;
/** @type {__VLS_StyleScopedClasses['w-4']} */ ;
/** @type {__VLS_StyleScopedClasses['h-4']} */ ;
/** @type {__VLS_StyleScopedClasses['fill-current']} */ ;
/** @type {__VLS_StyleScopedClasses['absolute']} */ ;
/** @type {__VLS_StyleScopedClasses['bottom-6']} */ ;
/** @type {__VLS_StyleScopedClasses['right-6']} */ ;
/** @type {__VLS_StyleScopedClasses['z-10']} */ ;
/** @type {__VLS_StyleScopedClasses['pointer-events-auto']} */ ;
/** @type {__VLS_StyleScopedClasses['w-12']} */ ;
/** @type {__VLS_StyleScopedClasses['h-12']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-brand-500']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-lg']} */ ;
/** @type {__VLS_StyleScopedClasses['text-white']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-full']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-center']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:bg-brand-600']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:scale-105']} */ ;
/** @type {__VLS_StyleScopedClasses['transition-all']} */ ;
/** @type {__VLS_StyleScopedClasses['w-6']} */ ;
/** @type {__VLS_StyleScopedClasses['h-6']} */ ;
/** @type {__VLS_StyleScopedClasses['h-full']} */ ;
/** @type {__VLS_StyleScopedClasses['w-full']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-[#f8f9fa]']} */ ;
/** @type {__VLS_StyleScopedClasses['n8n-canvas']} */ ;
/** @type {__VLS_StyleScopedClasses['!bg-white']} */ ;
/** @type {__VLS_StyleScopedClasses['!border-slate-200']} */ ;
/** @type {__VLS_StyleScopedClasses['!shadow-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['!rounded-md']} */ ;
/** @type {__VLS_StyleScopedClasses['overflow-hidden']} */ ;
/** @type {__VLS_StyleScopedClasses['mb-6']} */ ;
/** @type {__VLS_StyleScopedClasses['ml-6']} */ ;
/** @type {__VLS_StyleScopedClasses['!bg-white']} */ ;
/** @type {__VLS_StyleScopedClasses['!border-slate-200']} */ ;
/** @type {__VLS_StyleScopedClasses['!shadow-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['!rounded-md']} */ ;
/** @type {__VLS_StyleScopedClasses['mr-20']} */ ;
/** @type {__VLS_StyleScopedClasses['mb-6']} */ ;
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
            Loader2: Loader2,
            CustomNode: CustomNode,
            NodeCreator: NodeCreator,
            NodePanel: NodePanel,
            workflowStore: workflowStore,
            nodes: nodes,
            edges: edges,
            selectedNode: selectedNode,
            showNodeCreator: showNodeCreator,
            onNodeClick: onNodeClick,
            handleExecute: handleExecute,
            handleSave: handleSave,
            onDragStart: onDragStart,
            onDrop: onDrop,
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