use std::collections::HashMap;
use std::mem::transmute_copy;
use std::os::raw::c_char;
use std::sync::Arc;

use anyhow::anyhow;
use baml_cffi::baml::cffi::{
    cffi_object_response::Response as CffiObjectResponseVariant,
    cffi_object_response_success::Result as CffiObjectResponseSuccess,
    cffi_raw_object::Object as RawObjectVariant, CffiMapEntry, CffiObjectMethodArguments,
    CffiObjectResponse, CffiRawObject,
};
use baml_cffi::DecodeFromBuffer;
use prost::Message;

use super::{BamlMap, BamlValue, FromBamlValue};
use crate::{errors::BamlError, ffi, runtime::RuntimeHandleArc, BamlResult};

#[derive(Debug)]
struct ObjectInner {
    runtime: RuntimeHandleArc,
    raw: CffiRawObject,
}

impl ObjectInner {
    fn new(runtime: RuntimeHandleArc, raw: CffiRawObject) -> Self {
        Self { runtime, raw }
    }

    fn call(&self, method: &str, kwargs: Vec<CffiMapEntry>) -> BamlResult<ObjectResponse> {
        call_object_method(&self.runtime, &self.raw, method, kwargs)
    }
}

impl Drop for ObjectInner {
    fn drop(&mut self) {
        let _ = call_object_method(&self.runtime, &self.raw, "~destructor", Vec::new());
    }
}

#[derive(Debug, Clone)]
pub struct Usage {
    inner: Arc<ObjectInner>,
}

#[derive(Debug, Clone)]
pub struct Timing {
    inner: Arc<ObjectInner>,
}

#[derive(Debug, Clone)]
pub struct StreamTiming {
    inner: Arc<ObjectInner>,
}

#[derive(Debug, Clone)]
pub struct FunctionLog {
    inner: Arc<ObjectInner>,
}

#[derive(Debug, Clone)]
pub struct LlmCall {
    inner: Arc<ObjectInner>,
}

#[derive(Debug, Clone)]
pub struct LlmStreamCall {
    call: LlmCall,
    inner: Arc<ObjectInner>,
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    inner: Arc<ObjectInner>,
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    inner: Arc<ObjectInner>,
}

#[derive(Debug, Clone)]
pub struct HttpBody {
    inner: Arc<ObjectInner>,
}

#[derive(Debug, Clone)]
pub struct SseResponse {
    inner: Arc<ObjectInner>,
}

#[derive(Debug, Clone)]
pub enum LlmCallKind {
    Basic(LlmCall),
    Stream(LlmStreamCall),
}

#[derive(Debug)]
pub(crate) enum ObjectResponse {
    Object(CffiRawObject),
    Objects(Vec<CffiRawObject>),
    Value(BamlValue),
    Null,
}

pub(crate) fn call_object_method(
    runtime: &RuntimeHandleArc,
    object: &CffiRawObject,
    method_name: &str,
    kwargs: Vec<CffiMapEntry>,
) -> BamlResult<ObjectResponse> {
    let args = CffiObjectMethodArguments {
        object: Some(object.clone()),
        method_name: method_name.to_string(),
        kwargs,
    };

    let encoded_args = args.encode_to_vec();

    let buffer = ffi::call_object_method(
        runtime.ptr(),
        encoded_args.as_ptr() as *const c_char,
        encoded_args.len(),
    );

    let (ptr, len): (*const i8, usize) = unsafe { transmute_copy(&buffer) };
    if ptr.is_null() || len == 0 {
        ffi::free_buffer(buffer);
        return Err(BamlError::Runtime(anyhow!(format!(
            "object method {method_name} returned empty response"
        ))));
    }

    let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) }.to_vec();
    ffi::free_buffer(buffer);

    let response = CffiObjectResponse::decode(bytes.as_slice()).map_err(|err| {
        BamlError::Deserialization(format!(
            "Failed to decode object response for {method_name}: {err}"
        ))
    })?;

    decode_object_response(response)
}

