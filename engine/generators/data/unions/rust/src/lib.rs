#[cfg(test)]
mod tests {
    use anyhow::Result;
    use baml_client::{
        BamlClient, ExistingSystemComponent, StreamState, Union2KResourceOrKService,
    };
    use futures::{pin_mut, StreamExt};

    fn sample_component() -> ExistingSystemComponent {
        ExistingSystemComponent::new(
            1,
            "Example Service".to_string(),
            "service".to_string(),
            Union2KResourceOrKService::k_service(),
            "An example component used in tests".to_string(),
        )
    }

    #[tokio::test]
    async fn json_input_returns_summary() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.json_input(vec![sample_component()]).await?;
        assert!(!result.is_empty(), "expected non-empty summaries");
        for summary in result {
            assert!(!summary.trim().is_empty(), "expected summary content");
        }
        Ok(())
    }

    #[tokio::test]
    async fn json_input_stream_produces_final_result() -> Result<()> {
        let client = BamlClient::new()?;
        let stream = client.json_input_stream(vec![sample_component()]).await?;
        pin_mut!(stream);

        let mut saw_partial = false;
        let mut final_value: Option<Vec<String>> = None;

        while let Some(item) = stream.next().await {
            match item? {
                StreamState::Partial(values) => {
                    saw_partial = true;
                    assert!(values.len() >= 0);
                }
                StreamState::Final(values) => {
                    final_value = Some(values);
                }
            }
        }

        assert!(saw_partial, "expected at least one partial update");
        let final_values = final_value.expect("expected final stream result");
        assert!(
            !final_values.is_empty(),
            "expected summaries in final result"
        );
        for summary in final_values {
            assert!(!summary.trim().is_empty(), "expected summary content");
        }
        Ok(())
    }
}
