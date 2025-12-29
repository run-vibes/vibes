//! Route registry for plugin HTTP routes

use std::collections::HashMap;
use vibes_plugin_api::{HttpMethod, RouteSpec};

/// Registry of all plugin HTTP routes
pub struct RouteRegistry {
    /// Registered routes with compiled path matchers
    routes: Vec<RegisteredPluginRoute>,
}

/// A route registered by a plugin
pub struct RegisteredPluginRoute {
    /// Name of the plugin that owns this route
    pub plugin_name: String,
    /// Route specification
    pub spec: RouteSpec,
    /// Full path including /api/<plugin>/ prefix
    pub full_path: String,
    /// Compiled path matcher
    matcher: PathMatcher,
}

/// Simple path matcher supporting :param patterns
struct PathMatcher {
    segments: Vec<PathSegment>,
}

enum PathSegment {
    Literal(String),
    Param(String),
}

impl PathMatcher {
    fn new(path: &str) -> Self {
        let segments = path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| {
                if let Some(name) = s.strip_prefix(':') {
                    PathSegment::Param(name.to_string())
                } else {
                    PathSegment::Literal(s.to_string())
                }
            })
            .collect();

        Self { segments }
    }

    fn match_path(&self, path: &str) -> Option<HashMap<String, String>> {
        let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        if path_parts.len() != self.segments.len() {
            return None;
        }

        let mut params = HashMap::new();

        for (segment, part) in self.segments.iter().zip(path_parts.iter()) {
            match segment {
                PathSegment::Literal(expected) => {
                    if expected != *part {
                        return None;
                    }
                }
                PathSegment::Param(name) => {
                    params.insert(name.clone(), (*part).to_string());
                }
            }
        }

        Some(params)
    }
}

impl RouteRegistry {
    /// Create a new empty route registry
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    /// Register routes for a plugin
    ///
    /// Routes are prefixed with /api/<plugin>/
    pub fn register(&mut self, plugin_name: &str, routes: Vec<RouteSpec>) {
        for spec in routes {
            let full_path = format!("/api/{}{}", plugin_name, spec.path);
            let matcher = PathMatcher::new(&full_path);

            self.routes.push(RegisteredPluginRoute {
                plugin_name: plugin_name.to_string(),
                spec,
                full_path,
                matcher,
            });
        }
    }

    /// Check if a route would conflict with existing registrations
    ///
    /// Returns the name of the plugin that owns the conflicting route, if any
    pub fn check_conflict(&self, plugin_name: &str, spec: &RouteSpec) -> Option<&str> {
        let full_path = format!("/api/{}{}", plugin_name, spec.path);

        self.routes
            .iter()
            .find(|r| r.spec.method == spec.method && r.full_path == full_path)
            .map(|r| r.plugin_name.as_str())
    }

    /// Find a route matching the given method and path
    ///
    /// Returns the route and extracted path parameters
    pub fn match_route(
        &self,
        method: HttpMethod,
        path: &str,
    ) -> Option<(&RegisteredPluginRoute, HashMap<String, String>)> {
        for route in &self.routes {
            if route.spec.method == method
                && let Some(params) = route.matcher.match_path(path)
            {
                return Some((route, params));
            }
        }
        None
    }

    /// Unregister all routes for a plugin
    pub fn unregister(&mut self, plugin_name: &str) {
        self.routes.retain(|r| r.plugin_name != plugin_name);
    }
}

impl Default for RouteRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_routes() {
        let mut registry = RouteRegistry::new();

        let routes = vec![RouteSpec {
            method: HttpMethod::Get,
            path: "/policy".into(),
        }];

        registry.register("groove", routes);

