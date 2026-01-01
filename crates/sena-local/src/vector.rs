use async_trait::async_trait;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, DeletePointsBuilder, Distance, PointId, PointStruct,
    GetPointsBuilder, ScrollPointsBuilder, SearchPointsBuilder, UpsertPointsBuilder,
    Value, VectorParamsBuilder,
};
use qdrant_client::Qdrant;
use sena_core::{Error, Filter, Result, SearchResult, VectorPoint, VectorStore};
use std::collections::HashMap;

pub struct QdrantStore {
    client: Qdrant,
    _dimension: usize,
}

impl QdrantStore {
    pub async fn new(config: &sena_core::config::VectorStoreConfig, dimension: usize) -> Result<Self> {
        let client = Qdrant::from_url(&config.url)
            .build()
            .map_err(|e| Error::vector_store(format!("failed to connect: {}", e)))?;

        Ok(Self { client, _dimension: dimension })
    }

    pub async fn connect(url: &str, dimension: usize) -> Result<Self> {
        let client = Qdrant::from_url(url)
            .build()
            .map_err(|e| Error::vector_store(format!("failed to connect: {}", e)))?;

        Ok(Self { client, _dimension: dimension })
    }

    fn json_to_value(value: &serde_json::Value) -> Value {
        match value {
            serde_json::Value::String(s) => Value::from(s.as_str()),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::from(i)
                } else if let Some(f) = n.as_f64() {
                    Value::from(f)
                } else {
                    Value::from(n.to_string())
                }
            }
            serde_json::Value::Bool(b) => Value::from(*b),
            _ => Value::from(value.to_string()),
        }
    }
}

#[async_trait]
impl VectorStore for QdrantStore {
    async fn create_collection(&self, name: &str, dimension: usize) -> Result<()> {
        let exists = self.collection_exists(name).await?;
        if exists {
            return Ok(());
        }

        self.client
            .create_collection(
                CreateCollectionBuilder::new(name)
                    .vectors_config(VectorParamsBuilder::new(dimension as u64, Distance::Cosine)),
            )
            .await
            .map_err(|e| Error::vector_store(format!("create collection failed: {}", e)))?;

        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        self.client
            .delete_collection(name)
            .await
            .map_err(|e| Error::vector_store(format!("delete collection failed: {}", e)))?;
        Ok(())
    }

    async fn collection_exists(&self, name: &str) -> Result<bool> {
        self.client
            .collection_exists(name)
            .await
            .map_err(|e| Error::vector_store(format!("check collection failed: {}", e)))
    }

    async fn upsert(&self, collection: &str, points: Vec<VectorPoint>) -> Result<()> {
        let qdrant_points: Vec<PointStruct> = points
            .into_iter()
            .map(|p| {
                let payload: HashMap<String, Value> = p
                    .payload
                    .iter()
                    .map(|(k, v)| (k.clone(), Self::json_to_value(v)))
                    .collect();

                PointStruct::new(p.id, p.vector, payload)
            })
            .collect();

        self.client
            .upsert_points(UpsertPointsBuilder::new(collection, qdrant_points))
            .await
            .map_err(|e| Error::vector_store(format!("upsert failed: {}", e)))?;

        Ok(())
    }

    async fn search(
        &self,
        collection: &str,
        vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let results = self
            .client
            .search_points(
                SearchPointsBuilder::new(collection, vector, limit as u64).with_payload(true),
            )
            .await
            .map_err(|e| Error::vector_store(format!("search failed: {}", e)))?;

        Ok(results
            .result
            .into_iter()
            .map(|p| {
                let payload: HashMap<String, serde_json::Value> = p
                    .payload
                    .into_iter()
                    .map(|(k, v)| (k, value_to_json(v)))
                    .collect();

                SearchResult {
                    id: p.id.map(|id| format!("{:?}", id)).unwrap_or_default(),
                    score: p.score,
                    payload,
                }
            })
            .collect())
    }

    async fn search_with_filter(
        &self,
        collection: &str,
        vector: Vec<f32>,
        limit: usize,
        _filter: Filter,
    ) -> Result<Vec<SearchResult>> {
        self.search(collection, vector, limit).await
    }

    async fn delete(&self, collection: &str, ids: &[&str]) -> Result<()> {
        let point_ids: Vec<PointId> = ids.iter().map(|id| PointId::from(id.to_string())).collect();

        self.client
            .delete_points(DeletePointsBuilder::new(collection).points(point_ids))
            .await
            .map_err(|e| Error::vector_store(format!("delete failed: {}", e)))?;

        Ok(())
    }

    async fn get(&self, collection: &str, ids: &[&str]) -> Result<Vec<VectorPoint>> {
        let point_ids: Vec<PointId> = ids.iter().map(|id| PointId::from(id.to_string())).collect();

        let results = self
            .client
            .get_points(GetPointsBuilder::new(collection, point_ids).with_vectors(true).with_payload(true))
            .await
            .map_err(|e| Error::vector_store(format!("get failed: {}", e)))?;

        Ok(results
            .result
            .into_iter()
            .filter_map(|p| {
                let id = p.id.map(|id| format!("{:?}", id))?;
                #[allow(deprecated)]
                let vector = p.vectors.and_then(|v| v.vectors_options).and_then(|vo| {
                    use qdrant_client::qdrant::vectors_output::VectorsOptions;
                    match vo {
                        VectorsOptions::Vector(v) => Some(v.data),
                        _ => None,
                    }
                })?;
                let payload: HashMap<String, serde_json::Value> = p
                    .payload
                    .into_iter()
                    .map(|(k, v)| (k, value_to_json(v)))
                    .collect();

                Some(VectorPoint { id, vector, payload })
            })
            .collect())
    }

    async fn count(&self, collection: &str) -> Result<u64> {
        let result = self
            .client
            .scroll(ScrollPointsBuilder::new(collection).limit(0))
            .await
            .map_err(|e| Error::vector_store(format!("count failed: {}", e)))?;

        Ok(result.result.len() as u64)
    }
}

fn value_to_json(value: Value) -> serde_json::Value {
    match value.kind {
        Some(qdrant_client::qdrant::value::Kind::StringValue(s)) => serde_json::Value::String(s),
        Some(qdrant_client::qdrant::value::Kind::IntegerValue(i)) => serde_json::json!(i),
        Some(qdrant_client::qdrant::value::Kind::DoubleValue(f)) => serde_json::json!(f),
        Some(qdrant_client::qdrant::value::Kind::BoolValue(b)) => serde_json::Value::Bool(b),
        _ => serde_json::Value::Null,
    }
}
