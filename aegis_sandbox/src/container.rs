//! Container isolation (Qubes-inspired)
use aegis_core::ContainerId;
use crate::policy::SandboxPolicy;
use tracing::info;

#[derive(Debug)]
pub struct Container {
    pub id: ContainerId,
    pub name: String,
    pub policy: SandboxPolicy,
    pub origin_pattern: Option<String>,
}

impl Container {
    pub fn new(name: &str) -> Self {
        Self {
            id: ContainerId::new(),
            name: name.to_string(),
            policy: SandboxPolicy::default(),
            origin_pattern: None,
        }
    }

    pub fn with_policy(mut self, policy: SandboxPolicy) -> Self {
        self.policy = policy;
        self
    }

    pub fn for_origin(mut self, pattern: &str) -> Self {
        self.origin_pattern = Some(pattern.to_string());
        self
    }

    pub fn matches_origin(&self, origin: &str) -> bool {
        match &self.origin_pattern {
            Some(pattern) => origin.contains(pattern),
            None => true,
        }
    }
}

#[derive(Default)]
pub struct ContainerManager {
    containers: Vec<Container>,
}

impl ContainerManager {
    pub fn new() -> Self { Self::default() }
    
    pub fn add(&mut self, container: Container) {
        info!("Added container: {}", container.name);
        self.containers.push(container);
    }
    
    pub fn get_for_origin(&self, origin: &str) -> Option<&Container> {
        self.containers.iter().find(|c| c.matches_origin(origin))
    }
    
    pub fn default_container() -> Container {
        Container::new("default").with_policy(SandboxPolicy::default())
    }
    
    pub fn banking_container() -> Container {
        Container::new("banking")
            .with_policy(SandboxPolicy::strict())
            .for_origin("bank")
    }
    
    pub fn social_container() -> Container {
        Container::new("social")
            .with_policy(SandboxPolicy { allow_camera: true, allow_microphone: true, ..Default::default() })
            .for_origin("social")
    }
}