fn decode_object_response(response: CffiObjectResponse) -> BamlResult<ObjectResponse> {
    match response.response {
        Some(CffiObjectResponseVariant::Success(success)) => match success.result {
            Some(CffiObjectResponseSuccess::Object(object)) => Ok(ObjectResponse::Object(object)),
            Some(CffiObjectResponseSuccess::Objects(objects)) => {
                Ok(ObjectResponse::Objects(objects.objects))
            }
            Some(CffiObjectResponseSuccess::Value(holder)) => {
                let encoded = holder.encode_to_vec();
                let value =
                    BamlValue::from_c_buffer(encoded.as_ptr() as *const c_char, encoded.len())
                        .map_err(|err| {
                            BamlError::Deserialization(format!(
                                "Failed to decode object value: {err}"
                            ))
                        })?;

                Ok(match value {
                    BamlValue::Null => ObjectResponse::Null,
                    other => ObjectResponse::Value(other),
                })
            }
            None => Err(BamlError::Deserialization(
                "Object response missing result".to_string(),
            )),
        },
        Some(CffiObjectResponseVariant::Error(error)) => {
            Err(BamlError::Runtime(anyhow!(error.error)))
        }
        None => Err(BamlError::Deserialization(
            "Object response missing payload".to_string(),
        )),
    }
}

fn ensure_variant(raw: &CffiRawObject, expected: &'static str) -> BamlResult<()> {
    let actual = raw
        .object
        .as_ref()
        .map(|variant| match variant {
            RawObjectVariant::Collector(_) => "Collector",
            RawObjectVariant::FunctionLog(_) => "FunctionLog",
            RawObjectVariant::Usage(_) => "Usage",
            RawObjectVariant::Timing(_) => "Timing",
            RawObjectVariant::StreamTiming(_) => "StreamTiming",
            RawObjectVariant::LlmCall(_) => "LLMCall",
            RawObjectVariant::LlmStreamCall(_) => "LLMStreamCall",
            RawObjectVariant::HttpRequest(_) => "HTTPRequest",
            RawObjectVariant::HttpResponse(_) => "HTTPResponse",
            RawObjectVariant::HttpBody(_) => "HTTPBody",
            RawObjectVariant::SseResponse(_) => "SSEResponse",
            RawObjectVariant::MediaImage(_) => "MediaImage",
            RawObjectVariant::MediaAudio(_) => "MediaAudio",
            RawObjectVariant::MediaPdf(_) => "MediaPdf",
            RawObjectVariant::MediaVideo(_) => "MediaVideo",
            RawObjectVariant::TypeBuilder(_) => "TypeBuilder",
            RawObjectVariant::Type(_) => "Type",
            RawObjectVariant::EnumBuilder(_) => "EnumBuilder",
            RawObjectVariant::EnumValueBuilder(_) => "EnumValueBuilder",
            RawObjectVariant::ClassBuilder(_) => "ClassBuilder",
            RawObjectVariant::ClassPropertyBuilder(_) => "ClassPropertyBuilder",
        })
        .unwrap_or("unknown");

    if actual != expected {
        return Err(BamlError::Deserialization(format!(
            "Expected {expected} object, got {actual}"
        )));
    }
    Ok(())
}

fn expect_object(
    response: ObjectResponse,
    runtime: RuntimeHandleArc,
    expected: &'static str,
) -> BamlResult<Arc<ObjectInner>> {
    let raw = match response {
        ObjectResponse::Object(obj) => obj,
        ObjectResponse::Null => {
            return Err(BamlError::Deserialization(format!(
                "Expected {expected} object, got null"
            )))
        }
        other => {
            return Err(BamlError::Deserialization(format!(
                "Expected {expected} object, got {other:?}"
            )))
        }
    };

    ensure_variant(&raw, expected)?;
    Ok(Arc::new(ObjectInner::new(runtime, raw)))
}

