# Node Schema Contract

## Purpose

The node schema contract is the backend payload used by the frontend to render node pickers, parameter editors, credential requirements, and defaults.

## Shape

```json
{
  "name": "n8n-nodes-base.httpRequest",
  "displayName": "HTTP Request",
  "description": "Make an HTTP request",
  "isTrigger": false,
  "typeVersion": 1,
  "maxInputs": 1,
  "documentationUrl": null,
  "defaults": {
    "method": "GET"
  },
  "properties": [],
  "credentials": []
}
```

## Fields

- `name`: internal node type id
- `displayName`: node label shown in picker and NDV
- `description`: short summary
- `isTrigger`: whether the node starts flows
- `typeVersion`: latest UI-exposed type version
- `maxInputs`: expected input branch count
- `documentationUrl`: optional doc link
- `defaults`: default parameter map derived from schema properties
- `properties`: array of node property definitions
- `credentials`: array of credential references required by the node

## Property Contract

Each property entry follows the `INodeProperty` wire format:

```json
{
  "displayName": "URL",
  "name": "url",
  "type": "string",
  "default": "https://example.com",
  "description": "Target URL",
  "hint": null,
  "required": true,
  "options": null,
  "displayOptions": null
}
```

## Rules

- every node schema returned to the frontend must have `defaults`, even if empty
- every UI-exposed node must return `typeVersion`
- credential requirements are part of the node schema contract, not inferred in the frontend
- field names use camelCase on the wire
