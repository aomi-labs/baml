#[cfg(test)]
mod tests {
    use anyhow::Result;
    use baml_client::{BamlClient, NullValue};

    #[tokio::test]
    async fn kitchen_sink_has_expected_shape() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_kitchen_sink("test kitchen sink").await?;

        assert!(result.id > 0, "expected positive id");
        assert!(!result.name.trim().is_empty(), "expected non-empty name");
        assert!(result.score.is_finite(), "expected finite score");
        assert_eq!(result.nothing, NullValue);

        assert!(
            result.status.is_k_draft()
                || result.status.is_k_published()
                || result.status.is_k_archived(),
            "unexpected status variant"
        );
        assert!(
            result.priority.is_intk1()
                || result.priority.is_intk2()
                || result.priority.is_intk3()
                || result.priority.is_intk4()
                || result.priority.is_intk5(),
            "unexpected priority variant"
        );

        assert!(
            result.data.is_string() || result.data.is_int() || result.data.is_data_object(),
            "unexpected data union variant"
        );
        assert!(
            result.result.is_success() || result.result.is_error(),
            "unexpected result union variant"
        );

        assert!(result.user.id > 0, "expected user id to be positive");
        assert!(
            !result.user.profile.name.trim().is_empty(),
            "expected user profile name"
        );
        assert!(
            !result.user.profile.email.trim().is_empty(),
            "expected user profile email"
        );

        assert!(
            !result.config.version.trim().is_empty(),
            "expected configuration version"
        );
        assert!(result.config.features.len() >= 0);
        assert!(result.config.environments.len() >= 0);
        assert!(result.config.rules.len() >= 0);
        Ok(())
    }

    #[tokio::test]
    async fn ultra_complex_has_widgets_and_tree() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_ultra_complex("test ultra complex").await?;

        assert!(result.tree.id > 0, "expected tree id to be positive");
        assert!(
            matches!(result.tree.r#type.as_str(), "leaf" | "branch"),
            "unexpected tree type: {}",
            result.tree.r#type
        );
        assert!(
            result.tree.value.is_string()
                || result.tree.value.is_int()
                || result.tree.value.is_list_node()
                || result.tree.value.is_map_string_key_node_value(),
            "unexpected tree value variant"
        );

        assert!(!result.widgets.is_empty(), "expected at least one widget");
        for widget in &result.widgets {
            match widget.r#type.as_str() {
                "button" => assert!(widget.button.is_some(), "button widget missing data"),
                "text" => {
                    let text = widget.text.as_ref().expect("text widget missing data");
                    assert!(
                        text.format.is_k_plain()
                            || text.format.is_k_markdown()
                            || text.format.is_k_html(),
                        "unexpected text widget format"
                    );
                }
                "image" => assert!(widget.image.is_some(), "image widget missing data"),
                "container" => {
                    assert!(widget.container.is_some(), "container widget missing data");
                }
                other => panic!("unexpected widget type: {other}"),
            }
        }

        if let Some(data) = &result.data {
            assert!(!data.primary.values.is_empty(), "expected primary values");
        }

        assert!(
            result.response.status.is_k_success() || result.response.status.is_k_error(),
            "unexpected response status"
        );
        assert!(result.assets.len() >= 0);
        Ok(())
    }

    #[tokio::test]
    async fn recursive_complexity_contains_nested_nodes() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client
            .test_recursive_complexity("test recursive complexity")
            .await?;

        assert!(result.id > 0, "expected node id to be positive");
        assert!(
            matches!(result.r#type.as_str(), "leaf" | "branch"),
            "unexpected node type: {}",
            result.r#type
        );
        assert!(
            result.value.is_string()
                || result.value.is_int()
                || result.value.is_list_node()
                || result.value.is_map_string_key_node_value(),
            "unexpected node value variant"
        );
        Ok(())
    }
}
