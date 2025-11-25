# SENA Performance Patterns & Optimization Guide

This document contains performance optimization knowledge and patterns for persistent application across all SENA sessions.

---

## 1. ALGORITHMIC COMPLEXITY PATTERNS

### Big O Performance Guide
```
O(1)       - Constant      - Hash table lookup, array index access
O(log n)   - Logarithmic   - Binary search, balanced tree operations
O(n)       - Linear        - Single loop, linear search
O(n log n) - Linearithmic  - Efficient sorting (merge sort, quicksort)
O(n²)      - Quadratic     - Nested loops, bubble sort
O(2ⁿ)      - Exponential   - Recursive Fibonacci, brute force
O(n!)      - Factorial     - Permutations, traveling salesman (brute force)
```

### Common Optimizations

**Bad (O(n²)):**
```javascript
// Nested loop for finding duplicates
for (let i = 0; i < arr.length; i++) {
    for (let j = i + 1; j < arr.length; j++) {
        if (arr[i] === arr[j]) {
            duplicates.push(arr[i]);
        }
    }
}
```

**Good (O(n)):**
```javascript
// Hash set for O(1) lookup
const seen = new Set();
const duplicates = [];
for (const item of arr) {
    if (seen.has(item)) {
        duplicates.push(item);
    }
    seen.add(item);
}
```

---

## 2. DATABASE OPTIMIZATION PATTERNS

### N+1 Query Problem

**Bad (N+1 queries):**
```typescript
// 1 query for users
const users = await User.findAll();

// N queries for posts (one per user)
for (const user of users) {
    user.posts = await Post.findAll({ where: { userId: user.id } });
}
// Total: 1 + N queries
```

**Good (2 queries with eager loading):**
```typescript
// Single query with JOIN
const users = await User.findAll({
    include: [{ model: Post }]
});
// Total: 1 query (or 2 if using separate queries strategy)
```

### Index Optimization
```sql
-- Slow: Full table scan
SELECT * FROM users WHERE email = 'user@example.com';

-- Fast: Index lookup
CREATE INDEX idx_users_email ON users(email);
SELECT * FROM users WHERE email = 'user@example.com';

-- Composite index for multi-column queries
CREATE INDEX idx_users_status_created ON users(status, created_at);
SELECT * FROM users WHERE status = 'active' ORDER BY created_at DESC;
```

### Query Batching
```typescript
// Bad: Multiple individual queries
for (const id of userIds) {
    await User.findByPk(id);
}
// N database round-trips

// Good: Single batch query
const users = await User.findAll({
    where: { id: userIds }
});
// 1 database round-trip
```

---

## 3. CACHING STRATEGIES

### Cache-Aside (Lazy Loading)
```typescript
async function getUser(id: string): Promise<User> {
    // Try cache first
    const cached = await redis.get(`user:${id}`);
    if (cached) {
        return JSON.parse(cached);
    }

    // Cache miss: fetch from database
    const user = await User.findByPk(id);

    // Store in cache for future requests
    await redis.setex(`user:${id}`, 3600, JSON.stringify(user));

    return user;
}
```

### Write-Through Cache
```typescript
async function updateUser(id: string, data: Partial<User>): Promise<User> {
    // Update database
    const user = await User.update(data, { where: { id } });

    // Update cache immediately
    await redis.setex(`user:${id}`, 3600, JSON.stringify(user));

    return user;
}
```

### Cache Invalidation
```typescript
// Pattern 1: TTL-based (time to live)
await redis.setex('key', 3600, value); // Expires after 1 hour

// Pattern 2: Event-based invalidation
async function deleteUser(id: string) {
    await User.destroy({ where: { id } });

    // Invalidate cache
    await redis.del(`user:${id}`);
}

// Pattern 3: Cache tagging
await redis.set('user:123', data);
await redis.sadd('tag:users', 'user:123');

// Invalidate all users
const keys = await redis.smembers('tag:users');
await redis.del(...keys);
```

---

## 4. MEMORY OPTIMIZATION