fn expect_optional_object(
    response: ObjectResponse,
    runtime: RuntimeHandleArc,
    expected: &'static str,
) -> BamlResult<Option<Arc<ObjectInner>>> {
    match response {
        ObjectResponse::Null => Ok(None),
        ObjectResponse::Object(raw) => {
            ensure_variant(&raw, expected)?;
            Ok(Some(Arc::new(ObjectInner::new(runtime, raw))))
        }
        other => Err(BamlError::Deserialization(format!(
            "Expected optional {expected} object, got {other:?}"
        ))),
    }
}

fn expect_objects(
    response: ObjectResponse,
    runtime: RuntimeHandleArc,
    expected: &'static str,
) -> BamlResult<Vec<Arc<ObjectInner>>> {
    match response {
        ObjectResponse::Objects(raw_objects) => raw_objects
            .into_iter()
            .map(|raw| {
                ensure_variant(&raw, expected)?;
                Ok(Arc::new(ObjectInner::new(runtime.clone(), raw)))
            })
            .collect(),
        ObjectResponse::Null => Ok(vec![]),
        other => Err(BamlError::Deserialization(format!(
            "Expected list of {expected} objects, got {other:?}"
        ))),
    }
}

fn expect_value(response: ObjectResponse, method: &str) -> BamlResult<BamlValue> {
    match response {
        ObjectResponse::Value(value) => Ok(value),
        ObjectResponse::Null => Ok(BamlValue::Null),
        other => Err(BamlError::Deserialization(format!(
            "Expected value result from {method}, got {other:?}"
        ))),
    }
}

fn expect_bool(response: ObjectResponse, method: &str) -> BamlResult<bool> {
    let value = expect_value(response, method)?;
    bool::from_baml_value(value)
}

fn expect_int(response: ObjectResponse, method: &str) -> BamlResult<i64> {
    let value = expect_value(response, method)?;
    i64::from_baml_value(value)
}

fn expect_string(response: ObjectResponse, method: &str) -> BamlResult<String> {
    let value = expect_value(response, method)?;
    String::from_baml_value(value)
}

fn expect_optional_string(response: ObjectResponse, method: &str) -> BamlResult<Option<String>> {
    match response {
        ObjectResponse::Null => Ok(None),
        ObjectResponse::Value(value) => match value {
            BamlValue::Null => Ok(None),
            _ => String::from_baml_value(value).map(Some),
        },
        other => Err(BamlError::Deserialization(format!(
            "Expected optional string from {method}, got {other:?}"
        ))),
    }
}

fn expect_map_string_string(
    response: ObjectResponse,
    method: &str,
) -> BamlResult<HashMap<String, String>> {
    let value = expect_value(response, method)?;
    match value {
        BamlValue::Map(map) => map
            .into_iter()
            .map(|(k, v)| String::from_baml_value(v).map(|v_str| (k, v_str)))
            .collect(),
        BamlValue::Class(_, map) => map
            .into_iter()
            .map(|(k, v)| String::from_baml_value(v).map(|v_str| (k, v_str)))
            .collect(),
        BamlValue::Null => Ok(HashMap::new()),
        other => Err(BamlError::Deserialization(format!(
            "Expected map<string, string> from {method}, got {other:?}"
        ))),
    }
}

pub(crate) fn string_arg<K: Into<String>, V: Into<String>>(key: K, value: V) -> CffiMapEntry {
    CffiMapEntry {
        key: key.into(),
        value: Some(baml_cffi::baml::cffi::CffiValueHolder {
            value: Some(baml_cffi::baml::cffi::cffi_value_holder::Value::StringValue(value.into())),
            r#type: None,
        }),
    }
}

// Usage ---------------------------------------------------------------------

impl Usage {
    pub fn input_tokens(&self) -> BamlResult<i64> {
        expect_int(self.inner.call("input_tokens", Vec::new())?, "input_tokens")
    }

    pub fn output_tokens(&self) -> BamlResult<i64> {
        expect_int(
            self.inner.call("output_tokens", Vec::new())?,
            "output_tokens",
        )
    }
}

