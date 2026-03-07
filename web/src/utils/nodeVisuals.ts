import { Settings2, Globe, Database, Bot, Variable, GitBranch, GitMerge, FileCode2, Clock, Play, Send, Hash, Github, Table, MessageSquare, Notebook, Table2, DatabaseBackup, Layers, HardDrive, Cloud, Ticket, CreditCard, Mail } from 'lucide-vue-next'
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
        case 'barqflow-nodes.telegram':
            return { icon: Send, color: '#229ED9', bgColor: '#FFFFFF', iconBgColor: '#e0f2fe', iconColor: '#0284c7' }
        case 'barqflow-nodes.slack':
            return { icon: Hash, color: '#E01E5A', bgColor: '#FFFFFF', iconBgColor: '#ffe4e6', iconColor: '#e11d48' }
        case 'barqflow-nodes.github':
            return { icon: Github, color: '#181717', bgColor: '#FFFFFF', iconBgColor: '#f1f5f9', iconColor: '#334155' }
        case 'barqflow-nodes.googleSheets':
            return { icon: Table, color: '#0F9D58', bgColor: '#FFFFFF', iconBgColor: '#dcfce7', iconColor: '#16a34a' }
        case 'barqflow-nodes.discord':
            return { icon: MessageSquare, color: '#5865F2', bgColor: '#FFFFFF', iconBgColor: '#e0e7ff', iconColor: '#4f46e5' }
        case 'barqflow-nodes.notion':
            return { icon: Notebook, color: '#000000', bgColor: '#FFFFFF', iconBgColor: '#f1f5f9', iconColor: '#1e293b' }
        case 'barqflow-nodes.airtable':
            return { icon: Table2, color: '#18BFFF', bgColor: '#FFFFFF', iconBgColor: '#e0f2fe', iconColor: '#0284c7' }
        case 'barqflow-nodes.mysql':
            return { icon: DatabaseBackup, color: '#00758F', bgColor: '#FFFFFF', iconBgColor: '#e0f2fe', iconColor: '#0369a1' }
        case 'barqflow-nodes.redis':
            return { icon: Layers, color: '#D82C20', bgColor: '#FFFFFF', iconBgColor: '#fee2e2', iconColor: '#dc2626' }
        case 'barqflow-nodes.awsS3':
            return { icon: HardDrive, color: '#FF9900', bgColor: '#FFFFFF', iconBgColor: '#fff7ed', iconColor: '#ea580c' }
        case 'barqflow-nodes.googleDrive':
            return { icon: Cloud, color: '#1FA463', bgColor: '#FFFFFF', iconBgColor: '#dcfce7', iconColor: '#16a34a' }
        case 'barqflow-nodes.jira':
            return { icon: Ticket, color: '#0052CC', bgColor: '#FFFFFF', iconBgColor: '#eff6ff', iconColor: '#2563eb' }
        case 'barqflow-nodes.stripe':
            return { icon: CreditCard, color: '#635BFF', bgColor: '#FFFFFF', iconBgColor: '#ede9fe', iconColor: '#7c3aed' }
        case 'barqflow-nodes.sendGrid':
            return { icon: Mail, color: '#1A82E2', bgColor: '#FFFFFF', iconBgColor: '#e0f2fe', iconColor: '#0284c7' }
        default:
            return { icon: Settings2, color: '#94a3b8', bgColor: '#FFFFFF', iconBgColor: '#f1f5f9', iconColor: '#64748b' } // Default
    }
}
