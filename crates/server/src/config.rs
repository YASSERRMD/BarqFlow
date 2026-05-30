use std::env;

/// Validated startup configuration.
///
/// Call `AppConfig::from_env()` once at boot. It reads all required
/// environment variables and panics with a clear message if any are
/// missing or invalid, so misconfigured deployments fail fast before
/// any database connections or background workers are started.
#[derive(Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub port: u16,
}

impl AppConfig {
    /// Read and validate all required environment variables.
    ///
    /// Panics with a human-readable message that names every missing or
    /// invalid variable so operators can fix all problems in one pass.
    pub fn from_env() -> Self {
        let mut errors: Vec<String> = Vec::new();

        let database_url = require_env("DATABASE_URL", &mut errors);
        let port = parse_port(
            env::var("PORT").unwrap_or_else(|_| "3000".to_string()),
            &mut errors,
        );

        // Validate JWT_SECRET only when running in production.
        let barqflow_env = env::var("BARQFLOW_ENV").unwrap_or_default();
        if barqflow_env.eq_ignore_ascii_case("production") {
            if env::var("JWT_SECRET")
                .map(|s| s.trim().is_empty())
                .unwrap_or(true)
            {
                errors
                    .push("JWT_SECRET must be set to a non-empty value in production".to_string());
            }

            let enc_key = env::var("BARQFLOW_ENCRYPTION_KEY").unwrap_or_default();
            if enc_key.len() != 32 {
                errors.push(format!(
                    "BARQFLOW_ENCRYPTION_KEY must be exactly 32 bytes \
                     (got {} bytes) in production",
                    enc_key.len()
                ));
            }
        }

        if !errors.is_empty() {
            let list = errors
                .iter()
                .enumerate()
                .map(|(i, msg)| format!("  {}. {}", i + 1, msg))
                .collect::<Vec<_>>()
                .join("\n");
            panic!(
                "BarqFlow failed to start due to configuration errors:\n{}\n\
                 Fix the variables above and restart.",
                list
            );
        }

        Self {
            database_url: database_url.unwrap_or_default(),
            port: port.unwrap_or(3000),
        }
    }
}

fn require_env(name: &str, errors: &mut Vec<String>) -> Option<String> {
    match env::var(name) {
        Ok(value) if !value.trim().is_empty() => Some(value),
        Ok(_) => {
            errors.push(format!("{name} must not be empty"));
            None
        }
        Err(_) => {
            errors.push(format!("{name} is required but not set"));
            None
        }
    }
}

fn parse_port(raw: String, errors: &mut Vec<String>) -> Option<u16> {
    match raw.parse::<u16>() {
        Ok(port) if port > 0 => Some(port),
        Ok(_) => {
            errors.push("PORT must be greater than 0".to_string());
            None
        }
        Err(_) => {
            errors.push(format!(
                "PORT '{}' is not a valid port number (1–65535)",
                raw
            ));
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_env_collects_missing_var_as_error() {
        let mut errors = Vec::new();
        // Use a name that definitely won't be set in the test environment.
        let result = require_env("__BARQFLOW_TEST_MISSING_VAR__", &mut errors);
        assert!(result.is_none());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("required but not set"));
    }

    #[test]
    fn parse_port_rejects_non_numeric_input() {
        let mut errors = Vec::new();
        let result = parse_port("not-a-port".to_string(), &mut errors);
        assert!(result.is_none());
        assert!(!errors.is_empty());
    }

    #[test]
    fn parse_port_accepts_valid_port() {
        let mut errors = Vec::new();
        let result = parse_port("8080".to_string(), &mut errors);
        assert_eq!(result, Some(8080));
        assert!(errors.is_empty());
    }
}
