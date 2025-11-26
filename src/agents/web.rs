use super::{DomainAgentType, DomainAnalysis, Finding, Severity};
use once_cell::sync::Lazy;
use regex::Regex;

static REACT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(useState|useEffect|useCallback|useMemo|React\.FC|jsx|tsx)"#)
        .expect("invalid react regex")
});

static VUE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(defineComponent|ref\(|reactive\(|computed\(|v-model|v-if|<template>)"#)
        .expect("invalid vue regex")
});

static ANGULAR_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(@Component|@Injectable|ngOnInit|ngOnDestroy|\*ngIf|\*ngFor)"#)
        .expect("invalid angular regex")
});

static ARIA_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(aria-|role=)"#).expect("invalid aria regex")
});

pub struct WebAgent {
    name: String,
}

impl WebAgent {
    pub fn new() -> Self {
        Self {
            name: "Web Agent".to_string(),
        }
    }

    pub fn analyze(&self, command: &str, input: &str) -> DomainAnalysis {
        match command {
            "vitals" | "cwv" => self.analyze_core_web_vitals(input),
            "a11y" | "accessibility" => self.analyze_accessibility(input),
            "seo" => self.analyze_seo(input),
            "bundle" => self.analyze_bundle(input),
            "perf" | "performance" => self.analyze_performance(input),
            "audit" => self.analyze_security(input),
            _ => self.full_analysis(input),
        }
    }

