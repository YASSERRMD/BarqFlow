//! Branching and Conditional Routing
//!
//! Implements branching logic for IF/Switch nodes and merge logic
//! for combining data from multiple branches.

use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::types::IDataObject;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

/// Condition type for branching decisions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConditionType {
    /// Boolean equals check
    Boolean,
    /// String equals check
    String,
    /// Numeric comparison
    Number,
    /// Expression evaluation (future: use Rhai)
    Expression,
}

/// Comparison operator for conditions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ComparisonOperator {
    /// Equals
    Equals,
    /// Not equals
    NotEquals,
    /// Less than
    LessThan,
    /// Less than or equal
    LessThanOrEqual,
    /// Greater than
    GreaterThan,
    /// Greater than or equal
    GreaterThanOrEqual,
    /// Contains (for strings/arrays)
    Contains,
    /// Starts with (for strings)
    StartsWith,
    /// Ends with (for strings)
    EndsWith,
    /// Is empty
    IsEmpty,
    /// Is null
    IsNull,
}

/// A single condition for routing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingCondition {
    /// The field to check (JSON path expression)
    pub field: String,
    /// The condition type
    pub condition_type: ConditionType,
    /// The comparison operator
    pub operator: ComparisonOperator,
    /// The value to compare against
    pub value: serde_json::Value,
}

/// Output configuration for a branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchOutput {
    /// The output index
    pub output_index: usize,
    /// The condition(s) that must be met for this output
    pub conditions: Vec<RoutingCondition>,
    /// Whether all conditions must be met (AND) or any (OR)
    pub combine_operation: CombineOperation,
    /// Display name for this branch
    pub name: Option<String>,
}

/// How to combine multiple conditions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CombineOperation {
    /// All conditions must be true
    All,
    /// At least one condition must be true
    Any,
    /// All conditions must be false
    None,
}

/// Merge strategy for combining data from multiple inputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MergeStrategy {
    /// Append all items from all inputs into a single output
    Append,
    /// Merge by waiting for all inputs to have data at each index
    MergeByIndex,
    /// Keep only the first input that has data
    FirstAvailable,
    /// Keep all inputs as separate outputs
    PassThrough,
}

/// Configuration for merging data from multiple branches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeConfig {
    /// The merge strategy to use
    pub strategy: MergeStrategy,
    /// Number of inputs to merge
    pub input_count: usize,
    /// Options specific to the merge strategy
    pub options: Option<MergeOptions>,
}

/// Options for merge strategies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeOptions {
    /// For MergeByIndex: what to do if indexes don't align
    pub mode: Option<MergeMode>,
}

/// How to handle misaligned indexes in MergeByIndex.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MergeMode {
    /// Use the first available value
    FirstAvailable,
    /// Wait for all inputs to have values
    WaitForAll,
    /// Use default values for missing data
    UseDefaults,
}

/// Evaluates routing conditions against input data.
pub struct ConditionEvaluator;

impl ConditionEvaluator {
    /// Evaluate a single condition against data.
    ///
    /// # Arguments
    /// * `condition` - The condition to evaluate
    /// * `data` - The data to check
    ///
    /// # Returns
    /// true if the condition is met, false otherwise
    pub fn evaluate_condition(
        condition: &RoutingCondition,
        data: &INodeExecutionData,
    ) -> bool {
        // Extract the field value from the data using the field path
        let field_value = Self::extract_field_value(&condition.field, &data.json);

        match condition.operator {
            ComparisonOperator::Equals => Self::check_equals(&field_value, &condition.value),
            ComparisonOperator::NotEquals => !Self::check_equals(&field_value, &condition.value),
            ComparisonOperator::LessThan => Self::check_compare(&field_value, &condition.value, |a, b| a < b),
            ComparisonOperator::LessThanOrEqual => Self::check_compare(&field_value, &condition.value, |a, b| a <= b),
            ComparisonOperator::GreaterThan => Self::check_compare(&field_value, &condition.value, |a, b| a > b),
            ComparisonOperator::GreaterThanOrEqual => Self::check_compare(&field_value, &condition.value, |a, b| a >= b),
            ComparisonOperator::Contains => Self::check_contains(&field_value, &condition.value),
            ComparisonOperator::StartsWith => Self::check_starts_with(&field_value, &condition.value),
            ComparisonOperator::EndsWith => Self::check_ends_with(&field_value, &condition.value),
            ComparisonOperator::IsEmpty => Self::check_is_empty(&field_value),
            ComparisonOperator::IsNull => field_value.is_null(),
        }
    }

