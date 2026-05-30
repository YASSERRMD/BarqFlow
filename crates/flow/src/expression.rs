use md5::{Digest as Md5Digest, Md5};
use regex::Regex;
use rhai::packages::{Package, StandardPackage};
use rhai::{Dynamic, Engine, Scope, AST};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Shared, interior-mutable workflow cache backing the `$item`/`$items` helpers.
type SharedWorkflowCache = Arc<RwLock<HashMap<String, Vec<serde_json::Value>>>>;

#[derive(Debug, Clone)]
pub struct ExpressionContext {
    pub json_data: serde_json::Value,
    pub binary_keys: Vec<String>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub workflow_cache: HashMap<String, Vec<serde_json::Value>>,
}

pub struct ExpressionEngine {
    engine: Engine,
    /// Per-invocation workflow cache for the `$item`/`$items` helpers. It is
    /// refreshed at the start of every `eval_with_context` call so the engine
    /// itself stays immutable and evaluation only needs `&self`.
    workflow_cache: SharedWorkflowCache,
}

impl ExpressionEngine {
    pub fn new() -> Self {
        let mut engine = Engine::new_raw();

        // `new_raw()` ships without the StandardPackage, so helpers like
        // `len()`, arithmetic, and string utilities are unavailable. Register it
        // explicitly so expressions such as `json.tags.len()` evaluate.
        StandardPackage::new().register_into_engine(&mut engine);

        engine.set_allow_looping(true);
        engine.set_allow_shadowing(true);
        engine.set_strict_variables(false);

        engine.set_max_expr_depths(32, 32);
        engine.set_max_operations(10000);
        engine.set_max_call_levels(16);

        engine.on_print(|s| {
            tracing::debug!("Rhai print: {}", s);
        });

        let workflow_cache: SharedWorkflowCache = Arc::new(RwLock::new(HashMap::new()));

        // The dollar-prefixed helpers are rewritten to item/items before
        // evaluation because the dollar sign is a reserved token in the rhai
        // lexer. The cached node outputs already carry their own
        // { "json": ... } shape, so the helper returns each item verbatim
        // rather than re-wrapping it.
        let item_cache = workflow_cache.clone();
        engine.register_fn("item", move |node_name: &str| -> Dynamic {
            if let Ok(cache) = item_cache.read() {
                if let Some(first) = cache.get(node_name).and_then(|items| items.first()) {
                    return Self::json_to_dynamic(first);
                }
            }
            Dynamic::from(rhai::Map::new())
        });

        let items_cache = workflow_cache.clone();
        engine.register_fn("items", move |node_name: &str| -> rhai::Array {
            let mut array = rhai::Array::new();
            if let Ok(cache) = items_cache.read() {
                if let Some(items) = cache.get(node_name) {
                    for item in items {
                        array.push(Self::json_to_dynamic(item));
                    }
                }
            }
            array
        });

        Self {
            engine,
            workflow_cache,
        }
    }

