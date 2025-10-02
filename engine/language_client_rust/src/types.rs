//! Core BAML types for the Rust client
//!
//! This module provides the type system used by BAML functions.

use crate::{runtime::RuntimeHandleArc, BamlError, BamlResult};
use anyhow::anyhow;
use baml_cffi::{
    baml::cffi::CffiRawObject,
    rust::{CollectorHandle, TypeBuilderHandle},
};
use std::cell::Cell;
use std::sync::{Arc, Mutex};

// No additional imports needed for basic type conversions

// Re-export BamlValue and BamlMap from baml-types to maintain compatibility
pub use baml_types::{BamlMap, BamlValue};

mod raw_objects;
pub use raw_objects::{
    FunctionLog, HttpBody, HttpRequest, HttpResponse, LlmCall, LlmCallKind, LlmStreamCall,
    SseResponse, StreamTiming, Timing, Usage,
};

thread_local! {
    static PARTIAL_DESERIALIZATION: Cell<bool> = Cell::new(false);
}

/// Enable partial deserialization for the scope of the provided closure.
///
/// When enabled, missing or `Null` values will be replaced with sensible
/// defaults instead of returning a deserialization error. This is primarily
/// used to allow streaming partial updates to succeed while the model is
/// still filling in required fields.
pub fn with_partial_deserialization<R>(f: impl FnOnce() -> R) -> R {
    struct Reset(bool);
    impl Drop for Reset {
        fn drop(&mut self) {
            PARTIAL_DESERIALIZATION.with(|flag| flag.set(self.0));
        }
    }

    let previous = PARTIAL_DESERIALIZATION.with(|flag| {
        let prev = flag.get();
        flag.set(true);
        prev
    });
    let _reset = Reset(previous);
    f()
}

/// Returns true when partial deserialization mode is enabled.
pub fn is_partial_deserialization() -> bool {
    PARTIAL_DESERIALIZATION.with(|flag| flag.get())
}

/// Merge a newer `BamlValue` into an optional existing value, preserving the
/// previous data whenever the new value is still absent (Null) due to
/// incremental streaming.
pub fn overlay_baml_value(base: Option<BamlValue>, update: BamlValue) -> BamlValue {
    match update {
        BamlValue::Null => base.unwrap_or(BamlValue::Null),
        BamlValue::Class(name, update_map) => {
            let mut merged = match base {
                Some(BamlValue::Class(_, base_map)) => base_map,
                _ => BamlMap::new(),
            };
            for (key, update_value) in update_map.into_iter() {
                let previous = merged.get(&key).cloned();
                let merged_value = overlay_baml_value(previous, update_value);
                merged.insert(key, merged_value);
            }
            BamlValue::Class(name, merged)
        }
        BamlValue::Map(update_map) => {
            let mut merged = match base {
                Some(BamlValue::Map(base_map)) => base_map,
                _ => BamlMap::new(),
            };
            for (key, update_value) in update_map.into_iter() {
                let previous = merged.get(&key).cloned();
                let merged_value = overlay_baml_value(previous, update_value);
                merged.insert(key, merged_value);
            }
            BamlValue::Map(merged)
        }
        BamlValue::List(update_list) => {
            if update_list.is_empty() {
                if let Some(BamlValue::List(base_list)) = base {
                    BamlValue::List(base_list)
                } else {
                    BamlValue::List(update_list)
                }
            } else {
                BamlValue::List(update_list)
            }
        }
        other => other,
    }
}

/// Determine if a `BamlValue` contains any non-null data.
pub fn baml_value_has_data(value: &BamlValue) -> bool {
    match value {
        BamlValue::Null => false,
        BamlValue::String(s) => !s.is_empty(),
        BamlValue::Int(_) | BamlValue::Float(_) | BamlValue::Bool(_) => true,
        BamlValue::Media(_) => true,
        BamlValue::Enum(_, v) => !v.is_empty(),
        BamlValue::List(items) => items.iter().any(baml_value_has_data),
        BamlValue::Map(map) | BamlValue::Class(_, map) => map.values().any(baml_value_has_data),
    }
}

