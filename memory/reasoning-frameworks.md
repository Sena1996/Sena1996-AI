# SENA Reasoning Frameworks

This document contains advanced reasoning frameworks that enhance SENA's analytical capabilities across all sessions.

---

## 1. FIRST PRINCIPLES THINKING

Break complex problems down to fundamental truths and rebuild from there.

### Process:
1. **Identify and define current assumptions**
   - What do we currently believe?
   - Why do we believe it?
   - What evidence supports it?

2. **Break down the problem into fundamental principles**
   - What are the basic laws/truths that apply?
   - What can we know with absolute certainty?
   - What are the immutable constraints?

3. **Rebuild from the ground up**
   - Start with verified fundamentals
   - Add only what's necessary
   - Question each addition

### Example Application:
```
Problem: "Why is system slow?"

Bad approach: "Add more servers" (assumption-based)

First principles:
- Fundamental: Response time = Processing + I/O + Network
- Question: Which component dominates?
- Measure: Profile to find bottleneck
- Solution: Optimize the actual bottleneck
```

---

## 2. ROOT CAUSE ANALYSIS

Identify underlying causes, not just symptoms.

### 5 Whys Technique:
```
Problem: Website is down
Why? → Server crashed
Why? → Out of memory
Why? → Memory leak in code
Why? → Unclosed database connections
Why? → Missing connection pool cleanup
Root Cause: No connection lifecycle management
```

### Fishbone Diagram (Ishikawa):
```
           People    Process    Technology
              │         │           │
              ├─────────┼───────────┤
              │                     │
              └─────── PROBLEM ─────┘
              │                     │
              ├─────────┼───────────┤
              │         │           │
        Environment  Materials  Measurement
```

### Fault Tree Analysis:
- Start with failure
- Work backward through AND/OR gates
- Identify all possible causes
- Prioritize by probability and impact

---

## 3. STRUCTURED DECISION MAKING

### Decision Matrix:
```
┌────────────────┬──────────┬──────────┬──────────┬──────────┐
│ Option         │ Cost (3) │ Speed (5)│ Quality(4│ Total    │
├────────────────┼──────────┼──────────┼──────────┼──────────┤
│ Option A       │ 8 (24)   │ 6 (30)   │ 9 (36)   │ 90       │
│ Option B       │ 5 (15)   │ 9 (45)   │ 7 (28)   │ 88       │
│ Option C       │ 9 (27)   │ 4 (20)   │ 8 (32)   │ 79       │
└────────────────┴──────────┴──────────┴──────────┴──────────┘
(Weight in parentheses)
```

### Cost-Benefit Analysis:
```
Benefits:
  + Revenue increase: $100K/year
  + Time savings: 200 hours/year @ $50/hr = $10K
  + Risk reduction: Estimated $20K/year
  Total Benefits: $130K/year

Costs:
  - Initial development: $50K
  - Annual maintenance: $15K/year
  - Training: $5K
  Total First Year: $70K
  Total Annual: $15K/year

ROI: Year 1 = ($130K - $70K) / $70K = 86%
     Year 2+ = ($130K - $15K) / $15K = 767%
```

### Risk Assessment Matrix:
```
        │ Low Impact │ Medium Impact │ High Impact │
────────┼────────────┼───────────────┼─────────────┤
High    │   Medium   │     High      │  Critical   │
Prob.   │            │               │             │
────────┼────────────┼───────────────┼─────────────┤
Medium  │    Low     │    Medium     │    High     │
Prob.   │            │               │             │
────────┼────────────┼───────────────┼─────────────┤
Low     │   Accept   │      Low      │   Medium    │
Prob.   │            │               │             │
```

---

## 4. SYSTEMS THINKING

View problems as part of larger interconnected systems.

### Feedback Loops:
- **Reinforcing (Positive)**: Growth or decline accelerates
  ```
  More customers → More revenue → More marketing → More customers
  ```

- **Balancing (Negative)**: System self-corrects
  ```
  High prices → Reduced demand → Lower prices → Increased demand
  ```

### Leverage Points:
1. **Constants, parameters** (weak leverage)
2. **Buffers** (stock sizes)
3. **Stock and flow structures**
4. **Delays** (timing)
5. **Balancing feedback loops**
6. **Reinforcing feedback loops**
7. **Information flows**
8. **Rules** (incentives, constraints)
9. **Power to self-organize**
10. **Goals** (system purpose)
11. **Paradigms** (mindsets)
12. **Transcending paradigms** (strongest leverage)

