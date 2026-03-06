/// <reference types="../../node_modules/.vue-global-types/vue_3.5_0_0_0.d.ts" />
import { Handle, Position } from '@vue-flow/core';
import { Settings2, Play, AlertCircle, CheckCircle2 } from 'lucide-vue-next';
const props = defineProps();
debugger; /* PartiallyEnd: #3632/scriptSetup.vue */
const __VLS_ctx = {};
let __VLS_components;
let __VLS_directives;
// CSS variable injection 
// CSS variable injection end 
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: ([
            'min-w-[180px] bg-white rounded-xl shadow-node border-2 transition-all duration-200 group',
            __VLS_ctx.selected ? 'border-brand-500 ring-4 ring-brand-500/10 shadow-lg scale-102' : 'border-slate-200 hover:border-slate-300'
        ]) },
});
if (__VLS_ctx.data.status) {
    __VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
        ...{ class: "absolute -top-2 -right-2 z-10" },
    });
    if (__VLS_ctx.data.status === 'success') {
        __VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
            ...{ class: "bg-green-500 text-white p-1 rounded-full shadow-sm" },
        });
        const __VLS_0 = {}.CheckCircle2;
        /** @type {[typeof __VLS_components.CheckCircle2, ]} */ ;
        // @ts-ignore
        const __VLS_1 = __VLS_asFunctionalComponent(__VLS_0, new __VLS_0({
            ...{ class: "w-3.5 h-3.5" },
        }));
        const __VLS_2 = __VLS_1({
            ...{ class: "w-3.5 h-3.5" },
        }, ...__VLS_functionalComponentArgsRest(__VLS_1));
    }
    else if (__VLS_ctx.data.status === 'error') {
        __VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
            ...{ class: "bg-red-500 text-white p-1 rounded-full shadow-sm" },
        });
        const __VLS_4 = {}.AlertCircle;
        /** @type {[typeof __VLS_components.AlertCircle, ]} */ ;
        // @ts-ignore
        const __VLS_5 = __VLS_asFunctionalComponent(__VLS_4, new __VLS_4({
            ...{ class: "w-3.5 h-3.5" },
        }));
        const __VLS_6 = __VLS_5({
            ...{ class: "w-3.5 h-3.5" },
        }, ...__VLS_functionalComponentArgsRest(__VLS_5));
    }
    else if (__VLS_ctx.data.status === 'running') {
        __VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
            ...{ class: "bg-brand-500 text-white p-1 rounded-full shadow-sm animate-pulse" },
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
    }
}
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "px-4 py-3 border-b border-slate-100 flex items-center gap-3" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: ([
            'w-8 h-8 rounded-lg flex items-center justify-center shadow-sm transition-colors',
            __VLS_ctx.data.type === 'trigger' ? 'bg-purple-100 text-purple-600' : 'bg-brand-100 text-brand-600'
        ]) },
});
const __VLS_12 = {}.Settings2;
/** @type {[typeof __VLS_components.Settings2, ]} */ ;
// @ts-ignore
const __VLS_13 = __VLS_asFunctionalComponent(__VLS_12, new __VLS_12({
    ...{ class: "w-5 h-5" },
}));
const __VLS_14 = __VLS_13({
    ...{ class: "w-5 h-5" },
}, ...__VLS_functionalComponentArgsRest(__VLS_13));
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex-1 min-w-0" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.h3, __VLS_intrinsicElements.h3)({
    ...{ class: "text-sm font-semibold text-slate-800 truncate" },
});
(__VLS_ctx.data.label);
__VLS_asFunctionalElement(__VLS_intrinsicElements.p, __VLS_intrinsicElements.p)({
    ...{ class: "text-[10px] text-slate-400 font-medium uppercase tracking-wider" },
});
(__VLS_ctx.data.type);
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "px-4 py-3 bg-slate-50/50 rounded-b-xl" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.p, __VLS_intrinsicElements.p)({
    ...{ class: "text-[11px] text-slate-500 line-clamp-2 leading-relaxed italic" },
});
(__VLS_ctx.data.description || 'No configuration set');
if (__VLS_ctx.data.type !== 'trigger') {
    const __VLS_16 = {}.Handle;
    /** @type {[typeof __VLS_components.Handle, ]} */ ;
    // @ts-ignore
    const __VLS_17 = __VLS_asFunctionalComponent(__VLS_16, new __VLS_16({
        type: "target",
        position: (__VLS_ctx.Position.Left),
        ...{ class: "!w-3 !h-3 !bg-white !border-2 !border-slate-300 hover:!border-brand-400 !transition-colors !z-20" },
    }));
    const __VLS_18 = __VLS_17({
        type: "target",
        position: (__VLS_ctx.Position.Left),
        ...{ class: "!w-3 !h-3 !bg-white !border-2 !border-slate-300 hover:!border-brand-400 !transition-colors !z-20" },
    }, ...__VLS_functionalComponentArgsRest(__VLS_17));
}
const __VLS_20 = {}.Handle;
/** @type {[typeof __VLS_components.Handle, ]} */ ;
// @ts-ignore
const __VLS_21 = __VLS_asFunctionalComponent(__VLS_20, new __VLS_20({
    type: "source",
    position: (__VLS_ctx.Position.Right),
    ...{ class: "!w-3 !h-3 !bg-white !border-2 !border-slate-300 hover:!border-brand-400 !transition-colors !z-20" },
}));
const __VLS_22 = __VLS_21({
    type: "source",
    position: (__VLS_ctx.Position.Right),
    ...{ class: "!w-3 !h-3 !bg-white !border-2 !border-slate-300 hover:!border-brand-400 !transition-colors !z-20" },
}, ...__VLS_functionalComponentArgsRest(__VLS_21));
/** @type {__VLS_StyleScopedClasses['absolute']} */ ;
/** @type {__VLS_StyleScopedClasses['-top-2']} */ ;
/** @type {__VLS_StyleScopedClasses['-right-2']} */ ;
/** @type {__VLS_StyleScopedClasses['z-10']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-green-500']} */ ;
/** @type {__VLS_StyleScopedClasses['text-white']} */ ;
/** @type {__VLS_StyleScopedClasses['p-1']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-full']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['w-3.5']} */ ;
/** @type {__VLS_StyleScopedClasses['h-3.5']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-red-500']} */ ;
/** @type {__VLS_StyleScopedClasses['text-white']} */ ;
/** @type {__VLS_StyleScopedClasses['p-1']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-full']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['w-3.5']} */ ;
/** @type {__VLS_StyleScopedClasses['h-3.5']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-brand-500']} */ ;
/** @type {__VLS_StyleScopedClasses['text-white']} */ ;
/** @type {__VLS_StyleScopedClasses['p-1']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-full']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['animate-pulse']} */ ;
/** @type {__VLS_StyleScopedClasses['w-3.5']} */ ;
/** @type {__VLS_StyleScopedClasses['h-3.5']} */ ;
/** @type {__VLS_StyleScopedClasses['px-4']} */ ;
/** @type {__VLS_StyleScopedClasses['py-3']} */ ;
/** @type {__VLS_StyleScopedClasses['border-b']} */ ;
/** @type {__VLS_StyleScopedClasses['border-slate-100']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-3']} */ ;
/** @type {__VLS_StyleScopedClasses['w-5']} */ ;
/** @type {__VLS_StyleScopedClasses['h-5']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-1']} */ ;
/** @type {__VLS_StyleScopedClasses['min-w-0']} */ ;
/** @type {__VLS_StyleScopedClasses['text-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['font-semibold']} */ ;
/** @type {__VLS_StyleScopedClasses['text-slate-800']} */ ;
/** @type {__VLS_StyleScopedClasses['truncate']} */ ;
/** @type {__VLS_StyleScopedClasses['text-[10px]']} */ ;
/** @type {__VLS_StyleScopedClasses['text-slate-400']} */ ;
/** @type {__VLS_StyleScopedClasses['font-medium']} */ ;
/** @type {__VLS_StyleScopedClasses['uppercase']} */ ;
/** @type {__VLS_StyleScopedClasses['tracking-wider']} */ ;
/** @type {__VLS_StyleScopedClasses['px-4']} */ ;
/** @type {__VLS_StyleScopedClasses['py-3']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-slate-50/50']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-b-xl']} */ ;
/** @type {__VLS_StyleScopedClasses['text-[11px]']} */ ;
/** @type {__VLS_StyleScopedClasses['text-slate-500']} */ ;
/** @type {__VLS_StyleScopedClasses['line-clamp-2']} */ ;
/** @type {__VLS_StyleScopedClasses['leading-relaxed']} */ ;
/** @type {__VLS_StyleScopedClasses['italic']} */ ;
/** @type {__VLS_StyleScopedClasses['!w-3']} */ ;
/** @type {__VLS_StyleScopedClasses['!h-3']} */ ;
/** @type {__VLS_StyleScopedClasses['!bg-white']} */ ;
/** @type {__VLS_StyleScopedClasses['!border-2']} */ ;
/** @type {__VLS_StyleScopedClasses['!border-slate-300']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:!border-brand-400']} */ ;
/** @type {__VLS_StyleScopedClasses['!transition-colors']} */ ;
/** @type {__VLS_StyleScopedClasses['!z-20']} */ ;
/** @type {__VLS_StyleScopedClasses['!w-3']} */ ;
/** @type {__VLS_StyleScopedClasses['!h-3']} */ ;
/** @type {__VLS_StyleScopedClasses['!bg-white']} */ ;
/** @type {__VLS_StyleScopedClasses['!border-2']} */ ;
/** @type {__VLS_StyleScopedClasses['!border-slate-300']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:!border-brand-400']} */ ;
/** @type {__VLS_StyleScopedClasses['!transition-colors']} */ ;
/** @type {__VLS_StyleScopedClasses['!z-20']} */ ;
var __VLS_dollars;
const __VLS_self = (await import('vue')).defineComponent({
    setup() {
        return {
            Handle: Handle,
            Position: Position,
            Settings2: Settings2,
            Play: Play,
            AlertCircle: AlertCircle,
            CheckCircle2: CheckCircle2,
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