/// Convert a Rust value to a BAML value
pub trait ToBamlValue {
    /// Convert self to a BamlValue
    fn to_baml_value(self) -> crate::BamlResult<BamlValue>;
}

/// Convert a BAML value to a Rust type
pub trait FromBamlValue: Sized {
    /// Try to convert a BamlValue to Self
    fn from_baml_value(value: BamlValue) -> crate::BamlResult<Self>;
}

// Implementations for common types
impl ToBamlValue for String {
    fn to_baml_value(self) -> crate::BamlResult<BamlValue> {
        Ok(BamlValue::String(self))
    }
}

impl ToBamlValue for &str {
    fn to_baml_value(self) -> crate::BamlResult<BamlValue> {
        Ok(BamlValue::String(self.to_string()))
    }
}

impl FromBamlValue for String {
    fn from_baml_value(value: BamlValue) -> crate::BamlResult<Self> {
        match value {
            BamlValue::String(s) => Ok(s),
            BamlValue::Null if is_partial_deserialization() => Ok(String::new()),
            _ => Err(crate::BamlError::deserialization(format!(
                "Expected string, got {:?}",
                value
            ))),
        }
    }
}

impl ToBamlValue for i32 {
    fn to_baml_value(self) -> crate::BamlResult<BamlValue> {
        Ok(BamlValue::Int(self as i64))
    }
}

impl FromBamlValue for i32 {
    fn from_baml_value(value: BamlValue) -> crate::BamlResult<Self> {
        match value {
            BamlValue::Int(i) => i
                .try_into()
                .map_err(|_| crate::BamlError::deserialization("Integer overflow".to_string())),
            BamlValue::Null if is_partial_deserialization() => Ok(0),
            _ => Err(crate::BamlError::deserialization(format!(
                "Expected int, got {:?}",
                value
            ))),
        }
    }
}

impl ToBamlValue for i64 {
    fn to_baml_value(self) -> crate::BamlResult<BamlValue> {
        Ok(BamlValue::Int(self))
    }
}

impl FromBamlValue for i64 {
    fn from_baml_value(value: BamlValue) -> crate::BamlResult<Self> {
        match value {
            BamlValue::Int(i) => Ok(i),
            BamlValue::Null if is_partial_deserialization() => Ok(0),
            _ => Err(crate::BamlError::deserialization(format!(
                "Expected int, got {:?}",
                value
            ))),
        }
    }
}

impl ToBamlValue for f64 {
    fn to_baml_value(self) -> crate::BamlResult<BamlValue> {
        Ok(BamlValue::Float(self))
    }
}

impl FromBamlValue for f64 {
    fn from_baml_value(value: BamlValue) -> crate::BamlResult<Self> {
        match value {
            BamlValue::Float(f) => Ok(f),
            BamlValue::Int(i) => Ok(i as f64),
            BamlValue::Null if is_partial_deserialization() => Ok(0.0),
            _ => Err(crate::BamlError::deserialization(format!(
                "Expected float, got {:?}",
                value
            ))),
        }
    }
}

impl ToBamlValue for bool {
    fn to_baml_value(self) -> crate::BamlResult<BamlValue> {
        Ok(BamlValue::Bool(self))
    }
}

impl FromBamlValue for bool {
    fn from_baml_value(value: BamlValue) -> crate::BamlResult<Self> {
        match value {
            BamlValue::Bool(b) => Ok(b),
            BamlValue::Null if is_partial_deserialization() => Ok(false),
            _ => Err(crate::BamlError::deserialization(format!(
                "Expected bool, got {:?}",
                value
            ))),
        }
    }
}

impl<T: ToBamlValue> ToBamlValue for Vec<T> {
    fn to_baml_value(self) -> crate::BamlResult<BamlValue> {
        let values: Result<Vec<_>, _> = self.into_iter().map(|v| v.to_baml_value()).collect();
        Ok(BamlValue::List(values?))
    }
}

impl<T: FromBamlValue> FromBamlValue for Vec<T> {
    fn from_baml_value(value: BamlValue) -> crate::BamlResult<Self> {
        match value {
            BamlValue::List(list) => list
                .into_iter()
                .map(T::from_baml_value)
                .collect::<Result<Vec<_>, _>>(),
            BamlValue::Null if is_partial_deserialization() => Ok(Vec::new()),
            _ => Err(crate::BamlError::deserialization(format!(
                "Expected list, got {:?}",
                value
            ))),
        }
    }
}

