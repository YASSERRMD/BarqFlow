use regex::Regex;
use rhai::{Dynamic, Engine, Scope, AST};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ExpressionContext {
    pub json_data: serde_json::Value,
    pub binary_keys: Vec<String>,
    pub parameters: HashMap<String, serde_json::Value>,
}

pub struct ExpressionEngine {
    engine: Engine,
}

impl ExpressionEngine {
    pub fn new() -> Self {
        let mut engine = Engine::new_raw();

        engine.set_allow_looping(true);
        engine.set_allow_shadowing(true);
        engine.set_strict_variables(false);

        engine.set_max_expr_depths(32, 32);
        engine.set_max_operations(10000);
        engine.set_max_call_levels(16);

        engine.on_print(|s| {
            tracing::debug!("Rhai print: {}", s);
        });

        Self { engine }
    }

    pub fn with_custom_functions(mut self) -> Self {
        self.engine
            .register_fn("now", || chrono::Utc::now().to_rfc3339());

        self.engine.register_fn("today", || {
            chrono::Utc::now().format("%Y-%m-%d").to_string()
        });

        self.engine.register_fn("hash_md5", |s: &str| {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            s.hash(&mut hasher);
            format!("{:x}", hasher.finish())
        });

        self.engine.register_fn("hash_sha256", |s: &str| {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            s.hash(&mut hasher);
            format!("{:x}", hasher.finish())
        });

        self.engine
            .register_fn("url_encode", |s: &str| urlencoding::encode(s).to_string());

        self.engine.register_fn("url_decode", |s: &str| {
            urlencoding::decode(s)
                .map(|s| s.to_string())
                .unwrap_or_else(|_| s.to_string())
        });

        self
    }

    pub fn compile(&self, script: &str) -> Result<AST, String> {
        self.engine.compile(script).map_err(|e| e.to_string())
    }

    pub fn eval_with_context(
        &self,
        script: &str,
        context: &ExpressionContext,
    ) -> Result<Dynamic, String> {
        let mut scope = self.create_scope(context);

        self.engine
            .eval_with_scope::<Dynamic>(&mut scope, script)
            .map_err(|e| e.to_string())
    }

    pub fn create_scope(&self, context: &ExpressionContext) -> Scope {
        let mut scope = Scope::new();

        Self::add_json_to_scope(&mut scope, "json", &context.json_data);

        let binary_array: Vec<Dynamic> = context
            .binary_keys
            .iter()
            .map(|s| Dynamic::from(s.clone()))
            .collect();
        scope.push_constant("binary", Dynamic::from(binary_array));

        for (key, value) in &context.parameters {
            Self::add_json_to_scope(&mut scope, key, value);
        }

        let env_vars: HashMap<String, Dynamic> = std::env::vars()
            .map(|(k, v)| (k, Dynamic::from(v)))
            .collect();
        scope.push_constant("env", Dynamic::from(env_vars));

        scope
    }

    fn add_json_to_scope(scope: &mut Scope, name: &str, value: &serde_json::Value) {
        let dynamic = Self::json_to_dynamic(value);
        scope.push_constant(name.to_string(), dynamic);
    }

    fn json_to_dynamic(value: &serde_json::Value) -> Dynamic {
        match value {
            serde_json::Value::Null => Dynamic::from(()),
            serde_json::Value::Bool(b) => Dynamic::from(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Dynamic::from(i)
                } else if let Some(f) = n.as_f64() {
                    Dynamic::from(f)
                } else {
                    Dynamic::from(n.to_string())
                }
            }
            serde_json::Value::String(s) => Dynamic::from(s.clone()),
            serde_json::Value::Array(arr) => {
                let converted: Vec<Dynamic> =
                    arr.iter().map(|v| Self::json_to_dynamic(v)).collect();
                Dynamic::from(converted)
            }
            serde_json::Value::Object(obj) => {
                let map: HashMap<String, Dynamic> = obj
                    .iter()
                    .map(|(k, v)| (k.clone(), Self::json_to_dynamic(v)))
                    .collect();
                Dynamic::from(map)
            }
        }
    }

    pub fn validate_ast(&self, script: &str) -> Result<bool, String> {
        match self.engine.compile(script) {
            Ok(_) => Ok(true),
            Err(e) => Err(e.to_string()),
        }
    }
}

impl Default for ExpressionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionExtractor;