// Timing --------------------------------------------------------------------

impl Timing {
    pub fn start_time_utc_ms(&self) -> BamlResult<i64> {
        expect_int(
            self.inner.call("start_time_utc_ms", Vec::new())?,
            "start_time_utc_ms",
        )
    }

    pub fn duration_ms(&self) -> BamlResult<Option<i64>> {
        let value = expect_value(self.inner.call("duration_ms", Vec::new())?, "duration_ms")?;
        match value {
            BamlValue::Null => Ok(None),
            other => i64::from_baml_value(other).map(Some),
        }
    }
}

impl StreamTiming {
    pub fn start_time_utc_ms(&self) -> BamlResult<i64> {
        expect_int(
            self.inner.call("start_time_utc_ms", Vec::new())?,
            "stream_start_time_utc_ms",
        )
    }

    pub fn duration_ms(&self) -> BamlResult<i64> {
        expect_int(
            self.inner.call("duration_ms", Vec::new())?,
            "stream_duration_ms",
        )
    }
}

// HTTP Body -----------------------------------------------------------------

impl HttpBody {
    pub fn text(&self) -> BamlResult<String> {
        expect_string(self.inner.call("text", Vec::new())?, "http_body_text")
    }

    pub fn json(&self) -> BamlResult<Option<BamlValue>> {
        match self.inner.call("json", Vec::new())? {
            ObjectResponse::Value(value) => Ok(Some(value)),
            ObjectResponse::Null => Ok(None),
            other => Err(BamlError::Deserialization(format!(
                "Expected value for HTTPBody::json, got {other:?}"
            ))),
        }
    }
}

// HTTP Request ---------------------------------------------------------------

impl HttpRequest {
    pub fn request_id(&self) -> BamlResult<String> {
        expect_string(self.inner.call("request_id", Vec::new())?, "request_id")
    }

    pub fn url(&self) -> BamlResult<String> {
        expect_string(self.inner.call("url", Vec::new())?, "url")
    }

    pub fn method(&self) -> BamlResult<String> {
        expect_string(self.inner.call("method", Vec::new())?, "method")
    }

    pub fn headers(&self) -> BamlResult<HashMap<String, String>> {
        expect_map_string_string(self.inner.call("headers", Vec::new())?, "headers")
    }

    pub fn body(&self) -> BamlResult<HttpBody> {
        let runtime = self.inner.runtime.clone();
        let response = self.inner.call("body", Vec::new())?;
        let inner = expect_object(response, runtime.clone(), "HTTPBody")?;
        Ok(HttpBody { inner })
    }
}

// HTTP Response --------------------------------------------------------------

impl HttpResponse {
    pub fn request_id(&self) -> BamlResult<String> {
        expect_string(
            self.inner.call("request_id", Vec::new())?,
            "response_request_id",
        )
    }

    pub fn status(&self) -> BamlResult<i64> {
        expect_int(self.inner.call("status", Vec::new())?, "status")
    }

    pub fn headers(&self) -> BamlResult<HashMap<String, String>> {
        expect_map_string_string(self.inner.call("headers", Vec::new())?, "response_headers")
    }

    pub fn body(&self) -> BamlResult<HttpBody> {
        let runtime = self.inner.runtime.clone();
        let response = self.inner.call("body", Vec::new())?;
        let inner = expect_object(response, runtime.clone(), "HTTPBody")?;
        Ok(HttpBody { inner })
    }
}

// SSE Response ---------------------------------------------------------------

impl SseResponse {
    pub fn text(&self) -> BamlResult<String> {
        expect_string(self.inner.call("text", Vec::new())?, "sse_text")
    }

    pub fn json(&self) -> BamlResult<Option<BamlValue>> {
        match self.inner.call("json", Vec::new())? {
            ObjectResponse::Null => Ok(None),
            ObjectResponse::Value(value) => Ok(Some(value)),
            other => Err(BamlError::Deserialization(format!(
                "Expected value for SSEResponse::json, got {other:?}"
            ))),
        }
    }
}

