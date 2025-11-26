use super::{DomainAgentType, DomainAnalysis, Finding, Severity};
use once_cell::sync::Lazy;
use regex::Regex;

static SWIFTUI_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(struct\s+\w+\s*:\s*View|@State|@Binding|@ObservedObject|@Published|@EnvironmentObject)"#)
        .expect("invalid swiftui regex")
});

static UIKIT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(UIViewController|UIView|UITableView|UICollectionView|UINavigationController)"#)
        .expect("invalid uikit regex")
});

static MEMORY_LEAK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(\[self\s|self\.|self\]|\.self)"#).expect("invalid memory leak regex")
});

static MAIN_THREAD_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(DispatchQueue\.main|@MainActor|MainActor\.run)"#).expect("invalid main thread regex")
});

pub struct IOSAgent {
    #[allow(dead_code)]
    name: String,
}

impl IOSAgent {
    pub fn new() -> Self {
        Self {
            name: "iOS Agent".to_string(),
        }
    }

    pub fn analyze(&self, command: &str, input: &str) -> DomainAnalysis {
        match command {
            "ui" | "hig" => self.analyze_ui_hig(input),
            "perf" | "performance" => self.analyze_performance(input),
            "a11y" | "accessibility" => self.analyze_accessibility(input),
            "device" => self.analyze_device_features(input),
            "memory" => self.analyze_memory(input),
            _ => self.full_analysis(input),
        }
    }

