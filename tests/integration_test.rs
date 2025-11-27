use sena1996_ai::{
    ProcessingRequest, SenaUnifiedSystem, SystemHealth, ThinkingDepth, VERSION,
};

#[test]
fn test_version_exists() {
    assert!(!VERSION.is_empty());
}

#[test]
fn test_system_creation() {
    let system = SenaUnifiedSystem::new();
    let health = system.get_health();
    assert!(matches!(
        health,
        SystemHealth::Excellent | SystemHealth::Good | SystemHealth::Unknown
    ));
}

#[test]
fn test_processing_request_creation() {
    let request = ProcessingRequest::new("test input", "test");
    assert_eq!(request.content, "test input");
    assert_eq!(request.request_type, "test");
}

#[test]
fn test_thinking_depth_variants() {
    let _quick = ThinkingDepth::Quick;
    let _standard = ThinkingDepth::Standard;
    let _deep = ThinkingDepth::Deep;
    let _maximum = ThinkingDepth::Maximum;
}

#[test]
fn test_system_health_from_score() {
    assert_eq!(SystemHealth::from_score(1.0), SystemHealth::Excellent);
    assert_eq!(SystemHealth::from_score(0.85), SystemHealth::Good);
    assert_eq!(SystemHealth::from_score(0.65), SystemHealth::Degraded);
    assert_eq!(SystemHealth::from_score(0.3), SystemHealth::Critical);
}

#[tokio::test]
async fn test_process_request() {
    let mut system = SenaUnifiedSystem::new();
    let request = ProcessingRequest::new("Hello, SENA!", "greeting");

    let result = system.process(request).await;
    assert!(result.success);
}

#[test]
fn test_knowledge_search() {
    let mut system = SenaUnifiedSystem::new();
    let results = system.knowledge().search("security");
    let _ = results;
}

#[test]
fn test_intelligence_analyze() {
    let mut system = SenaUnifiedSystem::new();
    let analysis = system
        .intelligence()
        .analyze("test problem", ThinkingDepth::Quick);

    assert!(!analysis.conclusion.is_empty());
}
