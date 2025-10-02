#[cfg(test)]
mod tests {
    use anyhow::Result;
    use baml_client::{BamlClient, MixedOptionalNullable, NullableTypes, OptionalFields};

    fn approx_eq(left: f64, right: f64, tol: f64) {
        assert!(
            (left - right).abs() <= tol,
            "expected {left} to be within {tol} of {right}"
        );
    }

    #[tokio::test]
    async fn optional_fields_cover_present_and_missing_values() -> Result<()> {
        let client = BamlClient::new()?;
        let result: OptionalFields = client.test_optional_fields("test optional fields").await?;

        assert_eq!(result.required_string, "hello");
        assert_eq!(result.optional_string.as_deref(), Some("world"));
        assert_eq!(result.required_int, 42);
        assert!(result.optional_int.is_none());
        assert!(result.required_bool);
        assert_eq!(result.optional_bool, Some(false));
        let array = result
            .optional_array
            .as_ref()
            .expect("expected optional_array to be present");
        assert_eq!(array.len(), 3);
        assert!(result.optional_map.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn nullable_types_reflect_null_and_present_values() -> Result<()> {
        let client = BamlClient::new()?;
        let result: NullableTypes = client.test_nullable_types("test nullable types").await?;

        assert_eq!(result.nullable_string.as_deref(), Some("present"));
        assert!(result.nullable_int.is_none());
        approx_eq(
            result.nullable_float.expect("expected nullable_float"),
            3.14,
            0.01,
        );
        assert!(result.nullable_bool.is_none());
        let array = result
            .nullable_array
            .as_ref()
            .expect("expected nullable_array to be present");
        assert_eq!(array.len(), 2);
        assert!(result.nullable_object.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn mixed_optional_nullable_returns_primary_user() -> Result<()> {
        let client = BamlClient::new()?;
        let result: MixedOptionalNullable = client
            .test_mixed_optional_nullable("test mixed optional nullable")
            .await?;

        assert!(result.id > 0, "expected positive id");
        assert!(!result.tags.is_empty(), "expected tags to be populated");
        assert!(
            result.primary_user.id > 0,
            "expected primary user to have id"
        );
        assert!(
            !result.primary_user.name.trim().is_empty(),
            "expected primary user to have a name"
        );
        Ok(())
    }

    #[tokio::test]
    async fn all_null_returns_all_fields_as_none() -> Result<()> {
        let client = BamlClient::new()?;
        let result: NullableTypes = client.test_all_null("test all null").await?;

        assert!(result.nullable_string.is_none());
        assert!(result.nullable_int.is_none());
        assert!(result.nullable_float.is_none());
        assert!(result.nullable_bool.is_none());
        assert!(result.nullable_array.is_none());
        assert!(result.nullable_object.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn all_optional_omitted_leaves_required_values_only() -> Result<()> {
        let client = BamlClient::new()?;
        let result: OptionalFields = client
            .test_all_optional_omitted("test all optional omitted")
            .await?;

        assert!(
            !result.required_string.trim().is_empty(),
            "expected required string to be populated"
        );
        assert_ne!(
            result.required_int, 0,
            "expected required_int to be non-zero"
        );
        assert!(result.optional_string.is_none());
        assert!(result.optional_int.is_none());
        assert!(result.optional_bool.is_none());
        assert!(result.optional_array.is_none());
        assert!(result.optional_map.is_none());
        Ok(())
    }
}
