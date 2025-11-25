//! SENA v5.0 - Layer 3: Relationship Data Model (Rust)
//!
//! Inspired by Mayan Mathematics and Calendar Systems
//!
//! The Maya didn't just track time - they tracked relationships between
//! cycles. Their Long Count, Tzolkin, and Haab calendars interlock to
//! create a richer understanding than any single system.
//!
//! Applied to AI: Store relationships, not just values.
//! Context emerges from connections.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};

/// Types of relationships between nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationType {
    /// A causes B
    Causes,
    /// A is caused by B
    CausedBy,
    /// A contains B
    Contains,
    /// A is contained in B
    ContainedIn,
    /// A is related to B (bidirectional)
    RelatedTo,
    /// A depends on B
    DependsOn,
    /// A is a prerequisite for B
    PrerequisiteFor,
    /// A contradicts B
    Contradicts,
    /// A supports B
    Supports,
    /// A is similar to B
    SimilarTo,
    /// A is the opposite of B
    OppositeOf,
    /// A transforms into B
    TransformsInto,
    /// A is an instance of B
    InstanceOf,
    /// A inherits from B
    InheritsFrom,
    /// Custom relationship type
    Custom,
}

impl RelationType {
    /// Get the inverse relationship type
    pub fn inverse(&self) -> Self {
        match self {
            RelationType::Causes => RelationType::CausedBy,
            RelationType::CausedBy => RelationType::Causes,
            RelationType::Contains => RelationType::ContainedIn,
            RelationType::ContainedIn => RelationType::Contains,
            RelationType::DependsOn => RelationType::PrerequisiteFor,
            RelationType::PrerequisiteFor => RelationType::DependsOn,
            RelationType::TransformsInto => RelationType::TransformsInto,
            RelationType::InheritsFrom => RelationType::InheritsFrom,
            // Symmetric relationships
            _ => *self,
        }
    }

    /// Check if relationship is bidirectional
    pub fn is_bidirectional(&self) -> bool {
        matches!(
            self,
            RelationType::RelatedTo
                | RelationType::Contradicts
                | RelationType::SimilarTo
                | RelationType::OppositeOf
        )
    }
}

/// Types of nodes in the relationship graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    Concept,
    Entity,
    Event,
    Action,
    State,
    Property,
    Value,
    Context,
    Custom,
}

/// A node in the relationship graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipNode {
    pub id: String,
    pub name: String,
    pub node_type: NodeType,
    pub properties: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub weight: f64,
    pub tags: Vec<String>,
}

impl RelationshipNode {
    pub fn new(name: impl Into<String>, node_type: NodeType) -> Self {
        let name = name.into();
        let id = Self::generate_id(&name);

        Self {
            id,
            name,
            node_type,
            properties: HashMap::new(),
            created_at: Utc::now(),
            weight: 1.0,
            tags: Vec::new(),
        }
    }

    fn generate_id(name: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(name.as_bytes());
        hasher.update(Utc::now().timestamp_nanos_opt().unwrap_or(0).to_string().as_bytes());
        format!("node_{}", hex::encode(&hasher.finalize()[..8]))
    }

    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// A relationship between two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relation_type: RelationType,
    pub strength: f64,
    pub confidence: f64,
    pub properties: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub evidence: Vec<String>,
    pub bidirectional: bool,
}

impl Relationship {
    pub fn new(
        source_id: impl Into<String>,
        target_id: impl Into<String>,
        relation_type: RelationType,
    ) -> Self {
        let source_id = source_id.into();
        let target_id = target_id.into();
        let id = Self::generate_id(&source_id, &target_id, &relation_type);

        Self {
            id,
            source_id,
            target_id,
            relation_type,
            strength: 1.0,
            confidence: 1.0,
            properties: HashMap::new(),
            created_at: Utc::now(),
            evidence: Vec::new(),
            bidirectional: relation_type.is_bidirectional(),
        }
    }

    fn generate_id(source: &str, target: &str, rel_type: &RelationType) -> String {
        let mut hasher = Sha256::new();
        hasher.update(source.as_bytes());
        hasher.update(target.as_bytes());
        hasher.update(format!("{:?}", rel_type).as_bytes());
        format!("rel_{}", hex::encode(&hasher.finalize()[..8]))
    }

    pub fn with_strength(mut self, strength: f64) -> Self {
        self.strength = strength.clamp(0.0, 1.0);
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn with_evidence(mut self, evidence: impl Into<String>) -> Self {
        self.evidence.push(evidence.into());
        self
    }

    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }
}

