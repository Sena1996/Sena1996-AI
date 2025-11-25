use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SolidPrinciple {
    SingleResponsibility,
    OpenClosed,
    LiskovSubstitution,
    InterfaceSegregation,
    DependencyInversion,
}

impl std::fmt::Display for SolidPrinciple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SolidPrinciple::SingleResponsibility => write!(f, "Single Responsibility (S)"),
            SolidPrinciple::OpenClosed => write!(f, "Open/Closed (O)"),
            SolidPrinciple::LiskovSubstitution => write!(f, "Liskov Substitution (L)"),
            SolidPrinciple::InterfaceSegregation => write!(f, "Interface Segregation (I)"),
            SolidPrinciple::DependencyInversion => write!(f, "Dependency Inversion (D)"),
        }
    }
}

impl SolidPrinciple {
    pub fn description(&self) -> &'static str {
        match self {
            SolidPrinciple::SingleResponsibility => "A class should have one, and only one, reason to change.",
            SolidPrinciple::OpenClosed => "Software entities should be open for extension, but closed for modification.",
            SolidPrinciple::LiskovSubstitution => "Subtypes must be substitutable for their base types.",
            SolidPrinciple::InterfaceSegregation => "Clients should not be forced to depend on interfaces they do not use.",
            SolidPrinciple::DependencyInversion => "Depend on abstractions, not concretions.",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesignPattern {
    Factory,
    Singleton,
    Builder,
    Prototype,

    Adapter,
    Decorator,
    Facade,
    Proxy,

    Strategy,
    Observer,
    Command,
    State,
}

impl std::fmt::Display for DesignPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DesignPattern::Factory => write!(f, "Factory"),
            DesignPattern::Singleton => write!(f, "Singleton"),
            DesignPattern::Builder => write!(f, "Builder"),
            DesignPattern::Prototype => write!(f, "Prototype"),
            DesignPattern::Adapter => write!(f, "Adapter"),
            DesignPattern::Decorator => write!(f, "Decorator"),
            DesignPattern::Facade => write!(f, "Facade"),
            DesignPattern::Proxy => write!(f, "Proxy"),
            DesignPattern::Strategy => write!(f, "Strategy"),
            DesignPattern::Observer => write!(f, "Observer"),
            DesignPattern::Command => write!(f, "Command"),
            DesignPattern::State => write!(f, "State"),
        }
    }
}

