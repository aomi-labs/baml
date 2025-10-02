#[cfg(test)]
mod tests {
    use anyhow::Result;
    use baml_client::BamlClient;

    #[tokio::test]
    async fn simple_nested_has_expected_fields() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_simple_nested("test simple nested").await?;

        let user = result.user;
        assert!(user.id > 0, "expected user id to be positive");
        assert!(!user.name.trim().is_empty(), "expected user name");

        let profile = user.profile;
        assert!(!profile.bio.trim().is_empty(), "expected bio");
        assert!(!profile.avatar.trim().is_empty(), "expected avatar");
        assert!(
            profile.preferences.theme.is_k_light() || profile.preferences.theme.is_k_dark(),
            "unexpected theme variant"
        );
        assert!(
            profile.preferences.notifications.frequency.is_k_immediate()
                || profile.preferences.notifications.frequency.is_k_daily()
                || profile.preferences.notifications.frequency.is_k_weekly(),
            "unexpected notification frequency"
        );
        assert!(
            user.settings.privacy.profile_visibility.is_k_public()
                || user.settings.privacy.profile_visibility.is_k_private()
                || user.settings.privacy.profile_visibility.is_k_friends(),
            "unexpected profile visibility"
        );
        assert!(
            user.settings.display.font_size > 0,
            "expected positive font size"
        );
        assert!(
            !user.settings.display.color_scheme.trim().is_empty(),
            "expected color scheme"
        );

        let address = result.address;
        assert!(!address.street.trim().is_empty(), "expected street");
        assert!(!address.city.trim().is_empty(), "expected city");
        assert!(!address.state.trim().is_empty(), "expected state");
        assert!(!address.country.trim().is_empty(), "expected country");
        assert!(
            !address.postal_code.trim().is_empty(),
            "expected postal code"
        );

        let metadata = result.metadata;
        assert!(
            !metadata.created_at.trim().is_empty(),
            "expected created_at"
        );
        assert!(
            !metadata.updated_at.trim().is_empty(),
            "expected updated_at"
        );
        assert!(metadata.version > 0, "expected positive version");
        assert!(metadata.tags.len() >= 0);
        assert!(metadata.attributes.len() >= 0);
        Ok(())
    }

    #[tokio::test]
    async fn deeply_nested_walks_levels() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_deeply_nested("test deeply nested").await?;

        let level1 = result.level1;
        assert!(!level1.data.trim().is_empty(), "expected level1 data");

        let level2 = level1.level2;
        assert!(!level2.data.trim().is_empty(), "expected level2 data");

        let level3 = level2.level3;
        assert!(!level3.data.trim().is_empty(), "expected level3 data");

        let level4 = level3.level4;
        assert!(!level4.data.trim().is_empty(), "expected level4 data");

        let level5 = level4.level5;
        assert!(!level5.data.trim().is_empty(), "expected level5 data");
        assert!(level5.items.len() >= 0);
        assert!(level5.mapping.len() >= 0);
        Ok(())
    }

    #[tokio::test]
    async fn complex_nested_covers_company_structure() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client.test_complex_nested("test complex nested").await?;

        let company = result.company;
        assert!(company.id > 0, "expected company id");
        assert!(!company.name.trim().is_empty(), "expected company name");
        assert!(company.departments.len() >= 2);
        assert!(
            company.metadata.size.is_k_small()
                || company.metadata.size.is_k_medium()
                || company.metadata.size.is_k_large()
                || company.metadata.size.is_k_enterprise(),
            "unexpected company size"
        );

        for department in &company.departments {
            assert!(department.id > 0, "department id");
            assert!(!department.name.trim().is_empty(), "department name");
            assert!(department.budget.is_finite(), "department budget");
        }

        assert!(!result.employees.is_empty(), "expected employees");
        for employee in &result.employees {
            assert!(employee.id > 0, "employee id");
            assert!(!employee.name.trim().is_empty(), "employee name");
            assert!(!employee.email.trim().is_empty(), "employee email");
            assert!(!employee.role.trim().is_empty(), "employee role");
            assert!(
                !employee.department.trim().is_empty(),
                "employee department"
            );
        }

        assert!(!result.projects.is_empty(), "expected projects");
        for project in &result.projects {
            assert!(project.id > 0, "project id");
            assert!(!project.name.trim().is_empty(), "project name");
            assert!(
                !project.description.trim().is_empty(),
                "project description"
            );
            assert!(
                project.status.is_k_planning()
                    || project.status.is_k_active()
                    || project.status.is_k_completed()
                    || project.status.is_k_cancelled(),
                "unexpected project status"
            );
            assert!(project.budget.total.is_finite(), "project budget total");
            assert!(project.budget.spent.is_finite(), "project budget spent");
        }
        Ok(())
    }

    #[tokio::test]
    async fn recursive_structure_has_parent_links() -> Result<()> {
        let client = BamlClient::new()?;
        let result = client
            .test_recursive_structure("test recursive structure")
            .await?;

        assert!(result.id > 0, "expected root id");
        assert!(!result.name.trim().is_empty(), "expected root name");
        assert!(result.children.len() >= 2, "expected children");

        for child in &result.children {
            assert!(child.id > 0, "child id");
            assert!(!child.name.trim().is_empty(), "child name");
            if let Some(parent) = child.parent.as_ref().as_ref() {
                assert_eq!(parent.id, result.id, "parent id mismatch");
            }
        }

        let has_grandchildren = result
            .children
            .iter()
            .any(|child| !child.children.is_empty());
        assert!(has_grandchildren, "expected nested grandchildren");
        Ok(())
    }
}