    /// Evaluate multiple conditions with a combine operation.
    ///
    /// # Arguments
    /// * `conditions` - The conditions to evaluate
    /// * `data` - The data to check
    /// * `combine_op` - How to combine the results (AND/OR/NONE)
    ///
    /// # Returns
    /// true if the combined conditions are met
    pub fn evaluate_conditions(
        conditions: &[RoutingCondition],
        data: &INodeExecutionData,
        combine_op: CombineOperation,
    ) -> bool {
        if conditions.is_empty() {
            return true;
        }

        let results: Vec<bool> = conditions
            .iter()
            .map(|c| Self::evaluate_condition(c, data))
            .collect();

        match combine_op {
            CombineOperation::All => results.iter().all(|&r| r),
            CombineOperation::Any => results.iter().any(|&r| r),
            CombineOperation::None => results.iter().all(|&r| !r),
        }
    }

    /// Extract a field value from data using JSON path.
    fn extract_field_value(field: &str, data: &IDataObject) -> serde_json::Value {
        // Simple JSON path implementation
        // Supports paths like "$json.field", "$json.nested.field"
        let parts: Vec<&str> = field.split('.').collect();

        let mut current = serde_json::Value::Object(data.0.clone());

        // Skip the first part if it's "$json"
        for part in parts.iter().skip(if parts.first().map(|p| p.starts_with('$')).unwrap_or(false) { 1 } else { 0 }) {
            current = match current.get(*part) {
                Some(value) => value.clone(),
                None => return serde_json::Value::Null,
            };
        }

        current
    }

    fn check_equals(a: &serde_json::Value, b: &serde_json::Value) -> bool {
        a == b
    }

    fn check_compare<F>(a: &serde_json::Value, b: &serde_json::Value, compare_fn: F) -> bool
    where
        F: Fn(f64, f64) -> bool,
    {
        match (a.as_f64(), b.as_f64()) {
            (Some(a_num), Some(b_num)) => compare_fn(a_num, b_num),
            _ => false,
        }
    }

    fn check_contains(haystack: &serde_json::Value, needle: &serde_json::Value) -> bool {
        match (haystack, needle) {
            (serde_json::Value::String(h), serde_json::Value::String(n)) => h.contains(n),
            (serde_json::Value::Array(arr), _) => arr.contains(needle),
            _ => false,
        }
    }

    fn check_starts_with(haystack: &serde_json::Value, needle: &serde_json::Value) -> bool {
        match (haystack, needle) {
            (serde_json::Value::String(h), serde_json::Value::String(n)) => h.starts_with(n),
            _ => false,
        }
    }

    fn check_ends_with(haystack: &serde_json::Value, needle: &serde_json::Value) -> bool {
        match (haystack, needle) {
            (serde_json::Value::String(h), serde_json::Value::String(n)) => h.ends_with(n),
            _ => false,
        }
    }

    fn check_is_empty(value: &serde_json::Value) -> bool {
        match value {
            serde_json::Value::String(s) => s.is_empty(),
            serde_json::Value::Array(arr) => arr.is_empty(),
            serde_json::Value::Object(obj) => obj.is_empty(),
            serde_json::Value::Null => true,
            _ => false,
        }
    }
}

/// Merges data from multiple inputs according to a strategy.
pub struct DataMerger;

impl DataMerger {
    /// Merge data from multiple inputs.
    ///
    /// # Arguments
    /// * `inputs` - Input data from multiple branches (index -> data)
    /// * `config` - Merge configuration
    ///
    /// # Returns
    /// Merged output data
    pub fn merge(
        inputs: &HashMap<usize, Vec<INodeExecutionData>>,
        config: &MergeConfig,
    ) -> Vec<INodeExecutionData> {
        match config.strategy {
            MergeStrategy::Append => Self::merge_append(inputs),
            MergeStrategy::MergeByIndex => Self::merge_by_index(inputs, config),
            MergeStrategy::FirstAvailable => Self::merge_first_available(inputs),
            MergeStrategy::PassThrough => Self::merge_pass_through(inputs),
        }
    }

