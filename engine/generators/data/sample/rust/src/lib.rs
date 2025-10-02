#[cfg(test)]
mod tests {
    use anyhow::Result;
    use baml_client::{BamlClient, Example, Example2, StreamState, Union2ExampleOrExample2};
    use futures::{pin_mut, StreamExt};

    fn assert_example(example: &Example) {
        assert!(example.a != 0, "expected 'a' to be non-zero");
        assert!(!example.b.trim().is_empty(), "expected 'b' to be populated");
        assert!(
            !example.r#type.trim().is_empty(),
            "expected 'type' to be populated"
        );
    }

    fn assert_example2(example: &Example2) {
        assert!(!example.r#type.trim().is_empty(), "expected example2.type");
        assert!(
            !example.element.trim().is_empty(),
            "expected example2.element"
        );
        assert!(
            !example.element2.trim().is_empty(),
            "expected example2.element2"
        );
        assert_example(&example.item);
    }

    fn assert_union_has_data(union: &Union2ExampleOrExample2) {
        if let Some(example) = union.as_example() {
            assert_example(example);
        } else if let Some(example2) = union.as_example2() {
            assert_example2(example2);
        } else {
            panic!("unexpected union variant");
        }
    }

    #[tokio::test]
    async fn foo_returns_valid_union() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.foo(123).await?;
        assert_union_has_data(&result);
        Ok(())
    }

    #[tokio::test]
    async fn bar_returns_valid_union() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.bar(456).await?;
        assert_union_has_data(&result);
        Ok(())
    }

    #[tokio::test]
    async fn foo_stream_emits_partial_and_final() -> Result<()> {
        let client = BamlClient::new()?;
        let stream = client.foo_stream(789).await?;
        pin_mut!(stream);

        let mut saw_partial = false;
        let mut final_union: Option<Union2ExampleOrExample2> = None;

        while let Some(item) = stream.next().await {
            match item? {
                StreamState::Partial(value) => {
                    saw_partial = true;
                    assert!(value.is_example() || value.is_example2());
                }
                StreamState::Final(value) => {
                    final_union = Some(value);
                }
            }
        }

        assert!(saw_partial, "expected at least one partial update");
        let final_union = final_union.expect("expected final union value");
        assert_union_has_data(&final_union);
        Ok(())
    }

    #[tokio::test]
    async fn bar_stream_emits_partial_and_final() -> Result<()> {
        let client = BamlClient::new()?;
        let stream = client.bar_stream(321).await?;
        pin_mut!(stream);

        let mut saw_partial = false;
        let mut final_union: Option<Union2ExampleOrExample2> = None;

        while let Some(item) = stream.next().await {
            match item? {
                StreamState::Partial(value) => {
                    saw_partial = true;
                    assert!(value.is_example() || value.is_example2());
                }
                StreamState::Final(value) => {
                    final_union = Some(value);
                }
            }
        }

        assert!(saw_partial, "expected at least one partial update");
        let final_union = final_union.expect("expected final union value");
        assert_union_has_data(&final_union);
        Ok(())
    }
}