    fn analyze_core_web_vitals(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        if input_lower.contains("lazy") || input.contains("loading=\"lazy\"") || input.contains("React.lazy") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Lazy loading implemented".to_string(),
                description: "Resources loaded on-demand for better LCP".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("preload") || input_lower.contains("prefetch") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Resource hints detected".to_string(),
                description: "Preload/prefetch for critical resources".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("async") && input_lower.contains("script") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Async scripts detected".to_string(),
                description: "Scripts loaded asynchronously".to_string(),
                location: None,
                suggestion: None,
            });
        } else if input_lower.contains("<script") && !input_lower.contains("defer") {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "Render-blocking scripts".to_string(),
                description: "Scripts may block page rendering".to_string(),
                location: None,
                suggestion: Some("Add async or defer attribute to scripts".to_string()),
            });
        }

        if input_lower.contains("font-display") || input.contains("fontDisplay") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Font display strategy".to_string(),
                description: "Custom fonts have display strategy".to_string(),
                location: None,
                suggestion: Some("Use font-display: swap for better CLS".to_string()),
            });
        }

        if input.contains("width=") && input.contains("height=") && input_lower.contains("img") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Image dimensions specified".to_string(),
                description: "Reduces Cumulative Layout Shift (CLS)".to_string(),
                location: None,
                suggestion: None,
            });
        } else if input_lower.contains("<img") {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "Images without dimensions".to_string(),
                description: "Missing dimensions can cause layout shift".to_string(),
                location: None,
                suggestion: Some("Add width and height attributes to img tags".to_string()),
            });
        }

        if input_lower.contains("webp") || input_lower.contains("avif") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Modern image formats".to_string(),
                description: "Using WebP/AVIF for better compression".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input.contains("IntersectionObserver") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "IntersectionObserver used".to_string(),
                description: "Efficient visibility detection".to_string(),
                location: None,
                suggestion: None,
            });
        }

        let score = 75;

        DomainAnalysis {
            agent: DomainAgentType::Web,
            category: "Core Web Vitals".to_string(),
            findings,
            recommendations: vec![
                "Optimize Largest Contentful Paint (LCP < 2.5s)".to_string(),
                "Minimize Cumulative Layout Shift (CLS < 0.1)".to_string(),
                "Improve First Input Delay (FID < 100ms)".to_string(),
                "Use async/defer for non-critical scripts".to_string(),
                "Specify image dimensions".to_string(),
                "Implement lazy loading for below-fold content".to_string(),
            ],
            score,
        }
    }

    fn analyze_accessibility(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        if ARIA_REGEX.is_match(input) {
            findings.push(Finding {
                severity: Severity::Success,
                title: "ARIA attributes detected".to_string(),
                description: "Accessibility roles and properties defined".to_string(),
                location: None,
                suggestion: None,
            });
        } else {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "No ARIA attributes detected".to_string(),
                description: "Screen reader experience may be limited".to_string(),
                location: None,
                suggestion: Some("Add ARIA labels and roles to interactive elements".to_string()),
            });
        }

        if input_lower.contains("alt=") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Alt text for images".to_string(),
                description: "Images have alternative text".to_string(),
                location: None,
                suggestion: None,
            });
        } else if input_lower.contains("<img") {
            findings.push(Finding {
                severity: Severity::Critical,
                title: "Missing alt text".to_string(),
                description: "Images without alt text are inaccessible".to_string(),
                location: None,
                suggestion: Some("Add alt attribute to all img tags".to_string()),
            });
        }

        if input_lower.contains("<label") && input_lower.contains("for=") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Labels associated with inputs".to_string(),
                description: "Form labels properly linked".to_string(),
                location: None,
                suggestion: None,
            });
        } else if input_lower.contains("<input") {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "Form accessibility check".to_string(),
                description: "Ensure inputs have associated labels".to_string(),
                location: None,
                suggestion: Some("Use <label for='id'> or aria-label".to_string()),
            });
        }

        if input_lower.contains("tabindex") {
            if input.contains("tabindex=\"-1\"") {
                findings.push(Finding {
                    severity: Severity::Info,
                    title: "Programmatic focus management".to_string(),
                    description: "Elements removed from tab order".to_string(),
                    location: None,
                    suggestion: Some("Ensure focus management is logical".to_string()),
                });
            } else if input.contains("tabindex=\"0\"") {
                findings.push(Finding {
                    severity: Severity::Success,
                    title: "Custom focusable elements".to_string(),
                    description: "Elements added to natural tab order".to_string(),
                    location: None,
                    suggestion: None,
                });
            }
        }

        if input_lower.contains("skip") && input_lower.contains("link") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Skip links detected".to_string(),
                description: "Keyboard users can skip navigation".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("focus-visible") || input.contains(":focus-visible") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Focus styles implemented".to_string(),
                description: "Keyboard focus is visible".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input.contains("prefers-reduced-motion") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Reduced motion support".to_string(),
                description: "Respects user motion preferences".to_string(),
                location: None,
                suggestion: None,
            });
        }

        let critical_count = findings.iter().filter(|f| f.severity == Severity::Critical).count();
        let score = if critical_count > 0 { 45 } else { 80 };

        DomainAnalysis {
            agent: DomainAgentType::Web,
            category: "Accessibility (WCAG 2.1)".to_string(),
            findings,
            recommendations: vec![
                "Add alt text to all images".to_string(),
                "Use semantic HTML elements".to_string(),
                "Ensure keyboard navigation works".to_string(),
                "Maintain sufficient color contrast".to_string(),
                "Provide skip links for navigation".to_string(),
                "Test with screen readers (NVDA, VoiceOver)".to_string(),
            ],
            score,
        }
    }

    fn analyze_seo(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        if input_lower.contains("<title>") || input.contains("document.title") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Page title detected".to_string(),
                description: "Title tag is present".to_string(),
                location: None,
                suggestion: Some("Keep titles under 60 characters".to_string()),
            });
        } else {
            findings.push(Finding {
                severity: Severity::Critical,
                title: "Missing page title".to_string(),
                description: "Title tag is essential for SEO".to_string(),
                location: None,
                suggestion: Some("Add unique, descriptive <title> tag".to_string()),
            });
        }

        if input_lower.contains("meta") && input_lower.contains("description") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Meta description detected".to_string(),
                description: "Description for search results".to_string(),
                location: None,
                suggestion: Some("Keep descriptions 150-160 characters".to_string()),
            });
        } else {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "Missing meta description".to_string(),
                description: "Improves click-through rate from search".to_string(),
                location: None,
                suggestion: Some("Add meta name='description' content='...'".to_string()),
            });
        }

        if input_lower.contains("<h1") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "H1 heading detected".to_string(),
                description: "Primary heading present".to_string(),
                location: None,
                suggestion: Some("Use only one H1 per page".to_string()),
            });
        }

        if input_lower.contains("og:") || input_lower.contains("opengraph") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Open Graph tags detected".to_string(),
                description: "Social media sharing optimized".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("twitter:") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Twitter Card tags detected".to_string(),
                description: "Twitter sharing optimized".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("canonical") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Canonical URL specified".to_string(),
                description: "Prevents duplicate content issues".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("schema.org") || input.contains("\"@type\"") || input_lower.contains("jsonld") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Structured data detected".to_string(),
                description: "Rich snippets in search results".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("robots") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Robots meta tag detected".to_string(),
                description: "Search engine indexing controlled".to_string(),
                location: None,
                suggestion: Some("Ensure important pages are not blocked".to_string()),
            });
        }

        let critical_count = findings.iter().filter(|f| f.severity == Severity::Critical).count();
        let score = if critical_count > 0 { 50 } else { 80 };

        DomainAnalysis {
            agent: DomainAgentType::Web,
            category: "SEO Analysis".to_string(),
            findings,
            recommendations: vec![
                "Add unique title and meta description".to_string(),
                "Implement Open Graph tags".to_string(),
                "Add structured data (JSON-LD)".to_string(),
                "Use semantic heading hierarchy".to_string(),
                "Create XML sitemap".to_string(),
                "Ensure mobile-friendly design".to_string(),
            ],
            score,
        }
    }

    fn analyze_bundle(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        let is_react = REACT_REGEX.is_match(input);
        let is_vue = VUE_REGEX.is_match(input);
        let is_angular = ANGULAR_REGEX.is_match(input);

        if is_react {
            findings.push(Finding {
                severity: Severity::Info,
                title: "React framework detected".to_string(),
                description: "React-based application".to_string(),
                location: None,
                suggestion: None,
            });

            if input.contains("React.lazy") || input.contains("Suspense") {
                findings.push(Finding {
                    severity: Severity::Success,
                    title: "Code splitting with React.lazy".to_string(),
                    description: "Components loaded on-demand".to_string(),
                    location: None,
                    suggestion: None,
                });
            }

            if input.contains("useMemo") || input.contains("useCallback") {
                findings.push(Finding {
                    severity: Severity::Success,
                    title: "React memoization used".to_string(),
                    description: "Preventing unnecessary re-renders".to_string(),
                    location: None,
                    suggestion: None,
                });
            }
        }

        if is_vue {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Vue.js framework detected".to_string(),
                description: "Vue-based application".to_string(),
                location: None,
                suggestion: None,
            });

            if input.contains("defineAsyncComponent") {
                findings.push(Finding {
                    severity: Severity::Success,
                    title: "Async components used".to_string(),
                    description: "Components loaded on-demand".to_string(),
                    location: None,
                    suggestion: None,
                });
            }
        }

        if is_angular {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Angular framework detected".to_string(),
                description: "Angular-based application".to_string(),
                location: None,
                suggestion: None,
            });

            if input.contains("loadChildren") {
                findings.push(Finding {
                    severity: Severity::Success,
                    title: "Lazy-loaded routes".to_string(),
                    description: "Route-based code splitting".to_string(),
                    location: None,
                    suggestion: None,
                });
            }
        }

        if input_lower.contains("import(") || input.contains("import(") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Dynamic imports detected".to_string(),
                description: "Code splitting with dynamic imports".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("tree-shaking") || input_lower.contains("treeshaking") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Tree-shaking mentioned".to_string(),
                description: "Dead code elimination".to_string(),
                location: None,
                suggestion: Some("Use ES modules for effective tree-shaking".to_string()),
            });
        }

        if input_lower.contains("terser") || input_lower.contains("uglify") || input_lower.contains("minify") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Code minification".to_string(),
                description: "JavaScript is minified".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("gzip") || input_lower.contains("brotli") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Compression configured".to_string(),
                description: "Assets served compressed".to_string(),
                location: None,
                suggestion: None,
            });
        }

        let score = 75;

        DomainAnalysis {
            agent: DomainAgentType::Web,
            category: "Bundle Analysis".to_string(),
            findings,
            recommendations: vec![
                "Implement code splitting".to_string(),
                "Use dynamic imports for routes".to_string(),
                "Enable tree-shaking".to_string(),
                "Minify JavaScript and CSS".to_string(),
                "Enable Gzip/Brotli compression".to_string(),
                "Analyze bundle with webpack-bundle-analyzer".to_string(),
            ],
            score,
        }
    }

    fn analyze_performance(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        if input.contains("requestAnimationFrame") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "requestAnimationFrame used".to_string(),
                description: "Smooth animations with proper timing".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input.contains("throttle") || input.contains("debounce") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Event throttling/debouncing".to_string(),
                description: "Preventing excessive event handlers".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input.contains("Web Worker") || input.contains("new Worker") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Web Workers detected".to_string(),
                description: "Heavy computation off main thread".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("localstorage") || input_lower.contains("sessionstorage") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Web Storage API used".to_string(),
                description: "Local data persistence".to_string(),
                location: None,
                suggestion: Some("Consider IndexedDB for larger data sets".to_string()),
            });
        }

        if input_lower.contains("indexeddb") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "IndexedDB used".to_string(),
                description: "Efficient client-side storage".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("serviceworker") || input.contains("serviceWorker") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Service Worker detected".to_string(),
                description: "Offline support and caching".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input.contains("virtual") && (input_lower.contains("scroll") || input_lower.contains("list")) {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Virtual scrolling implemented".to_string(),
                description: "Efficient rendering of large lists".to_string(),
                location: None,
                suggestion: None,
            });
        }

        let score = 75;

        DomainAnalysis {
            agent: DomainAgentType::Web,
            category: "Performance Analysis".to_string(),
            findings,
            recommendations: vec![
                "Use throttling/debouncing for events".to_string(),
                "Offload heavy work to Web Workers".to_string(),
                "Implement virtual scrolling for lists".to_string(),
                "Use Service Workers for caching".to_string(),
                "Optimize animations with requestAnimationFrame".to_string(),
                "Profile with Chrome DevTools Performance tab".to_string(),
            ],
            score,
        }
    }

    fn analyze_security(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        if input.contains("innerHTML") || input.contains("dangerouslySetInnerHTML") {
            findings.push(Finding {
                severity: Severity::Critical,
                title: "XSS vulnerability risk".to_string(),
                description: "Direct HTML injection detected".to_string(),
                location: None,
                suggestion: Some("Sanitize content with DOMPurify".to_string()),
            });
        }

        if input.contains("eval(") {
            findings.push(Finding {
                severity: Severity::Critical,
                title: "eval() usage detected".to_string(),
                description: "Code injection vulnerability".to_string(),
                location: None,
                suggestion: Some("Avoid eval(), use JSON.parse or alternatives".to_string()),
            });
        }

        if input_lower.contains("content-security-policy") || input_lower.contains("csp") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Content Security Policy".to_string(),
                description: "CSP headers configured".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("httponly") || input_lower.contains("secure") && input_lower.contains("cookie") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Secure cookie configuration".to_string(),
                description: "Cookies have security flags".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("cors") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "CORS handling detected".to_string(),
                description: "Cross-origin resource sharing configured".to_string(),
                location: None,
                suggestion: Some("Ensure CORS is restrictive".to_string()),
            });
        }

        if input_lower.contains("https") || input_lower.contains("ssl") || input_lower.contains("tls") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "HTTPS/TLS usage".to_string(),
                description: "Secure transport layer".to_string(),
                location: None,
                suggestion: None,
            });
        }

        let critical_count = findings.iter().filter(|f| f.severity == Severity::Critical).count();
        let score = if critical_count > 0 { 40 } else { 85 };

        DomainAnalysis {
            agent: DomainAgentType::Web,
            category: "Security Audit".to_string(),
            findings,
            recommendations: vec![
                "Implement Content Security Policy".to_string(),
                "Avoid innerHTML and eval()".to_string(),
                "Use HTTPS everywhere".to_string(),
                "Set secure cookie flags".to_string(),
                "Sanitize all user input".to_string(),
                "Keep dependencies updated".to_string(),
            ],
            score,
        }
    }

    fn full_analysis(&self, input: &str) -> DomainAnalysis {
        let mut all_findings = Vec::new();

        let analyses = vec![
            self.analyze_core_web_vitals(input),
            self.analyze_accessibility(input),
            self.analyze_seo(input),
            self.analyze_bundle(input),
            self.analyze_performance(input),
            self.analyze_security(input),
        ];

        let mut total_score = 0u32;
        for analysis in &analyses {
            all_findings.extend(analysis.findings.clone());
            total_score += analysis.score as u32;
        }

        let avg_score = (total_score / analyses.len() as u32) as u8;

        DomainAnalysis {
            agent: DomainAgentType::Web,
            category: "Full Web Analysis".to_string(),
            findings: all_findings,
            recommendations: vec![
                "Optimize Core Web Vitals".to_string(),
                "Ensure WCAG 2.1 accessibility".to_string(),
                "Implement SEO best practices".to_string(),
                "Minimize bundle size".to_string(),
                "Add security headers".to_string(),
                "Test across browsers and devices".to_string(),
            ],
            score: avg_score,
        }
    }
}

impl Default for WebAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_agent_creation() {
        let agent = WebAgent::new();
        assert_eq!(agent.name, "Web Agent");
    }

    #[test]
    fn test_react_detection() {
        let agent = WebAgent::new();
        let code = r#"
            const [count, setCount] = useState(0);
            useEffect(() => {
                document.title = `Count: ${count}`;
            }, [count]);
        "#;
        let result = agent.analyze("bundle", code);
        assert!(result.findings.iter().any(|f| f.title.contains("React")));
    }

    #[test]
    fn test_accessibility_analysis() {
        let agent = WebAgent::new();
        let code = r#"
            <img src="photo.jpg" alt="User profile picture">
            <button aria-label="Close dialog">X</button>
        "#;
        let result = agent.analyze("a11y", code);
        assert!(result.findings.iter().any(|f| f.severity == Severity::Success));
    }

    #[test]
    fn test_xss_detection() {
        let agent = WebAgent::new();
        let code = r#"
            element.innerHTML = userInput;
            eval(userCode);
        "#;
        let result = agent.analyze("audit", code);
        assert!(result.findings.iter().any(|f| f.severity == Severity::Critical));
    }
}