    /// Append all items from all inputs.
    fn merge_append(inputs: &HashMap<usize, Vec<INodeExecutionData>>) -> Vec<INodeExecutionData> {
        let mut result = Vec::new();
        for (_, data) in inputs {
            result.extend(data.clone());
        }
        result
    }

    /// Merge by index, combining data at the same index from all inputs.
    fn merge_by_index(
        inputs: &HashMap<usize, Vec<INodeExecutionData>>,
        _config: &MergeConfig,
    ) -> Vec<INodeExecutionData> {
        // Find the maximum length
        let max_len = inputs.values().map(|v| v.len()).max().unwrap_or(0);

        let mut result = Vec::new();

        for i in 0..max_len {
            let mut map = serde_json::Map::new();

            for (input_idx, data) in inputs {
                if let Some(item) = data.get(i) {
                    // Add data from this input to the merged object
                    map.insert(
                        format!("input{}", input_idx),
                        serde_json::Value::Object(item.json.0.clone())
                    );
                }
            }

            result.push(INodeExecutionData::new(IDataObject(map)));
        }

        result
    }

    /// Keep only the first input that has data.
    fn merge_first_available(inputs: &HashMap<usize, Vec<INodeExecutionData>>) -> Vec<INodeExecutionData> {
        for i in 0..inputs.len() {
            if let Some(data) = inputs.get(&i) {
                if !data.is_empty() {
                    return data.clone();
                }
            }
        }
        Vec::new()
    }

