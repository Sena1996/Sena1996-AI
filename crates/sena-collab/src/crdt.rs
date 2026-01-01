use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use uuid::Uuid;

pub type NodeId = Uuid;
pub type Timestamp = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
pub struct LWWRegister<T> {
    value: T,
    timestamp: Timestamp,
    node_id: NodeId,
}

impl<T: Clone + Default> LWWRegister<T> {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            value: T::default(),
            timestamp: 0,
            node_id,
        }
    }

    pub fn with_value(value: T, node_id: NodeId) -> Self {
        Self {
            value,
            timestamp: Self::now(),
            node_id,
        }
    }

    pub fn set(&mut self, value: T) {
        self.value = value;
        self.timestamp = Self::now();
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn merge(&mut self, other: &Self) {
        if other.timestamp > self.timestamp
            || (other.timestamp == self.timestamp && other.node_id > self.node_id)
        {
            self.value = other.value.clone();
            self.timestamp = other.timestamp;
        }
    }

    fn now() -> Timestamp {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCounter {
    counts: HashMap<NodeId, u64>,
}

impl GCounter {
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }

    pub fn increment(&mut self, node_id: NodeId) {
        *self.counts.entry(node_id).or_insert(0) += 1;
    }

    pub fn increment_by(&mut self, node_id: NodeId, amount: u64) {
        *self.counts.entry(node_id).or_insert(0) += amount;
    }

    pub fn value(&self) -> u64 {
        self.counts.values().sum()
    }

    pub fn merge(&mut self, other: &Self) {
        for (node_id, &count) in &other.counts {
            self.counts
                .entry(*node_id)
                .and_modify(|c| *c = (*c).max(count))
                .or_insert(count);
        }
    }
}

impl Default for GCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PNCounter {
    positive: GCounter,
    negative: GCounter,
}

impl PNCounter {
    pub fn new() -> Self {
        Self {
            positive: GCounter::new(),
            negative: GCounter::new(),
        }
    }

    pub fn increment(&mut self, node_id: NodeId) {
        self.positive.increment(node_id);
    }

    pub fn decrement(&mut self, node_id: NodeId) {
        self.negative.increment(node_id);
    }

    pub fn value(&self) -> i64 {
        self.positive.value() as i64 - self.negative.value() as i64
    }

    pub fn merge(&mut self, other: &Self) {
        self.positive.merge(&other.positive);
        self.negative.merge(&other.negative);
    }
}

impl Default for PNCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize + Eq + Hash",
    deserialize = "T: Deserialize<'de> + Eq + Hash"
))]
pub struct ORSet<T> {
    elements: HashMap<T, HashSet<(NodeId, Timestamp)>>,
    tombstones: HashMap<T, HashSet<(NodeId, Timestamp)>>,
}

impl<T: Clone + Eq + Hash> ORSet<T> {
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
            tombstones: HashMap::new(),
        }
    }

    pub fn add(&mut self, value: T, node_id: NodeId) {
        let tag = (node_id, Self::now());
        self.elements.entry(value).or_default().insert(tag);
    }

    pub fn remove(&mut self, value: &T) {
        if let Some(tags) = self.elements.remove(value) {
            self.tombstones.entry(value.clone()).or_default().extend(tags);
        }
    }

    pub fn contains(&self, value: &T) -> bool {
        self.elements.get(value).map(|t| !t.is_empty()).unwrap_or(false)
    }

    pub fn values(&self) -> Vec<&T> {
        self.elements
            .iter()
            .filter(|(_, tags)| !tags.is_empty())
            .map(|(v, _)| v)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.elements
            .iter()
            .filter(|(_, tags)| !tags.is_empty())
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn merge(&mut self, other: &Self) {
        for (value, tags) in &other.elements {
            let entry = self.elements.entry(value.clone()).or_default();
            entry.extend(tags.clone());
        }

        for (value, tombstones) in &other.tombstones {
            let local_tombstones = self.tombstones.entry(value.clone()).or_default();
            local_tombstones.extend(tombstones.clone());
        }

        for (value, tombstones) in &self.tombstones {
            if let Some(tags) = self.elements.get_mut(value) {
                for tombstone in tombstones {
                    tags.remove(tombstone);
                }
            }
        }
    }

    fn now() -> Timestamp {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0)
    }
}

impl<T: Clone + Eq + Hash> Default for ORSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(
    serialize = "K: Serialize + Eq + Hash, V: Serialize",
    deserialize = "K: Deserialize<'de> + Eq + Hash, V: Deserialize<'de>"
))]
pub struct LWWMap<K, V> {
    entries: HashMap<K, LWWRegister<Option<V>>>,
    node_id: NodeId,
}

