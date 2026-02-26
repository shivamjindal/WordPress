use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestRoute {
    pub path: String,
    pub methods: Vec<String>,
}

#[derive(Debug, Default)]
pub struct RestRouteRegistry {
    routes: Vec<RestRoute>,
}

impl RestRouteRegistry {
    pub fn register(&mut self, path: impl Into<String>, methods: &[&str]) {
        self.routes.push(RestRoute {
            path: path.into(),
            methods: methods.iter().map(|method| (*method).to_string()).collect(),
        });
    }

    pub fn all(&self) -> &[RestRoute] {
        &self.routes
    }
}

pub fn core_seed_routes() -> RestRouteRegistry {
    let mut registry = RestRouteRegistry::default();
    registry.register("/wp-json", &["GET"]);
    registry.register("/wp-json/wp/v2/posts", &["GET", "POST"]);
    registry.register("/wp-json/wp/v2/users/me", &["GET"]);
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_seed_routes() {
        let registry = core_seed_routes();
        assert!(!registry.all().is_empty());
    }
}
