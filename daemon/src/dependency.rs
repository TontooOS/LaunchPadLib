#![cfg(target_os = "linux")]
#![allow(dead_code)]

use std::collections::{HashMap, HashSet, VecDeque};

use launchpad_lib::types::ServiceConfig;

#[derive(Debug)]
pub struct DependencyGraph {
    services: HashMap<String, ServiceConfig>,
}

impl DependencyGraph {
    pub fn new(services: &HashMap<String, ServiceConfig>) -> Self {
        Self {
            services: services.clone(),
        }
    }

    pub fn resolve_boot_order(&self) -> Result<Vec<String>, String> {
        let mut in_degree: HashMap<String, u32> = HashMap::new();
        let mut dependents: HashMap<String, Vec<String>> = HashMap::new();

        for (name, config) in &self.services {
            in_degree.entry(name.clone()).or_insert(0);
            dependents.entry(name.clone()).or_insert_with(Vec::new);

            for dep in &config.depends_on {
                if !self.services.contains_key(dep) {
                    return Err(format!(
                        "Service '{}' depends on '{}' which does not exist",
                        name, dep
                    ));
                }
                in_degree.entry(name.clone()).and_modify(|d| *d += 1);
                dependents
                    .entry(dep.clone())
                    .and_modify(|l| l.push(name.clone()));
            }
        }

        // Topological sort (Kahn's algorithm)
        let mut queue: VecDeque<String> = VecDeque::new();
        for (name, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(name.clone());
            }
        }

        let mut order = Vec::new();
        while let Some(name) = queue.pop_front() {
            order.push(name.clone());
            if let Some(deps) = dependents.get(&name) {
                for dep in deps {
                    let degree = in_degree.get_mut(dep).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(dep.clone());
                    }
                }
            }
        }

        if order.len() != self.services.len() {
            return Err("Circular dependency detected".to_string());
        }

        Ok(order)
    }

    pub fn get_dependencies(&self, name: &str) -> Vec<String> {
        let mut deps = Vec::new();
        let mut visited = HashSet::new();
        self.collect_deps(name, &mut deps, &mut visited);
        deps
    }

    fn collect_deps(&self, name: &str, deps: &mut Vec<String>, visited: &mut HashSet<String>) {
        if let Some(config) = self.services.get(name) {
            for dep in &config.depends_on {
                if !visited.contains(dep) {
                    visited.insert(dep.clone());
                    self.collect_deps(dep, deps, visited);
                    deps.push(dep.clone());
                }
            }
        }
    }

    pub fn has_circular(&self) -> bool {
        self.resolve_boot_order().is_err()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use launchpad_lib::types::ServiceType;

    fn make_config(name: &str, depends_on: Vec<&str>) -> ServiceConfig {
        ServiceConfig {
            name: name.to_string(),
            execute: format!("/usr/bin/{}", name),
            service_type: ServiceType::Sys,
            user: "root".to_string(),
            depends_on: depends_on.into_iter().map(String::from).collect(),
            restart: true,
        }
    }

    #[test]
    fn test_simple_order() {
        let mut services = HashMap::new();
        services.insert("a".to_string(), make_config("a", vec![]));
        services.insert("b".to_string(), make_config("b", vec!["a"]));
        services.insert("c".to_string(), make_config("c", vec!["b"]));

        let graph = DependencyGraph::new(&services);
        let order = graph.resolve_boot_order().unwrap();
        assert_eq!(order, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_circular() {
        let mut services = HashMap::new();
        services.insert("a".to_string(), make_config("a", vec!["b"]));
        services.insert("b".to_string(), make_config("b", vec!["a"]));

        let graph = DependencyGraph::new(&services);
        assert!(graph.resolve_boot_order().is_err());
    }
}
