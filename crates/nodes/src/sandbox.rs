//! Sandbox Execution Nodes
//!
//! Implements sandboxed script execution for user-defined code.

use rhai::{Engine, Scope};
use std::collections::HashMap;

pub struct SandboxExecutor {
    engine: Engine,
}

impl SandboxExecutor {
    pub fn new() -> Self {
        let mut engine = Engine::new_raw();
        engine.set_allow_looping(false);
        engine.set_allow_shadowing(false);
        engine.set_strict_variables(true);
        engine.set_max_expr_depths(8, 8);
        engine.set_max_operations(1000);
        engine.set_max_call_levels(4);
        Self { engine }
    }

    pub fn execute(
        &self,
        script: &str,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        let mut scope = Scope::new();

        for (key, value) in context {
            let dynamic = json_to_dynamic(value);
            scope.push_constant(key.clone(), dynamic);
        }

        let result = self
            .engine
            .eval_with_scope::<rhai::Dynamic>(&mut scope, script)
            .map_err(|e| e.to_string())?;

        Ok(dynamic_to_json(&result))
    }
}

fn json_to_dynamic(value: &serde_json::Value) -> rhai::Dynamic {
    match value {
        serde_json::Value::Null => rhai::Dynamic::from(()),
        serde_json::Value::Bool(b) => rhai::Dynamic::from(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                rhai::Dynamic::from(i)
            } else if let Some(f) = n.as_f64() {
                rhai::Dynamic::from(f)
            } else {
                rhai::Dynamic::from(n.to_string())
            }
        }
        serde_json::Value::String(s) => rhai::Dynamic::from(s.clone()),
        serde_json::Value::Array(arr) => {
            let converted: Vec<rhai::Dynamic> = arr.iter().map(json_to_dynamic).collect();
            rhai::Dynamic::from(converted)
        }
        serde_json::Value::Object(obj) => {
            let map: HashMap<String, rhai::Dynamic> = obj
                .iter()
                .map(|(k, v)| (k.clone(), json_to_dynamic(v)))
                .collect();
            rhai::Dynamic::from(map)
        }
    }
}

fn dynamic_to_json(dynamic: &rhai::Dynamic) -> serde_json::Value {
    if dynamic.is::<i64>() {
        serde_json::Value::Number(dynamic.as_int().unwrap().into())
    } else if dynamic.is::<f64>() {
        serde_json::json!(dynamic.as_float().unwrap())
    } else if dynamic.is::<bool>() {
        serde_json::Value::Bool(dynamic.as_bool().unwrap())
    } else if let Ok(s) = dynamic.clone().into_string() {
        serde_json::Value::String(s)
    } else {
        serde_json::Value::Null
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_executor_creation() {
        let executor = SandboxExecutor::new();
        let context = HashMap::new();
        let result = executor.execute("1 + 1", &context);
        assert!(result.is_ok());
    }
}