        let (route, params) = registry
            .match_route(HttpMethod::Get, "/api/groove/policy")
            .unwrap();
        assert_eq!(route.plugin_name, "groove");
        assert!(params.is_empty());
    }

    #[test]
    fn test_path_parameter_extraction() {
        let mut registry = RouteRegistry::new();

        registry.register(
            "groove",
            vec![RouteSpec {
                method: HttpMethod::Get,
                path: "/quarantine/:id".into(),
            }],
        );

        let (route, params) = registry
            .match_route(HttpMethod::Get, "/api/groove/quarantine/123")
            .unwrap();
        assert_eq!(route.plugin_name, "groove");
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_no_match_wrong_method() {
        let mut registry = RouteRegistry::new();

        registry.register(
            "groove",
            vec![RouteSpec {
                method: HttpMethod::Get,
                path: "/policy".into(),
            }],
        );

        let result = registry.match_route(HttpMethod::Post, "/api/groove/policy");
        assert!(result.is_none());
    }

    #[test]
    fn test_no_match_wrong_path() {
        let mut registry = RouteRegistry::new();

        registry.register(
            "groove",
            vec![RouteSpec {
                method: HttpMethod::Get,
                path: "/policy".into(),
            }],
        );

        let result = registry.match_route(HttpMethod::Get, "/api/groove/other");
        assert!(result.is_none());
    }

    #[test]
    fn test_check_conflict_finds_existing() {
        let mut registry = RouteRegistry::new();

        let routes = vec![RouteSpec {
            method: HttpMethod::Post,
            path: "/action".into(),
        }];
        registry.register("plugin-a", routes);

        let new_spec = RouteSpec {
            method: HttpMethod::Post,
            path: "/action".into(),
        };

        // Same plugin registering same route conflicts
        let conflict = registry.check_conflict("plugin-a", &new_spec);
        assert_eq!(conflict, Some("plugin-a"));
    }

    #[test]
    fn test_check_conflict_different_plugin_no_conflict() {
        let mut registry = RouteRegistry::new();

        let routes = vec![RouteSpec {
            method: HttpMethod::Post,
            path: "/action".into(),
        }];
        registry.register("plugin-a", routes);

        let new_spec = RouteSpec {
            method: HttpMethod::Post,
            path: "/action".into(),
        };

        // Different plugin, same path - no conflict because namespaced
        let conflict = registry.check_conflict("plugin-b", &new_spec);
        assert!(conflict.is_none());
    }

    #[test]
    fn test_check_conflict_different_method_no_conflict() {
        let mut registry = RouteRegistry::new();

        let routes = vec![RouteSpec {
            method: HttpMethod::Get,
            path: "/resource".into(),
        }];
        registry.register("plugin-a", routes);

        let new_spec = RouteSpec {
            method: HttpMethod::Post,
            path: "/resource".into(),
        };

        // Same plugin, same path, different method - no conflict
        let conflict = registry.check_conflict("plugin-a", &new_spec);
        assert!(conflict.is_none());
    }

    #[test]
    fn test_unregister_removes_all_plugin_routes() {
        let mut registry = RouteRegistry::new();

        registry.register(
            "test-plugin",
            vec![
                RouteSpec {
                    method: HttpMethod::Get,
                    path: "/route1".into(),
                },
                RouteSpec {
                    method: HttpMethod::Post,
                    path: "/route2".into(),
                },
            ],
        );

        // Verify routes exist
        assert!(
            registry
                .match_route(HttpMethod::Get, "/api/test-plugin/route1")
                .is_some()
        );
        assert!(
            registry
                .match_route(HttpMethod::Post, "/api/test-plugin/route2")
                .is_some()
        );

        // Unregister
        registry.unregister("test-plugin");

        // Verify routes are gone
        assert!(
            registry
                .match_route(HttpMethod::Get, "/api/test-plugin/route1")
                .is_none()
        );
        assert!(
            registry
                .match_route(HttpMethod::Post, "/api/test-plugin/route2")
                .is_none()
        );
    }

    #[test]
    fn test_multiple_path_parameters() {
        let mut registry = RouteRegistry::new();

        registry.register(
            "groove",
            vec![RouteSpec {
                method: HttpMethod::Get,
                path: "/sessions/:session_id/messages/:msg_id".into(),
            }],
        );

        let (route, params) = registry
            .match_route(HttpMethod::Get, "/api/groove/sessions/abc/messages/123")
            .unwrap();

        assert_eq!(route.plugin_name, "groove");
        assert_eq!(params.get("session_id"), Some(&"abc".to_string()));
        assert_eq!(params.get("msg_id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_default_creates_empty_registry() {
        let registry = RouteRegistry::default();
        assert!(registry.match_route(HttpMethod::Get, "/any/path").is_none());
    }
}