impl<T: ToBamlValue> ToBamlValue for Option<T> {
    fn to_baml_value(self) -> crate::BamlResult<BamlValue> {
        match self {
            Some(value) => value.to_baml_value(),
            None => Ok(BamlValue::Null),
        }
    }
}

impl<T: FromBamlValue> FromBamlValue for Option<T> {
    fn from_baml_value(value: BamlValue) -> crate::BamlResult<Self> {
        match value {
            BamlValue::Null => Ok(None),
            other => Ok(Some(T::from_baml_value(other)?)),
        }
    }
}

impl<T: ToBamlValue> ToBamlValue for Box<T> {
    fn to_baml_value(self) -> crate::BamlResult<BamlValue> {
        (*self).to_baml_value()
    }
}

impl<T: FromBamlValue> FromBamlValue for Box<T> {
    fn from_baml_value(value: BamlValue) -> crate::BamlResult<Self> {
        T::from_baml_value(value).map(Box::new)
    }
}

// HashMap implementations
impl<K, V> ToBamlValue for std::collections::HashMap<K, V>
where
    K: ToString,
    V: ToBamlValue,
{
    fn to_baml_value(self) -> crate::BamlResult<BamlValue> {
        let mut map = BamlMap::new();
        for (key, value) in self {
            map.insert(key.to_string(), value.to_baml_value()?);
        }
        Ok(BamlValue::Map(map))
    }
}

impl<K, V> FromBamlValue for std::collections::HashMap<K, V>
where
    K: std::str::FromStr + std::hash::Hash + Eq,
    K::Err: std::fmt::Debug,
    V: FromBamlValue,
{
    fn from_baml_value(value: BamlValue) -> crate::BamlResult<Self> {
        match value {
            BamlValue::Map(map) => {
                let mut result = std::collections::HashMap::new();
                for (key_str, value) in map {
                    let key = K::from_str(&key_str).map_err(|e| {
                        crate::BamlError::deserialization(format!(
                            "Could not parse key '{}': {:?}",
                            key_str, e
                        ))
                    })?;
                    let parsed_value = V::from_baml_value(value)?;
                    result.insert(key, parsed_value);
                }
                Ok(result)
            }
            BamlValue::Null if is_partial_deserialization() => Ok(std::collections::HashMap::new()),
            _ => Err(crate::BamlError::deserialization(format!(
                "Expected map, got {:?}",
                value
            ))),
        }
    }
}

impl ToBamlValue for BamlMap<String, BamlValue> {
    fn to_baml_value(self) -> crate::BamlResult<BamlValue> {
        Ok(BamlValue::Map(self))
    }
}

impl FromBamlValue for BamlMap<String, BamlValue> {
    fn from_baml_value(value: BamlValue) -> crate::BamlResult<Self> {
        match value {
            BamlValue::Map(map) => Ok(map),
            BamlValue::Null if is_partial_deserialization() => Ok(BamlMap::new()),
            _ => Err(crate::BamlError::deserialization(format!(
                "Expected map, got {:?}",
                value
            ))),
        }
    }
}

// Stub implementations for BAML runtime components we're no longer using directly

/// Type builder for BAML types backed by the shared CFFI runtime
#[derive(Debug, Clone)]
pub struct TypeBuilder {
    handle: TypeBuilderHandle,
}

impl TypeBuilder {
    /// Create a new type builder
    pub fn new() -> BamlResult<Self> {
        let handle = TypeBuilderHandle::new().map_err(|e| BamlError::Runtime(anyhow!(e)))?;
        Ok(Self { handle })
    }

    pub(crate) fn to_cffi(&self) -> CffiRawObject {
        self.handle.to_cffi()
    }
}

impl Default for TypeBuilder {
    fn default() -> Self {
        TypeBuilder::new().expect("failed to create TypeBuilder handle")
    }
}

/// Client registry for BAML clients (stub implementation)
#[derive(Debug, Clone)]
pub struct ClientRegistry {
    // This is now just a placeholder - the real client registry is in the FFI layer
}

