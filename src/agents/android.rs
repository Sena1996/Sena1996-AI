use super::{DomainAgentType, DomainAnalysis, Finding, Severity};
use once_cell::sync::Lazy;
use regex::Regex;

static COMPOSE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(@Composable|remember\{|LazyColumn|LazyRow|mutableStateOf)"#)
        .expect("invalid compose regex")
});

static ACTIVITY_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(Activity|Fragment|onCreate|onResume|onPause|onDestroy)"#)
        .expect("invalid activity regex")
});

static VIEWMODEL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(ViewModel|viewModel|StateFlow|LiveData|MutableLiveData)"#)
        .expect("invalid viewmodel regex")
});

static COROUTINE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(viewModelScope|lifecycleScope|suspend\s+fun|launch\s*\{|async\s*\{)"#)
        .expect("invalid coroutine regex")
});

pub struct AndroidAgent {
    #[allow(dead_code)]
    name: String,
}

impl AndroidAgent {
    pub fn new() -> Self {
        Self {
            name: "Android Agent".to_string(),
        }
    }

    pub fn analyze(&self, command: &str, input: &str) -> DomainAnalysis {
        match command {
            "ui" | "material" | "compose" => self.analyze_ui_material(input),
            "perf" | "performance" => self.analyze_performance(input),
            "lifecycle" => self.analyze_lifecycle(input),
            "compat" | "compatibility" => self.analyze_compatibility(input),
            "a11y" | "accessibility" => self.analyze_accessibility(input),
            _ => self.full_analysis(input),
        }
    }

    fn analyze_ui_material(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        let is_compose = COMPOSE_REGEX.is_match(input);

        if is_compose {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Jetpack Compose detected".to_string(),
                description: "Modern declarative UI framework in use".to_string(),
                location: None,
                suggestion: None,
            });

            if input.contains("remember") {
                findings.push(Finding {
                    severity: Severity::Info,
                    title: "State management with remember".to_string(),
                    description: "Composable state is properly remembered".to_string(),
                    location: None,
                    suggestion: None,
                });
            }

