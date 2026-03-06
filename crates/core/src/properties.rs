//! Node Property UI Contracts
//!
//! This module defines the structures that describe how nodes should be rendered in the UI,
//! including parameter types, display options, and conditional visibility rules.

use serde::{Deserialize, Serialize};

/// The type of UI element to display for a node property.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NodePropertyType {
    /// Simple text input field
    String,
    /// Multi-line text input
    Text,
    /// Boolean checkbox
    Boolean,
    /// Numeric input (integer or float)
    Number,
    /// Dropdown/select with predefined options
    Options,
    /// Collection of key-value pairs
    Collection,
    /// Fixed collection with multiple values
    FixedCollection {
        #[serde(rename = "type")]
        collection_type: String,
        values: Vec<String>,
    },
    /// Multi-select dropdown
    MultiSelect,
    /// Load options from an external source/API
    LoadOptions,
}

/// Represents a single option in a dropdown/select property.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePropertyOption {
    /// Display name of the option
    pub name: String,
    /// Internal value of the option
    pub value: serde_json::Value,
    /// Optional description shown below the option
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Display options that control how a property is rendered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeDisplayOptions {
    /// Show this property only if another property has a specific value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#show: Option<NodeDisplayCondition>,
}

/// Condition for showing/hiding a property based on another property's value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeDisplayCondition {
    /// The name of the property to check
    pub property: String,
    /// The value(s) that trigger this property to be shown
    pub values: Vec<serde_json::Value>,
}

/// Represents a single property in a node's parameter form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct INodeProperty {
    /// Display name of the property
    pub display_name: String,
    /// Internal property name/identifier
    pub name: String,
    /// Type of UI element to render
    pub r#type: NodePropertyType,
    /// Default value for the property
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    /// Description shown to the user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Hint text displayed below the input
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    /// Whether this property is required
    #[serde(default)]
    pub required: bool,
    /// Available options (for dropdown/select types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<NodePropertyOption>>,
    /// Display options for conditional visibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_options: Option<NodeDisplayOptions>,
}

/// Collection of properties that define a node's parameter form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct INodeProperties {
    /// List of all properties for this node
    pub properties: Vec<INodeProperty>,
    /// Display name for this collection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Required values for fixed collections
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_values: Option<Vec<String>>,
}

/// Describes how the credential will be injected into HTTP requests for actual usage
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateRequestProperties {
    /// How the credential should be passed (e.g., "header", "body", "qs" for query string)
    pub r#in: String,
    /// The key/name of the credential in the request
    pub name: String,
    /// The value payload, often containing expression references to form properties (e.g., "={{\$credentials.apiKey}}")
    pub value: String,
}

/// The root credential description contract sent to the UI
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ICredentialProperties {
    /// Internal ID / name of the credential type
    pub name: String,
    /// Display name shown in UI dropdowns
    pub display_name: String,
    /// Notice shown at the top of the credential form
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notice: Option<String>,
    /// The list of input fields the user must fill out
    #[serde(default)]
    pub properties: Vec<INodeProperty>,
    /// Optional instructional text mapping how to get the credential
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_url: Option<String>,
    /// Specifies how the credential applies itself to a standard HTTP request automatically
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authenticate: Option<AuthenticateRequestProperties>,
}

/// Helper functions to generate standard, repetitive credential forms
pub struct CredentialStandardForms;

impl CredentialStandardForms {
    /// Generates a standard OAuth2 configuration property block
    pub fn oauth2_properties() -> Vec<INodeProperty> {
        vec![
            INodeProperty {
                display_name: "Grant Type".to_string(),
                name: "grantType".to_string(),
                r#type: NodePropertyType::Options,
                default: Some(serde_json::json!("authorizationCode")),
                description: None,
                hint: None,
                required: true,
                options: Some(vec![
                    NodePropertyOption {
                        name: "Authorization Code".to_string(),
                        value: serde_json::json!("authorizationCode"),
                        description: None,
                    },
                    NodePropertyOption {
                        name: "Client Credentials".to_string(),
                        value: serde_json::json!("clientCredentials"),
                        description: None,
                    },
                ]),
                display_options: None,
            },
            INodeProperty {
                display_name: "Authorization URL".to_string(),
                name: "authUrl".to_string(),
                r#type: NodePropertyType::String,
                default: None,
                description: Some("The URL to authorize the user".to_string()),
                hint: None,
                required: true,
                options: None,
                display_options: Some(NodeDisplayOptions {
                    r#show: Some(NodeDisplayCondition {
                        property: "grantType".to_string(),
                        values: vec![serde_json::json!("authorizationCode")],
                    }),
                }),
            },
            INodeProperty {
                display_name: "Access Token URL".to_string(),
                name: "accessTokenUrl".to_string(),
                r#type: NodePropertyType::String,
                default: None,
                description: Some("The URL to retrieve the access token".to_string()),
                hint: None,
                required: true,
                options: None,
                display_options: None,
            },
            INodeProperty {
                display_name: "Client ID".to_string(),
                name: "clientId".to_string(),
                r#type: NodePropertyType::String,
                default: None,
                description: Some("The OAuth2 Client ID".to_string()),
                hint: None,
                required: true,
                options: None,
                display_options: None,
            },
            INodeProperty {
                display_name: "Client Secret".to_string(),
                name: "clientSecret".to_string(),
                r#type: NodePropertyType::String,
                default: None,
                description: Some("The OAuth2 Client Secret".to_string()),
                hint: None,
                required: true,
                options: None,
                display_options: None,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_node_property_serialization() {
        let prop = INodeProperty {
            display_name: "API Endpoint".to_string(),
            name: "endpoint".to_string(),
            r#type: NodePropertyType::String,
            default: Some(json!("https://api.example.com")),
            description: Some("The API endpoint to connect to".to_string()),
            hint: Some("Include the protocol (https://)".to_string()),
            required: true,
            options: None,
            display_options: None,
        };

        let serialized = serde_json::to_string(&prop).unwrap();
        let deserialized: INodeProperty = serde_json::from_str(&serialized).unwrap();

        assert_eq!(prop, deserialized);
        assert!(serialized.contains("\"displayName\":\"API Endpoint\""));
        assert!(serialized.contains("\"type\":\"string\""));
    }

    #[test]
    fn test_node_property_with_options() {
        let prop = INodeProperty {
            display_name: "HTTP Method".to_string(),
            name: "method".to_string(),
            r#type: NodePropertyType::Options,
            default: Some(json!("GET")),
            description: None,
            hint: None,
            required: true,
            options: Some(vec![
                NodePropertyOption {
                    name: "GET".to_string(),
                    value: json!("GET"),
                    description: None,
                },
                NodePropertyOption {
                    name: "POST".to_string(),
                    value: json!("POST"),
                    description: Some("Create a new resource".to_string()),
                },
            ]),
            display_options: None,
        };

        let serialized = serde_json::to_string(&prop).unwrap();
        assert!(serialized.contains("\"type\":\"options\""));
        assert!(serialized.contains("\"name\":\"GET\""));

        let deserialized: INodeProperty = serde_json::from_str(&serialized).unwrap();
        assert_eq!(prop, deserialized);
        assert!(deserialized.options.is_some());
        assert_eq!(deserialized.options.unwrap().len(), 2);
    }

    #[test]
    fn test_display_options_conditional_visibility() {
        let display_options = NodeDisplayOptions {
            r#show: Some(NodeDisplayCondition {
                property: "authentication".to_string(),
                values: vec![json!("oauth2"), json!("apiKey")],
            }),
        };

