# Testing Requirements

## Test Coverage: current gate (D79/D84/D90; D50 ladder retired at the Stage-2 boundary)

80% hard is ACTIVE, measured at stage/acceptance boundaries; runtime bins + fixture
crates are boundary-excluded from the denominator (D84 - naive whole-workspace totals
false-FAIL; the metric definition is part of the claim).
Tool: `pixi run -e cov cargo llvm-cov` (tarpaulin successor - faster, maintained).

Test Types (ALL required):
1. **Unit Tests** - Individual functions, utilities, components
2. **Integration Tests** - API endpoints, database operations
3. **E2E Tests** - Critical user flows (framework chosen per language)

## Test-Driven Development

MANDATORY workflow:
1. Write test first (RED)
2. Run test - it should FAIL
3. Write minimal implementation (GREEN)
4. Run test - it should PASS
5. Refactor (IMPROVE)
6. Verify coverage against the staged gate (D50)

## Troubleshooting Test Failures

1. Use **tdd-guide** agent
2. Check test isolation
3. Verify mocks are correct
4. Fix implementation, not tests (unless tests are wrong)

## Agent Support

- **tdd-guide** - Use PROACTIVELY for new features, enforces write-tests-first

## Test Structure (AAA Pattern)

Prefer Arrange-Act-Assert structure for tests:

```typescript
test('calculates similarity correctly', () => {
  // Arrange
  const vector1 = [1, 0, 0]
  const vector2 = [0, 1, 0]

  // Act
  const similarity = calculateCosineSimilarity(vector1, vector2)

  // Assert
  expect(similarity).toBe(0)
})
```

### Test Naming

Use descriptive names that explain the behavior under test:

```typescript
test('returns empty array when no markets match query', () => {})
test('throws error when API key is missing', () => {})
test('falls back to substring search when Redis is unavailable', () => {})
```

## Metric Integrity (orchestrator rule, 2026-10-19)

Any metric an agent reports (coverage %, test totals, timing) gets RE-COMPUTED with the
orchestrator's own parser before it can move a gate - the S2-M4 "core 81.33" was a
different aggregation, rejected; the honest 78.52 kept the D84 hook armed until real
payment. The metric definition is part of the claim.

## Runnable Check

```bash
# Verify: tests pass (pixi-routed on this box - mise cargo shim broken, D65)
pixi run cargo test --workspace
# Verify: coverage (boundary exclusions per D84)
pixi run -e cov cargo llvm-cov --workspace 2>/dev/null || echo "llvm-cov not installed (gate per D79/D90)"
```
