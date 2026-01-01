use async_trait::async_trait;

use crate::error::Result;
use crate::types::{SearchResult, VectorPoint};

#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn create_collection(&self, name: &str, dimension: usize) -> Result<()>;

    async fn delete_collection(&self, name: &str) -> Result<()>;

    async fn collection_exists(&self, name: &str) -> Result<bool>;

    async fn upsert(&self, collection: &str, points: Vec<VectorPoint>) -> Result<()>;

    async fn search(
        &self,
        collection: &str,
        vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<SearchResult>>;

    async fn search_with_filter(
        &self,
        collection: &str,
        vector: Vec<f32>,
        limit: usize,
        filter: Filter,
    ) -> Result<Vec<SearchResult>>;

    async fn delete(&self, collection: &str, ids: &[&str]) -> Result<()>;

    async fn get(&self, collection: &str, ids: &[&str]) -> Result<Vec<VectorPoint>>;

    async fn count(&self, collection: &str) -> Result<u64>;
}

#[derive(Debug, Clone, Default)]
pub struct Filter {
    conditions: Vec<FilterCondition>,
    must: Vec<Filter>,
    should: Vec<Filter>,
    must_not: Vec<Filter>,
}

impl Filter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn must(mut self, filter: Filter) -> Self {
        self.must.push(filter);
        self
    }

    pub fn should(mut self, filter: Filter) -> Self {
        self.should.push(filter);
        self
    }

    pub fn must_not(mut self, filter: Filter) -> Self {
        self.must_not.push(filter);
        self
    }

    pub fn condition(mut self, condition: FilterCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    pub fn field_eq(field: impl Into<String>, value: serde_json::Value) -> Self {
        Self::new().condition(FilterCondition::Eq {
            field: field.into(),
            value,
        })
    }

    pub fn field_in(field: impl Into<String>, values: Vec<serde_json::Value>) -> Self {
        Self::new().condition(FilterCondition::In {
            field: field.into(),
            values,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.conditions.is_empty()
            && self.must.is_empty()
            && self.should.is_empty()
            && self.must_not.is_empty()
    }

    pub fn conditions(&self) -> &[FilterCondition] {
        &self.conditions
    }

    pub fn must_filters(&self) -> &[Filter] {
        &self.must
    }

    pub fn should_filters(&self) -> &[Filter] {
        &self.should
    }

    pub fn must_not_filters(&self) -> &[Filter] {
        &self.must_not
    }
}

#[derive(Debug, Clone)]
pub enum FilterCondition {
    Eq {
        field: String,
        value: serde_json::Value,
    },
    Ne {
        field: String,
        value: serde_json::Value,
    },
    In {
        field: String,
        values: Vec<serde_json::Value>,
    },
    Gt {
        field: String,
        value: serde_json::Value,
    },
    Gte {
        field: String,
        value: serde_json::Value,
    },
    Lt {
        field: String,
        value: serde_json::Value,
    },
    Lte {
        field: String,
        value: serde_json::Value,
    },
    Range {
        field: String,
        gte: Option<serde_json::Value>,
        lte: Option<serde_json::Value>,
    },
}

#[async_trait]
pub trait KeyValueStore: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;

    async fn set(&self, key: &str, value: Vec<u8>) -> Result<()>;

    async fn set_with_ttl(&self, key: &str, value: Vec<u8>, ttl_secs: u64) -> Result<()>;

    async fn delete(&self, key: &str) -> Result<()>;

    async fn exists(&self, key: &str) -> Result<bool>;

    async fn keys(&self, pattern: &str) -> Result<Vec<String>>;

    async fn clear(&self) -> Result<()>;
}
