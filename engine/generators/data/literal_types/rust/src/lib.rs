#[cfg(test)]
mod tests {
    use anyhow::Result;
    use baml_client::{BamlClient, ComplexLiterals, MixedLiterals, StreamState, StringLiterals};
    use futures::{pin_mut, StreamExt};

    #[tokio::test]
    async fn string_literals_match_expected_values() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_string_literals("test string literals").await?;

        assert!(
            result.status.is_k_active(),
            "expected status to be active, got {:?}",
            result.status
        );
        assert!(
            result.environment.is_k_prod(),
            "expected environment to be prod, got {:?}",
            result.environment
        );
        assert!(
            result.method.is_k_post(),
            "expected method to be POST, got {:?}",
            result.method
        );
        Ok(())
    }

    #[tokio::test]
    async fn integer_literals_match_expected_values() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client
            .test_integer_literals("test integer literals")
            .await?;

        assert!(
            result.priority.is_intk3(),
            "expected priority 3, got {:?}",
            result.priority
        );
        assert!(
            result.http_status.is_intk201(),
            "expected http status 201, got {:?}",
            result.http_status
        );
        assert!(
            result.max_retries.is_intk3(),
            "expected max retries 3, got {:?}",
            result.max_retries
        );
        Ok(())
    }

    #[tokio::test]
    async fn boolean_literals_match_expected_values() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client
            .test_boolean_literals("test boolean literals")
            .await?;

        assert!(result.always_true);
        assert!(!result.always_false);
        assert!(
            result.either_bool.is_boolk_true(),
            "expected either_bool to be true, got {:?}",
            result.either_bool
        );
        Ok(())
    }

    #[tokio::test]
    async fn mixed_literals_match_expected_values() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_mixed_literals("test mixed literals").await?;

        assert_eq!(result.id, 12_345);
        assert!(
            result.r#type.is_k_admin(),
            "expected type admin, got {:?}",
            result.r#type
        );
        assert!(
            result.level.is_intk2(),
            "expected level 2, got {:?}",
            result.level
        );
        assert!(
            result.is_active.is_boolk_true(),
            "expected is_active to be true, got {:?}",
            result.is_active
        );
        assert!(
            result.api_version.is_kv2(),
            "expected api_version v2, got {:?}",
            result.api_version
        );
        Ok(())
    }

    #[tokio::test]
    async fn complex_literals_match_expected_values() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client
            .test_complex_literals("test complex literals")
            .await?;

        assert!(
            result.state.is_k_published(),
            "expected state to be published, got {:?}",
            result.state
        );
        assert!(
            result.retry_count.is_intk5(),
            "expected retry_count 5, got {:?}",
            result.retry_count
        );
        assert!(
            result.response.is_k_success(),
            "expected response success, got {:?}",
            result.response
        );
        assert_eq!(result.flags.len(), 3);
        assert_eq!(result.codes.len(), 3);
        Ok(())
    }

    async fn collect_stream<T>(
        stream: impl futures::Stream<Item = baml_client::BamlResult<StreamState<T>>>,
    ) -> Result<(Vec<T>, Option<T>)> {
        let mut partials = Vec::new();
        let mut final_value = None;
        pin_mut!(stream);
        while let Some(item) = stream.next().await {
            match item? {
                StreamState::Partial(value) => partials.push(value),
                StreamState::Final(value) => final_value = Some(value),
            }
        }
        Ok((partials, final_value))
    }

    #[tokio::test]
    async fn string_literals_stream_yields_final_value() -> Result<()> {
        let client = BamlClient::new()?;
        let stream = client
            .test_string_literals_stream("test string literals stream")
            .await?;

        let (_partials, final_value) = collect_stream(stream).await?;
        assert!(final_value.is_some(), "expected a final result");

        if let Some(final_result) = final_value {
            assert!(
                final_result.status.is_k_active(),
                "expected status to be active, got {:?}",
                final_result.status
            );
            assert!(
                final_result.environment.is_k_prod(),
                "expected environment to be prod, got {:?}",
                final_result.environment
            );
            assert!(
                final_result.method.is_k_post(),
                "expected method to be POST, got {:?}",
                final_result.method
            );
        }
        Ok(())
    }

    #[tokio::test]
    async fn mixed_literals_stream_validates_final_payload() -> Result<()> {
        let client = BamlClient::new()?;
        let stream = client
            .test_mixed_literals_stream("test mixed literals stream")
            .await?;

        let (_partials, final_value) = collect_stream(stream).await?;
        let final_value: MixedLiterals = final_value.expect("expected final mixed literals result");

        assert_eq!(final_value.id, 12_345);
        assert!(
            final_value.r#type.is_k_admin(),
            "expected type admin, got {:?}",
            final_value.r#type
        );
        assert!(
            final_value.level.is_intk2(),
            "expected level 2, got {:?}",
            final_value.level
        );
        assert!(
            final_value.is_active.is_boolk_true(),
            "expected is_active to be true, got {:?}",
            final_value.is_active
        );
        assert!(
            final_value.api_version.is_kv2(),
            "expected api_version v2, got {:?}",
            final_value.api_version
        );
        Ok(())
    }

    #[tokio::test]
    async fn complex_literals_stream_validates_final_payload() -> Result<()> {
        let client = BamlClient::new()?;
        let stream = client
            .test_complex_literals_stream("test complex literals stream")
            .await?;

        let (_partials, final_value) = collect_stream(stream).await?;
        let final_value: ComplexLiterals =
            final_value.expect("expected final complex literals result");

        assert!(
            final_value.state.is_k_published(),
            "expected state to be published, got {:?}",
            final_value.state
        );
        assert!(
            final_value.retry_count.is_intk5(),
            "expected retry_count 5, got {:?}",
            final_value.retry_count
        );
        assert!(
            final_value.response.is_k_success(),
            "expected response success, got {:?}",
            final_value.response
        );
        assert_eq!(final_value.flags.len(), 3);
        assert_eq!(final_value.codes.len(), 3);
        Ok(())
    }
}