### Object Pooling
```typescript
class ObjectPool<T> {
    private available: T[] = [];
    private inUse = new Set<T>();

    constructor(
        private factory: () => T,
        private reset: (obj: T) => void,
        initialSize: number = 10
    ) {
        for (let i = 0; i < initialSize; i++) {
            this.available.push(factory());
        }
    }

    acquire(): T {
        let obj = this.available.pop();
        if (!obj) {
            obj = this.factory();
        }
        this.inUse.add(obj);
        return obj;
    }

    release(obj: T): void {
        this.inUse.delete(obj);
        this.reset(obj);
        this.available.push(obj);
    }
}

// Usage for expensive objects (database connections, etc.)
const dbPool = new ObjectPool(
    () => createConnection(),
    (conn) => conn.reset(),
    20
);
```

### Lazy Initialization
```typescript
class DataService {
    private _expensiveResource?: ExpensiveResource;

    get expensiveResource(): ExpensiveResource {
        if (!this._expensiveResource) {
            this._expensiveResource = new ExpensiveResource();
        }
        return this._expensiveResource;
    }
}
```

### Pagination
```typescript
// Bad: Load all records
const users = await User.findAll(); // Could be millions of rows!

// Good: Paginate
const users = await User.findAll({
    limit: 20,
    offset: (page - 1) * 20,
    order: [['created_at', 'DESC']]
});

// Better: Cursor-based pagination (more efficient for large datasets)
const users = await User.findAll({
    where: {
        id: { [Op.gt]: lastSeenId }
    },
    limit: 20,
    order: [['id', 'ASC']]
});
```

---

## 5. NETWORK OPTIMIZATION

### Request Batching
```typescript
// Bad: Multiple API calls
const user = await fetch('/api/user/123');
const posts = await fetch('/api/user/123/posts');
const comments = await fetch('/api/user/123/comments');

// Good: Single batched request
const data = await fetch('/api/user/123?include=posts,comments');
```

### Compression
```typescript
import compression from 'compression';

app.use(compression({
    level: 6, // Balance between compression ratio and CPU usage
    threshold: 1024, // Only compress responses > 1KB
    filter: (req, res) => {
        if (req.headers['x-no-compression']) {
            return false;
        }
        return compression.filter(req, res);
    }
}));
```

### HTTP/2 Server Push
```typescript
// Push critical resources before they're requested
app.get('/', (req, res) => {
    res.push('/styles.css', {}, (err, stream) => {
        if (err) return;
        stream.respondWithFile('./public/styles.css');
    });

    res.push('/app.js', {}, (err, stream) => {
        if (err) return;
        stream.respondWithFile('./public/app.js');
    });

    res.sendFile('./public/index.html');
});
```

### CDN for Static Assets
```
✅ Best practices:
- Use CDN for static assets (images, CSS, JS)
- Set far-future cache headers (1 year)
- Use content-based hashing for filenames (cache busting)
- Serve from geographically distributed edge locations
```

---

## 6. ASYNC & CONCURRENCY PATTERNS

### Parallel Execution
```typescript
// Bad: Sequential (slow)
const user = await fetchUser(id);
const posts = await fetchPosts(id);
const comments = await fetchComments(id);
// Total time: T1 + T2 + T3

// Good: Parallel (fast)
const [user, posts, comments] = await Promise.all([
    fetchUser(id),
    fetchPosts(id),
    fetchComments(id)
]);
// Total time: max(T1, T2, T3)
```

### Rate-Limited Concurrency
```typescript
async function processWithConcurrencyLimit<T>(
    items: T[],
    limit: number,
    processor: (item: T) => Promise<void>
): Promise<void> {
    const executing: Promise<void>[] = [];

    for (const item of items) {
        const promise = processor(item).then(() => {
            executing.splice(executing.indexOf(promise), 1);
        });

        executing.push(promise);

        if (executing.length >= limit) {
            await Promise.race(executing);
        }
    }

    await Promise.all(executing);
}

// Process 1000 items with max 10 concurrent operations
await processWithConcurrencyLimit(items, 10, async (item) => {
    await processItem(item);
});
```

---

## 7. DATA STRUCTURE SELECTION

### Performance Characteristics

