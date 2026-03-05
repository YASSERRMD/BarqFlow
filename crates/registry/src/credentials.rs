//! Credential Form Descriptions (UI Contracts)
//!
//! This module defines the structures that describe how credentials should be
//! rendered in the UI, including OAuth2 specific forms and authentication injection.

use serde::{Deserialize, Serialize};

/// The type of credential authentication.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CredentialAuthType {
    /// No authentication (public API)
    None,
    /// Basic authentication with username/password
    BasicAuth,
    /// Header-based authentication (e.g., Bearer token)
    HeaderAuth,
    /// Query parameter authentication
    QueryAuth,
    /// OAuth2 authentication
    #[serde(rename = "oauth2")]
    OAuth2,
    /// API Key authentication
    ApiKey,
    /// Custom authentication type
    Custom,
}

/// How to inject the authentication into the HTTP request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AuthenticateBy {
    /// Inject via HTTP header
    Header {
        #[serde(rename = "type")]
        auth_type: String,
        #[serde(rename = "headerName")]
        header_name: String,
    },
    /// Inject via query parameters
    Query {
        #[serde(rename = "type")]
        auth_type: String,
        #[serde(rename = "parameterName")]
        parameter_name: String,
    },
    /// Inject via request body
    Body {
        #[serde(rename = "type")]
        auth_type: String,
        #[serde(rename = "fieldName")]
        field_name: String,
    },
}

/// OAuth2 specific configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2Options {
    /// OAuth2 grant type
    pub grant_type: OAuth2GrantType,
    /// Authorization URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_url: Option<String>,
    /// Access token URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token_url: Option<String>,
    /// Scope(s) to request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Client authentication method
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_auth_method: Option<String>,
}

/// OAuth2 grant types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OAuth2GrantType {
    /// Authorization code grant
    AuthorizationCode,
    /// Client credentials grant
    ClientCredentials,
    /// Implicit grant
    Implicit,
    /// Resource owner password credentials
    Password,
}

/// Represents a single property in a credential's form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ICredentialProperty {
    /// Display name of the property
    pub display_name: String,
    /// Internal property name/identifier
    pub name: String,
    /// Type of UI element to render
    pub r#type: CredentialPropertyType,
    /// Default value for the property
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    /// Description shown to the user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether this property is required
    #[serde(default)]
    pub required: bool,
    /// For password type - whether to mask the input
    #[serde(default)]
    pub password: bool,
}

/// The type of UI element to display for a credential property.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CredentialPropertyType {
    /// Simple text input
    String,
    /// Multi-line text input
    Text,
    /// Password input (masked)
    Password,
    /// Hidden field
    Hidden,
    /// Boolean checkbox
    Boolean,
    /// Dropdown/select with predefined options
    Options,
}

/// Authentication injection configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Authenticate {
    /// How to inject the authentication
    pub r#type: AuthenticateBy,
    /// The credential property name to use for the value
    pub property: String,
}

/// Collection of properties that define a credential's form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ICredentialsProperties {
    /// Display name for this credential type
    pub display_name: String,
    /// Type of authentication
    pub auth_type: CredentialAuthType,
    /// List of all properties for this credential
    pub properties: Vec<ICredentialProperty>,
    /// How to inject authentication into requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authenticate: Option<Authenticate>,
    /// OAuth2 specific options (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth2_options: Option<OAuth2Options>,
}

