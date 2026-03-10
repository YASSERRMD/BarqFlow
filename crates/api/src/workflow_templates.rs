use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct WorkflowTemplateDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub difficulty: &'static str,
    pub tags: &'static [&'static str],
    pub highlights: &'static [&'static str],
    pub nodes: Value,
    pub connections: Value,
    pub settings: Value,
}

impl WorkflowTemplateDefinition {
    pub fn tag_names(&self) -> Vec<String> {
        self.tags.iter().map(|tag| (*tag).to_string()).collect()
    }

    pub fn highlight_list(&self) -> Vec<String> {
        self.highlights
            .iter()
            .map(|highlight| (*highlight).to_string())
            .collect()
    }
}

pub fn list_workflow_templates() -> Vec<WorkflowTemplateDefinition> {
    vec![
        WorkflowTemplateDefinition {
            id: "incident-slack-escalation",
            name: "Incident Slack Escalation",
            description: "Fetch an external status endpoint, evaluate severity, and fan the alert into Slack.",
            category: "Ops",
            difficulty: "starter",
            tags: &["starter", "ops", "alerting"],
            highlights: &[
                "Manual trigger starter with a production-style incident branch.",
                "Shows HTTP Request, If, and Slack working together.",
                "Good baseline for health checks and release monitors.",
            ],
            nodes: json!([
                {
                    "id": "manual-trigger-1",
                    "name": "Manual Trigger",
                    "type": "n8n-nodes-base.manualTrigger",
                    "typeVersion": 1,
                    "position": [120, 240],
                    "parameters": {},
                    "credentials": [],
                    "disabled": false
                },
                {
                    "id": "http-request-1",
                    "name": "Fetch Status",
                    "type": "n8n-nodes-base.httpRequest",
                    "typeVersion": 1,
                    "position": [360, 240],
                    "parameters": {
                        "url": "https://status.example.com/api/incidents",
                        "method": "GET",
                        "headers": [],
                        "queryParameters": [],
                        "body": "",
                        "authentication": "none",
                        "responseFormat": "json",
                        "timeout": 30000
                    },
                    "credentials": [],
                    "disabled": false
                },
                {
                    "id": "if-1",
                    "name": "Incident Active",
                    "type": "n8n-nodes-base.if",
                    "typeVersion": 1,
                    "position": [620, 240],
                    "parameters": {
                        "combineOperation": "all",
                        "conditions": [{
                            "value1": "={{$json.status}}",
                            "operation": "equals",
                            "value2": "critical"
                        }],
                        "operation": "equals",
                        "value1": "={{$json.status}}",
                        "value2": "critical"
                    },
                    "credentials": [],
                    "disabled": false
                },
                {
                    "id": "slack-1",
                    "name": "Notify Slack",
                    "type": "barqflow-nodes.slack",
                    "typeVersion": 1,
                    "position": [900, 240],
                    "parameters": {
                        "operation": "postMessage",
                        "baseUrl": "https://slack.com",
                        "channel": "#incident-room",
                        "text": "Critical incident detected. Review the upstream status payload.",
                        "timeout": 60000
                    },
                    "credentials": [],
                    "disabled": false
                }
            ]),
            connections: json!({
                "Manual Trigger": {
                    "main": [[{
                        "node": "Fetch Status",
                        "type": "main",
                        "index": 0
                    }]]
                },
                "Fetch Status": {
                    "main": [[{
                        "node": "Incident Active",
                        "type": "main",
                        "index": 0
                    }]]
                },
                "Incident Active": {
                    "main": [[{
                        "node": "Notify Slack",
                        "type": "main",
                        "index": 0
                    }]]
                }
            }),
            settings: json!({
                "saveExecutionProgress": true,
                "saveManualExecutions": true,
                "timezone": "Asia/Dubai"
            }),
        },
        WorkflowTemplateDefinition {
            id: "webhook-lead-triage",
            name: "Webhook Lead Triage",
            description: "Receive inbound webhook payloads, qualify them, and post high-signal submissions to Slack.",
            category: "Growth",
            difficulty: "starter",
            tags: &["starter", "webhook", "triage"],
            highlights: &[
                "Includes a real webhook trigger configuration.",
                "Splits responses with an If node so qualification logic is visible.",
                "Useful as a base for lead capture, support forms, or intake pipelines.",
            ],
            nodes: json!([
                {
                    "id": "webhook-1",
                    "name": "Lead Intake",
                    "type": "barqflow-nodes.webhook",
                    "typeVersion": 1,
                    "position": [120, 220],
                    "parameters": {
                        "path": "lead-triage",
                        "httpMethod": "POST",
                        "responseMode": "onReceived",
                        "responseCode": 200,
                        "responseData": "{\"received\":true}"
                    },
                    "credentials": [],
                    "disabled": false
                },
                {
                    "id": "if-2",
                    "name": "High Intent",
                    "type": "n8n-nodes-base.if",
                    "typeVersion": 1,
                    "position": [400, 220],
                    "parameters": {
                        "combineOperation": "all",
                        "conditions": [{
                            "value1": "={{$json.companySize}}",
                            "operation": "larger",
                            "value2": 50
                        }],
                        "operation": "larger",
                        "value1": "={{$json.companySize}}",
                        "value2": "50"
                    },
                    "credentials": [],
                    "disabled": false
                },
                {
                    "id": "slack-2",
                    "name": "Send To Sales",
                    "type": "barqflow-nodes.slack",
                    "typeVersion": 1,
                    "position": [700, 220],
                    "parameters": {
                        "operation": "postMessage",
                        "baseUrl": "https://slack.com",
                        "channel": "#sales-intake",
                        "text": "New high-intent lead received from the webhook intake flow.",
                        "timeout": 60000
                    },
                    "credentials": [],
                    "disabled": false
                }
            ]),
            connections: json!({
                "Lead Intake": {
                    "main": [[{
                        "node": "High Intent",
                        "type": "main",
                        "index": 0
                    }]]
                },
                "High Intent": {
                    "main": [[{
                        "node": "Send To Sales",
                        "type": "main",
                        "index": 0
                    }]]
                }
            }),
            settings: json!({
                "saveExecutionProgress": true,
                "callerPolicy": "workflowsFromSameOwner"
            }),
        },
        WorkflowTemplateDefinition {
            id: "github-release-brief",
            name: "GitHub Release Brief",
            description: "Pull repository issues from GitHub and hand off a release-ready brief into Notion.",
            category: "Engineering",
            difficulty: "intermediate",
            tags: &["engineering", "github", "notion"],
            highlights: &[
                "Demonstrates multi-system handoff across GitHub and Notion.",
                "Designed for release notes, sprint summaries, and engineering reporting.",
                "Comes in inactive so credentials and IDs can be configured safely first.",
            ],
            nodes: json!([
                {
                    "id": "manual-trigger-2",
                    "name": "Manual Trigger",
                    "type": "n8n-nodes-base.manualTrigger",
                    "typeVersion": 1,
                    "position": [120, 260],
                    "parameters": {},
                    "credentials": [],
                    "disabled": false
                },
                {
                    "id": "github-1",
                    "name": "Fetch Issues",
                    "type": "barqflow-nodes.github",
                    "typeVersion": 1,
                    "position": [380, 260],
                    "parameters": {
                        "operation": "listIssues",
                        "baseUrl": "https://api.github.com",
                        "owner": "your-org",
                        "repo": "your-repo",
                        "state": "open",
                        "perPage": 20,
                        "timeout": 60000
                    },
                    "credentials": [],
                    "disabled": false
                },
                {
                    "id": "notion-1",
                    "name": "Create Brief",
                    "type": "barqflow-nodes.notion",
                    "typeVersion": 1,
                    "position": [680, 260],
                    "parameters": {
                        "operation": "createPage",
                        "baseUrl": "https://api.notion.com",
                        "databaseId": "replace-with-your-database-id",
                        "properties": "{\"Name\":{\"title\":[{\"text\":{\"content\":\"GitHub Release Brief\"}}]}}",
                        "timeout": 60000
                    },
                    "credentials": [],
                    "disabled": false
                }
            ]),
            connections: json!({
                "Manual Trigger": {
                    "main": [[{
                        "node": "Fetch Issues",
                        "type": "main",
                        "index": 0
                    }]]
                },
                "Fetch Issues": {
                    "main": [[{
                        "node": "Create Brief",
                        "type": "main",
                        "index": 0
                    }]]
                }
            }),
            settings: json!({
                "saveExecutionProgress": true,
                "saveManualExecutions": true,
                "timezone": "UTC"
            }),
        },
    ]
}

pub fn find_workflow_template(template_id: &str) -> Option<WorkflowTemplateDefinition> {
    list_workflow_templates()
        .into_iter()
        .find(|template| template.id == template_id)
}