// LLM Call ------------------------------------------------------------------

impl LlmCall {
    pub fn request_id(&self) -> BamlResult<String> {
        expect_string(self.inner.call("request_id", Vec::new())?, "request_id")
    }

    pub fn client_name(&self) -> BamlResult<String> {
        expect_string(self.inner.call("client_name", Vec::new())?, "client_name")
    }

    pub fn provider(&self) -> BamlResult<String> {
        expect_string(self.inner.call("provider", Vec::new())?, "provider")
    }

    pub fn http_request(&self) -> BamlResult<HttpRequest> {
        let runtime = self.inner.runtime.clone();
        let response = self.inner.call("http_request", Vec::new())?;
        let inner = expect_object(response, runtime.clone(), "HTTPRequest")?;
        Ok(HttpRequest { inner })
    }

    pub fn http_response(&self) -> BamlResult<Option<HttpResponse>> {
        let runtime = self.inner.runtime.clone();
        let response = self.inner.call("http_response", Vec::new())?;
        let maybe_inner = expect_optional_object(response, runtime.clone(), "HTTPResponse")?;
        Ok(maybe_inner.map(|inner| HttpResponse { inner }))
    }

    pub fn usage(&self) -> BamlResult<Option<Usage>> {
        let runtime = self.inner.runtime.clone();
        let response = self.inner.call("usage", Vec::new())?;
        let maybe_inner = expect_optional_object(response, runtime.clone(), "Usage")?;
        Ok(maybe_inner.map(|inner| Usage { inner }))
    }

    pub fn selected(&self) -> BamlResult<bool> {
        expect_bool(self.inner.call("selected", Vec::new())?, "selected")
    }

    pub fn timing(&self) -> BamlResult<Timing> {
        let runtime = self.inner.runtime.clone();
        let response = self.inner.call("timing", Vec::new())?;
        let inner = expect_object(response, runtime.clone(), "Timing")?;
        Ok(Timing { inner })
    }
}

impl LlmStreamCall {
    pub fn base(&self) -> &LlmCall {
        &self.call
    }

    pub fn sse_chunks(&self) -> BamlResult<Vec<SseResponse>> {
        let runtime = self.inner.runtime.clone();
        let response = self.inner.call("sse_chunks", Vec::new())?;
        let inners = expect_objects(response, runtime.clone(), "SSEResponse")?;
        Ok(inners
            .into_iter()
            .map(|inner| SseResponse { inner })
            .collect())
    }
}

// Function Log ---------------------------------------------------------------

impl FunctionLog {
    pub fn id(&self) -> BamlResult<String> {
        expect_string(self.inner.call("id", Vec::new())?, "function_log_id")
    }

    pub fn function_name(&self) -> BamlResult<String> {
        expect_string(
            self.inner.call("function_name", Vec::new())?,
            "function_name",
        )
    }

    pub fn log_type(&self) -> BamlResult<String> {
        expect_string(self.inner.call("log_type", Vec::new())?, "log_type")
    }

    pub fn timing(&self) -> BamlResult<Timing> {
        let runtime = self.inner.runtime.clone();
        let response = self.inner.call("timing", Vec::new())?;
        let inner = expect_object(response, runtime.clone(), "Timing")?;
        Ok(Timing { inner })
    }

    pub fn usage(&self) -> BamlResult<Usage> {
        let runtime = self.inner.runtime.clone();
        let response = self.inner.call("usage", Vec::new())?;
        let inner = expect_object(response, runtime.clone(), "Usage")?;
        Ok(Usage { inner })
    }

    pub fn raw_llm_response(&self) -> BamlResult<Option<String>> {
        expect_optional_string(
            self.inner.call("raw_llm_response", Vec::new())?,
            "raw_llm_response",
        )
    }

