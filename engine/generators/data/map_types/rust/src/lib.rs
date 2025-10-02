#[cfg(test)]
mod tests {
    use anyhow::Result;
    use baml_client::{BamlClient, ComplexMaps, EdgeCaseMaps, NestedMaps, SimpleMaps};
    use serde_json::Value;

    #[tokio::test]
    async fn simple_maps_have_expected_entries() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_simple_maps("test simple maps").await?;

        assert_eq!(result.string_to_string.len(), 2);
        assert_eq!(
            result.string_to_string.get("key1"),
            Some(&"value1".to_string())
        );

        assert_eq!(result.string_to_int.len(), 3);
        assert_eq!(result.string_to_int.get("one"), Some(&1));

        assert_eq!(result.string_to_float.len(), 2);
        assert!(
            (result
                .string_to_float
                .get("pi")
                .copied()
                .unwrap_or_default()
                - 3.14159)
                .abs()
                < 0.001
        );

        assert_eq!(result.string_to_bool.len(), 2);
        assert_eq!(result.string_to_bool.get("isTrue"), Some(&true));

        assert_eq!(result.int_to_string.len(), 3);
        assert_eq!(result.int_to_string.get("1"), Some(&"one".to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn complex_maps_contain_valid_data() -> Result<()> {
        let client = BamlClient::new()?;
        let result: ComplexMaps = client.test_complex_maps("test complex maps").await?;

        assert!(result.user_map.len() >= 2);
        for (key, user) in &result.user_map {
            assert!(
                !user.name.trim().is_empty(),
                "user {key} should have a name"
            );
            assert!(
                !user.email.trim().is_empty(),
                "user {key} should have an email"
            );
        }

        assert!(result.product_map.len() >= 3);
        for (key, product) in &result.product_map {
            assert!(
                !product.name.trim().is_empty(),
                "product {key} should have a name"
            );
            assert!(product.price > 0.0, "product {key} price must be positive");
        }

        assert!(!result.nested_map.is_empty());
        assert_eq!(result.array_map.len(), 2);
        assert!(result.map_array.len() >= 2);
        Ok(())
    }

    #[tokio::test]
    async fn nested_maps_have_multiple_levels() -> Result<()> {
        let client = BamlClient::new()?;
        let result: NestedMaps = client.test_nested_maps("test nested maps").await?;

        assert!(result.simple.len() >= 2);

        assert!(result.one_level_nested.len() >= 2);
        for inner in result.one_level_nested.values() {
            assert!(inner.len() >= 2);
        }

        assert!(result.two_level_nested.len() >= 2);
        for inner in result.two_level_nested.values() {
            for deeper in inner.values() {
                assert!(deeper.len() >= 1);
            }
        }

        assert!(!result.map_of_arrays.is_empty());
        assert!(!result.map_of_maps.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn edge_case_maps_handle_nulls_and_unions() -> Result<()> {
        let client = BamlClient::new()?;
        let result: EdgeCaseMaps = client.test_edge_case_maps("test edge case maps").await?;

        assert!(result.empty_map.is_empty());

        assert_eq!(
            result.nullable_values.get("present").unwrap(),
            &Some("value".to_string())
        );
        assert_eq!(result.nullable_values.get("absent"), Some(&None));

        assert!(result.optional_values.contains_key("required"));

        let unions_json: Value = serde_json::to_value(&result.union_values)?;
        let union_map = unions_json
            .as_object()
            .expect("union map should serialize as object");
        assert_eq!(
            union_map.get("string").and_then(|v| v.as_str()),
            Some("hello")
        );
        assert_eq!(union_map.get("number").and_then(|v| v.as_i64()), Some(42));
        assert_eq!(
            union_map.get("boolean").and_then(|v| v.as_bool()),
            Some(true)
        );
        Ok(())
    }

    #[tokio::test]
    async fn large_maps_have_many_entries() -> Result<()> {
        let client = BamlClient::new()?;
        let result: SimpleMaps = client.test_large_maps("test large structure").await?;

        assert!(result.string_to_string.len() >= 20);
        assert!(result.string_to_int.len() >= 20);
        assert!(result.string_to_float.len() >= 20);
        assert!(result.string_to_bool.len() >= 20);
        assert!(result.int_to_string.len() >= 20);
        Ok(())
    }

    #[tokio::test]
    async fn top_level_string_map_matches_expected_values() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_top_level_string_map("test string map").await?;

        assert_eq!(result.len(), 3);
        assert_eq!(result.get("first"), Some(&"Hello".to_string()));
        assert_eq!(result.get("second"), Some(&"World".to_string()));
        assert_eq!(result.get("third"), Some(&"BAML".to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn top_level_int_map_matches_expected_values() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_top_level_int_map("test int map").await?;

        assert_eq!(result.len(), 4);
        assert_eq!(result.get("one"), Some(&1));
        assert_eq!(result.get("two"), Some(&2));
        assert_eq!(result.get("ten"), Some(&10));
        assert_eq!(result.get("hundred"), Some(&100));
        Ok(())
    }

    #[tokio::test]
    async fn top_level_float_map_matches_expected_values() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_top_level_float_map("test float map").await?;

        assert_eq!(result.len(), 3);
        assert!((result.get("pi").copied().unwrap_or_default() - 3.14159).abs() < 0.001);
        assert!((result.get("e").copied().unwrap_or_default() - 2.71828).abs() < 0.001);
        assert!((result.get("golden").copied().unwrap_or_default() - 1.61803).abs() < 0.001);
        Ok(())
    }

    #[tokio::test]
    async fn top_level_bool_map_matches_expected_values() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_top_level_bool_map("test bool map").await?;

        assert_eq!(result.len(), 3);
        assert_eq!(result.get("isActive"), Some(&true));
        assert_eq!(result.get("isDisabled"), Some(&false));
        assert_eq!(result.get("isEnabled"), Some(&true));
        Ok(())
    }

    #[tokio::test]
    async fn top_level_nested_map_matches_expected_shape() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_top_level_nested_map("test nested map").await?;

        assert_eq!(result.len(), 2);
        assert_eq!(result.get("users").map(|m| m.len()), Some(2));
        assert_eq!(result.get("roles").map(|m| m.len()), Some(2));
        Ok(())
    }

    #[tokio::test]
    async fn top_level_map_of_arrays_has_expected_lengths() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client
            .test_top_level_map_of_arrays("test map of arrays")
            .await?;

        assert_eq!(result.len(), 3);
        assert_eq!(result.get("evens").map(|v| v.len()), Some(4));
        assert_eq!(result.get("odds").map(|v| v.len()), Some(4));
        assert_eq!(result.get("primes").map(|v| v.len()), Some(5));
        Ok(())
    }

    #[tokio::test]
    async fn top_level_empty_map_returns_no_entries() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_top_level_empty_map("test empty map").await?;
        assert!(result.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn top_level_map_with_nullable_contains_expected_values() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client
            .test_top_level_map_with_nullable("use jsut a json map")
            .await?;

        assert_eq!(result.len(), 3);
        assert_eq!(result.get("present"), Some(&Some("value".to_string())));
        assert_eq!(result.get("absent"), Some(&None));
        Ok(())
    }

    #[tokio::test]
    async fn top_level_map_of_objects_has_valid_users() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client
            .test_top_level_map_of_objects("test object map")
            .await?;

        assert_eq!(result.len(), 2);
        for (key, user) in &result {
            assert!(
                !user.name.trim().is_empty(),
                "expected user {key} to have a name"
            );
            assert!(
                !user.email.trim().is_empty(),
                "expected user {key} to have an email"
            );
        }
        Ok(())
    }
}