impl<K: Clone + Eq + Hash, V: Clone + Default> LWWMap<K, V> {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            entries: HashMap::new(),
            node_id,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let register = self.entries.entry(key).or_insert_with(|| {
            LWWRegister::new(self.node_id)
        });
        register.set(Some(value));
    }

    pub fn remove(&mut self, key: &K) {
        if let Some(register) = self.entries.get_mut(key) {
            register.set(None);
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.entries.get(key).and_then(|r| r.get().as_ref())
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.entries.get(key).map(|r| r.get().is_some()).unwrap_or(false)
    }

    pub fn keys(&self) -> Vec<&K> {
        self.entries
            .iter()
            .filter(|(_, r)| r.get().is_some())
            .map(|(k, _)| k)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.entries.iter().filter(|(_, r)| r.get().is_some()).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn merge(&mut self, other: &Self) {
        for (key, other_register) in &other.entries {
            self.entries
                .entry(key.clone())
                .and_modify(|r| r.merge(other_register))
                .or_insert_with(|| other_register.clone());
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorClock {
    clocks: HashMap<NodeId, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        Self {
            clocks: HashMap::new(),
        }
    }

    pub fn increment(&mut self, node_id: NodeId) {
        *self.clocks.entry(node_id).or_insert(0) += 1;
    }

    pub fn get(&self, node_id: &NodeId) -> u64 {
        *self.clocks.get(node_id).unwrap_or(&0)
    }

    pub fn merge(&mut self, other: &Self) {
        for (node_id, &clock) in &other.clocks {
            self.clocks
                .entry(*node_id)
                .and_modify(|c| *c = (*c).max(clock))
                .or_insert(clock);
        }
    }

    pub fn happens_before(&self, other: &Self) -> bool {
        let mut dominated = false;
        for (node_id, &clock) in &self.clocks {
            let other_clock = other.get(node_id);
            if clock > other_clock {
                return false;
            }
            if clock < other_clock {
                dominated = true;
            }
        }
        for (node_id, &clock) in &other.clocks {
            if !self.clocks.contains_key(node_id) && clock > 0 {
                dominated = true;
            }
        }
        dominated
    }

    pub fn dominates(&self, other: &Self) -> bool {
        for (node_id, &clock) in &other.clocks {
            if self.get(node_id) < clock {
                return false;
            }
        }
        true
    }

    pub fn concurrent_with(&self, other: &Self) -> bool {
        !self.happens_before(other) && !other.happens_before(self)
    }
}

impl Default for VectorClock {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
pub struct CrdtState<T> {
    pub data: T,
    pub clock: VectorClock,
    pub node_id: NodeId,
}

impl<T: Clone> CrdtState<T> {
    pub fn new(data: T, node_id: NodeId) -> Self {
        Self {
            data,
            clock: VectorClock::new(),
            node_id,
        }
    }

    pub fn update<F>(&mut self, f: F)
    where
        F: FnOnce(&mut T),
    {
        f(&mut self.data);
        self.clock.increment(self.node_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcounter() {
        let mut c1 = GCounter::new();
        let mut c2 = GCounter::new();
        let n1 = Uuid::new_v4();
        let n2 = Uuid::new_v4();

        c1.increment(n1);
        c1.increment(n1);
        c2.increment(n2);

        c1.merge(&c2);
        assert_eq!(c1.value(), 3);
    }

    #[test]
    fn test_pncounter() {
        let mut c1 = PNCounter::new();
        let n1 = Uuid::new_v4();

        c1.increment(n1);
        c1.increment(n1);
        c1.decrement(n1);

        assert_eq!(c1.value(), 1);
    }

    #[test]
    fn test_orset() {
        let mut s1: ORSet<String> = ORSet::new();
        let mut s2: ORSet<String> = ORSet::new();
        let n1 = Uuid::new_v4();
        let n2 = Uuid::new_v4();

        s1.add("a".to_string(), n1);
        s2.add("b".to_string(), n2);

        s1.merge(&s2);
        assert!(s1.contains(&"a".to_string()));
        assert!(s1.contains(&"b".to_string()));
        assert_eq!(s1.len(), 2);
    }

    #[test]
    fn test_lww_register() {
        let n1 = Uuid::new_v4();
        let n2 = Uuid::new_v4();
        let mut r1: LWWRegister<i32> = LWWRegister::with_value(10, n1);

        std::thread::sleep(std::time::Duration::from_millis(1));
        let r2: LWWRegister<i32> = LWWRegister::with_value(20, n2);

        r1.merge(&r2);
        assert_eq!(*r1.get(), 20);
    }

    #[test]
    fn test_vector_clock() {
        let n1 = Uuid::new_v4();
        let n2 = Uuid::new_v4();
        let mut vc1 = VectorClock::new();
        let mut vc2 = VectorClock::new();

        vc1.increment(n1);
        vc2.increment(n1);
        vc2.increment(n2);

        assert!(vc1.happens_before(&vc2));
        assert!(!vc2.happens_before(&vc1));
    }
}
