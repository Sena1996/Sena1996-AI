use sena1996_ai::{ProcessingRequest, SenaUnifiedSystem};

#[tokio::main]
async fn main() {
    println!("=== Sena1996 AI Tool - Multi-Agent Example ===\n");

    let mut system = SenaUnifiedSystem::new();

    let agents = ["backend", "ios", "android", "web", "iot"];

    for agent in agents {
        println!("Agent: {}", agent.to_uppercase());
        println!("{}", "-".repeat(40));

        let request = ProcessingRequest::new(
            &format!("Analyze typical {} project structure", agent),
            "analysis",
        );

        let result = system.process(request).await;
        if result.success {
            println!("Output: {}", result.content);
        } else {
            println!("Processing failed");
        }
        println!();
    }

    println!("=== Multi-Agent Example Complete ===");
}
