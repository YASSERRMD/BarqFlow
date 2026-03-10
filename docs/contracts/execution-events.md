# Execution Event Contract

## Purpose

Execution events define the stream payload used to update the workflow editor and execution views during a run.

## Event Envelope

```json
{
  "executionId": "f3180e91-f1fb-4c8f-9108-c10840d1943a",
  "workflowId": "7f595f8b-61d2-4cb3-b9f7-d4590f258fb2",
  "runId": "4be8e6d1-a5d0-45b0-9ad0-9891503176de",
  "eventType": "nodeFinished",
  "status": "running",
  "nodeId": "http-request-1",
  "nodeName": "HTTP Request",
  "message": "Node completed",
  "timestamp": "2026-03-10T10:05:00Z",
  "sequence": 12,
  "data": {
    "outputItems": 1
  }
}
```

## Event Types

- `queued`
- `started`
- `nodeStarted`
- `nodeFinished`
- `waiting`
- `resumed`
- `failed`
- `stopped`
- `completed`

## Status Values

- `queued`
- `running`
- `waiting`
- `success`
- `failed`
- `stopped`
- `cancelled`

## Rules

- `sequence` is monotonically increasing for a single execution
- `nodeId` and `nodeName` are optional for workflow-level events and required for node-level events
- `data` is an open JSON object for event-specific payloads
- `message` is user-visible text intended for UI rendering and logs
