//! Dependency graph for assets.
//!
//! Tracks which assets depend on which other assets, enabling
//! topological sort for build order and cycle detection.

use std::collections::{HashMap, HashSet, VecDeque};

use super::types::AssetId;

/// A dependency graph tracking relationships between assets.
#[derive(Debug, Default)]
pub struct DependencyGraph {
    /// Adjacency list: asset -> assets it depends on.
    dependencies: HashMap<AssetId, HashSet<AssetId>>,

    /// Reverse adjacency list: asset -> assets that depend on it.
    dependents: HashMap<AssetId, HashSet<AssetId>>,

    /// All known assets (including those with no dependencies).
    assets: HashSet<AssetId>,
}

impl DependencyGraph {
    /// Create a new empty dependency graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an asset in the graph (even if it has no dependencies).
    pub fn register(&mut self, id: AssetId) {
        self.assets.insert(id);
    }

    /// Add a dependency: `from` depends on `to`.
    ///
    /// Both assets are automatically registered in the graph.
    pub fn add_dependency(&mut self, from: AssetId, to: AssetId) {
        self.assets.insert(from.clone());
        self.assets.insert(to.clone());

        self.dependencies
            .entry(from.clone())
            .or_default()
            .insert(to.clone());

        self.dependents.entry(to).or_default().insert(from);
    }

    /// Get all assets that `id` depends on (direct dependencies).
    pub fn dependencies_of(&self, id: &AssetId) -> impl Iterator<Item = &AssetId> {
        self.dependencies
            .get(id)
            .map(|s| s.iter())
            .into_iter()
            .flatten()
    }

    /// Get all assets that depend on `id` (direct dependents).
    pub fn dependents_of(&self, id: &AssetId) -> impl Iterator<Item = &AssetId> {
        self.dependents
            .get(id)
            .map(|s| s.iter())
            .into_iter()
            .flatten()
    }

    /// Get the number of dependencies for an asset.
    pub fn dependency_count(&self, id: &AssetId) -> usize {
        self.dependencies.get(id).map_or(0, |s| s.len())
    }

    /// Get all registered assets.
    pub fn assets(&self) -> impl Iterator<Item = &AssetId> {
        self.assets.iter()
    }

    /// Get the total number of assets.
    pub fn len(&self) -> usize {
        self.assets.len()
    }

    /// Check if the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }

    /// Perform topological sort using Kahn's algorithm.
    ///
    /// Returns assets in dependency order (dependencies come before dependents).
    /// Returns an error if a cycle is detected, including the cycle path.
    pub fn topological_sort(&self) -> Result<Vec<AssetId>, CycleError> {
        // Calculate in-degree for each asset
        let mut in_degree: HashMap<&AssetId, usize> = HashMap::new();

        for asset in &self.assets {
            in_degree.entry(asset).or_insert(0);
        }

        for deps in self.dependencies.values() {
            for dep in deps {
                // Only count if the dependency is in our asset set
                if self.assets.contains(dep) {
                    // in_degree counts how many assets depend on this one
                    // But for Kahn's algorithm, we need: how many does this asset depend on
                }
            }
        }

        // Actually, let's recalculate correctly:
        // in_degree[x] = number of assets that x depends on (that haven't been processed)
        let mut in_degree: HashMap<&AssetId, usize> = HashMap::new();

        for asset in &self.assets {
            let count = self
                .dependencies
                .get(asset)
                .map_or(0, |deps| deps.iter().filter(|d| self.assets.contains(*d)).count());
            in_degree.insert(asset, count);
        }

        // Queue of assets with no remaining dependencies
        let mut queue: VecDeque<&AssetId> = in_degree
            .iter()
            .filter(|(_, &count)| count == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut result = Vec::with_capacity(self.assets.len());

        while let Some(asset) = queue.pop_front() {
            result.push(asset.clone());

            // For each asset that depends on this one, decrement its in-degree
            if let Some(dependents) = self.dependents.get(asset) {
                for dependent in dependents {
                    if let Some(count) = in_degree.get_mut(dependent) {
                        *count = count.saturating_sub(1);
                        if *count == 0 {
                            queue.push_back(dependent);
                        }
                    }
                }
            }
        }

        // If we didn't process all assets, there's a cycle
        if result.len() != self.assets.len() {
            let cycle = self.find_cycle();
            return Err(CycleError { cycle });
        }

        Ok(result)
    }

    /// Find a cycle in the graph (for error reporting).
    fn find_cycle(&self) -> Vec<AssetId> {
        // DFS to find a cycle
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for start in &self.assets {
            if !visited.contains(start) {
                if let Some(cycle) = self.dfs_find_cycle(start, &mut visited, &mut rec_stack, &mut path) {
                    return cycle;
                }
            }
        }

        Vec::new() // No cycle found (shouldn't happen if topological sort failed)
    }

    fn dfs_find_cycle(
        &self,
        node: &AssetId,
        visited: &mut HashSet<AssetId>,
        rec_stack: &mut HashSet<AssetId>,
        path: &mut Vec<AssetId>,
    ) -> Option<Vec<AssetId>> {
        visited.insert(node.clone());
        rec_stack.insert(node.clone());
        path.push(node.clone());

        if let Some(deps) = self.dependencies.get(node) {
            for dep in deps {
                if !self.assets.contains(dep) {
                    continue; // Skip external dependencies
                }

                if !visited.contains(dep) {
                    if let Some(cycle) = self.dfs_find_cycle(dep, visited, rec_stack, path) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(dep) {
                    // Found a cycle - extract it from the path
                    let cycle_start = path.iter().position(|x| x == dep).unwrap();
                    let mut cycle: Vec<_> = path[cycle_start..].to_vec();
                    cycle.push(dep.clone()); // Complete the cycle
                    return Some(cycle);
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
        None
    }
}

/// Error returned when a cycle is detected in the dependency graph.
#[derive(Debug)]
pub struct CycleError {
    /// The assets involved in the cycle.
    pub cycle: Vec<AssetId>,
}

impl std::fmt::Display for CycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Circular dependency detected: ")?;
        for (i, asset) in self.cycle.iter().enumerate() {
            if i > 0 {
                write!(f, " -> ")?;
            }
            write!(f, "{}", asset)?;
        }
        Ok(())
    }
}

impl std::error::Error for CycleError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let graph = DependencyGraph::new();
        assert!(graph.is_empty());

        let sorted = graph.topological_sort().unwrap();
        assert!(sorted.is_empty());
    }

    #[test]
    fn test_single_asset_no_deps() {
        let mut graph = DependencyGraph::new();
        graph.register(AssetId::shape("wall"));

        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted.len(), 1);
        assert_eq!(sorted[0], AssetId::shape("wall"));
    }