/// Pre-built OAuth2 form models for common OAuth2 providers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2Provider {
    /// Provider name (e.g., "Google", "GitHub")
    pub name: String,
    /// Provider-specific OAuth2 configuration
    pub options: OAuth2Options,
    /// Provider-specific credential properties
    pub properties: Vec<ICredentialProperty>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_properties_serialization() {
        let props = ICredentialsProperties {
            display_name: "HTTP Basic Auth".to_string(),
            auth_type: CredentialAuthType::BasicAuth,
            properties: vec![
                ICredentialProperty {
                    display_name: "Username".to_string(),
                    name: "username".to_string(),
                    r#type: CredentialPropertyType::String,
                    default: None,
                    description: Some("The username for authentication".to_string()),
                    required: true,
                    password: false,
                },
                ICredentialProperty {
                    display_name: "Password".to_string(),
                    name: "password".to_string(),
                    r#type: CredentialPropertyType::Password,
                    default: None,
                    description: None,
                    required: true,
                    password: true,
                },
            ],
            authenticate: Some(Authenticate {
                r#type: AuthenticateBy::Header {
                    auth_type: "header".to_string(),
                    header_name: "Authorization".to_string(),
                },
                property: "credentials".to_string(),
            }),
            oauth2_options: None,
        };

        let serialized = serde_json::to_string(&props).unwrap();
        let deserialized: ICredentialsProperties = serde_json::from_str(&serialized).unwrap();

        assert_eq!(props, deserialized);
        assert!(serialized.contains("\"authType\":\"basicAuth\""));
        assert!(serialized.contains("\"authenticate\":"));
    }

    #[test]
    fn test_oauth2_properties_serialization() {
        let props = ICredentialsProperties {
            display_name: "OAuth2".to_string(),
            auth_type: CredentialAuthType::OAuth2,
            properties: vec![
                ICredentialProperty {
                    display_name: "Client ID".to_string(),
                    name: "clientId".to_string(),
                    r#type: CredentialPropertyType::String,
                    default: None,
                    description: Some("The OAuth2 client ID".to_string()),
                    required: true,
                    password: false,
                },
                ICredentialProperty {
                    display_name: "Client Secret".to_string(),
                    name: "clientSecret".to_string(),
                    r#type: CredentialPropertyType::Password,
                    default: None,
                    description: Some("The OAuth2 client secret".to_string()),
                    required: true,
                    password: true,
                },
            ],
            authenticate: Some(Authenticate {
                r#type: AuthenticateBy::Header {
                    auth_type: "header".to_string(),
                    header_name: "Authorization".to_string(),
                },
                property: "accessToken".to_string(),
            }),
            oauth2_options: Some(OAuth2Options {
                grant_type: OAuth2GrantType::AuthorizationCode,
                auth_url: Some("https://example.com/oauth/authorize".to_string()),
                access_token_url: Some("https://example.com/oauth/token".to_string()),
                scope: Some("read write".to_string()),
                client_auth_method: Some("basic".to_string()),
            }),
        };

        let serialized = serde_json::to_string(&props).unwrap();
        let deserialized: ICredentialsProperties = serde_json::from_str(&serialized).unwrap();

        assert_eq!(props, deserialized);
        assert!(serialized.contains("\"authType\":\"oauth2\""));
        assert!(serialized.contains("\"grantType\":\"authorizationCode\""));
    }

    #[test]
    fn test_authenticate_by_variants() {
        let header_auth = AuthenticateBy::Header {
            auth_type: "header".to_string(),
            header_name: "Authorization".to_string(),
        };
        let serialized = serde_json::to_string(&header_auth).unwrap();
        assert!(serialized.contains("\"type\":\"header\""));
        assert!(serialized.contains("\"headerName\":\"Authorization\""));

        let query_auth = AuthenticateBy::Query {
            auth_type: "query".to_string(),
            parameter_name: "api_key".to_string(),
        };
        let serialized = serde_json::to_string(&query_auth).unwrap();
        assert!(serialized.contains("\"type\":\"query\""));
        assert!(serialized.contains("\"parameterName\":\"api_key\""));

        let body_auth = AuthenticateBy::Body {
            auth_type: "body".to_string(),
            field_name: "token".to_string(),
        };
        let serialized = serde_json::to_string(&body_auth).unwrap();
        assert!(serialized.contains("\"type\":\"body\""));
        assert!(serialized.contains("\"fieldName\":\"token\""));
    }

    #[test]
    fn test_oauth2_provider_serialization() {
        let provider = OAuth2Provider {
            name: "Google".to_string(),
            options: OAuth2Options {
                grant_type: OAuth2GrantType::AuthorizationCode,
                auth_url: Some("https://accounts.google.com/o/oauth2/v2/auth".to_string()),
                access_token_url: Some("https://oauth2.googleapis.com/token".to_string()),
                scope: Some("openid profile email".to_string()),
                client_auth_method: Some("basic".to_string()),
            },
            properties: vec![ICredentialProperty {
                display_name: "Client ID".to_string(),
                name: "clientId".to_string(),
                r#type: CredentialPropertyType::String,
                default: None,
                description: None,
                required: true,
                password: false,
            }],
        };

        let serialized = serde_json::to_string(&provider).unwrap();
        let deserialized: OAuth2Provider = serde_json::from_str(&serialized).unwrap();

        assert_eq!(provider, deserialized);
        assert!(serialized.contains("\"name\":\"Google\""));
    }
}
