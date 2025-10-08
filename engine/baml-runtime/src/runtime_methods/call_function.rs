use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use baml_types::{
    BamlMap, BamlValue, Constraint, EvaluationContext,
    tracing::events::{FunctionEnd, FunctionStart, TraceData, TraceEvent},
};
use internal_baml_core::{
    internal_baml_diagnostics::SourceFile,
    ir::{
        ArgCoercer, ExprFunctionWalker, FunctionWalker, IRHelper, TestCase,
        repr::{IntermediateRepr, Node, TypeBuilderEntry},
    },
    validate,
};
use internal_baml_jinja::RenderedPrompt;
use internal_llm_client::{AllowedRoleMetadata, ClientSpec};

use super::prepare_function::PreparedFunction;
use crate::{
    FunctionResult, FunctionResultStream, InternalRuntimeInterface, RenderCurlSettings,
    RuntimeContext, RuntimeContextManager, TripWire,
    client_registry::ClientProperty,
    internal::{
        ir_features::{IrFeatures, WithInternal},
        llm_client::{
            LLMResponse,
            llm_provider::LLMProvider,
            orchestrator::{
                IterOrchestrator, OrchestrationScope, OrchestratorNode, orchestrate_call,
            },
            primitive::LLMPrimitiveProvider,
            retry_policy::CallablePolicy,
            traits::{WithClientProperties, WithPrompt, WithRenderRawCurl},
        },
        prompt_renderer::PromptRenderer,
    },
    runtime::InternalBamlRuntime,
    runtime_interface::{InternalClientLookup, RuntimeConstructor},
    tracing::BamlTracer,
    tracingv2::storage::storage::{BAML_TRACER, Collector},
    type_builder::TypeBuilder,
};

impl InternalBamlRuntime {
    pub(crate) async fn call_function_impl<'ir>(
        &'ir self,
        prepared_func_call: PreparedFunction<'ir>,
        ctx: RuntimeContext,
        cancel_tripwire: Arc<TripWire>,
    ) -> Result<crate::FunctionResult> {
        let future = async {
            let renderer =
                PromptRenderer::from_function(&prepared_func_call.func, self.ir(), &ctx)?;
            let orchestrator = self.orchestration_graph(renderer.client_spec(), &ctx)?;

            let baml_args = BamlValue::Map(prepared_func_call.baml_args.value);

            // Now actually execute the code.
            let (history, _) = orchestrate_call(
                orchestrator,
                self.ir(),
                &ctx,
                &renderer,
                &baml_args,
                |s| renderer.parse(self.ir(), &ctx, s, false),
                cancel_tripwire.trip_wire(),
            )
            .await;

            FunctionResult::new_chain(history)
        };

        future.await
    }
}
