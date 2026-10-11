//! DD-61: create the slug for the current goal and its work tree.
use super::*;

impl Startup {
    pub(super) fn create_slug(
        &self,
        saved: &mut Saved,
        requested: Option<String>,
    ) -> Result<serde_json::Value, String> {
        let goal = saved.config.goal.trim().to_string();
        if goal.is_empty() {
            return Err("enter a goal before creating its slug".into());
        }
        let existing: Vec<String> = saved.config.slugs.iter().map(|s| s.slug.clone()).collect();
        let slug = match requested {
            Some(slug) => {
                crate::slugs::validate(&slug)?;
                if existing.contains(&slug) {
                    return Err(format!("slug {slug} already exists in this workspace"));
                }
                slug
            }
            None => crate::slugs::unique(&crate::slugs::suggest(&goal), &existing),
        };
        if !saved.config.repository.as_os_str().is_empty() {
            crate::slugs::exclude_from_git(&Repository::open(&saved.config.repository)?)?;
        }
        let root = crate::slugs::root(&saved.config.repository, &self.broker.dir, &slug);
        crate::slugs::create_tree(&root)?;
        let mut next = saved.clone();
        next.config.slugs.push(crate::slugs::Slug {
            slug: slug.clone(),
            goal,
            created_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_millis()
                .try_into()
                .map_err(|_| "clock out of range")?,
        });
        next.config.active_slug = Some(slug.clone());
        self.store(&next)?;
        *saved = next;
        Ok(serde_json::json!({"slug":slug,"root":root}))
    }
}