**Array:**
- Access: O(1)
- Search: O(n)
- Insert/Delete (end): O(1)
- Insert/Delete (middle): O(n)
- Use when: Indexed access, iteration

**Hash Map:**
- Access: O(1)
- Search: O(1)
- Insert/Delete: O(1)
- Use when: Key-value lookups, counting, deduplication

**Set:**
- Search: O(1)
- Insert/Delete: O(1)
- Use when: Uniqueness, membership testing

**Linked List:**
- Access: O(n)
- Insert/Delete (known position): O(1)
- Use when: Frequent insertions/deletions at known positions

**Binary Search Tree:**
- Access/Search/Insert/Delete: O(log n)
- Use when: Sorted data, range queries

**Heap:**
- Find min/max: O(1)
- Insert/Delete: O(log n)
- Use when: Priority queues, top K elements

---

## 8. CODE-LEVEL OPTIMIZATIONS

### Memoization
```typescript
// Expensive Fibonacci (exponential time)
function fib(n: number): number {
    if (n <= 1) return n;
    return fib(n - 1) + fib(n - 2); // Recalculates same values
}

// Memoized (linear time)
const fibMemo = (() => {
    const cache = new Map<number, number>();
    return function fib(n: number): number {
        if (n <= 1) return n;
        if (cache.has(n)) return cache.get(n)!;

        const result = fib(n - 1) + fib(n - 2);
        cache.set(n, result);
        return result;
    };
})();
```

### Debouncing & Throttling
```typescript
// Debounce: Execute after delay with no new calls
function debounce<T extends (...args: any[]) => any>(
    func: T,
    delay: number
): (...args: Parameters<T>) => void {
    let timeoutId: NodeJS.Timeout;
    return (...args) => {
        clearTimeout(timeoutId);
        timeoutId = setTimeout(() => func(...args), delay);
    };
}

// Throttle: Execute at most once per interval
function throttle<T extends (...args: any[]) => any>(
    func: T,
    interval: number
): (...args: Parameters<T>) => void {
    let lastCall = 0;
    return (...args) => {
        const now = Date.now();
        if (now - lastCall >= interval) {
            lastCall = now;
            func(...args);
        }
    };
}

// Usage
const handleScroll = debounce(() => {
    // Expensive operation
}, 300);

const handleResize = throttle(() => {
    // Update layout
}, 100);
```

### Loop Optimization
```typescript
// Bad: Recalculate length every iteration
for (let i = 0; i < arr.length; i++) {
    process(arr[i]);
}

// Good: Cache length
const len = arr.length;
for (let i = 0; i < len; i++) {
    process(arr[i]);
}

// Better: forEach (JIT-optimized)
arr.forEach(item => process(item));
```

---

## 9. MONITORING & PROFILING

### Performance Metrics to Track
```typescript
// Response time percentiles
const metrics = {
    p50: 45,  // 50th percentile (median)
    p95: 120, // 95th percentile
    p99: 250  // 99th percentile
};

// Throughput
const requestsPerSecond = 1000;

// Resource usage
const cpuPercent = 45;
const memoryMB = 512;
const diskIOPS = 100;
```

### Profiling Tools
```bash
# Node.js CPU profiling
node --prof app.js
node --prof-process isolate-*.log > profile.txt

# Memory profiling
node --inspect app.js
# Then use Chrome DevTools Memory Profiler

# Load testing
# Apache Bench
ab -n 10000 -c 100 http://localhost:3000/

# k6
k6 run load-test.js
```

---

## 10. PERFORMANCE BUDGET

### Establish Targets
```
Page Load Time:
  - Time to First Byte (TTFB): < 200ms
  - First Contentful Paint (FCP): < 1.0s
  - Largest Contentful Paint (LCP): < 2.5s
  - Time to Interactive (TTI): < 3.5s

API Response Time:
  - GET requests: < 100ms (p95)
  - POST requests: < 200ms (p95)
  - Complex queries: < 500ms (p95)

Resource Limits:
  - Bundle size: < 200KB (gzipped)
  - Images: < 100KB each
  - Total page weight: < 1MB
```

---

**Updated:** November 23, 2025
**Version:** 3.3.1
**Part of:** SENA Multi-Level Memory System
