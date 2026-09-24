use super::builtin::{
    CargoTomlProfile, DockerComposeProfile, GitHubActionsProfile, PackageJsonProfile,
    TsconfigJsonProfile,
};
use super::profile::{ConfigProfile, ProfileDescriptor, ProfileDiagnostic};
use crate::document::DocumentNode;
use std::sync::{Arc, OnceLock};

pub struct ProfileRegistry {
    profiles: Vec<Arc<dyn ConfigProfile>>,
}

impl Default for ProfileRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileRegistry {
    pub fn new() -> Self {
        let profiles: Vec<Arc<dyn ConfigProfile>> = vec![
            Arc::new(PackageJsonProfile),
            Arc::new(TsconfigJsonProfile),
            Arc::new(DockerComposeProfile),
            Arc::new(GitHubActionsProfile),
            Arc::new(CargoTomlProfile),
        ];
        Self { profiles }
    }

    pub fn list(&self) -> Vec<ProfileDescriptor> {
        self.profiles
            .iter()
            .map(|p| ProfileDescriptor {
                id: p.id().to_string(),
                name: p.name().to_string(),
                description: p.description().to_string(),
            })
            .collect()
    }

    pub fn detect(&self, filename: &str, content: &str, document: &DocumentNode) -> Option<String> {
        let mut best_score = 0u8;
        let mut best_id = None;

        for profile in &self.profiles {
            let score = profile.detect(filename, content, document);
            if score > best_score {
                best_score = score;
                best_id = Some(profile.id().to_string());
            }
        }

        if best_score >= 80 {
            best_id
        } else {
            None
        }
    }

    pub fn validate(&self, profile_id: &str, document: &DocumentNode) -> Vec<ProfileDiagnostic> {
        let lower = profile_id.to_ascii_lowercase();
        if let Some(profile) = self.profiles.iter().find(|p| p.id() == lower) {
            profile.validate(document)
        } else {
            Vec::new()
        }
    }
}

static GLOBAL_PROFILES: OnceLock<ProfileRegistry> = OnceLock::new();

pub fn get_profile_registry() -> &'static ProfileRegistry {
    GLOBAL_PROFILES.get_or_init(ProfileRegistry::new)
}
