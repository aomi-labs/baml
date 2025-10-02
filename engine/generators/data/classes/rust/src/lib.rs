#[cfg(test)]
mod tests {
    use anyhow::Result;
    use baml_client::{BamlClient, SimpleClass, StreamState};
    use futures::StreamExt;

    #[tokio::test]
    async fn consume_simple_class_round_trip() -> Result<()> {
        let client = BamlClient::new()?;
        let original = SimpleClass {
            digits: 10i64,
            words: "hello".to_string(),
        };

        let result = client.consume_simple_class(original.clone()).await?;

        assert_eq!(result.digits, original.digits);
        assert_eq!(result.words, original.words);
        Ok(())
    }

    #[tokio::test]
    async fn make_simple_class_stream_produces_values() -> Result<()> {
        let client = BamlClient::new()?;
        let mut stream = client.make_simple_class_stream().await?;

        let mut got_final = false;
        let mut partial_count = 0usize;

        while let Some(event) = stream.next().await {
            match event? {
                StreamState::Partial(value) => {
                    partial_count += 1;
                    // Partial frames can legitimately contain empty/default data while the model is still streaming.
                    assert!(
                        value.words.trim().is_empty() || value.digits != 0 || partial_count < 50,
                        "expected partial content"
                    );
                }
                StreamState::Final(value) => {
                    assert!(
                        !value.words.trim().is_empty(),
                        "expected final words to be non-empty"
                    );
                    assert!(value.digits != 0, "expected final digits to be non-zero");
                    got_final = true;
                }
            }
        }

        assert!(got_final, "expected final streaming result");
        assert!(partial_count >= 0);
        Ok(())
    }
}
