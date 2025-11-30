# SENA Architecture Patterns & Design Principles

This document contains architectural knowledge and design patterns for persistent application across all SENA sessions.

---

## 1. CORE ARCHITECTURAL PATTERNS

### Layered Architecture
```
┌──────────────────────────────────────┐
│   Presentation Layer (UI)            │
├──────────────────────────────────────┤
│   Application/Service Layer          │
├──────────────────────────────────────┤
│   Business Logic/Domain Layer        │
├──────────────────────────────────────┤
│   Data Access Layer (DAL)            │
├──────────────────────────────────────┤
│   Database                           │
└──────────────────────────────────────┘

✅ Benefits: Clear separation, easy to understand
❌ Drawbacks: Can become monolithic, tight coupling
```

### Microservices Architecture
```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   User      │     │   Order     │     │   Payment   │
│   Service   │────▶│   Service   │────▶│   Service   │
└─────────────┘     └─────────────┘     └─────────────┘
       │                   │                   │
       └───────────────────┴───────────────────┘
                           │
                    ┌──────▼──────┐
                    │  API Gateway │
                    └─────────────┘

✅ Benefits: Independent scaling, technology flexibility
❌ Drawbacks: Distributed complexity, data consistency
```

### Event-Driven Architecture
```
Producer ──▶ Event Bus ──▶ Consumer
                 │
                 ├──▶ Consumer
                 │
                 └──▶ Consumer

Events: UserCreated, OrderPlaced, PaymentProcessed

✅ Benefits: Loose coupling, scalability, real-time processing
❌ Drawbacks: Eventual consistency, debugging complexity
```

---

## 2. DOMAIN-DRIVEN DESIGN (DDD)

### Bounded Contexts
```
┌─────────────────────┐    ┌─────────────────────┐
│   Sales Context     │    │  Shipping Context   │
│                     │    │                     │
│   Customer (entity) │    │   Customer (VO)     │
│   Order (aggregate) │    │   Shipment (aggr)   │
│   Product (entity)  │    │   Package (entity)  │
└─────────────────────┘    └─────────────────────┘

Same concept (Customer) has different meaning in each context
```

### Aggregate Pattern
```typescript
// Aggregate Root: Order
class Order {
    private id: OrderId;
    private customer: CustomerId;
    private items: OrderItem[] = []; // Entities
    private status: OrderStatus; // Value Object

    // All modifications go through aggregate root
    addItem(product: Product, quantity: number): void {
        // Business rule: Can't add items to completed orders
        if (this.status === OrderStatus.Completed) {
            throw new Error('Cannot modify completed order');
        }

        this.items.push(new OrderItem(product, quantity));
    }

    // Transaction boundary at aggregate
    async save(): Promise<void> {
        await db.transaction(async (trx) => {
            await trx('orders').insert(this.toDTO());
            await trx('order_items').insert(this.items.map(i => i.toDTO()));
        });
    }
}

Rules:
- Only modify aggregate through root
- Reference other aggregates by ID only
- Transaction boundary = aggregate boundary
```

### Value Objects
```typescript
// Value Object: immutable, defined by attributes
class Money {
    constructor(
        public readonly amount: number,
        public readonly currency: string
    ) {
        if (amount < 0) throw new Error('Amount cannot be negative');
    }

    add(other: Money): Money {
        if (this.currency !== other.currency) {
            throw new Error('Currency mismatch');
        }
        return new Money(this.amount + other.amount, this.currency);
    }

    equals(other: Money): boolean {
        return this.amount === other.amount &&
               this.currency === other.currency;
    }
}
```

---

## 3. HEXAGONAL ARCHITECTURE (Ports & Adapters)

```
           ┌──────────────────────────┐
           │     Domain Core          │
           │   (Business Logic)       │
           └────────▲──────────▲──────┘
                    │          │
         ┌──────────┘          └──────────┐
         │ Port                      Port │
         │ (Interface)         (Interface)│
    ┌────▼─────┐                  ┌───────▼───┐
    │  Adapter │                  │  Adapter  │
    │  (HTTP)  │                  │   (DB)    │
    └──────────┘                  └───────────┘

Domain doesn't know about HTTP or DB - only interfaces
```

**Example:**
```typescript
// Port (interface in domain)
interface UserRepository {
    save(user: User): Promise<void>;
    findById(id: string): Promise<User | null>;
}

// Domain service uses port
class UserService {
    constructor(private userRepo: UserRepository) {}

    async createUser(email: string): Promise<User> {
        const user = new User(email);
        await this.userRepo.save(user);
        return user;
    }
}

// Adapter (implementation outside domain)
class PostgresUserRepository implements UserRepository {
    async save(user: User): Promise<void> {
        await db.query('INSERT INTO users...', [user.id, user.email]);
    }

    async findById(id: string): Promise<User | null> {
        const row = await db.query('SELECT * FROM users WHERE id = $1', [id]);
        return row ? User.fromDTO(row) : null;
    }
}

// Easy to swap adapters (testing, migration)
class InMemoryUserRepository implements UserRepository {
    private users = new Map<string, User>();

    async save(user: User): Promise<void> {
        this.users.set(user.id, user);
    }

    async findById(id: string): Promise<User | null> {
        return this.users.get(id) || null;
    }
}
```