    #[test]
    fn test_linear_dependencies() {
        let mut graph = DependencyGraph::new();

        // C depends on B, B depends on A
        let a = AssetId::palette("base");
        let b = AssetId::shader("default");
        let c = AssetId::shape("wall");

        graph.add_dependency(b.clone(), a.clone()); // shader depends on palette
        graph.add_dependency(c.clone(), b.clone()); // shape depends on shader

        let sorted = graph.topological_sort().unwrap();

        // A should come before B, B before C
        let pos_a = sorted.iter().position(|x| x == &a).unwrap();
        let pos_b = sorted.iter().position(|x| x == &b).unwrap();
        let pos_c = sorted.iter().position(|x| x == &c).unwrap();

        assert!(pos_a < pos_b);
        assert!(pos_b < pos_c);
    }

    #[test]
    fn test_diamond_dependencies() {
        let mut graph = DependencyGraph::new();

        // D depends on B and C, both B and C depend on A
        let a = AssetId::palette("base");
        let b = AssetId::stamp("brick");
        let c = AssetId::stamp("stone");
        let d = AssetId::shape("wall");

        graph.add_dependency(b.clone(), a.clone());
        graph.add_dependency(c.clone(), a.clone());
        graph.add_dependency(d.clone(), b.clone());
        graph.add_dependency(d.clone(), c.clone());

        let sorted = graph.topological_sort().unwrap();

        let pos_a = sorted.iter().position(|x| x == &a).unwrap();
        let pos_b = sorted.iter().position(|x| x == &b).unwrap();
        let pos_c = sorted.iter().position(|x| x == &c).unwrap();
        let pos_d = sorted.iter().position(|x| x == &d).unwrap();

        // A must come first
        assert!(pos_a < pos_b);
        assert!(pos_a < pos_c);
        // D must come last
        assert!(pos_d > pos_b);
        assert!(pos_d > pos_c);
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = DependencyGraph::new();

        let a = AssetId::shape("a");
        let b = AssetId::shape("b");
        let c = AssetId::shape("c");

        // A -> B -> C -> A (cycle)
        graph.add_dependency(a.clone(), b.clone());
        graph.add_dependency(b.clone(), c.clone());
        graph.add_dependency(c.clone(), a.clone());

        let result = graph.topological_sort();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(!err.cycle.is_empty());
        // Cycle should contain all three
        assert!(err.cycle.len() >= 3);
    }

    #[test]
    fn test_self_reference_cycle() {
        let mut graph = DependencyGraph::new();

        let a = AssetId::shape("recursive");
        graph.add_dependency(a.clone(), a.clone());

        let result = graph.topological_sort();
        assert!(result.is_err());
    }

    #[test]
    fn test_dependency_count() {
        let mut graph = DependencyGraph::new();

        let a = AssetId::shape("a");
        let b = AssetId::shape("b");
        let c = AssetId::shape("c");

        graph.add_dependency(c.clone(), a.clone());
        graph.add_dependency(c.clone(), b.clone());

        assert_eq!(graph.dependency_count(&c), 2);
        assert_eq!(graph.dependency_count(&a), 0);
        assert_eq!(graph.dependency_count(&b), 0);
    }

    #[test]
    fn test_dependencies_of() {
        let mut graph = DependencyGraph::new();

        let wall = AssetId::shape("wall");
        let brick = AssetId::stamp("brick");
        let stone = AssetId::stamp("stone");

        graph.add_dependency(wall.clone(), brick.clone());
        graph.add_dependency(wall.clone(), stone.clone());

        let deps: HashSet<_> = graph.dependencies_of(&wall).collect();
        assert!(deps.contains(&brick));
        assert!(deps.contains(&stone));
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn test_dependents_of() {
        let mut graph = DependencyGraph::new();

        let palette = AssetId::palette("base");
        let shader1 = AssetId::shader("dark");
        let shader2 = AssetId::shader("light");

        graph.add_dependency(shader1.clone(), palette.clone());
        graph.add_dependency(shader2.clone(), palette.clone());

        let dependents: HashSet<_> = graph.dependents_of(&palette).collect();
        assert!(dependents.contains(&shader1));
        assert!(dependents.contains(&shader2));
        assert_eq!(dependents.len(), 2);
    }
}