    pub fn with_custom_functions(mut self) -> Self {
        self.engine
            .register_fn("now", || chrono::Utc::now().to_rfc3339());

        self.engine.register_fn("today", || {
            chrono::Utc::now().format("%Y-%m-%d").to_string()
        });

        self.engine.register_fn("hash_md5", |s: &str| {
            let mut hasher = Md5::new();
            hasher.update(s.as_bytes());
            hex::encode(hasher.finalize())
        });

        self.engine.register_fn("hash_sha256", |s: &str| {
            let mut hasher = Sha256::new();
            hasher.update(s.as_bytes());
            hex::encode(hasher.finalize())
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
        let transformed_script = Self::transform_script(script);
        self.engine
            .compile(&transformed_script)
            .map_err(|e| e.to_string())
    }

    /// Rewrites n8n-style dollar-prefixed identifiers into the plain
    /// identifiers and helper function names the rhai engine understands. The
    /// "items"/"item" tokens must be substituted before "json"/"input" so the
    /// longer tokens win.
    fn transform_script(script: &str) -> String {
        script
            .replace("$items", "items")
            .replace("$item", "item")
            .replace("$json", "json")
            .replace("$env", "env")
            .replace("$input", "input")
    }

    pub fn eval_with_context(
        &self,
        script: &str,
        context: &ExpressionContext,
    ) -> Result<Dynamic, String> {
        let transformed_script = Self::transform_script(script);

        // Publish this invocation's workflow cache so the pre-registered
        // `$item`/`$items` helpers resolve against the current node outputs.
        if let Ok(mut cache) = self.workflow_cache.write() {
            *cache = context.workflow_cache.clone();
        }

        let mut scope = self.create_scope(context);

        self.engine
            .eval_with_scope::<Dynamic>(&mut scope, &transformed_script)
            .map_err(|e| e.to_string())
    }

    pub fn create_scope(&self, context: &ExpressionContext) -> Scope<'_> {
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
                let converted: Vec<Dynamic> = arr.iter().map(Self::json_to_dynamic).collect();
                Dynamic::from(converted)
            }
            serde_json::Value::Object(obj) => {
                let mut map = rhai::Map::new();
                for (k, v) in obj.iter() {
                    map.insert(k.clone().into(), Self::json_to_dynamic(v));
                }
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

    fn ctx(json_data: serde_json::Value) -> ExpressionContext {
        ExpressionContext {
            json_data,
            binary_keys: vec![],
            parameters: HashMap::new(),
            workflow_cache: HashMap::new(),
        }
    }

    #[test]
    fn test_null_json_value_becomes_rhai_unit() {
        let result = ExpressionEngine::new()
            .eval_with_context("json.nothing", &ctx(serde_json::json!({"nothing": null})));
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().type_name(),
            "()",
            "JSON null should map to Rhai unit type"
        );
    }

    #[test]
    fn test_nested_object_deep_access() {
        let result = ExpressionEngine::new().eval_with_context(
            "json.a.b.c",
            &ctx(serde_json::json!({"a": {"b": {"c": 42}}})),
        );
        assert_eq!(result.unwrap().as_int().unwrap(), 42);
    }

    #[test]
    fn test_array_index_access() {
        let result = ExpressionEngine::new().eval_with_context(
            "json.items[1]",
            &ctx(serde_json::json!({"items": ["alpha", "beta", "gamma"]})),
        );
        assert_eq!(result.unwrap().into_string().unwrap(), "beta");
    }

    #[test]
    fn test_array_length() {
        let result = ExpressionEngine::new().eval_with_context(
            "json.tags.len()",
            &ctx(serde_json::json!({"tags": ["a", "b", "c", "d"]})),
        );
        assert_eq!(result.unwrap().as_int().unwrap(), 4);
    }

    #[test]
    fn test_boolean_json_value() {
        let result = ExpressionEngine::new()
            .eval_with_context("json.active", &ctx(serde_json::json!({"active": true})));
        assert!(result.unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_float_json_value_comparison() {
        let result = ExpressionEngine::new()
            .eval_with_context("json.price > 3.0", &ctx(serde_json::json!({"price": 3.15})));
        assert!(result.unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_string_concatenation_from_json() {
        let result = ExpressionEngine::new().eval_with_context(
            "json.first + \" \" + json.last",
            &ctx(serde_json::json!({"first": "John", "last": "Doe"})),
        );
        assert_eq!(result.unwrap().into_string().unwrap(), "John Doe");
    }

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
                workflow_cache: HashMap::new(),
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
    fn test_expression_engine_json_access() {
        let engine = ExpressionEngine::new();

        let context = ExpressionContext {
            json_data: serde_json::json!({
                "name": "BarqFlow",
                "count": 42
            }),
            binary_keys: vec![],
            parameters: HashMap::new(),
            workflow_cache: HashMap::new(),
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
            workflow_cache: HashMap::new(),
        };

        let result = engine.eval_with_context("a + b * 2", &context);
        assert_eq!(result.unwrap().as_int().unwrap(), 20);
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
                workflow_cache: HashMap::new(),
            },
        );

        assert!(result
            .unwrap()
            .into_string()
            .unwrap()
            .contains("hello%20world"));
    }

    #[test]
    fn test_hash_md5_uses_real_md5_digest() {
        let engine = ExpressionEngine::new().with_custom_functions();

        let result = engine
            .eval_with_context(
                "hash_md5(\"hello\")",
                &ExpressionContext {
                    json_data: serde_json::json!({}),
                    binary_keys: vec![],
                    parameters: HashMap::new(),
                    workflow_cache: HashMap::new(),
                },
            )
            .unwrap()
            .into_string()
            .unwrap();

        assert_eq!(result, "5d41402abc4b2a76b9719d911017c592");
    }

    #[test]
    fn test_hash_sha256_uses_real_sha256_digest() {
        let engine = ExpressionEngine::new().with_custom_functions();

        let result = engine
            .eval_with_context(
                "hash_sha256(\"hello\")",
                &ExpressionContext {
                    json_data: serde_json::json!({}),
                    binary_keys: vec![],
                    parameters: HashMap::new(),
                    workflow_cache: HashMap::new(),
                },
            )
            .unwrap()
            .into_string()
            .unwrap();

        assert_eq!(
            result,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_item_backward_traversal() {
        let engine = ExpressionEngine::new().with_custom_functions();

        // Mock the global execution workflow cache with historical data
        let mut workflow_cache = HashMap::new();
        workflow_cache.insert(
            "SetNode".to_string(),
            vec![serde_json::json!({"json": {"count": 99}})],
        );

        let context = ExpressionContext {
            json_data: serde_json::json!({}),
            binary_keys: vec![],
            parameters: HashMap::new(),
            workflow_cache,
        };

        let result = engine.eval_with_context("$item(\"SetNode\").$json.count", &context);

        assert!(
            result.is_ok(),
            "Failed to evaluate expression: {:?}",
            result.err()
        );
        assert_eq!(result.unwrap().as_int().unwrap(), 99);
    }
}
