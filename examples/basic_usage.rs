use sena1996_ai::{ProcessingRequest, SenaUnifiedSystem, ThinkingDepth};

#[tokio::main]
async fn main() {
    println!("=== Sena1996 AI Tool - Basic Usage Example ===\n");

    let mut system = SenaUnifiedSystem::new();

    println!("1. System Health Check");
    println!("-----------------------");
    let health = system.get_health();
    println!("Health Status: {:?}\n", health);

    println!("2. Process a Request");
    println!("--------------------");
    let request = ProcessingRequest::new("Analyze code quality best practices", "analysis");
    let result = system.process(request).await;
    if result.success {
        println!("Result: {}\n", result.content);
    } else {
        println!("Processing failed\n");
    }

    println!("3. Knowledge Search");
    println!("-------------------");
    let results = system.knowledge().search("security");
    println!("Found {} results for 'security'\n", results.len());

    println!("4. Intelligence Analysis");
    println!("------------------------");
    let analysis = system
        .intelligence()
        .analyze("How to improve code performance?", ThinkingDepth::Standard);
    println!("Conclusion: {}\n", analysis.conclusion);

    println!("=== Example Complete ===");
}