/// A path through the relationship graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipPath {
    pub nodes: Vec<String>,
    pub relationships: Vec<String>,
    pub total_strength: f64,
    pub total_confidence: f64,
    pub path_length: usize,
}

impl RelationshipPath {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            relationships: Vec::new(),
            total_strength: 1.0,
            total_confidence: 1.0,
            path_length: 0,
        }
    }

    pub fn add_step(&mut self, node_id: String, relationship_id: String, strength: f64, confidence: f64) {
        self.nodes.push(node_id);
        self.relationships.push(relationship_id);
        self.total_strength *= strength;
        self.total_confidence *= confidence;
        self.path_length += 1;
    }
}

impl Default for RelationshipPath {
    fn default() -> Self {
        Self::new()
    }
}

/// A cluster of related nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipCluster {
    pub id: String,
    pub name: String,
    pub node_ids: HashSet<String>,
    pub central_node_id: Option<String>,
    pub cohesion_score: f64,
    pub created_at: DateTime<Utc>,
}

impl RelationshipCluster {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let mut hasher = Sha256::new();
        hasher.update(name.as_bytes());
        hasher.update(Utc::now().timestamp().to_string().as_bytes());
        let id = format!("cluster_{}", hex::encode(&hasher.finalize()[..8]));

        Self {
            id,
            name,
            node_ids: HashSet::new(),
            central_node_id: None,
            cohesion_score: 0.0,
            created_at: Utc::now(),
        }
    }

    pub fn add_node(&mut self, node_id: impl Into<String>) {
        self.node_ids.insert(node_id.into());
    }
}

/// Query for finding relationships
#[derive(Debug, Clone)]
pub struct RelationshipQuery {
    pub source_type: Option<NodeType>,
    pub target_type: Option<NodeType>,
    pub relation_types: Vec<RelationType>,
    pub min_strength: f64,
    pub min_confidence: f64,
    pub max_depth: usize,
    pub tags: Vec<String>,
}

impl Default for RelationshipQuery {
    fn default() -> Self {
        Self {
            source_type: None,
            target_type: None,
            relation_types: Vec::new(),
            min_strength: 0.0,
            min_confidence: 0.0,
            max_depth: 5,
            tags: Vec::new(),
        }
    }
}

impl RelationshipQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_relation_type(mut self, rel_type: RelationType) -> Self {
        self.relation_types.push(rel_type);
        self
    }

    pub fn with_min_strength(mut self, strength: f64) -> Self {
        self.min_strength = strength;
        self
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }
}

/// The main Relationship Data Model engine
pub struct RelationshipDataModel {
    nodes: HashMap<String, RelationshipNode>,
    relationships: HashMap<String, Relationship>,
    outgoing: HashMap<String, Vec<String>>,
    incoming: HashMap<String, Vec<String>>,
    clusters: HashMap<String, RelationshipCluster>,
    node_by_name: HashMap<String, String>,
}

impl Default for RelationshipDataModel {
    fn default() -> Self {
        Self::new()
    }
}

