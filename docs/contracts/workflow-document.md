# Workflow Document Contract

## Purpose

The workflow document is the canonical object edited by the frontend and executed by the backend.

## Top-Level Shape

```json
{
  "id": "7f595f8b-61d2-4cb3-b9f7-d4590f258fb2",
  "name": "Daily Slack Summary",
  "active": false,
  "nodes": [],
  "connections": {},
  "settings": {},
  "createdAt": "2026-03-10T10:00:00Z",
  "updatedAt": "2026-03-10T10:00:00Z"
}
```

## Fields

- `id`: workflow UUID
- `name`: user-visible workflow name
- `active`: whether triggers are active
- `nodes`: ordered array of workflow nodes
- `connections`: map of source node name to outgoing connection groups
- `settings`: workflow-wide settings
- `createdAt`: creation time
- `updatedAt`: last update time

## Node Shape

```json
{
  "id": "manual-trigger-1",
  "name": "Manual Trigger",
  "type": "n8n-nodes-base.manualTrigger",
  "typeVersion": 1,
  "position": [120, 180],
  "parameters": {},
  "credentials": [],
  "disabled": false
}
```

## Connection Shape

`connections` is indexed by source node name.

```json
{
  "Manual Trigger": {
    "main": [
      [
        {
          "node": "HTTP Request",
          "type": "main",
          "index": 0
        }
      ]
    ]
  }
}
```

## Settings Shape

```json
{
  "timezone": "Asia/Dubai",
  "saveExecutionProgress": true,
  "saveManualExecutions": true,
  "callerPolicy": "workflowsFromSameOwner"
}
```

## Rules

- `nodes[].id` must be unique within a workflow document.
- `connections` keys must refer to existing source node names.
- each `connection.node` must refer to an existing target node name.
- `typeVersion` is required even when the current version is `1`.
- `credentials` is always present and defaults to an empty array.
- `settings` is always present and defaults to `{}`.
