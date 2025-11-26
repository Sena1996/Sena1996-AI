//! SENA Controller v9.0 - Main Binary
//!
//! Production Ready - Robust Error Handling & Configuration
//!
//! This binary provides a CLI interface to the SENA v9.0 system.
//! Supports multiple modes:
//! - MCP server mode for Claude Code integration
//! - Hook mode for Claude Code hooks
//! - Interactive mode for direct usage
//! - Command mode for single operations

use clap::Parser;
use sena_v9::{
    Cli, execute_command,
    create_system, ProcessingRequest, SystemHealth, VERSION, CODENAME,
};
use std::io::{self, BufRead, Write};

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // If a command is provided, execute it
    if cli.command.is_some() {
        match execute_command(&cli).await {
            Ok(output) => {
                println!("{}", output);
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }

    // No command provided - run interactive mode
    run_interactive().await;
}

/// Run the interactive REPL mode
async fn run_interactive() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                                                              ║");
    println!("║     SENA Controller v{} - {}                  ║", VERSION, CODENAME);
    println!("║                                                              ║");
    println!("║     Truth-Embedded Architecture in Rust                      ║");
    println!("║                                                              ║");
    println!("║     7 Ancient Wisdom Layers:                                 ║");
    println!("║       0. First Principles (Eratosthenes, 240 BCE)            ║");
    println!("║       1. Constraint-as-Feature (Persian Qanats)              ║");
    println!("║       2. Negative Space (Sushruta, 600 BCE)                  ║");
    println!("║       3. Relationship Model (Mayan Mathematics)              ║");
    println!("║       4. Self-Healing (Roman Concrete)                       ║");
    println!("║       5. Harmony Validation (Antikythera, 150 BCE)           ║");
    println!("║       6. Millennium Test (All Ancient Wisdom)                ║");
    println!("║                                                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // Create the unified system
    let mut system = create_system();

    println!("System initialized. Health: {:?}", system.get_health());
    println!();
    println!("Commands:");
    println!("  /help     - Show available commands");
    println!("  /status   - Show system status");
    println!("  /report   - Show detailed system report");
    println!("  /test     - Run millennium test on system");
    println!("  /quit     - Exit the program");
    println!();
    println!("Enter your request (or command):");
    println!();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("SENA 🦁> ");
        let _ = stdout.flush();

        let mut input = String::new();
        match stdin.lock().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                continue;
            }
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // Handle commands
        if input.starts_with('/') {
            match input.to_lowercase().as_str() {
                "/help" => {
                    println!();
                    println!("Available Commands:");
                    println!("  /help     - Show this help message");
                    println!("  /status   - Show current system health status");
                    println!("  /report   - Show detailed system report");
                    println!("  /test     - Run millennium durability test");
                    println!("  /layers   - Show information about the 7 layers");
                    println!("  /quit     - Exit the program");
                    println!();
                    println!("CLI Commands (run with --help for details):");
                    println!("  sena mcp          - Start MCP server for Claude Code");
                    println!("  sena hook         - Handle Claude Code hooks");
                    println!("  sena process      - Process a request");
                    println!("  sena health       - Get system health");
                    println!("  sena metrics      - Get system metrics");
                    println!("  sena daemon       - Control background daemon");
                    println!();
                }
                "/status" => {
                    println!();
                    let health = system.get_health();
                    let health_indicator = match health {
                        SystemHealth::Excellent => "🟢 Excellent",
                        SystemHealth::Good => "🟡 Good",
                        SystemHealth::Degraded => "🟠 Degraded",
                        SystemHealth::Critical => "🔴 Critical",
                        SystemHealth::Unknown => "⚪ Unknown",
                    };
                    println!("System Health: {}", health_indicator);
                    println!();
                }
                "/report" => {
                    println!();
                    let report = system.get_system_report();
                    println!("╔══════════════════════════════════════════════════════════════╗");
                    println!("║                    SYSTEM REPORT                             ║");
                    println!("╚══════════════════════════════════════════════════════════════╝");
                    println!();
                    println!("Version: {} ({})", report.version, report.codename);
                    println!("Health: {:?}", report.health);
                    println!("Uptime: {} seconds", report.uptime_seconds);
                    println!();
                    println!("Request Statistics:");
                    println!("  Total Requests: {}", report.request_count);
                    println!("  Successful: {}", report.successful_count);
                    println!("  Failed: {}", report.failed_count);
                    println!("  Success Rate: {:.1}%", report.success_rate * 100.0);
                    println!();
                    println!("Layer Statistics:");
                    println!("  Harmony Validations: {}", report.harmony_stats.total_validations);
                    println!("  Harmony Rate: {:.1}%", report.harmony_stats.harmony_rate * 100.0);
                    println!("  Healing Components: {}", report.healing_stats.total_components);
                    println!("  Healing Operations: {}", report.healing_stats.total_healing_operations);
                    println!("  Millennium Criteria: {}", report.millennium_stats.total_criteria);
                    println!();
                }
                "/test" => {
                    println!();
                    println!("Running Millennium Test on SENA system...");
                    println!();

                    system.millennium_test().register_component("sena_core", VERSION);
                    let result = system.millennium_test().run_millennium_test("sena_core");

                    println!("╔══════════════════════════════════════════════════════════════╗");
                    println!("║                 MILLENNIUM TEST RESULT                       ║");
                    println!("╚══════════════════════════════════════════════════════════════╝");
                    println!();
                    println!("Component: {}", result.component_name);
                    println!("Overall Passed: {}", if result.passed { "✅ YES" } else { "❌ NO" });
                    println!("Rating: {:?}", result.assessment.overall_rating);
                    println!("Score: {:.1}%", result.assessment.overall_score * 100.0);
                    println!("Estimated Lifespan: {} years", result.assessment.estimated_lifespan_years);
                    println!();
                    println!("Passed Criteria: {}", result.passed_criteria.len());
                    println!("Failed Criteria: {}", result.failed_criteria.len());
                    println!();

                    if !result.assessment.recommendations.is_empty() {
                        println!("Recommendations:");
                        for rec in &result.assessment.recommendations {
                            println!("  - {}", rec);
                        }
                        println!();
                    }
                }
                "/layers" => {
                    println!();
                    println!("╔══════════════════════════════════════════════════════════════╗");
                    println!("║              THE 7 ANCIENT WISDOM LAYERS                     ║");
                    println!("╚══════════════════════════════════════════════════════════════╝");
                    println!();
                    println!("Layer 0: First Principles Engine");
                    println!("  Inspired by: Eratosthenes (240 BCE)");
                    println!("  Principle: Understand WHY before building");
                    println!();
                    println!("Layer 1: Constraint-as-Feature Engine");
                    println!("  Inspired by: Persian Qanats (3000+ years)");
                    println!("  Principle: Treat limitations as features");
                    println!();
                    println!("Layer 2: Negative Space Architecture");
                    println!("  Inspired by: Sushruta (600 BCE)");
                    println!("  Principle: Define failure before success");
                    println!();
                    println!("Layer 3: Relationship Data Model");
                    println!("  Inspired by: Mayan Mathematics");
                    println!("  Principle: Store connections, not just values");
                    println!();
                    println!("Layer 4: Embedded Self-Healing");
                    println!("  Inspired by: Roman Concrete (2000+ years)");
                    println!("  Principle: Embed repair in damage pathways");
                    println!();
                    println!("Layer 5: Harmony Validation Engine");
                    println!("  Inspired by: Antikythera Mechanism (150 BCE)");
                    println!("  Principle: Ensure model mirrors reality");
                    println!();
                    println!("Layer 6: Millennium Test Framework");
                    println!("  Inspired by: All Ancient Wisdom");
                    println!("  Principle: Build for 1,000+ years");
                    println!();
                }
                "/quit" | "/exit" | "/q" => {
                    println!();
                    println!("Shutting down SENA system...");
                    println!("Thank you for using SENA v{}! 🦁", VERSION);
                    break;
                }
                _ => {
                    println!();
                    println!("Unknown command: {}", input);
                    println!("Type /help for available commands.");
                    println!();
                }
            }
            continue;
        }

        // Process as a regular request
        let request = ProcessingRequest::new(input, "user_input");

        println!();
        println!("Processing through 7 Ancient Wisdom Layers...");

        let result = system.process(request).await;

        println!();
        if result.success {
            println!("╔══════════════════════════════════════════════════════════════╗");
            println!("║                      RESPONSE                                ║");
            println!("╚══════════════════════════════════════════════════════════════╝");
            println!();
            println!("{}", result.content);
            println!();
            println!("────────────────────────────────────────────────────────────────");
            println!("Processing Time: {}ms", result.processing_time_ms);
            println!("Safety Score: {:.1}%", result.safety_score * 100.0);
            println!("Harmony Score: {:.1}%", result.harmony_score * 100.0);
            println!("Overall Score: {:.1}%", result.overall_score() * 100.0);

            if !result.warnings.is_empty() {
                println!();
                println!("Warnings:");
                for warning in &result.warnings {
                    println!("  ⚠️  {}", warning);
                }
            }
        } else {
            println!("❌ Processing failed!");
            for error in &result.errors {
                println!("  Error: {}", error);
            }
        }
        println!();
    }
}