impl RelationshipDataModel {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            relationships: HashMap::new(),
            outgoing: HashMap::new(),
            incoming: HashMap::new(),
            clusters: HashMap::new(),
            node_by_name: HashMap::new(),
        }
    }

    /// Add a node to the model
    pub fn add_node(&mut self, node: RelationshipNode) -> String {
        let id = node.id.clone();
        self.node_by_name.insert(node.name.clone(), id.clone());
        self.outgoing.entry(id.clone()).or_default();
        self.incoming.entry(id.clone()).or_default();
        self.nodes.insert(id.clone(), node);
        id
    }

    /// Create and add a node
    pub fn create_node(&mut self, name: impl Into<String>, node_type: NodeType) -> String {
        let node = RelationshipNode::new(name, node_type);
        self.add_node(node)
    }

    /// Get a node by ID
    pub fn get_node(&self, id: &str) -> Option<&RelationshipNode> {
        self.nodes.get(id)
    }

    /// Get a node by name
    pub fn get_node_by_name(&self, name: &str) -> Option<&RelationshipNode> {
        self.node_by_name
            .get(name)
            .and_then(|id| self.nodes.get(id))
    }

    /// Add a relationship between nodes
    pub fn add_relationship(&mut self, relationship: Relationship) -> String {
        let id = relationship.id.clone();

        // Add to adjacency lists
        self.outgoing
            .entry(relationship.source_id.clone())
            .or_default()
            .push(id.clone());
        self.incoming
            .entry(relationship.target_id.clone())
            .or_default()
            .push(id.clone());

        // If bidirectional, add reverse edges too
        if relationship.bidirectional {
            self.outgoing
                .entry(relationship.target_id.clone())
                .or_default()
                .push(id.clone());
            self.incoming
                .entry(relationship.source_id.clone())
                .or_default()
                .push(id.clone());
        }

        self.relationships.insert(id.clone(), relationship);
        id
    }

    /// Create and add a relationship
    pub fn create_relationship(
        &mut self,
        source_id: impl Into<String>,
        target_id: impl Into<String>,
        relation_type: RelationType,
    ) -> String {
        let rel = Relationship::new(source_id, target_id, relation_type);
        self.add_relationship(rel)
    }

    /// Relate two nodes by name
    pub fn relate(
        &mut self,
        source_name: &str,
        target_name: &str,
        relation_type: RelationType,
    ) -> Option<String> {
        let source_id = self.node_by_name.get(source_name)?.clone();
        let target_id = self.node_by_name.get(target_name)?.clone();
        Some(self.create_relationship(source_id, target_id, relation_type))
    }

    /// Get a relationship by ID
    pub fn get_relationship(&self, id: &str) -> Option<&Relationship> {
        self.relationships.get(id)
    }

    /// Get all outgoing relationships from a node
    pub fn get_outgoing(&self, node_id: &str) -> Vec<&Relationship> {
        self.outgoing
            .get(node_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.relationships.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all incoming relationships to a node
    pub fn get_incoming(&self, node_id: &str) -> Vec<&Relationship> {
        self.incoming
            .get(node_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.relationships.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find paths between two nodes using BFS
    pub fn find_path(
        &self,
        start_id: &str,
        end_id: &str,
        max_depth: usize,
    ) -> Option<RelationshipPath> {
        if !self.nodes.contains_key(start_id) || !self.nodes.contains_key(end_id) {
            return None;
        }

        if start_id == end_id {
            let mut path = RelationshipPath::new();
            path.nodes.push(start_id.to_string());
            return Some(path);
        }

        // BFS to find shortest path
        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<(String, RelationshipPath)> = VecDeque::new();

        let mut initial_path = RelationshipPath::new();
        initial_path.nodes.push(start_id.to_string());
        queue.push_back((start_id.to_string(), initial_path));
        visited.insert(start_id.to_string());

        while let Some((current_id, mut current_path)) = queue.pop_front() {
            if current_path.path_length >= max_depth {
                continue;
            }

            for rel in self.get_outgoing(&current_id) {
                let next_id = &rel.target_id;

                if next_id == end_id {
                    current_path.add_step(
                        next_id.clone(),
                        rel.id.clone(),
                        rel.strength,
                        rel.confidence,
                    );
                    return Some(current_path);
                }

                if !visited.contains(next_id) {
                    visited.insert(next_id.clone());
                    let mut new_path = current_path.clone();
                    new_path.add_step(
                        next_id.clone(),
                        rel.id.clone(),
                        rel.strength,
                        rel.confidence,
                    );
                    queue.push_back((next_id.clone(), new_path));
                }
            }
        }

        None
    }

    /// Find all nodes related to a given node within a depth
    pub fn find_related(&self, node_id: &str, max_depth: usize) -> Vec<(String, usize)> {
        let mut related = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<(String, usize)> = VecDeque::new();

        queue.push_back((node_id.to_string(), 0));
        visited.insert(node_id.to_string());

        while let Some((current_id, depth)) = queue.pop_front() {
            if depth > 0 {
                related.push((current_id.clone(), depth));
            }

            if depth >= max_depth {
                continue;
            }

            for rel in self.get_outgoing(&current_id) {
                if !visited.contains(&rel.target_id) {
                    visited.insert(rel.target_id.clone());
                    queue.push_back((rel.target_id.clone(), depth + 1));
                }
            }
        }

        related
    }

    /// Query relationships based on criteria
    pub fn query(&self, query: &RelationshipQuery) -> Vec<&Relationship> {
        self.relationships
            .values()
            .filter(|rel| {
                // Filter by relation type
                if !query.relation_types.is_empty()
                    && !query.relation_types.contains(&rel.relation_type)
                {
                    return false;
                }

                // Filter by strength
                if rel.strength < query.min_strength {
                    return false;
                }

                // Filter by confidence
                if rel.confidence < query.min_confidence {
                    return false;
                }

                // Filter by source type
                if let Some(source_type) = query.source_type {
                    if let Some(source) = self.nodes.get(&rel.source_id) {
                        if source.node_type != source_type {
                            return false;
                        }
                    }
                }

                // Filter by target type
                if let Some(target_type) = query.target_type {
                    if let Some(target) = self.nodes.get(&rel.target_id) {
                        if target.node_type != target_type {
                            return false;
                        }
                    }
                }

                true
            })
            .collect()
    }

    /// Infer new relationships based on existing patterns
    pub fn infer_relationship(
        &self,
        source_id: &str,
        target_id: &str,
    ) -> Vec<(RelationType, f64)> {
        let mut inferences = Vec::new();

        // Check for transitive relationships
        // If A -> B -> C, infer A -> C
        if let Some(path) = self.find_path(source_id, target_id, 2) {
            if path.path_length == 2 {
                // Get the relationship types in the path
                let rel1 = self.get_relationship(&path.relationships[0]);
                let rel2 = self.get_relationship(&path.relationships[1]);

                if let (Some(r1), Some(r2)) = (rel1, rel2) {
                    // Transitive inference rules
                    let inferred = match (r1.relation_type, r2.relation_type) {
                        (RelationType::Causes, RelationType::Causes) => {
                            Some((RelationType::Causes, path.total_confidence * 0.8))
                        }
                        (RelationType::Contains, RelationType::Contains) => {
                            Some((RelationType::Contains, path.total_confidence * 0.9))
                        }
                        (RelationType::DependsOn, RelationType::DependsOn) => {
                            Some((RelationType::DependsOn, path.total_confidence * 0.7))
                        }
                        (RelationType::InheritsFrom, RelationType::InheritsFrom) => {
                            Some((RelationType::InheritsFrom, path.total_confidence * 0.95))
                        }
                        _ => None,
                    };

                    if let Some((rel_type, confidence)) = inferred {
                        inferences.push((rel_type, confidence));
                    }
                }
            }
        }

        // Check for similar neighbors
        let source_neighbors: HashSet<String> = self
            .get_outgoing(source_id)
            .iter()
            .map(|r| r.target_id.clone())
            .collect();

        let target_neighbors: HashSet<String> = self
            .get_outgoing(target_id)
            .iter()
            .map(|r| r.target_id.clone())
            .collect();

        let common = source_neighbors.intersection(&target_neighbors).count();
        if common > 0 && !source_neighbors.is_empty() && !target_neighbors.is_empty() {
            let jaccard =
                common as f64 / (source_neighbors.len() + target_neighbors.len() - common) as f64;
            if jaccard > 0.3 {
                inferences.push((RelationType::SimilarTo, jaccard));
            }
        }

        inferences
    }

    /// Create a cluster from related nodes
    pub fn create_cluster(&mut self, name: impl Into<String>, seed_node_id: &str, depth: usize) -> Option<String> {
        if !self.nodes.contains_key(seed_node_id) {
            return None;
        }

        let mut cluster = RelationshipCluster::new(name);
        cluster.central_node_id = Some(seed_node_id.to_string());

        // Find all related nodes
        let related = self.find_related(seed_node_id, depth);
        cluster.add_node(seed_node_id);
        for (node_id, _) in related {
            cluster.add_node(node_id);
        }

        // Calculate cohesion (internal edge density)
        let node_count = cluster.node_ids.len();
        if node_count > 1 {
            let mut internal_edges = 0;
            for rel in self.relationships.values() {
                if cluster.node_ids.contains(&rel.source_id)
                    && cluster.node_ids.contains(&rel.target_id)
                {
                    internal_edges += 1;
                }
            }
            let max_edges = node_count * (node_count - 1);
            cluster.cohesion_score = internal_edges as f64 / max_edges as f64;
        }

        let id = cluster.id.clone();
        self.clusters.insert(id.clone(), cluster);
        Some(id)
    }

    /// Get statistics about the model
    pub fn get_statistics(&self) -> ModelStatistics {
        let mut type_counts: HashMap<RelationType, usize> = HashMap::new();
        for rel in self.relationships.values() {
            *type_counts.entry(rel.relation_type).or_insert(0) += 1;
        }

        let mut node_type_counts: HashMap<NodeType, usize> = HashMap::new();
        for node in self.nodes.values() {
            *node_type_counts.entry(node.node_type).or_insert(0) += 1;
        }

        // Calculate average degree
        let total_edges: usize = self.outgoing.values().map(|v| v.len()).sum();
        let avg_degree = if self.nodes.is_empty() {
            0.0
        } else {
            total_edges as f64 / self.nodes.len() as f64
        };

        ModelStatistics {
            node_count: self.nodes.len(),
            relationship_count: self.relationships.len(),
            cluster_count: self.clusters.len(),
            relationship_types: type_counts,
            node_types: node_type_counts,
            average_degree: avg_degree,
        }
    }

    /// Get all nodes
    pub fn get_all_nodes(&self) -> Vec<&RelationshipNode> {
        self.nodes.values().collect()
    }

    /// Get all relationships
    pub fn get_all_relationships(&self) -> Vec<&Relationship> {
        self.relationships.values().collect()
    }
}

/// Statistics about the relationship model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStatistics {
    pub node_count: usize,
    pub relationship_count: usize,
    pub cluster_count: usize,
    pub relationship_types: HashMap<RelationType, usize>,
    pub node_types: HashMap<NodeType, usize>,
    pub average_degree: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = RelationshipNode::new("TestConcept", NodeType::Concept)
            .with_property("key", "value")
            .with_weight(0.8)
            .with_tag("important");

        assert!(node.id.starts_with("node_"));
        assert_eq!(node.name, "TestConcept");
        assert_eq!(node.node_type, NodeType::Concept);
        assert_eq!(node.weight, 0.8);
        assert!(node.properties.contains_key("key"));
        assert!(node.tags.contains(&"important".to_string()));
    }

    #[test]
    fn test_relationship_creation() {
        let rel = Relationship::new("node1", "node2", RelationType::Causes)
            .with_strength(0.9)
            .with_confidence(0.8)
            .with_evidence("Test evidence");

        assert!(rel.id.starts_with("rel_"));
        assert_eq!(rel.source_id, "node1");
        assert_eq!(rel.target_id, "node2");
        assert_eq!(rel.relation_type, RelationType::Causes);
        assert_eq!(rel.strength, 0.9);
        assert_eq!(rel.confidence, 0.8);
    }

    #[test]
    fn test_model_add_nodes_and_relationships() {
        let mut model = RelationshipDataModel::new();

        let node1_id = model.create_node("Concept A", NodeType::Concept);
        let node2_id = model.create_node("Concept B", NodeType::Concept);

        let rel_id = model.create_relationship(&node1_id, &node2_id, RelationType::Causes);

        assert!(model.get_node(&node1_id).is_some());
        assert!(model.get_node(&node2_id).is_some());
        assert!(model.get_relationship(&rel_id).is_some());
    }

    #[test]
    fn test_find_path() {
        let mut model = RelationshipDataModel::new();

        let a = model.create_node("A", NodeType::Concept);
        let b = model.create_node("B", NodeType::Concept);
        let c = model.create_node("C", NodeType::Concept);

        model.create_relationship(&a, &b, RelationType::Causes);
        model.create_relationship(&b, &c, RelationType::Causes);

        let path = model.find_path(&a, &c, 5);
        assert!(path.is_some());
        assert_eq!(path.unwrap().path_length, 2);
    }

    #[test]
    fn test_find_related() {
        let mut model = RelationshipDataModel::new();

        let center = model.create_node("Center", NodeType::Concept);
        let n1 = model.create_node("N1", NodeType::Concept);
        let n2 = model.create_node("N2", NodeType::Concept);

        model.create_relationship(&center, &n1, RelationType::RelatedTo);
        model.create_relationship(&center, &n2, RelationType::RelatedTo);

        let related = model.find_related(&center, 1);
        assert_eq!(related.len(), 2);
    }

    #[test]
    fn test_model_statistics() {
        let mut model = RelationshipDataModel::new();

        model.create_node("A", NodeType::Concept);
        model.create_node("B", NodeType::Entity);
        let a = model.node_by_name.get("A").unwrap().clone();
        let b = model.node_by_name.get("B").unwrap().clone();
        model.create_relationship(a, b, RelationType::RelatedTo);

        let stats = model.get_statistics();
        assert_eq!(stats.node_count, 2);
        assert_eq!(stats.relationship_count, 1);
    }
}
