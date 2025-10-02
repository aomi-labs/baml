#[cfg(test)]
mod tests {
    use anyhow::Result;
    use baml_client::{
        BamlClient, MixedPrimitives, NullValue, PrimitiveArrays, PrimitiveMaps, PrimitiveTypes,
    };

    fn approx_eq(left: f64, right: f64, tol: f64) {
        assert!(
            (left - right).abs() <= tol,
            "expected {left} to be within {tol} of {right}"
        );
    }

    #[tokio::test]
    async fn top_level_string_returns_expected_value() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_top_level_string("test string").await?;
        assert_eq!(result, "Hello from BAML!");
        Ok(())
    }

    #[tokio::test]
    async fn top_level_int_returns_expected_value() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_top_level_int("test int").await?;
        assert_eq!(result, 42);
        Ok(())
    }

    #[tokio::test]
    async fn top_level_float_is_close_to_pi() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_top_level_float("test float").await?;
        approx_eq(result, 3.14159, 0.01);
        Ok(())
    }

    #[tokio::test]
    async fn top_level_bool_is_true() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_top_level_bool("test bool").await?;
        assert!(result, "expected true, got false");
        Ok(())
    }

    #[tokio::test]
    async fn primitive_types_populate_all_fields() -> Result<()> {
        let client = BamlClient::new()?;
        let result: PrimitiveTypes = client.test_primitive_types("test input").await?;

        assert_eq!(result.string_field, "Hello, BAML!");
        assert_eq!(result.int_field, 42);
        approx_eq(result.float_field, 3.14159, 0.01);
        assert!(result.bool_field);
        assert_eq!(result.null_field, NullValue);
        Ok(())
    }

    #[tokio::test]
    async fn primitive_arrays_have_expected_lengths() -> Result<()> {
        let client = BamlClient::new()?;
        let result: PrimitiveArrays = client.test_primitive_arrays("test arrays").await?;

        assert_eq!(result.string_array.len(), 3);
        assert_eq!(result.int_array.len(), 5);
        assert_eq!(result.float_array.len(), 4);
        assert_eq!(result.bool_array.len(), 4);
        Ok(())
    }

    #[tokio::test]
    async fn primitive_maps_have_expected_lengths() -> Result<()> {
        let client = BamlClient::new()?;
        let result: PrimitiveMaps = client.test_primitive_maps("test maps").await?;

        assert_eq!(result.string_map.len(), 2);
        assert_eq!(result.int_map.len(), 3);
        assert_eq!(result.float_map.len(), 2);
        assert_eq!(result.bool_map.len(), 2);
        Ok(())
    }

    #[tokio::test]
    async fn mixed_primitives_have_basic_values() -> Result<()> {
        let client = BamlClient::new()?;
        let result: MixedPrimitives = client.test_mixed_primitives("test mixed").await?;

        assert!(
            !result.name.trim().is_empty(),
            "expected name to be non-empty"
        );
        assert!(result.age > 0, "expected positive age, got {}", result.age);
        Ok(())
    }

    #[tokio::test]
    async fn empty_collections_are_returned_for_empty_request() -> Result<()> {
        let client = BamlClient::new()?;
        let result: PrimitiveArrays = client.test_empty_collections("test empty").await?;

        assert!(result.string_array.is_empty());
        assert!(result.int_array.is_empty());
        assert!(result.float_array.is_empty());
        assert!(result.bool_array.is_empty());
        Ok(())
    }
}
