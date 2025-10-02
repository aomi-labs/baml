#[cfg(test)]
mod tests {
    use anyhow::Result;
    use baml_client::{BamlClient, Person, StreamState};
    use futures::StreamExt;

    #[tokio::test]
    async fn person_test_returns_adult() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.person_test().await?;

        assert!(result.age > 0, "expected positive age, got {}", result.age);
        assert!(
            !result.name.trim().is_empty(),
            "expected non-empty name, got {:?}",
            result.name
        );
        Ok(())
    }

    #[tokio::test]
    async fn person_test_stream_emits_partial_and_final() -> Result<()> {
        let client = BamlClient::new()?;
        let mut stream = client.person_test_stream().await?;

        let mut got_final = false;
        let mut partial_count = 0usize;
        let mut partials = Vec::<Person>::new();

        while let Some(chunk) = stream.next().await {
            match chunk? {
                StreamState::Partial(value) => {
                    partial_count += 1;
                    partials.push(value);
                }
                StreamState::Final(value) => {
                    assert!(value.age > 0, "expected positive age, got {}", value.age);
                    assert!(
                        !value.name.trim().is_empty(),
                        "expected non-empty name, got {:?}",
                        value.name
                    );
                    got_final = true;
                }
            }
        }

        assert!(got_final, "expected to receive a final result");
        assert_eq!(partial_count, partials.len());
        Ok(())
    }
}
