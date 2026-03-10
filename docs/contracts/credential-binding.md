# Credential Binding Contract

## Purpose

Credential binding connects a saved credential record to a workflow node in a stable, explicit way.

## Node Binding Shape

```json
{
  "nodeId": "github-1",
  "credentialType": "githubApi",
  "credentialId": "2c1c80d3-33dc-4b78-8634-9751e2e50e11"
}
```

## Credential Summary Shape

```json
{
  "id": "2c1c80d3-33dc-4b78-8634-9751e2e50e11",
  "name": "Production GitHub",
  "credentialType": "githubApi",
  "data": {
    "accessToken": "******"
  },
  "createdAt": "2026-03-10T10:00:00Z",
  "updatedAt": "2026-03-10T10:00:00Z"
}
```

## Rules

- workflow documents store only `credentialId`, never raw secrets
- the backend always returns masked credential data in summary/detail responses
- the frontend treats bindings as explicit objects, not inferred string maps
- node schema credential requirements must match `credentialType`
- a node can bind multiple credentials of different types
