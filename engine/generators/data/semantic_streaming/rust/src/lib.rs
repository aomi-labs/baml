#[cfg(test)]
mod tests {
    use anyhow::Result;
    use baml_client::{BamlClient, SemanticContainer, StreamState};
    use futures::{pin_mut, StreamExt};

    fn assert_semantic_container(container: &SemanticContainer) {
        assert!(
            container.sixteen_digit_number != 0,
            "expected 16 digit number to be present"
        );
        assert!(
            !container.string_with_twenty_words.trim().is_empty(),
            "expected 20-word string"
        );
        assert!(
            !container.final_string.trim().is_empty(),
            "expected final string"
        );
        assert!(
            !container.three_small_things.is_empty(),
            "expected small things"
        );
        for thing in &container.three_small_things {
            assert!(thing.i_16_digits != 0, "small thing lacks digits");
        }
    }

    #[tokio::test]
    async fn make_semantic_container_produces_data() -> Result<()> {
        let client = BamlClient::new()?;
        let result: SemanticContainer = client.make_semantic_container().await?;
        assert_semantic_container(&result);
        Ok(())
    }

    #[tokio::test]
    async fn make_semantic_container_stream_yields_consistent_partials() -> Result<()> {
        let client = BamlClient::new()?;
        let stream = client.make_semantic_container_stream().await?;
        pin_mut!(stream);

        let mut saw_partial = false;
        let mut final_value: Option<SemanticContainer> = None;
        let mut observed_number: Option<i64> = None;
        let mut observed_string: Option<String> = None;

        while let Some(item) = stream.next().await {
            match item? {
                StreamState::Partial(value) => {
                    saw_partial = true;
                    if value.sixteen_digit_number != 0 {
                        if let Some(expected) = observed_number {
                            assert_eq!(expected, value.sixteen_digit_number);
                        } else {
                            observed_number = Some(value.sixteen_digit_number);
                        }
                    }
                    if !value.string_with_twenty_words.trim().is_empty() {
                        if let Some(expected) = &observed_string {
                            assert_eq!(expected, &value.string_with_twenty_words);
                        } else {
                            observed_string = Some(value.string_with_twenty_words.clone());
                        }
                    }
                }
                StreamState::Final(value) => {
                    final_value = Some(value);
                }
            }
        }

        assert!(saw_partial, "expected at least one partial update");
        let final_value = final_value.expect("expected final semantic container");
        assert_semantic_container(&final_value);

        if let Some(expected) = observed_number {
            assert_eq!(expected, final_value.sixteen_digit_number);
        }
        if let Some(expected) = observed_string {
            assert_eq!(expected, final_value.string_with_twenty_words);
        }
        Ok(())
    }
}
