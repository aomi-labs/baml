#[cfg(test)]
mod tests {
    use anyhow::Result;
    use baml_client::{BamlClient, TestEnum};

    #[tokio::test]
    async fn consume_test_enum_returns_response() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.consume_test_enum(TestEnum::Confused).await?;

        assert!(
            !result.as_str().trim().is_empty(),
            "expected non-empty response when consuming enum"
        );
        Ok(())
    }

    #[tokio::test]
    async fn aliased_enum_output_matches_expected_variant() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.fn_test_aliased_enum_output("mehhhhh").await?;

        assert_eq!(result, TestEnum::Bored);
        Ok(())
    }

    #[tokio::test]
    async fn aliased_enum_output_variants_cover_expected_inputs() -> Result<()> {
        let client = BamlClient::new()?;
        let cases = vec![
            ("I am so angry right now", TestEnum::Angry),
            ("I'm feeling really happy", TestEnum::Happy),
            ("This makes me sad", TestEnum::Sad),
            ("I don't understand", TestEnum::Confused),
            ("I'm so excited!", TestEnum::Excited),
            ("k5", TestEnum::Excited),
            ("I'm bored and this is a long message", TestEnum::Bored),
        ];

        for (input, expected) in cases {
            let result = client.fn_test_aliased_enum_output(input).await?;
            assert_eq!(result, expected, "unexpected variant for input: {}", input);
        }
        Ok(())
    }
}