impl ExpressionExtractor {
    pub fn extract_expressions(input: &str) -> Vec<ExtractedExpression> {
        let re = Regex::new(r#"\{\{([^}]+)\}\}"#).unwrap();

        let mut results = Vec::new();
        for cap in re.captures_iter(input) {
            if let Some(full_match) = cap.get(0) {
                if let Some(expr_match) = cap.get(1) {
                    results.push(ExtractedExpression {
                        index: results.len(),
                        full_match: full_match.as_str().to_string(),
                        expression: expr_match.as_str().trim().to_string(),
                        start: full_match.start(),
                        end: full_match.end(),
                    });
                }
            }
        }
        results
    }

    pub fn replace_expressions(input: &str, replacements: &[String]) -> String {
        let expressions = Self::extract_expressions(input);

        if expressions.is_empty() {
            return input.to_string();
        }

        let mut result = input.to_string();
        let mut offset: isize = 0;

        for (i, expr) in expressions.iter().enumerate() {
            if i < replacements.len() {
                let start = (expr.start as isize + offset) as usize;
                let end = (expr.end as isize + offset) as usize;

                if start < result.len() && end <= result.len() {
                    result.replace_range(start..end, &replacements[i]);
                    offset += replacements[i].len() as isize - expr.full_match.len() as isize;
                }
            }
        }

        result
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedExpression {
    pub index: usize,
    pub full_match: String,
    pub expression: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct ItemAccessor {
    graph_cache: HashMap<String, Vec<(String, serde_json::Value)>>,
}

impl ItemAccessor {
    pub fn new() -> Self {
        Self {
            graph_cache: HashMap::new(),
        }
    }

    pub fn cache_node_output(&mut self, node_id: &str, data: Vec<(String, serde_json::Value)>) {
        self.graph_cache.insert(node_id.to_string(), data);
    }

    pub fn get_item_json(&self, node_id: &str, index: usize) -> Option<serde_json::Value> {
        self.graph_cache
            .get(node_id)
            .and_then(|items| items.get(index))
            .map(|(_, v)| v.clone())
    }

    pub fn get_item_binary(
        &self,
        node_id: &str,
        index: usize,
    ) -> Option<HashMap<String, serde_json::Value>> {
        self.graph_cache
            .get(node_id)
            .and_then(|items| items.get(index))
            .and_then(|(binary_key, _)| serde_json::from_str(binary_key).ok())
    }
}

impl Default for ItemAccessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_extractor_single() {
        let input = "Hello {{name}}!";
        let extracted = ExpressionExtractor::extract_expressions(input);

        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted[0].expression, "name");
        assert_eq!(extracted[0].full_match, "{{name}}");
    }

    #[test]
    fn test_expression_extractor_multiple() {
        let input = "{{first}} and {{second}}";
        let extracted = ExpressionExtractor::extract_expressions(input);

        assert_eq!(extracted.len(), 2);
        assert_eq!(extracted[0].expression, "first");
        assert_eq!(extracted[1].expression, "second");
    }

    #[test]
    fn test_expression_extractor_nested_braces() {
        let input = "{{ outer {{ inner }} }}";
        let extracted = ExpressionExtractor::extract_expressions(input);

        assert_eq!(extracted.len(), 1);
    }

    #[test]
    fn test_expression_engine_basic() {
        let engine = ExpressionEngine::new();

        let result = engine.eval_with_context(
            "1 + 2",
            &ExpressionContext {
                json_data: serde_json::json!({}),
                binary_keys: vec![],
                parameters: HashMap::new(),
            },
        );

        assert_eq!(result.unwrap().as_int().unwrap(), 3);
    }

    #[test]
    fn test_replace_expressions() {
        let input = "Hello {{name}}, your score is {{score}}";
        let replacements = vec!["World".to_string(), "100".to_string()];

        let result = ExpressionExtractor::replace_expressions(input, &replacements);
        assert_eq!(result, "Hello World, your score is 100");
    }

    #[test]
    fn test_expression_with_parameters() {
    fn test_expression_engine_json_access() {
        let engine = ExpressionEngine::new();

        let context = ExpressionContext {
            json_data: serde_json::json!({
                "name": "BarqFlow",
                "count": 42
            }),
            binary_keys: vec![],
            parameters: HashMap::new(),
        };

        let result = engine.eval_with_context("1 + 1", &context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_expression_engine_math() {
        let engine = ExpressionEngine::new();

        let context = ExpressionContext {
            json_data: serde_json::json!({}),
            binary_keys: vec![],
            parameters: HashMap::from([
                ("a".to_string(), serde_json::json!(10)),
                ("b".to_string(), serde_json::json!(5)),
            ]),
        };

        let result = engine.eval_with_context("a + b * 2", &context);
        assert_eq!(result.unwrap().as_int().unwrap(), 20);
    }

    #[test]
    fn test_replace_expressions() {
        let input = "Hello {{name}}, your score is {{score}}";
        let replacements = vec!["World".to_string(), "100".to_string()];

        let result = ExpressionExtractor::replace_expressions(input, &replacements);
        assert_eq!(result, "Hello World, your score is 100");
    }

    #[test]
    fn test_item_accessor_cache() {
        let mut accessor = ItemAccessor::new();

        let data = vec![
            ("json".to_string(), serde_json::json!({"id": 1})),
            ("json".to_string(), serde_json::json!({"id": 2})),
        ];

        accessor.cache_node_output("Process", data);

        assert_eq!(
            accessor.get_item_json("Process", 0).unwrap(),
            serde_json::json!({"id": 1})
        );
        assert_eq!(
            accessor.get_item_json("Process", 1).unwrap(),
            serde_json::json!({"id": 2})
        );
    }

    #[test]
    fn test_expression_with_custom_functions() {
        let engine = ExpressionEngine::new().with_custom_functions();

        let result = engine.eval_with_context(
            "url_encode(\"hello world\")",
            &ExpressionContext {
                json_data: serde_json::json!({}),
                binary_keys: vec![],
                parameters: HashMap::new(),
            },
        );

        assert!(result
            .unwrap()
            .into_string()
            .unwrap()
            .contains("hello%20world"));
    }
}
