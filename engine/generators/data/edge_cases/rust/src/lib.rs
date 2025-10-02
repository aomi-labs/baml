#[cfg(test)]
mod tests {
    use anyhow::Result;
    use baml_client::{BamlClient, CircularReference, DeepRecursion};

    #[tokio::test]
    async fn test_empty_collections() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client
            .test_empty_collections("test empty collections")
            .await?;

        assert!(result.empty_string_array.is_empty());
        assert!(result.empty_int_array.is_empty());
        assert!(result.empty_object_array.is_empty());
        assert!(result.empty_map.is_empty());
        assert!(result.empty_nested_array.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_large_structure() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_large_structure("test large structure").await?;

        for (idx, value) in [
            &result.field1,
            &result.field2,
            &result.field3,
            &result.field4,
            &result.field5,
        ]
        .iter()
        .enumerate()
        {
            assert!(
                !value.trim().is_empty(),
                "expected field{} to be non-empty",
                idx + 1
            );
        }

        for (idx, value) in [
            result.field6,
            result.field7,
            result.field8,
            result.field9,
            result.field10,
        ]
        .into_iter()
        .enumerate()
        {
            assert!(value != 0, "expected field{} to be non-zero", idx + 6);
        }

        for (idx, value) in [
            result.field11,
            result.field12,
            result.field13,
            result.field14,
            result.field15,
        ]
        .into_iter()
        .enumerate()
        {
            assert!(value != 0.0, "expected field{} to be non-zero", idx + 11);
        }

        for (idx, array_len) in [
            result.array1.len(),
            result.array2.len(),
            result.array3.len(),
            result.array4.len(),
            result.array5.len(),
        ]
        .into_iter()
        .enumerate()
        {
            assert!(
                (3..=5).contains(&array_len),
                "expected array{} length between 3 and 5, got {}",
                idx + 1,
                array_len
            );
        }

        for (idx, map_len) in [
            result.map1.len(),
            result.map2.len(),
            result.map3.len(),
            result.map4.len(),
            result.map5.len(),
        ]
        .into_iter()
        .enumerate()
        {
            assert!(
                (2..=3).contains(&map_len),
                "expected map{} length between 2 and 3, got {}",
                idx + 1,
                map_len
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_deep_recursion() -> Result<()> {
        fn depth(node: Option<&DeepRecursion>) -> (usize, bool) {
            let mut current = node;
            let mut count = 0usize;
            let mut all_non_empty = true;
            while let Some(value) = current {
                count += 1;
                all_non_empty &= !value.value.trim().is_empty();
                current = value.next.as_deref();
            }
            (count, all_non_empty)
        }

        let client = BamlClient::new()?;
        let result = client.test_deep_recursion(5).await?;

        let (count, all_non_empty) = depth(Some(&result));
        assert_eq!(count, 5, "expected recursion depth 5");
        assert!(
            all_non_empty,
            "expected all recursion values to be non-empty"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_special_characters() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client
            .test_special_characters("test special characters")
            .await?;

        assert_eq!(result.normal_text, "Hello World");
        assert!(result.with_newlines.contains('\n'));
        assert!(result.with_tabs.contains('\t'));
        assert!(result.with_quotes.contains('"'));
        assert!(result.with_backslashes.contains('\\'));
        assert!(
            !result.with_unicode.trim().is_empty(),
            "expected unicode string to be non-empty"
        );
        assert!(
            !result.with_emoji.trim().is_empty(),
            "expected emoji string to be non-empty"
        );
        assert!(
            !result.with_mixed_special.trim().is_empty(),
            "expected mixed special string to be non-empty"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_number_edge_cases() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client
            .test_number_edge_cases("test number edge cases")
            .await?;

        assert_eq!(result.zero, 0);
        assert!(result.negative_int < 0);
        assert!(result.large_int > 1_000);
        assert!(result.very_large_int > 1_000_000);
        assert!(result.small_float < 1.0);
        assert!(result.large_float > 1_000.0);
        assert!(result.negative_float < 0.0);
        assert!(result.scientific_notation.abs() > 1_000.0);
        Ok(())
    }

    #[tokio::test]
    async fn test_circular_reference() -> Result<()> {
        fn assert_child_relationship(root: &CircularReference, child: &CircularReference) {
            if let Some(parent) = child.parent.as_deref() {
                assert_eq!(parent.id, root.id);
            }
        }

        let client = BamlClient::new()?;
        let result = client
            .test_circular_reference("test circular reference")
            .await?;

        assert_eq!(result.id, 1);
        assert!(
            !result.name.trim().is_empty(),
            "expected root name to be non-empty"
        );
        assert_eq!(result.children.len(), 2);

        let child_ids: Vec<_> = result.children.iter().map(|child| child.id).collect();
        assert_ne!(child_ids[0], child_ids[1], "expected unique child ids");

        for child in &result.children {
            assert_child_relationship(&result, child);
        }

        for related in &result.related_items {
            assert!(related.id > 0, "expected related item to have an id");
            assert!(
                !related.name.trim().is_empty(),
                "expected related item to have a name"
            );
        }
        Ok(())
    }
}
