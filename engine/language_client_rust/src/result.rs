use crate::types::BamlValue;
use serde::{Deserialize, Serialize};

/// Result of a BAML function call
pub type BamlResult<T> = std::result::Result<T, crate::BamlError>;

/// Function execution result containing the response and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionResult {
    /// The function response data
    pub data: BamlValue,
    /// Function call ID for tracing
    pub call_id: String,
    /// Metadata about the function execution
    pub metadata: FunctionMetadata,
}

/// Metadata about function execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetadata {
    /// Duration of the function call in milliseconds
    pub duration_ms: u64,
    /// Number of input tokens used (if available)
    pub input_tokens: Option<u64>,
    /// Number of output tokens used (if available)  
    pub output_tokens: Option<u64>,
    /// Total cost in USD (if available)
    pub cost_usd: Option<f64>,
    /// Provider used for the function call
    pub provider: Option<String>,
    /// Model used for the function call
    pub model: Option<String>,
}

impl FunctionResult {
    /// Create a new function result
    pub fn new(data: BamlValue, call_id: String) -> Self {
        Self {
            data,
            call_id,
            metadata: FunctionMetadata {
                duration_ms: 0,
                input_tokens: None,
                output_tokens: None,
                cost_usd: None,
                provider: None,
                model: None,
            },
        }
    }

    /// Create a function result with metadata
    pub fn with_metadata(data: BamlValue, call_id: String, metadata: FunctionMetadata) -> Self {
        Self {
            data,
            call_id,
            metadata,
        }
    }

    /// Extract the data as a specific type
    pub fn into_data<T>(self) -> BamlResult<T>
    where
        T: crate::types::FromBamlValue,
    {
        crate::types::FromBamlValue::from_baml_value(self.data)
    }

    /// Get a reference to the data
    pub fn data(&self) -> &BamlValue {
        &self.data
    }

    /// Get the function call ID
    pub fn call_id(&self) -> &str {
        &self.call_id
    }

    /// Get the metadata
    pub fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

impl FunctionMetadata {
    /// Create empty metadata
    pub fn empty() -> Self {
        Self {
            duration_ms: 0,
            input_tokens: None,
            output_tokens: None,
            cost_usd: None,
            provider: None,
            model: None,
        }
    }

    /// Set duration
    pub fn with_duration_ms(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    /// Set token counts
    pub fn with_tokens(mut self, input_tokens: u64, output_tokens: u64) -> Self {
        self.input_tokens = Some(input_tokens);
        self.output_tokens = Some(output_tokens);
        self
    }

    /// Set cost
    pub fn with_cost_usd(mut self, cost_usd: f64) -> Self {
        self.cost_usd = Some(cost_usd);
        self
    }

    /// Set provider and model
    pub fn with_provider_model(mut self, provider: String, model: String) -> Self {
        self.provider = Some(provider);
        self.model = Some(model);
        self
    }
}