        let prop = INodeProperty {
            display_name: "API Key".to_string(),
            name: "apiKey".to_string(),
            r#type: NodePropertyType::String,
            default: None,
            description: None,
            hint: None,
            required: false,
            options: None,
            display_options: Some(display_options),
        };

        let serialized = serde_json::to_string(&prop).unwrap();
        assert!(serialized.contains("\"property\":\"authentication\""));
        assert!(serialized.contains("\"values\":[\"oauth2\",\"apiKey\"]"));

        let deserialized: INodeProperty = serde_json::from_str(&serialized).unwrap();
        assert_eq!(prop, deserialized);
    }

    #[test]
    fn test_node_properties_collection() {
        let properties = INodeProperties {
            display_name: Some("HTTP Request Configuration".to_string()),
            properties: vec![
                INodeProperty {
                    display_name: "URL".to_string(),
                    name: "url".to_string(),
                    r#type: NodePropertyType::String,
                    default: None,
                    description: Some("The URL to request".to_string()),
                    hint: None,
                    required: true,
                    options: None,
                    display_options: None,
                },
                INodeProperty {
                    display_name: "Method".to_string(),
                    name: "method".to_string(),
                    r#type: NodePropertyType::Options,
                    default: Some(json!("GET")),
                    description: None,
                    hint: None,
                    required: true,
                    options: Some(vec![
                        NodePropertyOption {
                            name: "GET".to_string(),
                            value: json!("GET"),
                            description: None,
                        },
                    ]),
                    display_options: None,
                },
            ],
            required_values: None,
        };

        let serialized = serde_json::to_string(&properties).unwrap();
        let deserialized: INodeProperties = serde_json::from_str(&serialized).unwrap();

        assert_eq!(properties, deserialized);
        assert_eq!(deserialized.properties.len(), 2);
    }

    #[test]
    fn test_fixed_collection_property() {
        let prop = INodeProperty {
            display_name: "Headers".to_string(),
            name: "headerParameters".to_string(),
            r#type: NodePropertyType::FixedCollection {
                collection_type: "fixedArrayOfLength".to_string(),
                values: vec!["name".to_string(), "value".to_string()],
            },
            default: None,
            description: Some("HTTP headers to send".to_string()),
            hint: None,
            required: false,
            options: None,
            display_options: None,
        };

        let serialized = serde_json::to_string(&prop).unwrap();
        assert!(serialized.contains("\"type\":\"fixedArrayOfLength\""));
        assert!(serialized.contains("\"values\":[\"name\",\"value\"]"));

        let deserialized: INodeProperty = serde_json::from_str(&serialized).unwrap();
        assert_eq!(prop, deserialized);
    }
    
    #[test]
    fn test_credential_serialization() {
        let auth = AuthenticateRequestProperties {
            r#in: "header".to_string(),
            name: "Authorization".to_string(),
            value: "Bearer ={{$credentials.oauthToken}}".to_string(),
        };
        
        let props = ICredentialProperties {
            name: "githubOAuth2Api".to_string(),
            display_name: "GitHub OAuth2 API".to_string(),
            notice: None,
            properties: CredentialStandardForms::oauth2_properties(),
            documentation_url: Some("https://docs.github.com/en/rest".to_string()),
            authenticate: Some(auth),
        };
        
        let serialized = serde_json::to_string(&props).unwrap();
        
        assert!(serialized.contains("\"name\":\"githubOAuth2Api\""));
        assert!(serialized.contains("\"value\":\"Bearer ={{$credentials.oauthToken}}\""));
        assert!(serialized.contains("\"name\":\"grantType\""));
        
        let deserialized: ICredentialProperties = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.properties.len(), 5);
        assert_eq!(deserialized.authenticate.unwrap().r#in, "header");
    }
}