    /// Keep all inputs as separate outputs (multi-output).
    fn merge_pass_through(inputs: &HashMap<usize, Vec<INodeExecutionData>>) -> Vec<INodeExecutionData> {
        // For pass-through, we return a flattened structure
        // In a real implementation, this would need to handle multiple outputs
        Self::merge_append(inputs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data(json_value: serde_json::Value) -> INodeExecutionData {
        INodeExecutionData::new(IDataObject::from(json_value))
    }

    #[test]
    fn test_condition_equals() {
        let condition = RoutingCondition {
            field: "$json.status".to_string(),
            condition_type: ConditionType::String,
            operator: ComparisonOperator::Equals,
            value: json!("active"),
        };

        let data = create_test_data(json!({"status": "active"}));
        assert!(ConditionEvaluator::evaluate_condition(&condition, &data));

        let data2 = create_test_data(json!({"status": "inactive"}));
        assert!(!ConditionEvaluator::evaluate_condition(&condition, &data2));
    }

    #[test]
    fn test_condition_greater_than() {
        let condition = RoutingCondition {
            field: "$json.count".to_string(),
            condition_type: ConditionType::Number,
            operator: ComparisonOperator::GreaterThan,
            value: json!(5),
        };

        let data = create_test_data(json!({"count": 10}));
        assert!(ConditionEvaluator::evaluate_condition(&condition, &data));

        let data2 = create_test_data(json!({"count": 3}));
        assert!(!ConditionEvaluator::evaluate_condition(&condition, &data2));
    }

    #[test]
    fn test_condition_contains() {
        let condition = RoutingCondition {
            field: "$json.tags".to_string(),
            condition_type: ConditionType::String,
            operator: ComparisonOperator::Contains,
            value: json!("important"),
        };

        let data = create_test_data(json!({"tags": ["important", "test"]}));
        assert!(ConditionEvaluator::evaluate_condition(&condition, &data));

        let data2 = create_test_data(json!({"tags": ["other"]}));
        assert!(!ConditionEvaluator::evaluate_condition(&condition, &data2));
    }

    #[test]
    fn test_combine_all() {
        let cond1 = RoutingCondition {
            field: "$json.status".to_string(),
            condition_type: ConditionType::String,
            operator: ComparisonOperator::Equals,
            value: json!("active"),
        };

        let cond2 = RoutingCondition {
            field: "$json.count".to_string(),
            condition_type: ConditionType::Number,
            operator: ComparisonOperator::GreaterThan,
            value: json!(0),
        };

        let data = create_test_data(json!({"status": "active", "count": 5}));
        assert!(ConditionEvaluator::evaluate_conditions(
            &[cond1.clone(), cond2.clone()],
            &data,
            CombineOperation::All
        ));

        let data2 = create_test_data(json!({"status": "inactive", "count": 5}));
        assert!(!ConditionEvaluator::evaluate_conditions(
            &[cond1, cond2],
            &data2,
            CombineOperation::All
        ));
    }

    #[test]
    fn test_combine_any() {
        let cond1 = RoutingCondition {
            field: "$json.status".to_string(),
            condition_type: ConditionType::String,
            operator: ComparisonOperator::Equals,
            value: json!("active"),
        };

        let cond2 = RoutingCondition {
            field: "$json.count".to_string(),
            condition_type: ConditionType::Number,
            operator: ComparisonOperator::Equals,
            value: json!(100),
        };

        let data = create_test_data(json!({"status": "active", "count": 5}));
        assert!(ConditionEvaluator::evaluate_conditions(
            &[cond1.clone(), cond2.clone()],
            &data,
            CombineOperation::Any
        ));

        let data2 = create_test_data(json!({"status": "inactive", "count": 5}));
        assert!(!ConditionEvaluator::evaluate_conditions(
            &[cond1, cond2],
            &data2,
            CombineOperation::Any
        ));
    }

    #[test]
    fn test_merge_append() {
        let mut inputs = HashMap::new();
        inputs.insert(0, vec![create_test_data(json!({"id": 1}))]);
        inputs.insert(1, vec![create_test_data(json!({"id": 2}))]);

        let config = MergeConfig {
            strategy: MergeStrategy::Append,
            input_count: 2,
            options: None,
        };

        let result = DataMerger::merge(&inputs, &config);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_merge_by_index() {
        let mut inputs = HashMap::new();
        inputs.insert(0, vec![create_test_data(json!({"from": "input0"}))]);
        inputs.insert(1, vec![create_test_data(json!({"from": "input1"}))]);

        let config = MergeConfig {
            strategy: MergeStrategy::MergeByIndex,
            input_count: 2,
            options: None,
        };

        let result = DataMerger::merge(&inputs, &config);
        assert_eq!(result.len(), 1);
        assert!(result[0].json.0.get("input0").is_some());
        assert!(result[0].json.0.get("input1").is_some());
    }

    #[test]
    fn test_merge_first_available() {
        let mut inputs = HashMap::new();
        inputs.insert(0, vec![]);  // Empty input
        inputs.insert(1, vec![create_test_data(json!({"from": "input1"}))]);

        let config = MergeConfig {
            strategy: MergeStrategy::FirstAvailable,
            input_count: 2,
            options: None,
        };

        let result = DataMerger::merge(&inputs, &config);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].json.0.get("from").unwrap(), "input1");
    }

    #[test]
    fn test_condition_is_empty() {
        let condition = RoutingCondition {
            field: "$json.items".to_string(),
            condition_type: ConditionType::String,
            operator: ComparisonOperator::IsEmpty,
            value: json!(null),
        };

        let data = create_test_data(json!({"items": []}));
        assert!(ConditionEvaluator::evaluate_condition(&condition, &data));

        let data2 = create_test_data(json!({"items": ["test"]}));
        assert!(!ConditionEvaluator::evaluate_condition(&condition, &data2));
    }

    #[test]
    fn test_condition_starts_with() {
        let condition = RoutingCondition {
            field: "$json.name".to_string(),
            condition_type: ConditionType::String,
            operator: ComparisonOperator::StartsWith,
            value: json!("test"),
        };

        let data = create_test_data(json!({"name": "testing123"}));
        assert!(ConditionEvaluator::evaluate_condition(&condition, &data));

        let data2 = create_test_data(json!({"name": "123test"}));
        assert!(!ConditionEvaluator::evaluate_condition(&condition, &data2));
    }
}
