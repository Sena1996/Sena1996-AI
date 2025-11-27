use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplexityClass {
    Constant,
    Logarithmic,
    Linear,
    Linearithmic,
    Quadratic,
    Cubic,
    Exponential,
    Factorial,
}

impl std::fmt::Display for ComplexityClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComplexityClass::Constant => write!(f, "O(1)"),
            ComplexityClass::Logarithmic => write!(f, "O(log n)"),
            ComplexityClass::Linear => write!(f, "O(n)"),
            ComplexityClass::Linearithmic => write!(f, "O(n log n)"),
            ComplexityClass::Quadratic => write!(f, "O(n²)"),
            ComplexityClass::Cubic => write!(f, "O(n³)"),
            ComplexityClass::Exponential => write!(f, "O(2ⁿ)"),
            ComplexityClass::Factorial => write!(f, "O(n!)"),
        }
    }
}

impl ComplexityClass {
    pub fn examples(&self) -> Vec<&'static str> {
        match self {
            ComplexityClass::Constant => vec!["Hash table lookup", "Array index access"],
            ComplexityClass::Logarithmic => vec!["Binary search", "Balanced tree operations"],
            ComplexityClass::Linear => vec!["Single loop", "Linear search"],
            ComplexityClass::Linearithmic => vec!["Merge sort", "Quicksort (average)"],
            ComplexityClass::Quadratic => vec!["Nested loops", "Bubble sort"],
            ComplexityClass::Cubic => vec!["Matrix multiplication (naive)"],
            ComplexityClass::Exponential => vec!["Recursive Fibonacci", "Brute force"],
            ComplexityClass::Factorial => vec!["Permutations", "Traveling salesman (brute force)"],
        }
    }

    pub fn is_acceptable(&self) -> bool {
        matches!(
            self,
            ComplexityClass::Constant
                | ComplexityClass::Logarithmic
                | ComplexityClass::Linear
                | ComplexityClass::Linearithmic
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePattern {
    pub name: String,
    pub description: String,
    pub category: String,
    pub complexity_before: Option<ComplexityClass>,
    pub complexity_after: Option<ComplexityClass>,
    pub good_example: String,
    pub bad_example: String,
    pub benefits: Vec<String>,
}

impl PerformancePattern {
    pub fn new(name: &str, description: &str, category: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            category: category.to_string(),
            complexity_before: None,
            complexity_after: None,
            good_example: String::new(),
            bad_example: String::new(),
            benefits: Vec::new(),
        }
    }

    pub fn with_complexity(mut self, before: ComplexityClass, after: ComplexityClass) -> Self {
        self.complexity_before = Some(before);
        self.complexity_after = Some(after);
        self
    }

    pub fn with_good_example(mut self, example: &str) -> Self {
        self.good_example = example.to_string();
        self
    }

    pub fn with_bad_example(mut self, example: &str) -> Self {
        self.bad_example = example.to_string();
        self
    }

    pub fn with_benefit(mut self, benefit: &str) -> Self {
        self.benefits.push(benefit.to_string());
        self
    }

    pub fn with_benefits(mut self, benefits: &[&str]) -> Self {
        for b in benefits {
            self.benefits.push(b.to_string());
        }
        self
    }
}

impl std::fmt::Display for PerformancePattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══════════════════════════════════════")?;
        writeln!(f, "  ⚡ {}", self.name)?;
        writeln!(f, "═══════════════════════════════════════")?;
        writeln!(f)?;
        writeln!(f, "{}", self.description)?;
        writeln!(f, "Category: {}", self.category)?;

        if let (Some(before), Some(after)) = (&self.complexity_before, &self.complexity_after) {
            writeln!(f)?;
            writeln!(f, "Complexity: {} → {}", before, after)?;
        }

        if !self.bad_example.is_empty() {
            writeln!(f)?;
            writeln!(f, "❌ BAD:")?;
            writeln!(f, "{}", self.bad_example)?;
        }

        if !self.good_example.is_empty() {
            writeln!(f)?;
            writeln!(f, "✅ GOOD:")?;
            writeln!(f, "{}", self.good_example)?;
        }

        if !self.benefits.is_empty() {
            writeln!(f)?;
            writeln!(f, "Benefits:")?;
            for benefit in &self.benefits {
                writeln!(f, "  • {}", benefit)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub title: String,
    pub description: String,
    pub impact: u8,
    pub effort: u8,
    pub pattern: Option<String>,
}

pub fn default_patterns() -> Vec<PerformancePattern> {
    vec![
        PerformancePattern::new(
            "Hash Set for Lookups",
            "Use hash sets for O(1) membership testing instead of array iteration.",
            "Algorithms",
        )
        .with_complexity(ComplexityClass::Quadratic, ComplexityClass::Linear)
        .with_bad_example(
            "// O(n²) - Nested loops for finding duplicates\n\
            for (let i = 0; i < arr.length; i++) {\n\
                for (let j = i + 1; j < arr.length; j++) {\n\
                    if (arr[i] === arr[j]) duplicates.push(arr[i]);\n\
                }\n\
            }",
        )
        .with_good_example(
            "// O(n) - Hash set for O(1) lookup\n\
            const seen = new Set();\n\
            const duplicates = [];\n\
            for (const item of arr) {\n\
                if (seen.has(item)) duplicates.push(item);\n\
                seen.add(item);\n\
            }",
        )
        .with_benefits(&[
            "O(n) instead of O(n²)",
            "Scales to large datasets",
            "Constant-time lookups",
        ]),
        PerformancePattern::new(
            "N+1 Query Prevention",
            "Avoid N+1 queries by using eager loading or batch queries.",
            "Database",
        )
        .with_bad_example(
            "// N+1 queries (1 + N database calls)\n\
            const users = await User.findAll();\n\
            for (const user of users) {\n\
                user.posts = await Post.findAll({ where: { userId: user.id } });\n\
            }",
        )
        .with_good_example(
            "// Single query with JOIN or eager loading\n\
            const users = await User.findAll({\n\
                include: [{ model: Post }]\n\
            });",
        )
        .with_benefits(&[
            "Reduces database round-trips",
            "Dramatically faster for large datasets",
            "Reduces database load",
        ]),
        PerformancePattern::new(
            "Query Batching",
            "Batch multiple queries into a single database call.",
            "Database",
        )
        .with_bad_example(
            "// N database round-trips\n\
            for (const id of userIds) {\n\
                await User.findByPk(id);\n\
            }",
        )
        .with_good_example(
            "// 1 database round-trip\n\
            const users = await User.findAll({\n\
                where: { id: userIds }\n\
            });",
        )
        .with_benefits(&[
            "Single database round-trip",
            "Reduces network latency",
            "More efficient query execution",
        ]),
        PerformancePattern::new(
            "Database Indexing",
            "Add indexes to frequently queried columns.",
            "Database",
        )
        .with_bad_example(
            "-- Full table scan\n\
            SELECT * FROM users WHERE email = 'user@example.com';",
        )
        .with_good_example(
            "-- Index lookup\n\
            CREATE INDEX idx_users_email ON users(email);\n\
            SELECT * FROM users WHERE email = 'user@example.com';\n\n\
            -- Composite index for multi-column queries\n\
            CREATE INDEX idx_users_status_created ON users(status, created_at);",
        )
        .with_benefits(&[
            "O(log n) instead of O(n) lookups",
            "Faster ORDER BY operations",
            "Improved query performance",
        ]),
        PerformancePattern::new(
            "Cache-Aside Pattern",
            "Check cache first, load from database on miss, then cache the result.",
            "Caching",
        )
        .with_good_example(
            "async function getUser(id: string): Promise<User> {\n\
                // Try cache first\n\
                const cached = await redis.get(`user:${id}`);\n\
                if (cached) return JSON.parse(cached);\n\n\
                // Cache miss: fetch from database\n\
                const user = await User.findByPk(id);\n\n\
                // Store in cache\n\
                await redis.setex(`user:${id}`, 3600, JSON.stringify(user));\n\n\
                return user;\n\
            }",
        )
        .with_benefits(&[
            "Reduces database load",
            "Sub-millisecond response times for cached data",
            "Graceful degradation if cache fails",
        ]),
        PerformancePattern::new(
            "Parallel Execution",
            "Run independent async operations in parallel instead of sequentially.",
            "Async",
        )
        .with_bad_example(
            "// Sequential: Total time = T1 + T2 + T3\n\
            const user = await fetchUser(id);\n\
            const posts = await fetchPosts(id);\n\
            const comments = await fetchComments(id);",
        )
        .with_good_example(
            "// Parallel: Total time = max(T1, T2, T3)\n\
            const [user, posts, comments] = await Promise.all([\n\
                fetchUser(id),\n\
                fetchPosts(id),\n\
                fetchComments(id)\n\
            ]);",
        )
        .with_benefits(&[
            "Reduced total wait time",
            "Better utilization of I/O",
            "Improved user experience",
        ]),
        PerformancePattern::new(
            "Memoization",
            "Cache function results to avoid redundant computation.",
            "Algorithms",
        )
        .with_complexity(ComplexityClass::Exponential, ComplexityClass::Linear)
        .with_bad_example(
            "// Exponential time - recalculates same values\n\
            function fib(n: number): number {\n\
                if (n <= 1) return n;\n\
                return fib(n - 1) + fib(n - 2);\n\
            }",
        )
        .with_good_example(
            "// Linear time - cached results\n\
            const fibMemo = (() => {\n\
                const cache = new Map<number, number>();\n\
                return function fib(n: number): number {\n\
                    if (n <= 1) return n;\n\
                    if (cache.has(n)) return cache.get(n)!;\n\n\
                    const result = fib(n - 1) + fib(n - 2);\n\
                    cache.set(n, result);\n\
                    return result;\n\
                };\n\
            })();",
        )
        .with_benefits(&[
            "Avoids redundant computation",
            "Can turn exponential into linear",
            "Useful for recursive algorithms",
        ]),
        PerformancePattern::new(
            "Debouncing",
            "Delay execution until a pause in rapid calls (e.g., user typing).",
            "Frontend",
        )
        .with_good_example(
            "function debounce<T extends (...args: any[]) => any>(\n\
                func: T,\n\
                delay: number\n\
            ): (...args: Parameters<T>) => void {\n\
                let timeoutId: NodeJS.Timeout;\n\
                return (...args) => {\n\
                    clearTimeout(timeoutId);\n\
                    timeoutId = setTimeout(() => func(...args), delay);\n\
                };\n\
            }\n\n\
            // Usage: Only searches after user stops typing for 300ms\n\
            const handleSearch = debounce(search, 300);",
        )
        .with_benefits(&[
            "Reduces unnecessary API calls",
            "Improves perceived performance",
            "Reduces server load",
        ]),
        PerformancePattern::new(
            "Pagination",
            "Load data in chunks instead of all at once.",
            "Database",
        )
        .with_bad_example(
            "// DANGEROUS: Could be millions of rows!\n\
            const users = await User.findAll();",
        )
        .with_good_example(
            "// Offset pagination\n\
            const users = await User.findAll({\n\
                limit: 20,\n\
                offset: (page - 1) * 20,\n\
                order: [['created_at', 'DESC']]\n\
            });\n\n\
            // Cursor-based (more efficient for large datasets)\n\
            const users = await User.findAll({\n\
                where: { id: { [Op.gt]: lastSeenId } },\n\
                limit: 20,\n\
                order: [['id', 'ASC']]\n\
            });",
        )
        .with_benefits(&[
            "Bounded memory usage",
            "Faster initial load",
            "Better UX with progressive loading",
        ]),
        PerformancePattern::new(
            "Object Pooling",
            "Reuse expensive objects instead of creating new ones.",
            "Memory",
        )
        .with_good_example(
            "class ObjectPool<T> {\n\
                private available: T[] = [];\n\
                private inUse = new Set<T>();\n\n\
                constructor(private factory: () => T, initialSize: number = 10) {\n\
                    for (let i = 0; i < initialSize; i++) {\n\
                        this.available.push(factory());\n\
                    }\n\
                }\n\n\
                acquire(): T {\n\
                    let obj = this.available.pop() || this.factory();\n\
                    this.inUse.add(obj);\n\
                    return obj;\n\
                }\n\n\
                release(obj: T): void {\n\
                    this.inUse.delete(obj);\n\
                    this.available.push(obj);\n\
                }\n\
            }",
        )
        .with_benefits(&[
            "Reduces garbage collection pressure",
            "Faster object acquisition",
            "Useful for database connections, workers",
        ]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_patterns() {
        let patterns = default_patterns();
        assert!(patterns.len() >= 5);
    }

    #[test]
    fn test_complexity_display() {
        assert_eq!(format!("{}", ComplexityClass::Linear), "O(n)");
        assert_eq!(format!("{}", ComplexityClass::Quadratic), "O(n²)");
    }

    #[test]
    fn test_complexity_acceptable() {
        assert!(ComplexityClass::Linear.is_acceptable());
        assert!(!ComplexityClass::Exponential.is_acceptable());
    }

    #[test]
    fn test_pattern_display() {
        let pattern = &default_patterns()[0];
        let display = format!("{}", pattern);
        assert!(display.contains("Hash Set"));
    }
}
