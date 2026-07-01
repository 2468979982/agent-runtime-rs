use crate::tools::types::*;
use async_trait::async_trait;
use meval::eval_str;

/// Calculator tool for mathematical expressions
pub struct CalculatorTool {
    metadata: ToolMetadata,
}

impl CalculatorTool {
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                name: "calculator".to_string(),
                description: "Evaluate a mathematical expression. Supports basic arithmetic, parentheses, and mathematical functions.".to_string(),
                parameters: vec![
                    ToolParameter {
                        name: "expression".to_string(),
                        description: "Mathematical expression to evaluate (e.g., '2 + 2', 'sin(0.5)', '(1 + 2) * 3')".to_string(),
                        required: true,
                        parameter_type: ParameterType::String,
                    },
                ],
            },
        }
    }
}

#[async_trait]
impl ToolExecutor for CalculatorTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }
    
    async fn execute(&self, parameters: serde_json::Value) -> Result<ToolResult, ToolError> {
        let expression = parameters["expression"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParameters("Missing 'expression' parameter".to_string()))?;
        
        match eval_str(expression) {
            Ok(result) => Ok(ToolResult {
                success: true,
                output: result.to_string(),
                error: None,
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to evaluate expression: {}", e)),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_calculator_basic_arithmetic() {
        let tool = CalculatorTool::new();
        
        let result = tool.execute(json!({"expression": "2 + 2"})).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "4");
    }
    
    #[tokio::test]
    async fn test_calculator_complex_expression() {
        let tool = CalculatorTool::new();
        
        let result = tool.execute(json!({"expression": "(1 + 2) * 3"})).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "9");
    }
    
    #[tokio::test]
    async fn test_calculator_invalid_expression() {
        let tool = CalculatorTool::new();
        
        // Use an expression that will definitely fail
        let result = tool.execute(json!({"expression": "2 + + +"})).await.unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
    }
    
    #[tokio::test]
    async fn test_calculator_missing_parameter() {
        let tool = CalculatorTool::new();
        
        let result = tool.execute(json!({})).await;
        assert!(result.is_err());
    }
}