    fn analyze_ui_hig(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        let is_swiftui = SWIFTUI_REGEX.is_match(input);
        let is_uikit = UIKIT_REGEX.is_match(input);

        if is_swiftui {
            findings.push(Finding {
                severity: Severity::Info,
                title: "SwiftUI framework detected".to_string(),
                description: "Modern declarative UI framework in use".to_string(),
                location: None,
                suggestion: None,
            });

            if input.contains("@State") {
                findings.push(Finding {
                    severity: Severity::Info,
                    title: "@State property wrapper used".to_string(),
                    description: "Local view state management".to_string(),
                    location: None,
                    suggestion: None,
                });
            }

            if input.contains("NavigationView") || input.contains("NavigationStack") {
                findings.push(Finding {
                    severity: Severity::Success,
                    title: "Navigation structure implemented".to_string(),
                    description: "Proper navigation hierarchy".to_string(),
                    location: None,
                    suggestion: Some("Use NavigationStack for iOS 16+".to_string()),
                });
            }
        }

        if is_uikit {
            findings.push(Finding {
                severity: Severity::Info,
                title: "UIKit framework detected".to_string(),
                description: "Traditional imperative UI framework".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("44") && (input_lower.contains("width") || input_lower.contains("height")) {
            findings.push(Finding {
                severity: Severity::Success,
                title: "HIG touch target size (44pt)".to_string(),
                description: "Touch targets meet minimum 44x44pt requirement".to_string(),
                location: None,
                suggestion: None,
            });
        } else if input_lower.contains("button") || input_lower.contains("tap") {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "Verify touch target sizes".to_string(),
                description: "HIG recommends minimum 44x44pt touch targets".to_string(),
                location: None,
                suggestion: Some("Ensure all buttons are at least 44x44pt".to_string()),
            });
        }

        if input_lower.contains("safearea") || input_lower.contains("safe area") || input.contains(".safeAreaInset") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Safe area handling implemented".to_string(),
                description: "UI respects device safe areas (notch, home indicator)".to_string(),
                location: None,
                suggestion: None,
            });
        } else if is_swiftui || is_uikit {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "No safe area handling detected".to_string(),
                description: "Content may be obscured by notch or home indicator".to_string(),
                location: None,
                suggestion: Some("Use safeAreaInset or UILayoutGuide for safe areas".to_string()),
            });
        }

        if input.contains("@Environment(\\.colorScheme)") || input_lower.contains("traitcollection.userinterfacestyle") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Dark mode support detected".to_string(),
                description: "App adapts to light/dark appearance".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input.contains("DynamicTypeSize") || input_lower.contains("preferredcontentsizecategory") || input.contains(".dynamicTypeSize") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Dynamic Type support detected".to_string(),
                description: "Text scales with user accessibility settings".to_string(),
                location: None,
                suggestion: None,
            });
        } else if input_lower.contains("font") || input_lower.contains("text") {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "Dynamic Type not detected".to_string(),
                description: "Text may not scale with accessibility settings".to_string(),
                location: None,
                suggestion: Some("Use .font(.body) or UIFont.preferredFont for Dynamic Type".to_string()),
            });
        }

        if input.contains("SF Symbols") || input.contains("systemName") || input.contains("Image(systemName:") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "SF Symbols used".to_string(),
                description: "Using Apple's consistent icon set".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("alert") || input_lower.contains("actionsheet") || input.contains(".alert(") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "System dialogs used".to_string(),
                description: "Using standard iOS alert patterns".to_string(),
                location: None,
                suggestion: Some("Ensure alerts have clear, actionable buttons".to_string()),
            });
        }

        let score = if findings.iter().filter(|f| f.severity == Severity::Success).count() > 3 { 85 } else { 65 };

        DomainAnalysis {
            agent: DomainAgentType::IOS,
            category: "UI/UX & HIG Compliance".to_string(),
            findings,
            recommendations: vec![
                "Follow Human Interface Guidelines (HIG)".to_string(),
                "Ensure minimum 44x44pt touch targets".to_string(),
                "Support Dynamic Type for accessibility".to_string(),
                "Handle safe areas for all device sizes".to_string(),
                "Support both light and dark appearance".to_string(),
                "Use SF Symbols for consistent iconography".to_string(),
            ],
            score,
        }
    }

    fn analyze_performance(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        if input.contains("DispatchQueue.global") || input.contains("Task {") || input.contains("async {") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Background processing detected".to_string(),
                description: "Work is offloaded from main thread".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if MAIN_THREAD_REGEX.is_match(input) && input_lower.contains("ui") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "UI updates on main thread".to_string(),
                description: "UI operations correctly dispatched to main thread".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("lazyv") || input_lower.contains("lazyhstack") || input.contains("LazyVStack") || input.contains("LazyHStack") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Lazy loading views used".to_string(),
                description: "Views are loaded on-demand for performance".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if (input_lower.contains("list") || input_lower.contains("foreach"))
            && !input_lower.contains("lazy") && !input.contains("List")
        {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "Consider lazy loading for lists".to_string(),
                description: "Large lists should use lazy containers".to_string(),
                location: None,
                suggestion: Some("Use LazyVStack/List instead of VStack/ForEach for large datasets".to_string()),
            });
        }

        if input.contains("Image(") && !input_lower.contains("resizable") {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "Image optimization check".to_string(),
                description: "Ensure images are properly sized and optimized".to_string(),
                location: None,
                suggestion: Some("Use .resizable() and proper aspect ratio".to_string()),
            });
        }

        if input_lower.contains("asyncimage") || input.contains("URLSession") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Network image loading detected".to_string(),
                description: "Images are loaded from network".to_string(),
                location: None,
                suggestion: Some("Implement image caching for better performance".to_string()),
            });
        }

        if input_lower.contains("coredata") || input_lower.contains("@fetchrequest") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Core Data usage detected".to_string(),
                description: "Local persistence with Core Data".to_string(),
                location: None,
                suggestion: Some("Use batch operations for large data sets".to_string()),
            });

            if input_lower.contains("fetchbatchsize") || input_lower.contains("fetchlimit") {
                findings.push(Finding {
                    severity: Severity::Success,
                    title: "Core Data fetch optimization".to_string(),
                    description: "Fetch operations are limited/batched".to_string(),
                    location: None,
                    suggestion: None,
                });
            }
        }

        if input.contains(".animation(") || input.contains("withAnimation") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Animations detected".to_string(),
                description: "UI animations are implemented".to_string(),
                location: None,
                suggestion: Some("Avoid animating expensive operations".to_string()),
            });
        }

        let score = 75;

        DomainAnalysis {
            agent: DomainAgentType::IOS,
            category: "Performance Analysis".to_string(),
            findings,
            recommendations: vec![
                "Use Instruments to profile performance".to_string(),
                "Offload heavy work to background threads".to_string(),
                "Use lazy loading for large collections".to_string(),
                "Optimize images (size, format, caching)".to_string(),
                "Batch Core Data operations".to_string(),
                "Minimize view hierarchy depth".to_string(),
            ],
            score,
        }
    }

    fn analyze_accessibility(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        if input.contains(".accessibilityLabel") || input_lower.contains("accessibilitylabel") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Accessibility labels implemented".to_string(),
                description: "VoiceOver can describe UI elements".to_string(),
                location: None,
                suggestion: None,
            });
        } else {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "No accessibility labels detected".to_string(),
                description: "VoiceOver users may have difficulty navigating".to_string(),
                location: None,
                suggestion: Some("Add .accessibilityLabel() to interactive elements".to_string()),
            });
        }

        if input.contains(".accessibilityHint") || input_lower.contains("accessibilityhint") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Accessibility hints provided".to_string(),
                description: "Additional context for VoiceOver users".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input.contains(".accessibilityValue") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Accessibility values set".to_string(),
                description: "Dynamic values announced to VoiceOver".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input.contains("AccessibilityTraits") || input.contains(".accessibilityAddTraits") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Accessibility traits configured".to_string(),
                description: "UI elements have proper semantic roles".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("reducemotion") || input.contains("accessibilityReduceMotion") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Reduce Motion support".to_string(),
                description: "App respects user's motion preferences".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("voiceover") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "VoiceOver consideration detected".to_string(),
                description: "Code references VoiceOver".to_string(),
                location: None,
                suggestion: Some("Test thoroughly with VoiceOver enabled".to_string()),
            });
        }

        if input.contains(".accessibilityElement(children:") || input.contains(".accessibilityChildren") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Accessibility grouping implemented".to_string(),
                description: "Related elements are grouped for navigation".to_string(),
                location: None,
                suggestion: None,
            });
        }

        let a11y_count = findings.iter().filter(|f| f.severity == Severity::Success).count();
        let score = if a11y_count >= 3 { 85 } else if a11y_count >= 1 { 65 } else { 40 };

        DomainAnalysis {
            agent: DomainAgentType::IOS,
            category: "Accessibility Audit".to_string(),
            findings,
            recommendations: vec![
                "Add accessibility labels to all interactive elements".to_string(),
                "Test with VoiceOver enabled".to_string(),
                "Support Dynamic Type for text scaling".to_string(),
                "Respect Reduce Motion preference".to_string(),
                "Ensure sufficient color contrast".to_string(),
                "Group related elements for easier navigation".to_string(),
            ],
            score,
        }
    }

    fn analyze_device_features(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        if input_lower.contains("camera") || input.contains("AVCaptureSession") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Camera usage detected".to_string(),
                description: "App accesses device camera".to_string(),
                location: None,
                suggestion: Some("Add NSCameraUsageDescription to Info.plist".to_string()),
            });
        }

        if input_lower.contains("location") || input.contains("CLLocationManager") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Location services detected".to_string(),
                description: "App accesses user location".to_string(),
                location: None,
                suggestion: Some("Add appropriate location usage description to Info.plist".to_string()),
            });
        }

        if input.contains("HealthKit") || input_lower.contains("healthstore") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "HealthKit integration detected".to_string(),
                description: "App accesses health data".to_string(),
                location: None,
                suggestion: Some("Ensure proper HealthKit entitlements and descriptions".to_string()),
            });
        }

        if input.contains("UIDevice.current") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Device info access detected".to_string(),
                description: "App queries device information".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("haptic") || input.contains("UIImpactFeedbackGenerator") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Haptic feedback implemented".to_string(),
                description: "App provides tactile feedback".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("notificationcenter") || input.contains("UNUserNotificationCenter") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Push notifications detected".to_string(),
                description: "App uses push notifications".to_string(),
                location: None,
                suggestion: Some("Request notification permission at appropriate time".to_string()),
            });
        }

        if input.contains("#available") || input.contains("@available") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "API availability checks".to_string(),
                description: "Code handles different iOS versions".to_string(),
                location: None,
                suggestion: None,
            });
        }

        let score = 75;

        DomainAnalysis {
            agent: DomainAgentType::IOS,
            category: "Device Features".to_string(),
            findings,
            recommendations: vec![
                "Add usage descriptions for all sensitive permissions".to_string(),
                "Request permissions at contextually appropriate times".to_string(),
                "Handle permission denial gracefully".to_string(),
                "Use @available checks for newer APIs".to_string(),
                "Implement feature fallbacks for older devices".to_string(),
            ],
            score,
        }
    }

    fn analyze_memory(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        if input.contains("[weak self]") || input.contains("[unowned self]") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Weak/unowned references used".to_string(),
                description: "Closure capture lists prevent retain cycles".to_string(),
                location: None,
                suggestion: None,
            });
        } else if MEMORY_LEAK_REGEX.is_match(input) && input_lower.contains("closure") {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "Potential retain cycle".to_string(),
                description: "Closure captures self strongly".to_string(),
                location: None,
                suggestion: Some("Use [weak self] or [unowned self] in closures".to_string()),
            });
        }

        if input.contains("deinit") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Deinit implemented".to_string(),
                description: "Cleanup code in deinitializer".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("didreceivememorywarning") || input.contains("applicationDidReceiveMemoryWarning") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Memory warning handling".to_string(),
                description: "App responds to memory pressure".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("autoreleasepool") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Autorelease pool usage".to_string(),
                description: "Manual memory management for loops".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("nscache") || input.contains("NSCache") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "NSCache used for caching".to_string(),
                description: "Memory-aware caching implementation".to_string(),
                location: None,
                suggestion: None,
            });
        }

        let score = 70;

        DomainAnalysis {
            agent: DomainAgentType::IOS,
            category: "Memory Analysis".to_string(),
            findings,
            recommendations: vec![
                "Use [weak self] in closures to prevent retain cycles".to_string(),
                "Profile with Instruments Memory Leaks tool".to_string(),
                "Handle memory warnings appropriately".to_string(),
                "Use NSCache for automatic memory management".to_string(),
                "Implement deinit for cleanup verification".to_string(),
            ],
            score,
        }
    }

    fn full_analysis(&self, input: &str) -> DomainAnalysis {
        let mut all_findings = Vec::new();

        let analyses = vec![
            self.analyze_ui_hig(input),
            self.analyze_performance(input),
            self.analyze_accessibility(input),
            self.analyze_device_features(input),
            self.analyze_memory(input),
        ];

        let mut total_score = 0u32;
        for analysis in &analyses {
            all_findings.extend(analysis.findings.clone());
            total_score += analysis.score as u32;
        }

        let avg_score = (total_score / analyses.len() as u32) as u8;

        DomainAnalysis {
            agent: DomainAgentType::IOS,
            category: "Full iOS Analysis".to_string(),
            findings: all_findings,
            recommendations: vec![
                "Follow Human Interface Guidelines".to_string(),
                "Ensure full VoiceOver accessibility".to_string(),
                "Profile performance with Instruments".to_string(),
                "Test on multiple device sizes".to_string(),
                "Handle memory warnings properly".to_string(),
                "Support Dynamic Type and Dark Mode".to_string(),
            ],
            score: avg_score,
        }
    }
}

impl Default for IOSAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ios_agent_creation() {
        let agent = IOSAgent::new();
        assert_eq!(agent.name, "iOS Agent");
    }

    #[test]
    fn test_swiftui_detection() {
        let agent = IOSAgent::new();
        let code = r#"
            struct ContentView: View {
                @State private var count = 0
                var body: some View {
                    Text("Hello")
                }
            }
        "#;
        let result = agent.analyze("ui", code);
        assert!(result.findings.iter().any(|f| f.title.contains("SwiftUI")));
    }

    #[test]
    fn test_accessibility_analysis() {
        let agent = IOSAgent::new();
        let code = r#"
            Button("Submit")
                .accessibilityLabel("Submit form")
                .accessibilityHint("Double tap to submit")
        "#;
        let result = agent.analyze("a11y", code);
        assert!(result.findings.iter().any(|f| f.severity == Severity::Success));
    }

    #[test]
    fn test_memory_analysis() {
        let agent = IOSAgent::new();
        let code = r#"
            someAsyncOperation { [weak self] result in
                self?.handleResult(result)
            }
        "#;
        let result = agent.analyze("memory", code);
        assert!(result.findings.iter().any(|f| f.title.contains("Weak")));
    }
}
