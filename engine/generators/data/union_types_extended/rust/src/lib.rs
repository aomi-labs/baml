#[cfg(test)]
mod tests {
    use anyhow::Result;
    use baml_client::{
        BamlClient, ComplexUnions, DiscriminatedUnions, PrimitiveUnions, UnionArrays,
    };

    fn approx_eq(left: f64, right: f64, tol: f64) {
        assert!(
            (left - right).abs() <= tol,
            "expected {left} to be within {tol} of {right}"
        );
    }

    #[tokio::test]
    async fn primitive_unions_return_expected_literals() -> Result<()> {
        let client = BamlClient::new()?;
        let result: PrimitiveUnions = client
            .test_primitive_unions("test primitive unions")
            .await?;

        assert!(
            result.string_or_int.is_int(),
            "expected string_or_int to be an int, got {:?}",
            result.string_or_int
        );
        assert_eq!(
            result.string_or_int.as_int().copied(),
            Some(42),
            "expected string_or_int to be 42"
        );

        assert!(
            result.string_or_float.is_string(),
            "expected string_or_float to be a string, got {:?}",
            result.string_or_float
        );
        assert_eq!(
            result
                .string_or_float
                .as_string()
                .map(|value| value.as_str()),
            Some("hello"),
            "expected string_or_float to contain 'hello'"
        );

        assert!(
            result.int_or_float.is_float(),
            "expected int_or_float to be a float, got {:?}",
            result.int_or_float
        );
        approx_eq(
            *result
                .int_or_float
                .as_float()
                .expect("expected float variant"),
            3.14,
            0.01,
        );

        assert!(
            result.bool_or_string.is_bool(),
            "expected bool_or_string to be a bool, got {:?}",
            result.bool_or_string
        );
        assert_eq!(
            result.bool_or_string.as_bool(),
            Some(&true),
            "expected bool_or_string to be true"
        );

        assert!(
            result.any_primitive.is_string(),
            "expected any_primitive to be a string, got {:?}",
            result.any_primitive
        );
        assert_eq!(
            result.any_primitive.as_string().map(|value| value.as_str()),
            Some("mixed"),
            "expected any_primitive to contain 'mixed'"
        );

        Ok(())
    }