impl DesignPattern {
    pub fn category(&self) -> &'static str {
        match self {
            DesignPattern::Factory | DesignPattern::Singleton | DesignPattern::Builder | DesignPattern::Prototype => "Creational",
            DesignPattern::Adapter | DesignPattern::Decorator | DesignPattern::Facade | DesignPattern::Proxy => "Structural",
            DesignPattern::Strategy | DesignPattern::Observer | DesignPattern::Command | DesignPattern::State => "Behavioral",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            DesignPattern::Factory => "Creates objects without specifying the exact class to create.",
            DesignPattern::Singleton => "Ensures a class has only one instance with a global access point.",
            DesignPattern::Builder => "Constructs complex objects step by step.",
            DesignPattern::Prototype => "Creates new objects by cloning an existing object.",
            DesignPattern::Adapter => "Allows incompatible interfaces to work together.",
            DesignPattern::Decorator => "Adds behavior to objects dynamically.",
            DesignPattern::Facade => "Provides a simplified interface to a complex subsystem.",
            DesignPattern::Proxy => "Provides a placeholder for another object to control access.",
            DesignPattern::Strategy => "Defines a family of algorithms and makes them interchangeable.",
            DesignPattern::Observer => "Defines a one-to-many dependency between objects.",
            DesignPattern::Command => "Encapsulates a request as an object.",
            DesignPattern::State => "Allows an object to alter its behavior when its state changes.",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturePattern {
    pub name: String,
    pub description: String,
    pub category: String,
    pub diagram: Option<String>,
    pub example: String,
    pub benefits: Vec<String>,
    pub drawbacks: Vec<String>,
    pub use_cases: Vec<String>,
}

impl ArchitecturePattern {
    pub fn new(name: &str, description: &str, category: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            category: category.to_string(),
            diagram: None,
            example: String::new(),
            benefits: Vec::new(),
            drawbacks: Vec::new(),
            use_cases: Vec::new(),
        }
    }

    pub fn with_diagram(mut self, diagram: &str) -> Self {
        self.diagram = Some(diagram.to_string());
        self
    }

    pub fn with_example(mut self, example: &str) -> Self {
        self.example = example.to_string();
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

    pub fn with_drawback(mut self, drawback: &str) -> Self {
        self.drawbacks.push(drawback.to_string());
        self
    }

    pub fn with_drawbacks(mut self, drawbacks: &[&str]) -> Self {
        for d in drawbacks {
            self.drawbacks.push(d.to_string());
        }
        self
    }

    pub fn with_use_case(mut self, use_case: &str) -> Self {
        self.use_cases.push(use_case.to_string());
        self
    }
}

impl std::fmt::Display for ArchitecturePattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══════════════════════════════════════")?;
        writeln!(f, "  🏗️  {}", self.name)?;
        writeln!(f, "═══════════════════════════════════════")?;
        writeln!(f)?;
        writeln!(f, "{}", self.description)?;
        writeln!(f, "Category: {}", self.category)?;

        if let Some(diagram) = &self.diagram {
            writeln!(f)?;
            writeln!(f, "{}", diagram)?;
        }

        if !self.example.is_empty() {
            writeln!(f)?;
            writeln!(f, "Example:")?;
            writeln!(f, "{}", self.example)?;
        }

        if !self.benefits.is_empty() {
            writeln!(f)?;
            writeln!(f, "✅ Benefits:")?;
            for benefit in &self.benefits {
                writeln!(f, "  • {}", benefit)?;
            }
        }

        if !self.drawbacks.is_empty() {
            writeln!(f)?;
            writeln!(f, "❌ Drawbacks:")?;
            for drawback in &self.drawbacks {
                writeln!(f, "  • {}", drawback)?;
            }
        }

        if !self.use_cases.is_empty() {
            writeln!(f)?;
            writeln!(f, "Use cases:")?;
            for use_case in &self.use_cases {
                writeln!(f, "  • {}", use_case)?;
            }
        }

        Ok(())
    }
}

