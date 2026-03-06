/// <reference types="../../node_modules/.vue-global-types/vue_3.5_0_0_0.d.ts" />
import { ref } from 'vue';
import GlassNode from './NodeEditor/GlassNode.vue';
// Basic mock state
const nodes = ref([
    { id: '1', title: 'Webhook Trigger', description: 'Listens for incoming HTTP POSTs across your custom endpoints.', x: 150, y: 200 },
    { id: '2', title: 'Rhai Sandbox', description: 'Executes your custom scripts securely mapping outputs properly.', x: 550, y: 250 }
]);
const wires = ref([
    { startX: 410, startY: 280, endX: 550, endY: 330 } // Mock wire
]);
// Drag State
const draggedNode = ref(null);
const dragOffset = ref({ x: 0, y: 0 });
// Wire State
const isDrawingWire = ref(false);
const drawingWireStart = ref({ x: 0, y: 0 });
const mousePos = ref({ x: 0, y: 0 });
const generateBezierPath = (x1, y1, x2, y2) => {
    const dx = Math.abs(x2 - x1) * 0.5;
    return `M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}`;
};
const startDragNode = (event, node) => {
    draggedNode.value = node;
    dragOffset.value = {
        x: event.clientX - node.x,
        y: event.clientY - node.y
    };
};
const onMouseMove = (event) => {
    mousePos.value = { x: event.clientX, y: event.clientY - 60 }; // Account for 60px TopNav
    if (draggedNode.value) {
        draggedNode.value.x = mousePos.value.x - dragOffset.value.x;
        draggedNode.value.y = mousePos.value.y + 60 - dragOffset.value.y;
    }
};
const onMouseUp = () => {
    draggedNode.value = null;
    isDrawingWire.value = false;
};
debugger; /* PartiallyEnd: #3632/scriptSetup.vue */
const __VLS_ctx = {};
let __VLS_components;
let __VLS_directives;
/** @type {__VLS_StyleScopedClasses['wire-drawing']} */ ;
// CSS variable injection 
// CSS variable injection end 
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ onMousemove: (__VLS_ctx.onMouseMove) },
    ...{ onMouseup: (__VLS_ctx.onMouseUp) },
    ...{ onMouseleave: (__VLS_ctx.onMouseUp) },
    ...{ class: "canvas-wrapper" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.svg, __VLS_intrinsicElements.svg)({
    ...{ class: "wires-layer" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.defs, __VLS_intrinsicElements.defs)({});
__VLS_asFunctionalElement(__VLS_intrinsicElements.linearGradient, __VLS_intrinsicElements.linearGradient)({
    id: "neonGradient",
    x1: "0%",
    y1: "0%",
    x2: "100%",
    y2: "0%",
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.stop)({
    offset: "0%",
    'stop-color': "var(--neon-cyan)",
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.stop)({
    offset: "100%",
    'stop-color': "var(--neon-purple)",
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.filter, __VLS_intrinsicElements.filter)({
    id: "glow",
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.feGaussianBlur)({
    stdDeviation: "3",
    result: "coloredBlur",
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.feMerge, __VLS_intrinsicElements.feMerge)({});
__VLS_asFunctionalElement(__VLS_intrinsicElements.feMergeNode)({
    in: "coloredBlur",
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.feMergeNode)({
    in: "SourceGraphic",
});
for (const [wire, i] of __VLS_getVForSourceType((__VLS_ctx.wires))) {
    __VLS_asFunctionalElement(__VLS_intrinsicElements.path)({
        key: (i),
        d: (__VLS_ctx.generateBezierPath(wire.startX, wire.startY, wire.endX, wire.endY)),
        ...{ class: "wire-path" },
    });
}
if (__VLS_ctx.isDrawingWire) {
    __VLS_asFunctionalElement(__VLS_intrinsicElements.path)({
        d: (__VLS_ctx.generateBezierPath(__VLS_ctx.drawingWireStart.x, __VLS_ctx.drawingWireStart.y, __VLS_ctx.mousePos.x, __VLS_ctx.mousePos.y)),
        ...{ class: "wire-drawing" },
    });
}
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "nodes-layer" },
});
for (const [node] of __VLS_getVForSourceType((__VLS_ctx.nodes))) {
    /** @type {[typeof GlassNode, ]} */ ;
    // @ts-ignore
    const __VLS_0 = __VLS_asFunctionalComponent(GlassNode, new GlassNode({
        ...{ 'onDragStart': {} },
        key: (node.id),
        title: (node.title),
        description: (node.description),
        x: (node.x),
        y: (node.y),
    }));
    const __VLS_1 = __VLS_0({
        ...{ 'onDragStart': {} },
        key: (node.id),
        title: (node.title),
        description: (node.description),
        x: (node.x),
        y: (node.y),
    }, ...__VLS_functionalComponentArgsRest(__VLS_0));
    let __VLS_3;
    let __VLS_4;
    let __VLS_5;
    const __VLS_6 = {
        onDragStart: (...[$event]) => {
            __VLS_ctx.startDragNode($event, node);
        }
    };
    var __VLS_2;
}
/** @type {__VLS_StyleScopedClasses['canvas-wrapper']} */ ;
/** @type {__VLS_StyleScopedClasses['wires-layer']} */ ;
/** @type {__VLS_StyleScopedClasses['wire-path']} */ ;
/** @type {__VLS_StyleScopedClasses['wire-drawing']} */ ;
/** @type {__VLS_StyleScopedClasses['nodes-layer']} */ ;
var __VLS_dollars;
const __VLS_self = (await import('vue')).defineComponent({
    setup() {
        return {
            GlassNode: GlassNode,
            nodes: nodes,
            wires: wires,
            isDrawingWire: isDrawingWire,
            drawingWireStart: drawingWireStart,
            mousePos: mousePos,
            generateBezierPath: generateBezierPath,
            startDragNode: startDragNode,
            onMouseMove: onMouseMove,
            onMouseUp: onMouseUp,
        };
    },
});
export default (await import('vue')).defineComponent({
    setup() {
        return {};
    },
});
; /* PartiallyEnd: #4569/main.vue */
//# sourceMappingURL=Canvas.vue.js.map