---

## 4. CQRS (Command Query Responsibility Segregation)

```
Commands (Write)          Queries (Read)
      │                        │
      ▼                        ▼
┌──────────┐            ┌──────────┐
│  Write   │   Sync/    │   Read   │
│  Model   │───Async───▶│  Model   │
└──────────┘            └──────────┘
   (Normalized)         (Denormalized)
```

**Implementation:**
```typescript
// Command (write)
class CreateOrderCommand {
    constructor(
        public customerId: string,
        public items: OrderItemDTO[]
    ) {}
}

class CreateOrderHandler {
    async handle(cmd: CreateOrderCommand): Promise<void> {
        const order = new Order(cmd.customerId, cmd.items);
        await this.orderRepo.save(order);

        // Publish event for read model
        await this.eventBus.publish(new OrderCreatedEvent(order));
    }
}

// Query (read) - optimized for display
class GetOrderSummaryQuery {
    constructor(public orderId: string) {}
}

class GetOrderSummaryHandler {
    async handle(query: GetOrderSummaryQuery): Promise<OrderSummaryDTO> {
        // Read from denormalized view
        return await this.readModel.getOrderSummary(query.orderId);
    }
}
```

---

## 5. EVENT SOURCING

```
Instead of storing current state:
┌──────┬────────┬────────┐
│  ID  │  Name  │ Status │
├──────┼────────┼────────┤
│  123 │  John  │ Active │
└──────┴────────┴────────┘

Store events:
┌─────┬─────────────────┬──────────────────────┐
│ Seq │ Event Type      │ Data                 │
├─────┼─────────────────┼──────────────────────┤
│  1  │ UserCreated     │ {name: "John"}       │
│  2  │ EmailChanged    │ {email: "j@x.com"}   │
│  3  │ UserActivated   │ {}                   │
└─────┴─────────────────┴──────────────────────┘

Current state = replay all events
```

**Benefits:**
- Complete audit trail
- Time travel (reconstruct state at any point)
- Event-driven integration
- Bug fixing (replay events with new logic)

---

## 6. SOLID PRINCIPLES IN PRACTICE

### Single Responsibility Principle (SRP)
```typescript
// BAD: Class does too much
class User {
    save() { /* DB logic */ }
    sendEmail() { /* Email logic */ }
    validate() { /* Validation logic */ }
}

// GOOD: Separate responsibilities
class User {
    // Just domain logic
}

class UserRepository {
    save(user: User) { /* DB logic */ }
}

class EmailService {
    sendWelcomeEmail(user: User) { /* Email logic */ }
}

class UserValidator {
    validate(user: User) { /* Validation logic */ }
}
```

### Open/Closed Principle (OCP)
```typescript
// Open for extension, closed for modification

// BAD: Modify class to add new payment method
class PaymentProcessor {
    process(type: string, amount: number) {
        if (type === 'credit') { /* ... */ }
        else if (type === 'debit') { /* ... */ }
        // Adding PayPal requires modifying this class
    }
}

// GOOD: Extend via interface
interface PaymentMethod {
    process(amount: number): Promise<void>;
}

class CreditCardPayment implements PaymentMethod {
    async process(amount: number) { /* ... */ }
}

class PayPalPayment implements PaymentMethod {
    async process(amount: number) { /* ... */ }
}

class PaymentProcessor {
    async process(method: PaymentMethod, amount: number) {
        await method.process(amount); // No modification needed for new methods
    }
}
```

### Liskov Substitution Principle (LSP)
```typescript
// Subtypes must be substitutable for base types

// BAD: Square violates LSP
class Rectangle {
    setWidth(w: number) { this.width = w; }
    setHeight(h: number) { this.height = h; }
}

class Square extends Rectangle {
    setWidth(w: number) {
        this.width = w;
        this.height = w; // Breaks expected behavior!
    }
}

// GOOD: Separate types
interface Shape {
    area(): number;
}

class Rectangle implements Shape {
    constructor(private width: number, private height: number) {}
    area() { return this.width * this.height; }
}

class Square implements Shape {
    constructor(private side: number) {}
    area() { return this.side * this.side; }
}
```

### Interface Segregation Principle (ISP)
```typescript
// Clients shouldn't depend on interfaces they don't use

// BAD: Fat interface
interface Worker {
    work(): void;
    eat(): void;
    sleep(): void;
}

class Robot implements Worker {
    work() { /* OK */ }
    eat() { /* Robots don't eat! */ }
    sleep() { /* Robots don't sleep! */ }
}

// GOOD: Segregated interfaces
interface Workable {
    work(): void;
}

interface Eatable {
    eat(): void;
}

interface Sleepable {
    sleep(): void;
}

class Human implements Workable, Eatable, Sleepable {
    work() { /* ... */ }
    eat() { /* ... */ }
    sleep() { /* ... */ }
}

class Robot implements Workable {
    work() { /* ... */ }
}
```