            if input.contains("LazyColumn") || input.contains("LazyRow") {
                findings.push(Finding {
                    severity: Severity::Success,
                    title: "Lazy lists used".to_string(),
                    description: "Efficient list rendering with lazy composition".to_string(),
                    location: None,
                    suggestion: None,
                });
            }
        }

        if input_lower.contains("material") || input.contains("MaterialTheme") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Material Design theme".to_string(),
                description: "Using Material Design components".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input.contains("isSystemInDarkTheme")
            || input_lower.contains("dark_theme")
            || input_lower.contains("night")
        {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Dark theme support".to_string(),
                description: "App supports dark/light theme".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("48") && input_lower.contains("dp") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Touch target size (48dp)".to_string(),
                description: "Touch targets meet Material Design minimum 48dp".to_string(),
                location: None,
                suggestion: None,
            });
        } else if input_lower.contains("button") || input_lower.contains("clickable") {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "Verify touch target sizes".to_string(),
                description: "Material Design recommends minimum 48dp touch targets".to_string(),
                location: None,
                suggestion: Some("Ensure clickable areas are at least 48x48dp".to_string()),
            });
        }

        if input.contains("Scaffold") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Scaffold structure used".to_string(),
                description: "Using Material Design scaffold pattern".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input.contains("TopAppBar")
            || input.contains("BottomAppBar")
            || input_lower.contains("toolbar")
        {
            findings.push(Finding {
                severity: Severity::Success,
                title: "App bar implemented".to_string(),
                description: "Standard navigation pattern".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input.contains("FloatingActionButton") || input.contains("FAB") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "FAB detected".to_string(),
                description: "Floating Action Button for primary action".to_string(),
                location: None,
                suggestion: Some("Ensure FAB represents the primary screen action".to_string()),
            });
        }

        if input.contains("NavigationBar") || input.contains("BottomNavigation") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Bottom navigation implemented".to_string(),
                description: "Standard navigation for top-level destinations".to_string(),
                location: None,
                suggestion: None,
            });
        }

        let score = if is_compose { 80 } else { 70 };

        DomainAnalysis {
            agent: DomainAgentType::Android,
            category: "UI & Material Design".to_string(),
            findings,
            recommendations: vec![
                "Follow Material Design 3 guidelines".to_string(),
                "Use minimum 48dp touch targets".to_string(),
                "Support both light and dark themes".to_string(),
                "Use Scaffold for consistent app structure".to_string(),
                "Implement proper navigation patterns".to_string(),
                "Use Material components consistently".to_string(),
            ],
            score,
        }
    }

    fn analyze_performance(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        if COROUTINE_REGEX.is_match(input) {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Coroutines detected".to_string(),
                description: "Using Kotlin coroutines for async operations".to_string(),
                location: None,
                suggestion: None,
            });

            if input.contains("Dispatchers.IO") {
                findings.push(Finding {
                    severity: Severity::Success,
                    title: "IO Dispatcher used".to_string(),
                    description: "IO operations on proper dispatcher".to_string(),
                    location: None,
                    suggestion: None,
                });
            }

            if input.contains("Dispatchers.Main") || input.contains("withContext(Dispatchers.Main)")
            {
                findings.push(Finding {
                    severity: Severity::Info,
                    title: "Main dispatcher usage".to_string(),
                    description: "UI updates on main thread".to_string(),
                    location: None,
                    suggestion: Some("Ensure heavy work is not on Main dispatcher".to_string()),
                });
            }
        }

        if input.contains("RecyclerView") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "RecyclerView detected".to_string(),
                description: "Efficient list implementation".to_string(),
                location: None,
                suggestion: None,
            });

            if input.contains("DiffUtil") || input.contains("ListAdapter") {
                findings.push(Finding {
                    severity: Severity::Success,
                    title: "DiffUtil/ListAdapter used".to_string(),
                    description: "Efficient list updates".to_string(),
                    location: None,
                    suggestion: None,
                });
            } else {
                findings.push(Finding {
                    severity: Severity::Warning,
                    title: "Consider DiffUtil".to_string(),
                    description: "DiffUtil improves RecyclerView performance".to_string(),
                    location: None,
                    suggestion: Some("Use ListAdapter or DiffUtil.Callback".to_string()),
                });
            }
        }

        if input_lower.contains("glide")
            || input_lower.contains("coil")
            || input_lower.contains("picasso")
        {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Image loading library used".to_string(),
                description: "Efficient image loading with caching".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("room") || input.contains("@Dao") || input.contains("@Entity") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Room database detected".to_string(),
                description: "SQLite abstraction layer".to_string(),
                location: None,
                suggestion: Some("Use suspend functions for Room operations".to_string()),
            });
        }

        if input.contains("runBlocking") {
            findings.push(Finding {
                severity: Severity::Critical,
                title: "runBlocking detected".to_string(),
                description: "Blocks thread, can cause ANR on main thread".to_string(),
                location: None,
                suggestion: Some("Use launch or async instead of runBlocking".to_string()),
            });
        }

        if input.contains("Thread.sleep") || input.contains("SystemClock.sleep") {
            findings.push(Finding {
                severity: Severity::Critical,
                title: "Thread.sleep detected".to_string(),
                description: "Blocking sleep can cause ANR".to_string(),
                location: None,
                suggestion: Some("Use delay() in coroutines or Handler.postDelayed".to_string()),
            });
        }

        let critical_count = findings
            .iter()
            .filter(|f| f.severity == Severity::Critical)
            .count();
        let score = if critical_count > 0 { 45 } else { 80 };

        DomainAnalysis {
            agent: DomainAgentType::Android,
            category: "Performance Analysis".to_string(),
            findings,
            recommendations: vec![
                "Use coroutines with proper dispatchers".to_string(),
                "Avoid blocking operations on main thread".to_string(),
                "Use DiffUtil for RecyclerView updates".to_string(),
                "Implement image caching with Glide/Coil".to_string(),
                "Profile with Android Studio Profiler".to_string(),
                "Use R8/ProGuard for release builds".to_string(),
            ],
            score,
        }
    }

    fn analyze_lifecycle(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        if ACTIVITY_REGEX.is_match(input) {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Activity/Fragment lifecycle methods".to_string(),
                description: "Lifecycle callbacks detected".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if VIEWMODEL_REGEX.is_match(input) {
            findings.push(Finding {
                severity: Severity::Success,
                title: "ViewModel architecture".to_string(),
                description: "Using ViewModel for lifecycle-aware state".to_string(),
                location: None,
                suggestion: None,
            });

            if input.contains("StateFlow") || input.contains("MutableStateFlow") {
                findings.push(Finding {
                    severity: Severity::Success,
                    title: "StateFlow for state management".to_string(),
                    description: "Modern reactive state handling".to_string(),
                    location: None,
                    suggestion: None,
                });
            }
        }

        if input.contains("savedStateHandle") || input.contains("SavedStateHandle") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "SavedStateHandle used".to_string(),
                description: "State survives process death".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("onbackpressed") || input.contains("OnBackPressedCallback") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Back navigation handling".to_string(),
                description: "Custom back press handling".to_string(),
                location: None,
                suggestion: Some("Use OnBackPressedDispatcher for predictive back".to_string()),
            });
        }

        if input.contains("repeatOnLifecycle") || input.contains("flowWithLifecycle") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Lifecycle-aware flow collection".to_string(),
                description: "Flows are collected lifecycle-safely".to_string(),
                location: None,
                suggestion: None,
            });
        } else if input.contains("collect") && input.contains("Flow") {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "Flow collection lifecycle check".to_string(),
                description: "Ensure flows are collected lifecycle-aware".to_string(),
                location: None,
                suggestion: Some("Use repeatOnLifecycle(Lifecycle.State.STARTED)".to_string()),
            });
        }

        if input_lower.contains("oncleared") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "ViewModel onCleared implemented".to_string(),
                description: "Cleanup when ViewModel is destroyed".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input.contains("configChanges") {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "Manual config changes handling".to_string(),
                description: "Handling configuration changes manually".to_string(),
                location: None,
                suggestion: Some(
                    "Consider ViewModel for configuration change survival".to_string(),
                ),
            });
        }

        let score = 75;

        DomainAnalysis {
            agent: DomainAgentType::Android,
            category: "Lifecycle Analysis".to_string(),
            findings,
            recommendations: vec![
                "Use ViewModel for UI state".to_string(),
                "Implement SavedStateHandle for process death".to_string(),
                "Collect flows with repeatOnLifecycle".to_string(),
                "Use OnBackPressedDispatcher for back handling".to_string(),
                "Clean up resources in onCleared/onDestroy".to_string(),
                "Test with Don't Keep Activities enabled".to_string(),
            ],
            score,
        }
    }

    fn analyze_compatibility(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        if input.contains("Build.VERSION.SDK_INT") || input.contains("@RequiresApi") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "API level checks".to_string(),
                description: "Code handles different Android versions".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("minsdkversion") || input_lower.contains("minsdk") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Min SDK version configuration".to_string(),
                description: "Minimum supported Android version defined".to_string(),
                location: None,
                suggestion: Some("Consider API 24+ for modern features".to_string()),
            });
        }

        if input.contains("AppCompat") || input.contains("androidx.appcompat") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "AppCompat used".to_string(),
                description: "Backward-compatible components".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input.contains("ContextCompat") || input.contains("ActivityCompat") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Compat utilities used".to_string(),
                description: "Using compatibility helpers".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("permission")
            && (input.contains("requestPermissions") || input.contains("ActivityResultLauncher"))
        {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Runtime permissions handled".to_string(),
                description: "Using runtime permission requests".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("desugaring") || input.contains("coreLibraryDesugaring") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Java 8+ API desugaring".to_string(),
                description: "Modern Java APIs available on older devices".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("tablet") || input.contains("sw600dp") || input.contains("xlarge") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Tablet layouts detected".to_string(),
                description: "Supporting multiple screen sizes".to_string(),
                location: None,
                suggestion: None,
            });
        }

        let score = 75;

        DomainAnalysis {
            agent: DomainAgentType::Android,
            category: "Compatibility Analysis".to_string(),
            findings,
            recommendations: vec![
                "Use Build.VERSION.SDK_INT for API checks".to_string(),
                "Implement runtime permission handling".to_string(),
                "Use AndroidX/AppCompat libraries".to_string(),
                "Enable Java 8+ desugaring if needed".to_string(),
                "Test on multiple Android versions".to_string(),
                "Provide layouts for different screen sizes".to_string(),
            ],
            score,
        }
    }

    fn analyze_accessibility(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        if input.contains("contentDescription") || input.contains("semantics") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Content descriptions provided".to_string(),
                description: "TalkBack can describe UI elements".to_string(),
                location: None,
                suggestion: None,
            });
        } else {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "No content descriptions detected".to_string(),
                description: "TalkBack users may have difficulty".to_string(),
                location: None,
                suggestion: Some("Add contentDescription to images and buttons".to_string()),
            });
        }

        if input.contains("importantForAccessibility") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Accessibility importance set".to_string(),
                description: "Controlling accessibility tree".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input.contains("AccessibilityNodeInfo") || input.contains("AccessibilityEvent") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Custom accessibility implementation".to_string(),
                description: "Advanced accessibility handling".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input.contains("labelFor") || input.contains("setLabelFor") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Label associations set".to_string(),
                description: "Form labels linked to inputs".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("fontscale") || input_lower.contains("sp") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Scalable text units (SP)".to_string(),
                description: "Text sizes respect user preferences".to_string(),
                location: None,
                suggestion: None,
            });
        }

        let score = 65;

        DomainAnalysis {
            agent: DomainAgentType::Android,
            category: "Accessibility Audit".to_string(),
            findings,
            recommendations: vec![
                "Add contentDescription to all images".to_string(),
                "Test with TalkBack enabled".to_string(),
                "Use SP units for text sizes".to_string(),
                "Ensure sufficient color contrast".to_string(),
                "Associate labels with form inputs".to_string(),
                "Support Switch Access navigation".to_string(),
            ],
            score,
        }
    }

    fn full_analysis(&self, input: &str) -> DomainAnalysis {
        let mut all_findings = Vec::new();

        let analyses = vec![
            self.analyze_ui_material(input),
            self.analyze_performance(input),
            self.analyze_lifecycle(input),
            self.analyze_compatibility(input),
            self.analyze_accessibility(input),
        ];

        let mut total_score = 0u32;
        for analysis in &analyses {
            all_findings.extend(analysis.findings.clone());
            total_score += analysis.score as u32;
        }

        let avg_score = (total_score / analyses.len() as u32) as u8;

        DomainAnalysis {
            agent: DomainAgentType::Android,
            category: "Full Android Analysis".to_string(),
            findings: all_findings,
            recommendations: vec![
                "Follow Material Design guidelines".to_string(),
                "Use ViewModel and proper lifecycle handling".to_string(),
                "Avoid ANRs with coroutines".to_string(),
                "Support multiple Android versions".to_string(),
                "Implement TalkBack accessibility".to_string(),
                "Test on various devices and configurations".to_string(),
            ],
            score: avg_score,
        }
    }
}

