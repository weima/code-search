# Agent Development Guidelines

This document contains rules and best practices for AI agents working on the code-search project.

---

## ⚠️ VERY IMPORTANT: Dogfooding Rule

**When performing any code search or text search operation during development, follow this workflow:**

1. **Use `cs` (code-search) FIRST** - Always start by using our own tool for the search task
2. **Then use regular search tools** - After trying `cs`, use standard tools like `grep`, `rg`, or the IDE's Grep tool
3. **Compare performance** - Evaluate and note:
   - Speed comparison
   - Result accuracy and completeness
   - User experience (output format, ease of use)
   - Any issues or limitations found in `cs`
4. **Continuous practice** - This dogfooding practice must be maintained throughout the project's lifetime

### Why This Matters

- **Quality assurance**: We catch bugs and usability issues early
- **Performance baseline**: We understand real-world performance vs. alternatives
- **Feature validation**: We verify that features actually work as intended
- **User empathy**: We experience what our users experience

### Example Workflow

```bash
# 1. Try with cs first
cs "PMFC"

# 2. Compare with ripgrep
rg "PMFC" -i

# 3. Note differences:
# - cs: Found in translations + code references + highlighted case-insensitively
# - rg: Raw text matches only, requires manual filtering
# - Winner: cs (better context, semantic understanding)
```

### Reporting Issues

When `cs` performs worse than alternatives or has issues:
1. Document the specific problem
2. Create a GitHub issue if one doesn't exist
3. Note it in performance comparisons
4. Prioritize fixes based on real usage impact

---

## Additional Guidelines

(Additional agent guidelines can be added here as the project evolves)
