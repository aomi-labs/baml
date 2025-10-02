#[cfg(test)]
mod tests {
    use anyhow::Result;
    use baml_client::BamlClient;

    #[tokio::test]
    async fn test_simple_arrays() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_simple_arrays("test simple arrays").await?;

        // Verify array lengths
        assert_eq!(result.strings.len(), 3, "Expected strings length 3");
        assert_eq!(result.integers.len(), 5, "Expected integers length 5");
        assert_eq!(result.floats.len(), 3, "Expected floats length 3");
        assert_eq!(result.booleans.len(), 4, "Expected booleans length 4");

        // Verify specific values
        assert_eq!(result.strings, vec!["hello", "world", "test"]);
        assert_eq!(result.integers, vec![1, 2, 3, 4, 5]);
        assert_eq!(result.booleans, vec![true, false, true, false]);

        println!("✓ SimpleArrays test passed");
        Ok(())
    }

    #[tokio::test]
    async fn test_nested_arrays() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_nested_arrays("test nested arrays").await?;

        // Verify nested array structure
        assert_eq!(result.matrix.len(), 3, "Expected matrix length 3");
        assert_eq!(
            result.string_matrix.len(),
            2,
            "Expected string_matrix length 2"
        );
        assert_eq!(
            result.three_dimensional.len(),
            2,
            "Expected three_dimensional length 2"
        );

        // Verify matrix content
        assert_eq!(result.matrix[0], vec![1, 2, 3]);
        assert_eq!(result.matrix[1], vec![4, 5, 6]);
        assert_eq!(result.matrix[2], vec![7, 8, 9]);

        // Verify string matrix
        assert_eq!(result.string_matrix[0], vec!["a", "b"]);
        assert_eq!(result.string_matrix[1], vec!["c", "d"]);

        // Verify 3D structure dimensions
        assert_eq!(
            result.three_dimensional[0].len(),
            2,
            "First level should have 2 elements"
        );
        assert_eq!(
            result.three_dimensional[0][0].len(),
            2,
            "Second level should have 2 elements"
        );

        println!("✓ NestedArrays test passed");
        Ok(())
    }

    #[tokio::test]
    async fn test_object_arrays() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_object_arrays("test object arrays").await?;

        // Verify array lengths
        assert_eq!(result.users.len(), 3, "Expected 3 users");
        assert_eq!(result.products.len(), 2, "Expected 2 products");
        assert_eq!(result.tags.len(), 4, "Expected 4 tags");

        // Verify user objects have required fields
        for (i, user) in result.users.iter().enumerate() {
            assert!(user.id > 0, "User {} has invalid id: {}", i, user.id);
            assert!(!user.name.is_empty(), "User {} has empty name", i);
            assert!(!user.email.is_empty(), "User {} has empty email", i);
        }

        // Verify product objects
        for (i, product) in result.products.iter().enumerate() {
            assert!(
                product.id > 0,
                "Product {} has invalid id: {}",
                i,
                product.id
            );
            assert!(!product.name.is_empty(), "Product {} has empty name", i);
            assert!(
                product.price >= 0.0,
                "Product {} has negative price: {}",
                i,
                product.price
            );
        }

        // Verify tag objects
        for (i, tag) in result.tags.iter().enumerate() {
            assert!(tag.id > 0, "Tag {} has invalid id: {}", i, tag.id);
            assert!(!tag.name.is_empty(), "Tag {} has empty name", i);
            assert!(!tag.color.is_empty(), "Tag {} has empty color", i);
        }

        println!("✓ ObjectArrays test passed");
        Ok(())
    }

    #[tokio::test]
    async fn test_mixed_arrays() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_mixed_arrays("test mixed arrays").await?;

        // Verify mixed array contents
        assert_eq!(
            result.primitive_array.len(),
            4,
            "Expected primitive_array length 4"
        );
        assert_eq!(
            result.nullable_array.len(),
            4,
            "Expected nullable_array length 4"
        );
        assert!(
            result.optional_items.len() >= 2,
            "Expected at least 2 optional_items"
        );
        assert!(
            result.array_of_arrays.len() >= 2,
            "Expected at least 2 array_of_arrays"
        );
        assert!(
            result.complex_mixed.len() >= 2,
            "Expected at least 2 complex_mixed items"
        );

        println!("✓ MixedArrays test passed");
        Ok(())
    }

    #[tokio::test]
    async fn test_empty_arrays() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_empty_arrays("test empty arrays").await?;

        // Verify all arrays are empty
        assert_eq!(result.strings.len(), 0, "Expected empty strings array");
        assert_eq!(result.integers.len(), 0, "Expected empty integers array");
        assert_eq!(result.floats.len(), 0, "Expected empty floats array");
        assert_eq!(result.booleans.len(), 0, "Expected empty booleans array");

        println!("✓ EmptyArrays test passed");
        Ok(())
    }

    #[tokio::test]
    async fn test_large_arrays() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_large_arrays("test large arrays").await?;

        // Verify large array sizes
        assert!(
            result.strings.len() >= 40,
            "Expected at least 40 strings, got {}",
            result.strings.len()
        );
        assert!(
            result.integers.len() >= 50,
            "Expected at least 50 integers, got {}",
            result.integers.len()
        );
        assert!(
            result.floats.len() >= 20,
            "Expected at least 20 floats, got {}",
            result.floats.len()
        );
        assert!(
            result.booleans.len() >= 15,
            "Expected at least 15 booleans, got {}",
            result.booleans.len()
        );

        println!("✓ LargeArrays test passed");
        Ok(())
    }

    // Test top-level array return types
    #[tokio::test]
    async fn test_top_level_string_array() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client
            .test_top_level_string_array("test string array")
            .await?;
        assert_eq!(result.len(), 4, "Expected 4 strings");
        assert_eq!(result, vec!["apple", "banana", "cherry", "date"]);
        println!("✓ TopLevelStringArray test passed");
        Ok(())
    }

    #[tokio::test]
    async fn test_top_level_int_array() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_top_level_int_array("test int array").await?;
        assert_eq!(result.len(), 5, "Expected 5 integers");
        assert_eq!(result, vec![10, 20, 30, 40, 50]);
        println!("✓ TopLevelIntArray test passed");
        Ok(())
    }

    #[tokio::test]
    async fn test_top_level_float_array() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client
            .test_top_level_float_array("test float array")
            .await?;
        assert_eq!(result.len(), 4, "Expected 4 floats");
        assert_eq!(result, vec![1.5, 2.5, 3.5, 4.5]);
        println!("✓ TopLevelFloatArray test passed");
        Ok(())
    }

    #[tokio::test]
    async fn test_top_level_bool_array() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_top_level_bool_array("test bool array").await?;
        assert_eq!(result.len(), 5, "Expected 5 booleans");
        assert_eq!(result, vec![true, false, true, false, true]);
        println!("✓ TopLevelBoolArray test passed");
        Ok(())
    }

    #[tokio::test]
    async fn test_top_level_nested_array() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client
            .test_top_level_nested_array("test nested array")
            .await?;
        assert_eq!(result.len(), 3, "Expected 3 rows");
        for (i, row) in result.iter().enumerate() {
            assert_eq!(row.len(), 3, "Expected 3 columns in row {}", i);
        }
        println!("✓ TopLevelNestedArray test passed");
        Ok(())
    }

    #[tokio::test]
    async fn test_top_level_3d_array() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_top_level3_d_array("test 3D array").await?;
        assert_eq!(result.len(), 2, "Expected 2 levels");
        for (i, level) in result.iter().enumerate() {
            assert_eq!(level.len(), 2, "Expected 2 rows in level {}", i);
            for (j, row) in level.iter().enumerate() {
                assert_eq!(row.len(), 2, "Expected 2 columns in level {} row {}", i, j);
            }
        }
        println!("✓ TopLevel3DArray test passed");
        Ok(())
    }

    #[tokio::test]
    async fn test_top_level_empty_array() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client
            .test_top_level_empty_array("test empty array")
            .await?;
        assert_eq!(result.len(), 0, "Expected empty array");
        println!("✓ TopLevelEmptyArray test passed");
        Ok(())
    }

    #[tokio::test]
    async fn test_top_level_nullable_array() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client
            .test_top_level_nullable_array("test nullable array")
            .await?;
        assert_eq!(result.len(), 5, "Expected 5 elements in nullable array");
        assert_eq!(
            result[0],
            Some("hello".to_string()),
            "Expected first element to be 'hello'"
        );
        assert_eq!(result[1], None, "Expected second element to be None");
        println!("✓ TopLevelNullableArray test passed");
        Ok(())
    }

    #[tokio::test]
    async fn test_top_level_object_array() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client
            .test_top_level_object_array("test object array")
            .await?;
        assert_eq!(result.len(), 3, "Expected 3 users");
        for (i, user) in result.iter().enumerate() {
            assert!(!user.name.is_empty(), "User {} has empty name", i);
            assert!(!user.email.is_empty(), "User {} has empty email", i);
        }
        println!("✓ TopLevelObjectArray test passed");
        Ok(())
    }

    #[tokio::test]
    async fn test_top_level_mixed_array() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client
            .test_top_level_mixed_array("test mixed array")
            .await?;
        assert_eq!(result.len(), 6, "Expected 6 elements in mixed array");
        println!("✓ TopLevelMixedArray test passed");
        Ok(())
    }

    #[tokio::test]
    async fn test_top_level_array_of_maps() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client
            .test_top_level_array_of_maps("test array of maps")
            .await?;
        assert_eq!(result.len(), 3, "Expected 3 maps in array");
        for (i, map) in result.iter().enumerate() {
            assert_eq!(map.len(), 2, "Expected 2 entries in map {}", i);
        }
        println!("✓ TopLevelArrayOfMaps test passed");
        Ok(())
    }
}