impl Default for AndroidAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_android_agent_creation() {
        let agent = AndroidAgent::new();
        assert_eq!(agent.name, "Android Agent");
    }

    #[test]
    fn test_compose_detection() {
        let agent = AndroidAgent::new();
        let code = r#"
            @Composable
            fun Greeting(name: String) {
                Text(text = "Hello $name!")
            }
        "#;
        let result = agent.analyze("ui", code);
        assert!(result.findings.iter().any(|f| f.title.contains("Compose")));
    }

    #[test]
    fn test_viewmodel_detection() {
        let agent = AndroidAgent::new();
        let code = r#"
            class MainViewModel : ViewModel() {
                private val _uiState = MutableStateFlow(UiState())
                val uiState: StateFlow<UiState> = _uiState.asStateFlow()
            }
        "#;
        let result = agent.analyze("lifecycle", code);
        assert!(result
            .findings
            .iter()
            .any(|f| f.title.contains("ViewModel")));
    }

    #[test]
    fn test_anr_detection() {
        let agent = AndroidAgent::new();
        let code = r#"
            fun loadData() {
                runBlocking {
                    Thread.sleep(5000)
                }
            }
        "#;
        let result = agent.analyze("perf", code);
        assert!(result
            .findings
            .iter()
            .any(|f| f.severity == Severity::Critical));
    }
}