    pub fn tags(&self) -> BamlResult<BamlMap<String, BamlValue>> {
        let value = expect_value(self.inner.call("tags", Vec::new())?, "tags")?;
        match value {
            BamlValue::Map(map) => Ok(map),
            BamlValue::Class(_, map) => Ok(map),
            BamlValue::Null => Ok(BamlMap::new()),
            other => Err(BamlError::Deserialization(format!(
                "Expected map for FunctionLog::tags, got {other:?}"
            ))),
        }
    }

    pub fn calls(&self) -> BamlResult<Vec<LlmCallKind>> {
        let runtime = self.inner.runtime.clone();
        let response = self.inner.call("calls", Vec::new())?;
        let inners = expect_objects(response, runtime.clone(), "LLMCall")?;

        inners
            .into_iter()
            .map(|inner| match inner.raw.object {
                Some(RawObjectVariant::LlmCall(_)) => Ok(LlmCallKind::Basic(LlmCall { inner })),
                Some(RawObjectVariant::LlmStreamCall(_)) => {
                    let stream_inner = inner.clone();
                    let base = LlmCall {
                        inner: inner.clone(),
                    };
                    Ok(LlmCallKind::Stream(LlmStreamCall {
                        call: base,
                        inner: stream_inner,
                    }))
                }
                _ => Err(BamlError::Deserialization(
                    "Unexpected call type in FunctionLog::calls".to_string(),
                )),
            })
            .collect()
    }

    pub fn selected_call(&self) -> BamlResult<Option<LlmCallKind>> {
        let runtime = self.inner.runtime.clone();
        let response = self.inner.call("selected_call", Vec::new())?;
        let maybe_inner = expect_optional_object(response, runtime.clone(), "LLMCall")?;
        Ok(maybe_inner.map(|inner| match inner.raw.object {
            Some(RawObjectVariant::LlmCall(_)) => LlmCallKind::Basic(LlmCall { inner }),
            Some(RawObjectVariant::LlmStreamCall(_)) => {
                let base = LlmCall {
                    inner: inner.clone(),
                };
                LlmCallKind::Stream(LlmStreamCall { call: base, inner })
            }
            _ => unreachable!("Variant checked in expect_optional_object"),
        }))
    }
}

// Public constructors --------------------------------------------------------

pub(crate) fn usage_from_response(
    runtime: RuntimeHandleArc,
    response: ObjectResponse,
) -> BamlResult<Usage> {
    let inner = expect_object(response, runtime.clone(), "Usage")?;
    Ok(Usage { inner })
}

pub(crate) fn function_logs_from_response(
    runtime: RuntimeHandleArc,
    response: ObjectResponse,
) -> BamlResult<Vec<FunctionLog>> {
    let inners = expect_objects(response, runtime.clone(), "FunctionLog")?;
    Ok(inners
        .into_iter()
        .map(|inner| FunctionLog { inner })
        .collect())
}

pub(crate) fn optional_function_log_from_response(
    runtime: RuntimeHandleArc,
    response: ObjectResponse,
) -> BamlResult<Option<FunctionLog>> {
    let maybe_inner = expect_optional_object(response, runtime.clone(), "FunctionLog")?;
    Ok(maybe_inner.map(|inner| FunctionLog { inner }))
}

pub(crate) fn function_log_from_response(
    runtime: RuntimeHandleArc,
    response: ObjectResponse,
) -> BamlResult<FunctionLog> {
    let inner = expect_object(response, runtime.clone(), "FunctionLog")?;
    Ok(FunctionLog { inner })
}

pub(crate) fn clear_count_from_response(response: ObjectResponse) -> BamlResult<i64> {
    expect_int(response, "clear")
}

pub(crate) fn name_from_response(response: ObjectResponse) -> BamlResult<String> {
    expect_string(response, "name")
}

pub(crate) fn collector_usage_value_from_response(
    runtime: RuntimeHandleArc,
    response: ObjectResponse,
) -> BamlResult<Usage> {
    usage_from_response(runtime, response)
}
