#[cfg(test)]
mod tests {
    use anyhow::Result;
    use baml_client::{
        BamlClient, StreamState, Union5FloatOrIntOrListJSONOrMapStringKeyJSONValueOrString, JSON,
    };
    use futures::{pin_mut, StreamExt};

    #[tokio::test]
    async fn foo_returns_result() -> Result<()> {
        let client = BamlClient::new()?;
        let result: JSON = client.foo(8192).await?;
        assert!(result.is_some(), "expected foo to return JSON value");
        Ok(())
    }

    #[tokio::test]
    async fn json_input_accepts_union() -> Result<()> {
        let client = BamlClient::new()?;
        let payload = Some(
            Union5FloatOrIntOrListJSONOrMapStringKeyJSONValueOrString::string("Hello".to_string()),
        );

        let result = client.json_input(payload.clone()).await?;
        assert!(result.is_some(), "expected json_input to echo data");
        Ok(())
    }

    #[tokio::test]
    async fn foo_stream_emits_final_value() -> Result<()> {
        let client = BamlClient::new()?;
        let stream = client.foo_stream(8192).await?;
        pin_mut!(stream);

        let mut saw_partial = false;
        let mut got_final = false;

        while let Some(item) = stream.next().await {
            match item? {
                StreamState::Partial(value) => {
                    saw_partial = true;
                    if let Some(inner) = value {
                        assert!(
                            inner.is_string()
                                || inner.is_int()
                                || inner.is_float()
                                || inner.is_listjson()
                                || inner.is_map_string_keyjson_value(),
                            "unexpected partial variant"
                        );
                    }
                }
                StreamState::Final(value) => {
                    got_final = true;
                    assert!(value.is_some(), "expected final JSON payload");
                }
            }
        }

        assert!(saw_partial || got_final, "stream produced no events");
        assert!(got_final, "expected final event from foo_stream");
        Ok(())
    }
}