    #[tokio::test]
    async fn complex_unions_cover_all_variants() -> Result<()> {
        let client = BamlClient::new()?;
        let result: ComplexUnions = client.test_complex_unions("test complex unions").await?;

        assert!(
            result.user_or_product.is_user() || result.user_or_product.is_product(),
            "expected user_or_product to be user or product"
        );
        if let Some(user) = result.user_or_product.as_user() {
            assert!(user.id > 0, "expected user id to be positive");
            assert_eq!(user.r#type, "user", "expected user type discriminator");
        }
        if let Some(product) = result.user_or_product.as_product() {
            assert!(product.id > 0, "expected product id to be positive");
            assert!(
                product.price >= 0.0,
                "expected product price to be non-negative"
            );
            assert_eq!(
                product.r#type, "product",
                "expected product type discriminator"
            );
        }

        assert!(
            result.user_or_product_or_admin.is_user()
                || result.user_or_product_or_admin.is_product()
                || result.user_or_product_or_admin.is_admin(),
            "expected user_or_product_or_admin to have a variant"
        );
        if let Some(admin) = result.user_or_product_or_admin.as_admin() {
            assert_eq!(admin.r#type, "admin", "expected admin discriminator");
            assert!(
                !admin.permissions.is_empty(),
                "expected admin permissions to be populated"
            );
        }

        assert!(
            result.data_or_error.is_data_response() || result.data_or_error.is_error_response(),
            "expected data_or_error to be data or error"
        );
        if let Some(data) = result.data_or_error.as_data_response() {
            assert_eq!(data.status, "success", "expected success status");
            assert!(
                !data.data.trim().is_empty(),
                "expected data response to include data"
            );
        }
        if let Some(error) = result.data_or_error.as_error_response() {
            assert_eq!(error.status, "error", "expected error status");
            assert!(
                !error.error.trim().is_empty(),
                "expected error response to include message"
            );
        }

        if let Some(output) = &result.result_or_null {
            assert!(
                output.value.is_string() || output.value.is_int() || output.value.is_float(),
                "expected result_or_null.value to hold a primitive variant"
            );
            assert!(
                !output.metadata.is_empty(),
                "expected result metadata when present"
            );
        }

        assert!(
            result.multi_type_result.is_success()
                || result.multi_type_result.is_warning()
                || result.multi_type_result.is_error(),
            "expected multi_type_result variant"
        );
        if let Some(success) = result.multi_type_result.as_success() {
            assert_eq!(success.r#type, "success", "expected success type");
            assert!(
                !success.message.trim().is_empty(),
                "expected success message"
            );
        } else if let Some(warning) = result.multi_type_result.as_warning() {
            assert_eq!(warning.r#type, "warning", "expected warning type");
            assert!(
                warning.level >= 0,
                "expected warning level to be non-negative"
            );
        } else if let Some(error) = result.multi_type_result.as_error() {
            assert_eq!(error.r#type, "error", "expected error type");
            assert!(!error.message.trim().is_empty(), "expected error message");
        }

        Ok(())
    }

    #[tokio::test]
    async fn discriminated_unions_return_requested_variants() -> Result<()> {
        let client = BamlClient::new()?;
        let result: DiscriminatedUnions = client
            .test_discriminated_unions("test discriminated unions")
            .await?;

        assert!(
            result.shape.is_circle(),
            "expected shape to be a circle, got {:?}",
            result.shape
        );
        let circle = result
            .shape
            .as_circle()
            .expect("expected circle variant for shape");
        assert_eq!(circle.shape, "circle", "expected circle discriminator");
        approx_eq(circle.radius, 5.0, 0.01);

        assert!(
            result.animal.is_dog(),
            "expected animal to be a dog, got {:?}",
            result.animal
        );
        let dog = result
            .animal
            .as_dog()
            .expect("expected dog variant for animal");
        assert_eq!(dog.species, "dog", "expected dog discriminator");
        assert!(
            !dog.breed.trim().is_empty(),
            "expected dog breed to be populated"
        );
        assert!(dog.good_boy, "expected dog.good_boy to be true");

        assert!(
            result.response.is_api_error(),
            "expected response to be an API error, got {:?}",
            result.response
        );
        let api_error = result
            .response
            .as_api_error()
            .expect("expected api error variant");
        assert_eq!(api_error.status, "error", "expected error status");
        assert_eq!(api_error.message, "Not found", "expected error message");
        assert_eq!(api_error.code, 404, "expected error code 404");

        Ok(())
    }

    #[tokio::test]
    async fn union_arrays_match_expected_content() -> Result<()> {
        let client = BamlClient::new()?;
        let result: UnionArrays = client.test_union_arrays("test union arrays").await?;

        assert_eq!(result.mixed_array.len(), 4, "expected four mixed values");
        assert_eq!(
            result.mixed_array[0]
                .as_string()
                .map(|value| value.as_str()),
            Some("hello"),
            "expected first mixed value to be 'hello'"
        );
        assert_eq!(
            result.mixed_array[1].as_int().copied(),
            Some(1),
            "expected second mixed value to be 1"
        );
        assert_eq!(
            result.mixed_array[2]
                .as_string()
                .map(|value| value.as_str()),
            Some("world"),
            "expected third mixed value to be 'world'"
        );
        assert_eq!(
            result.mixed_array[3].as_int().copied(),
            Some(2),
            "expected fourth mixed value to be 2"
        );

        assert_eq!(
            result.nullable_items.len(),
            4,
            "expected four nullable entries"
        );
        assert_eq!(
            result.nullable_items[0].as_deref(),
            Some("present"),
            "expected first nullable item"
        );
        assert!(
            result.nullable_items[1].is_none(),
            "expected second to be null"
        );
        assert_eq!(
            result.nullable_items[2].as_deref(),
            Some("also present"),
            "expected third nullable item"
        );
        assert!(
            result.nullable_items[3].is_none(),
            "expected fourth to be null"
        );

        assert!(
            result.object_array.len() >= 2,
            "expected at least two objects in object_array"
        );
        let mut saw_user = false;
        let mut saw_product = false;
        for entry in &result.object_array {
            if let Some(user) = entry.as_user() {
                saw_user = true;
                assert_eq!(user.r#type, "user", "expected user discriminator");
                assert!(user.id > 0, "expected user id to be positive");
                assert!(
                    !user.name.trim().is_empty(),
                    "expected user name to be populated"
                );
            } else if let Some(product) = entry.as_product() {
                saw_product = true;
                assert_eq!(product.r#type, "product", "expected product discriminator");
                assert!(product.id > 0, "expected product id to be positive");
                assert!(
                    product.price >= 0.0,
                    "expected product price to be non-negative"
                );
            } else {
                panic!("unexpected union variant in object_array");
            }
        }
        assert!(saw_user, "expected at least one user in object_array");
        assert!(saw_product, "expected at least one product in object_array");

        assert_eq!(
            result.nested_union_array.len(),
            4,
            "expected four entries in nested_union_array"
        );
        assert_eq!(
            result.nested_union_array[0]
                .as_string()
                .map(|value| value.as_str()),
            Some("string"),
            "expected first nested union entry"
        );
        let second = result.nested_union_array[1]
            .as_list_int()
            .expect("expected second entry to be list");
        assert_eq!(
            second.as_slice(),
            &[1, 2, 3],
            "expected second nested entry to be [1, 2, 3]"
        );
        assert_eq!(
            result.nested_union_array[2]
                .as_string()
                .map(|value| value.as_str()),
            Some("another"),
            "expected third nested union entry"
        );
        let fourth = result.nested_union_array[3]
            .as_list_int()
            .expect("expected fourth entry to be list");
        assert_eq!(
            fourth.as_slice(),
            &[4, 5],
            "expected fourth nested entry to be [4, 5]"
        );

        Ok(())
    }
}