### Dependency Inversion Principle (DIP)
```typescript
// Depend on abstractions, not concretions

// BAD: High-level depends on low-level
class EmailService {
    send() { /* SMTP logic */ }
}

class UserService {
    private emailService = new EmailService(); // Concrete dependency

    createUser(user: User) {
        this.emailService.send(); // Tightly coupled
    }
}

// GOOD: Both depend on abstraction
interface NotificationService {
    send(message: string): Promise<void>;
}

class EmailNotification implements NotificationService {
    async send(message: string) { /* SMTP */ }
}

class SMSNotification implements NotificationService {
    async send(message: string) { /* SMS API */ }
}

class UserService {
    constructor(private notifier: NotificationService) {} // Abstract dependency

    async createUser(user: User) {
        await this.notifier.send('Welcome!'); // Loosely coupled
    }
}
```

---

## 7. DESIGN PATTERNS QUICK REFERENCE

### Creational Patterns

**Factory Method:**
```typescript
interface Product {
    operation(): string;
}

abstract class Creator {
    abstract factoryMethod(): Product;

    someOperation(): string {
        const product = this.factoryMethod();
        return product.operation();
    }
}

class ConcreteCreator extends Creator {
    factoryMethod(): Product {
        return new ConcreteProduct();
    }
}
```

**Singleton:**
```typescript
class DatabaseConnection {
    private static instance: DatabaseConnection;
    private constructor() {}

    static getInstance(): DatabaseConnection {
        if (!DatabaseConnection.instance) {
            DatabaseConnection.instance = new DatabaseConnection();
        }
        return DatabaseConnection.instance;
    }
}
```

**Builder:**
```typescript
class QueryBuilder {
    private sql = '';

    select(columns: string[]): this {
        this.sql += `SELECT ${columns.join(', ')} `;
        return this;
    }

    from(table: string): this {
        this.sql += `FROM ${table} `;
        return this;
    }

    where(condition: string): this {
        this.sql += `WHERE ${condition} `;
        return this;
    }

    build(): string {
        return this.sql.trim();
    }
}

// Usage:
const query = new QueryBuilder()
    .select(['id', 'name'])
    .from('users')
    .where('age > 18')
    .build();
```

### Structural Patterns

**Adapter:**
```typescript
// Legacy system
class LegacyPrinter {
    printOldFormat(text: string) { /* ... */ }
}

// New interface
interface ModernPrinter {
    print(document: Document): void;
}

// Adapter
class PrinterAdapter implements ModernPrinter {
    constructor(private legacy: LegacyPrinter) {}

    print(document: Document): void {
        const text = document.toString();
        this.legacy.printOldFormat(text);
    }
}
```

**Decorator:**
```typescript
interface DataSource {
    writeData(data: string): void;
    readData(): string;
}

class FileDataSource implements DataSource {
    writeData(data: string) { /* ... */ }
    readData(): string { /* ... */ }
}

class EncryptionDecorator implements DataSource {
    constructor(private wrapped: DataSource) {}

    writeData(data: string) {
        const encrypted = encrypt(data);
        this.wrapped.writeData(encrypted);
    }

    readData(): string {
        const data = this.wrapped.readData();
        return decrypt(data);
    }
}

// Usage:
let source = new FileDataSource();
source = new EncryptionDecorator(source);
source = new CompressionDecorator(source);
```

### Behavioral Patterns

**Strategy:**
```typescript
interface SortStrategy {
    sort(data: number[]): number[];
}

class QuickSort implements SortStrategy {
    sort(data: number[]): number[] { /* ... */ }
}

class MergeSort implements SortStrategy {
    sort(data: number[]): number[] { /* ... */ }
}

class Sorter {
    constructor(private strategy: SortStrategy) {}

    setStrategy(strategy: SortStrategy) {
        this.strategy = strategy;
    }

    sort(data: number[]): number[] {
        return this.strategy.sort(data);
    }
}
```

**Observer:**
```typescript
interface Observer {
    update(data: any): void;
}

class Subject {
    private observers: Observer[] = [];

    attach(observer: Observer): void {
        this.observers.push(observer);
    }

    detach(observer: Observer): void {
        const index = this.observers.indexOf(observer);
        this.observers.splice(index, 1);
    }

    notify(data: any): void {
        for (const observer of this.observers) {
            observer.update(data);
        }
    }
}
```

---

## 8. API DESIGN PRINCIPLES

### RESTful API Best Practices
```
Resources (nouns, not verbs):
✅ GET /users
✅ POST /users
✅ GET /users/123
✅ PUT /users/123
✅ DELETE /users/123

❌ GET /getUsers
❌ POST /createUser
❌ GET /user/delete/123

HTTP Methods:
- GET: Retrieve (idempotent, safe)
- POST: Create
- PUT: Update/Replace (idempotent)
- PATCH: Partial update
- DELETE: Remove (idempotent)

Status Codes:
- 200: OK
- 201: Created
- 204: No Content
- 400: Bad Request
- 401: Unauthorized
- 403: Forbidden
- 404: Not Found
- 500: Internal Server Error
```

---

**Updated:** November 23, 2025
**Version:** 13.1.3
**Part of:** SENA Multi-Level Memory System
