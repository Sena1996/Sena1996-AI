use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryLevel {
    Session,
    Project,
    Global,
    Permanent,
}

impl std::fmt::Display for MemoryLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryLevel::Session => write!(f, "Session"),
            MemoryLevel::Project => write!(f, "Project"),
            MemoryLevel::Global => write!(f, "Global"),
            MemoryLevel::Permanent => write!(f, "Permanent"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub level: MemoryLevel,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u64,
    pub metadata: HashMap<String, String>,
}

impl KnowledgeEntry {
    pub fn new(title: &str, content: &str, level: MemoryLevel) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            content: content.to_string(),
            tags: Vec::new(),
            level,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            metadata: HashMap::new(),
        }
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    pub fn with_tags(mut self, tags: &[&str]) -> Self {
        for tag in tags {
            self.tags.push(tag.to_string());
        }
        self
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn record_access(&mut self) {
        self.last_accessed = Utc::now();
        self.access_count += 1;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySystem {
    session_memory: HashMap<String, KnowledgeEntry>,
    project_memory: HashMap<String, KnowledgeEntry>,
    global_memory: HashMap<String, KnowledgeEntry>,
    permanent_memory: HashMap<String, KnowledgeEntry>,
    #[serde(skip)]
    memory_file: PathBuf,
}

impl MemorySystem {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let memory_file = home.join(".sena").join("memory.json");

        let mut system = Self {
            session_memory: HashMap::new(),
            project_memory: HashMap::new(),
            global_memory: HashMap::new(),
            permanent_memory: HashMap::new(),
            memory_file,
        };

        let _ = system.load();

        if system.permanent_memory.is_empty() {
            system.initialize_permanent_knowledge();
        }

        system
    }

    fn initialize_permanent_knowledge(&mut self) {
        self.store(KnowledgeEntry::new(
            "First Principles Thinking",
            "Break complex problems down to fundamental truths and rebuild from there.\n\
            Process:\n\
            1. Identify and define current assumptions\n\
            2. Break down the problem into fundamental principles\n\
            3. Rebuild from the ground up",
            MemoryLevel::Permanent,
        ).with_tags(&["reasoning", "analysis", "problem-solving"]));

        self.store(KnowledgeEntry::new(
            "Root Cause Analysis",
            "Identify underlying causes, not just symptoms.\n\
            5 Whys Technique: Ask 'Why?' repeatedly until you reach the root cause.\n\
            Fishbone Diagram: Categorize causes into People, Process, Technology, Environment.",
            MemoryLevel::Permanent,
        ).with_tags(&["reasoning", "debugging", "analysis"]));

        self.store(KnowledgeEntry::new(
            "SQL Injection Prevention",
            "SECURE: Use parameterized queries\n\
            const user = await db.query('SELECT * FROM users WHERE email = $1', [email]);\n\n\
            INSECURE: String concatenation\n\
            const user = await db.query(`SELECT * FROM users WHERE email = '${email}'`);",
            MemoryLevel::Permanent,
        ).with_tags(&["security", "database", "owasp"]));

        self.store(KnowledgeEntry::new(
            "XSS Prevention",
            "SECURE: Output encoding with DOMPurify or html-escaper\n\
            const safeHTML = DOMPurify.sanitize(userInput);\n\n\
            React automatically escapes: <div>{userInput}</div>\n\n\
            INSECURE: element.innerHTML = userInput;",
            MemoryLevel::Permanent,
        ).with_tags(&["security", "frontend", "owasp"]));

        self.store(KnowledgeEntry::new(
            "Big O Performance Guide",
            "O(1) - Constant - Hash table lookup\n\
            O(log n) - Logarithmic - Binary search\n\
            O(n) - Linear - Single loop\n\
            O(n log n) - Linearithmic - Merge sort\n\
            O(n²) - Quadratic - Nested loops\n\
            O(2ⁿ) - Exponential - Recursive Fibonacci",
            MemoryLevel::Permanent,
        ).with_tags(&["performance", "algorithms", "complexity"]));

        self.store(KnowledgeEntry::new(
            "N+1 Query Problem",
            "BAD: 1 query for users + N queries for posts\n\
            for (const user of users) { user.posts = await Post.findAll({userId: user.id}); }\n\n\
            GOOD: Single query with JOIN or eager loading\n\
            const users = await User.findAll({ include: [{ model: Post }] });",
            MemoryLevel::Permanent,
        ).with_tags(&["performance", "database", "optimization"]));

        self.store(KnowledgeEntry::new(
            "SOLID Principles",
            "S - Single Responsibility: One class, one responsibility\n\
            O - Open/Closed: Open for extension, closed for modification\n\
            L - Liskov Substitution: Subtypes must be substitutable\n\
            I - Interface Segregation: Small, specific interfaces\n\
            D - Dependency Inversion: Depend on abstractions",
            MemoryLevel::Permanent,
        ).with_tags(&["architecture", "design", "principles"]));

        self.store(KnowledgeEntry::new(
            "Design Patterns",
            "Creational: Factory, Singleton, Builder\n\
            Structural: Adapter, Decorator, Facade\n\
            Behavioral: Strategy, Observer, Command",
            MemoryLevel::Permanent,
        ).with_tags(&["architecture", "design", "patterns"]));
    }

    pub fn store(&mut self, entry: KnowledgeEntry) {
        let memory = match entry.level {
            MemoryLevel::Session => &mut self.session_memory,
            MemoryLevel::Project => &mut self.project_memory,
            MemoryLevel::Global => &mut self.global_memory,
            MemoryLevel::Permanent => &mut self.permanent_memory,
        };
        memory.insert(entry.id.clone(), entry);
    }

    pub fn retrieve(&mut self, id: &str) -> Option<&KnowledgeEntry> {
        for memory in [
            &mut self.session_memory,
            &mut self.project_memory,
            &mut self.global_memory,
            &mut self.permanent_memory,
        ] {
            if let Some(entry) = memory.get_mut(id) {
                entry.record_access();
                return Some(entry);
            }
        }
        None
    }

    pub fn search(&self, query: &str) -> Vec<&KnowledgeEntry> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<&KnowledgeEntry> = Vec::new();

        for memory in [
            &self.session_memory,
            &self.project_memory,
            &self.global_memory,
            &self.permanent_memory,
        ] {
            for entry in memory.values() {
                if entry.title.to_lowercase().contains(&query_lower)
                    || entry.content.to_lowercase().contains(&query_lower)
                    || entry.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
                {
                    results.push(entry);
                }
            }
        }

        results.sort_by(|a, b| b.access_count.cmp(&a.access_count));
        results
    }

    pub fn search_by_tag(&self, tag: &str) -> Vec<&KnowledgeEntry> {
        let tag_lower = tag.to_lowercase();
        let mut results: Vec<&KnowledgeEntry> = Vec::new();

        for memory in [
            &self.session_memory,
            &self.project_memory,
            &self.global_memory,
            &self.permanent_memory,
        ] {
            for entry in memory.values() {
                if entry.tags.iter().any(|t| t.to_lowercase() == tag_lower) {
                    results.push(entry);
                }
            }
        }

        results
    }

    pub fn get_by_level(&self, level: MemoryLevel) -> Vec<&KnowledgeEntry> {
        let memory = match level {
            MemoryLevel::Session => &self.session_memory,
            MemoryLevel::Project => &self.project_memory,
            MemoryLevel::Global => &self.global_memory,
            MemoryLevel::Permanent => &self.permanent_memory,
        };
        memory.values().collect()
    }

    pub fn clear_session(&mut self) {
        self.session_memory.clear();
    }

    pub fn total_entries(&self) -> usize {
        self.session_memory.len()
            + self.project_memory.len()
            + self.global_memory.len()
            + self.permanent_memory.len()
    }

    pub fn save(&self) -> Result<(), String> {
        #[derive(Serialize)]
        struct SaveData {
            project_memory: HashMap<String, KnowledgeEntry>,
            global_memory: HashMap<String, KnowledgeEntry>,
        }

        let data = SaveData {
            project_memory: self.project_memory.clone(),
            global_memory: self.global_memory.clone(),
        };

        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Failed to serialize memory: {}", e))?;

        if let Some(parent) = self.memory_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create memory directory: {}", e))?;
        }

        fs::write(&self.memory_file, json)
            .map_err(|e| format!("Failed to write memory file: {}", e))?;

        Ok(())
    }

    pub fn load(&mut self) -> Result<(), String> {
        if !self.memory_file.exists() {
            return Ok(());
        }

        #[derive(Deserialize)]
        struct SaveData {
            project_memory: HashMap<String, KnowledgeEntry>,
            global_memory: HashMap<String, KnowledgeEntry>,
        }

        let content = fs::read_to_string(&self.memory_file)
            .map_err(|e| format!("Failed to read memory file: {}", e))?;

        let data: SaveData = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse memory file: {}", e))?;

        self.project_memory = data.project_memory;
        self.global_memory = data.global_memory;

        Ok(())
    }

    pub fn stats(&self) -> MemoryStats {
        MemoryStats {
            session_count: self.session_memory.len(),
            project_count: self.project_memory.len(),
            global_count: self.global_memory.len(),
            permanent_count: self.permanent_memory.len(),
            total_count: self.total_entries(),
        }
    }
}

impl Default for MemorySystem {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub session_count: usize,
    pub project_count: usize,
    pub global_count: usize,
    pub permanent_count: usize,
    pub total_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_system_creation() {
        let system = MemorySystem::new();
        assert!(system.total_entries() > 0);
        assert!(system.permanent_memory.len() > 0);
    }

    #[test]
    fn test_store_and_retrieve() {
        let mut system = MemorySystem::new();
        let entry = KnowledgeEntry::new("Test Entry", "Test content", MemoryLevel::Session);
        let id = entry.id.clone();
        system.store(entry);

        let retrieved = system.retrieve(&id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title, "Test Entry");
    }

    #[test]
    fn test_search() {
        let system = MemorySystem::new();
        let results = system.search("sql injection");
        assert!(results.len() > 0);
    }

    #[test]
    fn test_search_by_tag() {
        let system = MemorySystem::new();
        let results = system.search_by_tag("security");
        assert!(results.len() > 0);
    }

    #[test]
    fn test_clear_session() {
        let mut system = MemorySystem::new();
        system.store(KnowledgeEntry::new("Test", "Content", MemoryLevel::Session));
        assert!(system.session_memory.len() > 0);

        system.clear_session();
        assert_eq!(system.session_memory.len(), 0);
    }
}
