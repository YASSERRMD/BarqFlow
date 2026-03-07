/// <reference types="../../node_modules/.vue-global-types/vue_3.5_0_0_0.d.ts" />
import { Handle, Position } from '@vue-flow/core';
import { Settings2, Play } from 'lucide-vue-next';
const props = defineProps();
debugger; /* PartiallyEnd: #3632/scriptSetup.vue */
const __VLS_ctx = {};
let __VLS_components;
let __VLS_directives;
// CSS variable injection 
// CSS variable injection end 
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: ([
            'min-w-[240px] glass-card rounded-[1.5rem] transition-all duration-300 group hover:shadow-node-hover hover:-translate-y-1',
            __VLS_ctx.selected ? 'border-brand-500 ring-4 ring-brand-500/20 shadow-[0_8px_30px_rgb(14,165,233,0.3)] scale-[1.02]' : 'border-white/60'
        ]) },
});
if (__VLS_ctx.data.schema.type !== 'trigger') {
    const __VLS_0 = {}.Handle;
    /** @type {[typeof __VLS_components.Handle, ]} */ ;
    // @ts-ignore
    const __VLS_1 = __VLS_asFunctionalComponent(__VLS_0, new __VLS_0({
        id: "a",
        type: "target",
        position: (__VLS_ctx.Position.Left),
        ...{ class: "!w-4 !h-6 !bg-slate-300 !border-2 !border-white !rounded-sm hover:!bg-brand-500 !-ml-2 !transition-colors !z-10" },
    }));
    const __VLS_2 = __VLS_1({
        id: "a",
        type: "target",
        position: (__VLS_ctx.Position.Left),
        ...{ class: "!w-4 !h-6 !bg-slate-300 !border-2 !border-white !rounded-sm hover:!bg-brand-500 !-ml-2 !transition-colors !z-10" },
    }, ...__VLS_functionalComponentArgsRest(__VLS_1));
}
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "px-3 py-2 flex items-center justify-between gap-3" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex items-center gap-2 overflow-hidden" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "w-6 h-6 rounded flex items-center justify-center shrink-0" },
    ...{ class: (__VLS_ctx.data.schema.type === 'trigger' ? 'text-purple-600 bg-purple-50' : 'text-brand-600 bg-brand-50') },
});
const __VLS_4 = {}.Settings2;
/** @type {[typeof __VLS_components.Settings2, ]} */ ;
// @ts-ignore
const __VLS_5 = __VLS_asFunctionalComponent(__VLS_4, new __VLS_4({
    ...{ class: "w-4 h-4" },
}));
const __VLS_6 = __VLS_5({
    ...{ class: "w-4 h-4" },
}, ...__VLS_functionalComponentArgsRest(__VLS_5));
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "min-w-0" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "text-sm font-semibold text-slate-800 truncate" },
});
(__VLS_ctx.data.label);
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "text-[10px] text-slate-500 uppercase tracking-wide truncate" },
});
(__VLS_ctx.data.schema.type);
__VLS_asFunctionalElement(__VLS_intrinsicElements.button, __VLS_intrinsicElements.button)({
    ...{ class: "opacity-0 group-hover:opacity-100 p-1.5 text-slate-400 hover:text-brand-600 hover:bg-brand-50 rounded transition-all shrink-0" },
    title: "Execute Node",
});
const __VLS_8 = {}.Play;
/** @type {[typeof __VLS_components.Play, ]} */ ;
// @ts-ignore
const __VLS_9 = __VLS_asFunctionalComponent(__VLS_8, new __VLS_8({
    ...{ class: "w-3.5 h-3.5" },
}));
const __VLS_10 = __VLS_9({
    ...{ class: "w-3.5 h-3.5" },
}, ...__VLS_functionalComponentArgsRest(__VLS_9));
if (__VLS_ctx.data.status) {
    __VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
        ...{ class: "px-3 py-1.5 border-t border-slate-100 bg-slate-50 rounded-b-lg flex items-center gap-2" },
    });
    __VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
        ...{ class: "w-2 h-2 rounded-full" },
        ...{ class: ({
                'bg-blue-500 animate-pulse': __VLS_ctx.data.status === 'running',
                'bg-green-500': __VLS_ctx.data.status === 'success',
                'bg-red-500': __VLS_ctx.data.status === 'error'
            }) },
    });
    __VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
        ...{ class: "text-xs font-medium text-slate-600 capitalize" },
    });
    (__VLS_ctx.data.status);
}
const __VLS_12 = {}.Handle;
/** @type {[typeof __VLS_components.Handle, ]} */ ;
// @ts-ignore
const __VLS_13 = __VLS_asFunctionalComponent(__VLS_12, new __VLS_12({
    id: "b",
    type: "source",
    position: (__VLS_ctx.Position.Right),
    ...{ class: "!w-4 !h-6 !bg-slate-300 !border-2 !border-white !rounded-sm hover:!bg-brand-500 !-mr-2 !transition-colors !z-10" },
}));
const __VLS_14 = __VLS_13({
    id: "b",
    type: "source",
    position: (__VLS_ctx.Position.Right),
    ...{ class: "!w-4 !h-6 !bg-slate-300 !border-2 !border-white !rounded-sm hover:!bg-brand-500 !-mr-2 !transition-colors !z-10" },
}, ...__VLS_functionalComponentArgsRest(__VLS_13));
/** @type {__VLS_StyleScopedClasses['!w-4']} */ ;
/** @type {__VLS_StyleScopedClasses['!h-6']} */ ;
/** @type {__VLS_StyleScopedClasses['!bg-slate-300']} */ ;
/** @type {__VLS_StyleScopedClasses['!border-2']} */ ;
/** @type {__VLS_StyleScopedClasses['!border-white']} */ ;
/** @type {__VLS_StyleScopedClasses['!rounded-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:!bg-brand-500']} */ ;
/** @type {__VLS_StyleScopedClasses['!-ml-2']} */ ;
/** @type {__VLS_StyleScopedClasses['!transition-colors']} */ ;
/** @type {__VLS_StyleScopedClasses['!z-10']} */ ;
/** @type {__VLS_StyleScopedClasses['px-3']} */ ;
/** @type {__VLS_StyleScopedClasses['py-2']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-between']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-3']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-2']} */ ;
/** @type {__VLS_StyleScopedClasses['overflow-hidden']} */ ;
/** @type {__VLS_StyleScopedClasses['w-6']} */ ;
/** @type {__VLS_StyleScopedClasses['h-6']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-center']} */ ;
/** @type {__VLS_StyleScopedClasses['shrink-0']} */ ;
/** @type {__VLS_StyleScopedClasses['w-4']} */ ;
/** @type {__VLS_StyleScopedClasses['h-4']} */ ;
/** @type {__VLS_StyleScopedClasses['min-w-0']} */ ;
/** @type {__VLS_StyleScopedClasses['text-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['font-semibold']} */ ;
/** @type {__VLS_StyleScopedClasses['text-slate-800']} */ ;
/** @type {__VLS_StyleScopedClasses['truncate']} */ ;
/** @type {__VLS_StyleScopedClasses['text-[10px]']} */ ;
/** @type {__VLS_StyleScopedClasses['text-slate-500']} */ ;
/** @type {__VLS_StyleScopedClasses['uppercase']} */ ;
/** @type {__VLS_StyleScopedClasses['tracking-wide']} */ ;
/** @type {__VLS_StyleScopedClasses['truncate']} */ ;
/** @type {__VLS_StyleScopedClasses['opacity-0']} */ ;
/** @type {__VLS_StyleScopedClasses['group-hover:opacity-100']} */ ;
/** @type {__VLS_StyleScopedClasses['p-1.5']} */ ;
/** @type {__VLS_StyleScopedClasses['text-slate-400']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:text-brand-600']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:bg-brand-50']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded']} */ ;
/** @type {__VLS_StyleScopedClasses['transition-all']} */ ;
/** @type {__VLS_StyleScopedClasses['shrink-0']} */ ;
/** @type {__VLS_StyleScopedClasses['w-3.5']} */ ;
/** @type {__VLS_StyleScopedClasses['h-3.5']} */ ;
/** @type {__VLS_StyleScopedClasses['px-3']} */ ;
/** @type {__VLS_StyleScopedClasses['py-1.5']} */ ;
/** @type {__VLS_StyleScopedClasses['border-t']} */ ;
/** @type {__VLS_StyleScopedClasses['border-slate-100']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-slate-50']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-b-lg']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-2']} */ ;
/** @type {__VLS_StyleScopedClasses['w-2']} */ ;
/** @type {__VLS_StyleScopedClasses['h-2']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-full']} */ ;
/** @type {__VLS_StyleScopedClasses['text-xs']} */ ;
/** @type {__VLS_StyleScopedClasses['font-medium']} */ ;
/** @type {__VLS_StyleScopedClasses['text-slate-600']} */ ;
/** @type {__VLS_StyleScopedClasses['capitalize']} */ ;
/** @type {__VLS_StyleScopedClasses['!w-4']} */ ;
/** @type {__VLS_StyleScopedClasses['!h-6']} */ ;
/** @type {__VLS_StyleScopedClasses['!bg-slate-300']} */ ;
/** @type {__VLS_StyleScopedClasses['!border-2']} */ ;
/** @type {__VLS_StyleScopedClasses['!border-white']} */ ;
/** @type {__VLS_StyleScopedClasses['!rounded-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:!bg-brand-500']} */ ;
/** @type {__VLS_StyleScopedClasses['!-mr-2']} */ ;
/** @type {__VLS_StyleScopedClasses['!transition-colors']} */ ;
/** @type {__VLS_StyleScopedClasses['!z-10']} */ ;
var __VLS_dollars;
const __VLS_self = (await import('vue')).defineComponent({
    setup() {
        return {
            Handle: Handle,
            Position: Position,
            Settings2: Settings2,
            Play: Play,
        };
    },
    __typeProps: {},
});
export default (await import('vue')).defineComponent({
    setup() {
        return {};
    },
    __typeProps: {},
});
; /* PartiallyEnd: #4569/main.vue */
//# sourceMappingURL=CustomNode.vue.js.map