pub fn default_patterns() -> Vec<ArchitecturePattern> {
    vec![
        ArchitecturePattern::new(
            "Single Responsibility Principle",
            "A class should have one, and only one, reason to change.",
            "SOLID",
        )
        .with_example(
            "// BAD: Class does too much\n\
            class User {\n\
                save() { /* DB logic */ }\n\
                sendEmail() { /* Email logic */ }\n\
                validate() { /* Validation logic */ }\n\
            }\n\n\
            // GOOD: Separate responsibilities\n\
            class User { /* domain logic only */ }\n\
            class UserRepository { save(user: User) { } }\n\
            class EmailService { sendWelcomeEmail(user: User) { } }\n\
            class UserValidator { validate(user: User) { } }"
        )
        .with_benefits(&["Easier to understand", "Easier to test", "More maintainable"])
        .with_use_case("Refactoring monolithic classes"),

        ArchitecturePattern::new(
            "Open/Closed Principle",
            "Open for extension, closed for modification.",
            "SOLID",
        )
        .with_example(
            "// BAD: Modify class to add new payment method\n\
            class PaymentProcessor {\n\
                process(type: string, amount: number) {\n\
                    if (type === 'credit') { /* ... */ }\n\
                    else if (type === 'debit') { /* ... */ }\n\
                }\n\
            }\n\n\
            // GOOD: Extend via interface\n\
            interface PaymentMethod {\n\
                process(amount: number): Promise<void>;\n\
            }\n\n\
            class CreditCardPayment implements PaymentMethod { }\n\
            class PayPalPayment implements PaymentMethod { }  // New!\n\n\
            class PaymentProcessor {\n\
                process(method: PaymentMethod, amount: number) {\n\
                    method.process(amount);  // No modification needed\n\
                }\n\
            }"
        )
        .with_benefits(&["Stable code", "Easy to extend", "Less regression risk"]),

        ArchitecturePattern::new(
            "Dependency Inversion Principle",
            "Depend on abstractions, not concretions.",
            "SOLID",
        )
        .with_example(
            "// BAD: High-level depends on low-level\n\
            class UserService {\n\
                private emailService = new EmailService();  // Concrete!\n\
                createUser() { this.emailService.send(); }\n\
            }\n\n\
            // GOOD: Both depend on abstraction\n\
            interface NotificationService {\n\
                send(message: string): Promise<void>;\n\
            }\n\n\
            class UserService {\n\
                constructor(private notifier: NotificationService) {}  // Abstract!\n\
                async createUser() { await this.notifier.send('Welcome!'); }\n\
            }"
        )
        .with_benefits(&["Loose coupling", "Easy to test (mock)", "Swappable implementations"]),

        ArchitecturePattern::new(
            "Factory Pattern",
            "Creates objects without specifying the exact class to create.",
            "Creational",
        )
        .with_example(
            "interface Product {\n\
                operation(): string;\n\
            }\n\n\
            abstract class Creator {\n\
                abstract factoryMethod(): Product;\n\n\
                someOperation(): string {\n\
                    const product = this.factoryMethod();\n\
                    return product.operation();\n\
                }\n\
            }\n\n\
            class ConcreteCreator extends Creator {\n\
                factoryMethod(): Product {\n\
                    return new ConcreteProduct();\n\
                }\n\
            }"
        )
        .with_benefits(&["Decouples creation from usage", "Easy to add new types"])
        .with_use_case("Creating objects based on configuration"),

        ArchitecturePattern::new(
            "Builder Pattern",
            "Constructs complex objects step by step.",
            "Creational",
        )
        .with_example(
            "class QueryBuilder {\n\
                private sql = '';\n\n\
                select(columns: string[]): this {\n\
                    this.sql += `SELECT ${columns.join(', ')} `;\n\
                    return this;\n\
                }\n\n\
                from(table: string): this {\n\
                    this.sql += `FROM ${table} `;\n\
                    return this;\n\
                }\n\n\
                where(condition: string): this {\n\
                    this.sql += `WHERE ${condition} `;\n\
                    return this;\n\
                }\n\n\
                build(): string {\n\
                    return this.sql.trim();\n\
                }\n\
            }\n\n\
            // Usage:\n\
            const query = new QueryBuilder()\n\
                .select(['id', 'name'])\n\
                .from('users')\n\
                .where('age > 18')\n\
                .build();"
        )
        .with_benefits(&["Fluent API", "Immutable construction", "Optional parameters"]),

        ArchitecturePattern::new(
            "Adapter Pattern",
            "Allows incompatible interfaces to work together.",
            "Structural",
        )
        .with_example(
            "// Legacy system\n\
            class LegacyPrinter {\n\
                printOldFormat(text: string) { /* ... */ }\n\
            }\n\n\
            // New interface\n\
            interface ModernPrinter {\n\
                print(document: Document): void;\n\
            }\n\n\
            // Adapter\n\
            class PrinterAdapter implements ModernPrinter {\n\
                constructor(private legacy: LegacyPrinter) {}\n\n\
                print(document: Document): void {\n\
                    const text = document.toString();\n\
                    this.legacy.printOldFormat(text);\n\
                }\n\
            }"
        )
        .with_benefits(&["Integrates legacy code", "No changes to existing code"])
        .with_use_case("Integrating third-party libraries"),

        ArchitecturePattern::new(
            "Decorator Pattern",
            "Adds behavior to objects dynamically.",
            "Structural",
        )
        .with_example(
            "interface DataSource {\n\
                writeData(data: string): void;\n\
                readData(): string;\n\
            }\n\n\
            class FileDataSource implements DataSource { /* ... */ }\n\n\
            class EncryptionDecorator implements DataSource {\n\
                constructor(private wrapped: DataSource) {}\n\n\
                writeData(data: string) {\n\
                    const encrypted = encrypt(data);\n\
                    this.wrapped.writeData(encrypted);\n\
                }\n\n\
                readData(): string {\n\
                    const data = this.wrapped.readData();\n\
                    return decrypt(data);\n\
                }\n\
            }\n\n\
            // Usage:\n\
            let source = new FileDataSource();\n\
            source = new EncryptionDecorator(source);\n\
            source = new CompressionDecorator(source);"
        )
        .with_benefits(&["Composable behaviors", "Single responsibility", "Runtime flexibility"]),

        ArchitecturePattern::new(
            "Strategy Pattern",
            "Defines a family of algorithms and makes them interchangeable.",
            "Behavioral",
        )
        .with_example(
            "interface SortStrategy {\n\
                sort(data: number[]): number[];\n\
            }\n\n\
            class QuickSort implements SortStrategy {\n\
                sort(data: number[]): number[] { /* ... */ }\n\
            }\n\n\
            class MergeSort implements SortStrategy {\n\
                sort(data: number[]): number[] { /* ... */ }\n\
            }\n\n\
            class Sorter {\n\
                constructor(private strategy: SortStrategy) {}\n\n\
                setStrategy(strategy: SortStrategy) {\n\
                    this.strategy = strategy;\n\
                }\n\n\
                sort(data: number[]): number[] {\n\
                    return this.strategy.sort(data);\n\
                }\n\
            }"
        )
        .with_benefits(&["Runtime algorithm swapping", "Easy to add new algorithms"]),

        ArchitecturePattern::new(
            "Observer Pattern",
            "Defines a one-to-many dependency between objects.",
            "Behavioral",
        )
        .with_example(
            "interface Observer {\n\
                update(data: any): void;\n\
            }\n\n\
            class Subject {\n\
                private observers: Observer[] = [];\n\n\
                attach(observer: Observer): void {\n\
                    this.observers.push(observer);\n\
                }\n\n\
                detach(observer: Observer): void {\n\
                    const index = this.observers.indexOf(observer);\n\
                    this.observers.splice(index, 1);\n\
                }\n\n\
                notify(data: any): void {\n\
                    for (const observer of this.observers) {\n\
                        observer.update(data);\n\
                    }\n\
                }\n\
            }"
        )
        .with_benefits(&["Loose coupling", "Dynamic subscription", "Event-driven"])
        .with_use_case("Event systems, pub/sub, reactive programming"),

        ArchitecturePattern::new(
            "Hexagonal Architecture",
            "Separates domain logic from external concerns through ports and adapters.",
            "Architecture",
        )
        .with_diagram(
            "           ┌──────────────────────────┐\n\
            │     Domain Core          │\n\
            │   (Business Logic)       │\n\
            └────────▲──────────▲──────┘\n\
                     │          │\n\
          ┌──────────┘          └──────────┐\n\
          │ Port                      Port │\n\
          │ (Interface)         (Interface)│\n\
     ┌────▼─────┐                  ┌───────▼───┐\n\
     │  Adapter │                  │  Adapter  │\n\
     │  (HTTP)  │                  │   (DB)    │\n\
     └──────────┘                  └───────────┘"
        )
        .with_example(
            "// Port (interface in domain)\n\
            interface UserRepository {\n\
                save(user: User): Promise<void>;\n\
                findById(id: string): Promise<User | null>;\n\
            }\n\n\
            // Domain service uses port\n\
            class UserService {\n\
                constructor(private userRepo: UserRepository) {}\n\
                async createUser(email: string): Promise<User> {\n\
                    const user = new User(email);\n\
                    await this.userRepo.save(user);\n\
                    return user;\n\
                }\n\
            }\n\n\
            // Adapter\n\
            class PostgresUserRepository implements UserRepository { }"
        )
        .with_benefits(&["Domain isolation", "Easy to swap adapters", "Testable"])
        .with_drawbacks(&["More boilerplate", "Learning curve"]),

        ArchitecturePattern::new(
            "CQRS",
            "Command Query Responsibility Segregation - separate read and write models.",
            "Architecture",
        )
        .with_diagram(
            "Commands (Write)          Queries (Read)\n\
                  │                        │\n\
                  ▼                        ▼\n\
            ┌──────────┐            ┌──────────┐\n\
            │  Write   │   Sync/    │   Read   │\n\
            │  Model   │───Async───▶│  Model   │\n\
            └──────────┘            └──────────┘\n\
               (Normalized)         (Denormalized)"
        )
        .with_benefits(&["Optimized read/write", "Scalability", "Complex domains"])
        .with_drawbacks(&["Eventual consistency", "Complexity"])
        .with_use_case("High-performance applications with different read/write patterns"),
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
    fn test_solid_principles() {
        let srp = SolidPrinciple::SingleResponsibility;
        assert!(srp.description().contains("one"));
    }

    #[test]
    fn test_design_patterns() {
        let factory = DesignPattern::Factory;
        assert_eq!(factory.category(), "Creational");
    }

    #[test]
    fn test_pattern_display() {
        let pattern = &default_patterns()[0];
        let display = format!("{}", pattern);
        assert!(display.contains("Single Responsibility"));
    }
}
