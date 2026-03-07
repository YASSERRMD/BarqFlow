import { Settings2, Globe, Database, Bot, Variable, GitBranch, GitMerge, FileCode2, Clock, Play } from 'lucide-vue-next'
import type { Component } from 'vue'

export interface NodeVisualMeta {
    icon: Component;
    color: string;
    bgColor: string;
    iconBgColor: string;
    iconColor: string;
}

export function getNodeVisuals(nodeTypeName: string): NodeVisualMeta {
    switch (nodeTypeName) {
        case 'barqflow-nodes.postgres':
            return { icon: Database, color: '#2563eb', bgColor: '#FFFFFF', iconBgColor: '#eff6ff', iconColor: '#2563eb' } // Blue
        case 'barqflow-nodes.openai':
        case 'barqflow-nodes.ollama':
            return { icon: Bot, color: '#10a37f', bgColor: '#FFFFFF', iconBgColor: '#f0fdf4', iconColor: '#16a34a' } // Green
        case 'barqflow-nodes.httpRequest':
            return { icon: Globe, color: '#0ea5e9', bgColor: '#FFFFFF', iconBgColor: '#f0f9ff', iconColor: '#0ea5e9' } // Light Blue
        case 'barqflow-nodes.if':
        case 'barqflow-nodes.switch':
            return { icon: GitBranch, color: '#8b5cf6', bgColor: '#FFFFFF', iconBgColor: '#faf5ff', iconColor: '#9333ea' } // Purple
        case 'barqflow-nodes.webhook':
        case 'barqflow-nodes.cron':
            return { icon: Play, color: '#ec4899', bgColor: '#FFFFFF', iconBgColor: '#fdf2f8', iconColor: '#db2777' } // Pink / Action
        case 'barqflow-nodes.code':
            return { icon: FileCode2, color: '#f59e0b', bgColor: '#FFFFFF', iconBgColor: '#fffbeb', iconColor: '#d97706' } // Amber
        case 'barqflow-nodes.merge':
            return { icon: GitMerge, color: '#6366f1', bgColor: '#FFFFFF', iconBgColor: '#eef2ff', iconColor: '#4f46e5' } // Indigo
        case 'barqflow-nodes.set':
        case 'barqflow-nodes.filter':
            return { icon: Variable, color: '#64748b', bgColor: '#FFFFFF', iconBgColor: '#f8fafc', iconColor: '#475569' } // Slate
        default:
            return { icon: Settings2, color: '#94a3b8', bgColor: '#FFFFFF', iconBgColor: '#f1f5f9', iconColor: '#64748b' } // Default
    }
}