---

## 5. LATERAL THINKING

Generate creative solutions by approaching from non-obvious angles.

### Techniques:

**Random Input:**
- Pick random word/concept
- Force connection to problem
- Extract useful ideas

**Provocation:**
- Make deliberately absurd statement
- Explore implications
- Find kernel of useful insight

**Six Thinking Hats:**
- 🎩 White: Facts and information
- 🧢 Red: Emotions and intuition
- ⛑️ Yellow: Benefits and optimism
- 🎩 Black: Risks and criticism
- 🧢 Green: Creativity and alternatives
- 👑 Blue: Process control

---

## 6. MULTI-CRITERIA DECISION ANALYSIS (MCDA)

### Analytical Hierarchy Process (AHP):
1. Define criteria and alternatives
2. Pairwise comparison of criteria (which is more important?)
3. Pairwise comparison of alternatives per criterion
4. Calculate weighted scores
5. Select highest-scoring alternative

### TOPSIS (Technique for Order of Preference by Similarity to Ideal Solution):
1. Normalize decision matrix
2. Weight normalized matrix
3. Identify ideal and anti-ideal solutions
4. Calculate distance from ideal
5. Rank alternatives by relative closeness

---

## 7. PROBABILISTIC THINKING

### Bayes' Theorem:
```
P(A|B) = P(B|A) × P(A) / P(B)

Example: Medical test accuracy
- Disease prevalence: 1%
- Test sensitivity: 95% (true positive rate)
- Test specificity: 90% (true negative rate)

If test is positive, actual probability of disease:
= (0.95 × 0.01) / [(0.95 × 0.01) + (0.10 × 0.99)]
= 0.0095 / 0.1085
= 8.8%
```

### Expected Value:
```
EV = Σ (Probability × Outcome)

Example: Investment decision
- 60% chance of $100K gain = 0.6 × $100K = $60K
- 30% chance of $0 = 0.3 × $0 = $0
- 10% chance of $50K loss = 0.1 × -$50K = -$5K
Expected Value = $55K
```

---

## 8. CONSTRAINT-BASED THINKING

### Theory of Constraints (TOC):
1. **Identify** the system constraint (bottleneck)
2. **Exploit** the constraint (get maximum from it)
3. **Subordinate** everything else to the constraint
4. **Elevate** the constraint (increase its capacity)
5. **Repeat** (find next constraint)

### Application:
```
Software development bottleneck:
1. Identify: Code review is slowest step
2. Exploit: Prioritize review, clear blockers
3. Subordinate: Slow down coding to match review capacity
4. Elevate: Add reviewers, automate checks
5. Repeat: Next bottleneck might be testing
```

---

## 9. INVERSION THINKING

### Ask the opposite question:
- Instead of "How to succeed?", ask "How to guarantee failure?"
- Identify failure modes
- Avoid them

### Example:
```
Question: "How to build a successful product?"

Invert: "How to guarantee product failure?"
- Ignore user feedback
- Add complexity
- Skip testing
- Poor documentation
- No clear value proposition

Prevention: Do the opposite!
```

---

## 10. MENTAL MODELS

### Key Models to Apply:

**Second-Order Thinking:**
- First order: Immediate consequence
- Second order: Consequence of the consequence
- Example: Fire employee → Save money (1st) → But lose expertise, morale drops, others leave (2nd)

**Circle of Competence:**
- Know what you know
- Know what you don't know
- Operate within competence
- Expand it systematically

**Margin of Safety:**
- Build in buffers
- Don't optimize to limits
- Account for uncertainty
- Examples: 2x time estimates, 50% capacity buffer

**Opportunity Cost:**
- Every choice has alternatives
- What are you giving up?
- Compare to best alternative
- Hidden cost of any decision

---

## USAGE INSTRUCTIONS

When analyzing complex problems:
1. **Select appropriate framework** based on problem type
2. **Apply systematically**, don't skip steps
3. **Document reasoning**, show work
4. **Question assumptions** at each stage
5. **Consider alternatives** before concluding
6. **Quantify when possible** (numbers over feelings)
7. **Acknowledge uncertainty** (confidence levels)

---

**Updated:** November 23, 2025
**Version:** 3.3.1
**Part of:** SENA Multi-Level Memory System