impl ClientRegistry {
    /// Create a new client registry
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ClientRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Collector for BAML tracing backed by the shared CFFI runtime
#[derive(Debug, Clone)]
pub struct Collector {
    handle: CollectorHandle,
    runtime: Arc<Mutex<Option<RuntimeHandleArc>>>,
}

impl Collector {
    /// Create a new collector
    pub fn new(name: Option<&str>) -> BamlResult<Self> {
        let handle = CollectorHandle::new(name).map_err(|e| BamlError::Runtime(anyhow!(e)))?;
        Ok(Self {
            handle,
            runtime: Arc::new(Mutex::new(None)),
        })
    }

    pub(crate) fn bind_runtime(&self, runtime: RuntimeHandleArc) -> BamlResult<()> {
        let mut guard = self.runtime.lock().unwrap();
        if let Some(existing) = guard.as_ref() {
            if !Arc::ptr_eq(existing, &runtime) {
                return Err(BamlError::Configuration(
                    "Collector is already bound to a different BAML runtime".to_string(),
                ));
            }
            return Ok(());
        }
        *guard = Some(runtime);
        Ok(())
    }

    fn runtime(&self) -> BamlResult<RuntimeHandleArc> {
        self.runtime
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| {
                BamlError::Configuration(
                    "Collector is not attached to a runtime. Pass it to a BamlClient call before querying it.".to_string(),
                )
            })
    }

    pub(crate) fn to_cffi(&self) -> CffiRawObject {
        self.handle.to_cffi()
    }

    /// Fetch usage statistics accumulated in this collector.
    pub fn usage(&self) -> BamlResult<Usage> {
        let runtime = self.runtime()?;
        let response =
            raw_objects::call_object_method(&runtime, &self.handle.to_cffi(), "usage", Vec::new())?;
        raw_objects::collector_usage_value_from_response(runtime, response)
    }

    /// Get the collector name (if provided during creation).
    pub fn name(&self) -> BamlResult<String> {
        let runtime = self.runtime()?;
        let response =
            raw_objects::call_object_method(&runtime, &self.handle.to_cffi(), "name", Vec::new())?;
        raw_objects::name_from_response(response)
    }

    /// Retrieve all function logs captured by this collector.
    pub fn logs(&self) -> BamlResult<Vec<FunctionLog>> {
        let runtime = self.runtime()?;
        let response =
            raw_objects::call_object_method(&runtime, &self.handle.to_cffi(), "logs", Vec::new())?;
        raw_objects::function_logs_from_response(runtime, response)
    }

    /// Retrieve the latest function log, if any.
    pub fn last(&self) -> BamlResult<Option<FunctionLog>> {
        let runtime = self.runtime()?;
        let response =
            raw_objects::call_object_method(&runtime, &self.handle.to_cffi(), "last", Vec::new())?;
        raw_objects::optional_function_log_from_response(runtime, response)
    }

    /// Lookup a log by its function call identifier.
    pub fn id(&self, function_id: &str) -> BamlResult<FunctionLog> {
        let runtime = self.runtime()?;
        let response = raw_objects::call_object_method(
            &runtime,
            &self.handle.to_cffi(),
            "id",
            vec![raw_objects::string_arg("id", function_id)],
        )?;
        raw_objects::function_log_from_response(runtime, response)
    }

    /// Clear the stored logs and return the number of entries removed.
    pub fn clear(&self) -> BamlResult<i64> {
        let runtime = self.runtime()?;
        let response =
            raw_objects::call_object_method(&runtime, &self.handle.to_cffi(), "clear", Vec::new())?;
        raw_objects::clear_count_from_response(response)
    }
}

impl Default for Collector {
    fn default() -> Self {
        Collector::new(None).expect("failed to create Collector handle")
    }
}

/// Runtime context manager (stub implementation)
#[derive(Debug, Clone)]
pub struct RuntimeContextManager {
    // This is now just a placeholder - the real context management is in the FFI layer
}

impl RuntimeContextManager {
    /// Create a new runtime context manager
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for RuntimeContextManager {
    fn default() -> Self {
        Self::new()
    }
}
