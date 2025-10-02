use crate::result::FunctionResult;
use crate::BamlResult;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::task::{Context, Poll};

/// State of a streaming function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamState<T> {
    /// Partial result received during streaming
    Partial(T),
    /// Final result received, streaming complete
    Final(T),
}

impl<T> StreamState<T> {
    /// Check if this is a partial result
    pub fn is_partial(&self) -> bool {
        matches!(self, StreamState::Partial(_))
    }

    /// Check if this is the final result
    pub fn is_final(&self) -> bool {
        matches!(self, StreamState::Final(_))
    }

    /// Get the inner value regardless of state
    pub fn into_inner(self) -> T {
        match self {
            StreamState::Partial(value) | StreamState::Final(value) => value,
        }
    }

    /// Get a reference to the inner value
    pub fn inner(&self) -> &T {
        match self {
            StreamState::Partial(value) | StreamState::Final(value) => value,
        }
    }

    /// Map the inner value to a different type
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> StreamState<U> {
        match self {
            StreamState::Partial(value) => StreamState::Partial(f(value)),
            StreamState::Final(value) => StreamState::Final(f(value)),
        }
    }

    /// Try to map the inner value, propagating errors
    pub fn try_map<U, E>(self, f: impl FnOnce(T) -> Result<U, E>) -> Result<StreamState<U>, E> {
        match self {
            StreamState::Partial(value) => Ok(StreamState::Partial(f(value)?)),
            StreamState::Final(value) => Ok(StreamState::Final(f(value)?)),
        }
    }
}

/// A stream of function results
pub struct FunctionResultStream {
    inner: Pin<Box<dyn Stream<Item = BamlResult<StreamState<FunctionResult>>> + Send + Sync>>,
}

impl FunctionResultStream {
    /// Create a new function result stream
    pub fn new(
        inner: impl Stream<Item = BamlResult<StreamState<FunctionResult>>> + Send + Sync + 'static,
    ) -> Self {
        Self {
            inner: Box::pin(inner),
        }
    }

    /// Map the stream results to a different type
    pub fn map<T, F>(self, mut f: F) -> impl Stream<Item = BamlResult<StreamState<T>>>
    where
        F: FnMut(FunctionResult) -> BamlResult<T> + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        futures::stream::StreamExt::map(self, move |result| match result {
            Ok(stream_state) => match stream_state.try_map(&mut f) {
                Ok(mapped_state) => Ok(mapped_state),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        })
    }

    /// Try to map the stream results, flattening errors
    pub fn try_map<T, F>(self, f: F) -> impl Stream<Item = BamlResult<StreamState<T>>>
    where
        F: FnMut(FunctionResult) -> BamlResult<T> + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        self.map(f)
    }

    /// Filter out partial results, only yielding final results
    pub fn finals_only(self) -> impl Stream<Item = BamlResult<FunctionResult>> {
        futures::stream::StreamExt::filter_map(self, |result| async move {
            match result {
                Ok(StreamState::Final(value)) => Some(Ok(value)),
                Ok(StreamState::Partial(_)) => None,
                Err(e) => Some(Err(e)),
            }
        })
    }

    /// Collect all partial and final results
    pub async fn collect_all(self) -> BamlResult<Vec<StreamState<FunctionResult>>> {
        use futures::StreamExt;

        let results: Vec<_> = self.collect().await;
        // Convert Vec<BamlResult<StreamState<FunctionResult>>> to BamlResult<Vec<StreamState<FunctionResult>>>
        let mut stream_states = Vec::new();
        for result in results {
            stream_states.push(result?);
        }
        Ok(stream_states)
    }

    /// Get only the final result, ignoring partials
    pub async fn final_result(mut self) -> BamlResult<FunctionResult> {
        use futures::StreamExt;

        while let Some(result) = self.next().await {
            match result? {
                StreamState::Final(value) => return Ok(value),
                StreamState::Partial(_) => continue,
            }
        }

        Err(crate::BamlError::Stream(
            "Stream ended without final result".to_string(),
        ))
    }
}

impl Stream for FunctionResultStream {
    type Item = BamlResult<StreamState<FunctionResult>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

impl futures::stream::FusedStream for FunctionResultStream {
    fn is_terminated(&self) -> bool {
        false // We don't track termination state for now
    }
}
