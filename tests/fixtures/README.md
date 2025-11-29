# Test Fixtures for Code Search CLI

This directory contains test fixtures for verifying the code search CLI tool's ability to trace UI text through i18n translation files to implementation code.

## Fixture Structure

### Rails Application (`rails-app/`)

Simulates a Ruby on Rails application with YAML-based i18n.

**Files:**
- `config/locales/en.yml` - YAML translation file with nested structure
- `app/components/invoices.ts` - TypeScript component using `I18n.t()` pattern

**Key Paths:**
- `invoice.labels.add_new` → "add new"
- `invoice.labels.edit` → "edit invoice"
- `invoice.labels.delete` → "delete invoice"
- `invoice.messages.created` → "Invoice created successfully"
- `user.labels.login` → "log in"
- `user.messages.welcome` → "Welcome back!"

**Patterns Used:**
- `I18n.t('key.path')` - Standard Rails i18n pattern (10 occurrences)

### React Application (`react-app/`)

Simulates a React application with JSON-based i18n using react-i18next.

**Files:**
- `src/locales/en.json` - JSON translation file with nested structure
- `src/components/InvoiceManager.tsx` - React component using multiple i18n patterns

**Key Paths:**
- `invoice.labels.add_new` → "add new"
- `invoice.labels.edit` → "edit invoice"
- `invoice.messages.created` → "Invoice created successfully"
- `user.labels.login` → "log in"
- `user.messages.welcome` → "Welcome back!"

**Patterns Used:**
- `t('key.path')` - useTranslation hook pattern
- `i18n.t('key.path')` - i18n instance pattern

### Vue Application (`vue-app/`)

Simulates a Vue.js application with JSON-based i18n using vue-i18n.

**Files:**
- `src/locales/en.json` - JSON translation file with nested structure
- `src/components/InvoiceManager.vue` - Vue component using template and script patterns
- `src/components/UserAuth.vue` - Vue component with additional usage examples

**Key Paths:**
- `invoice.labels.add_new` → "add new"
- `invoice.labels.edit` → "edit invoice"
- `invoice.messages.created` → "Invoice created successfully"
- `user.labels.login` → "log in"
- `user.messages.welcome` → "Welcome back!"

**Patterns Used:**
- `{{ $t('key.path') }}` - Template expression pattern (11 occurrences)
- `this.$t('key.path')` - Script method pattern (9 occurrences)

## Search Targets for Testing

### Primary Test Cases

1. **"add new"** (case-insensitive)
   - Found in all 3 translation files
   - Used in Rails, React, and Vue components
   - Key path: `invoice.labels.add_new`

2. **"log in"**
   - Found in all 3 translation files
   - Used in all 3 component implementations
   - Key path: `user.labels.login`

3. **"Welcome back!"**
   - Found in all 3 translation files
   - Used in all 3 component implementations
   - Key path: `user.messages.welcome`

### Additional Test Cases

4. **"edit invoice"** → `invoice.labels.edit`
5. **"Invoice created successfully"** → `invoice.messages.created`
6. **"Invoice not found"** → `invoice.errors.not_found`

## Verification

Run these commands from the `tests/fixtures/` directory to verify the fixtures:

```bash
# Search for text in translation files
rg "add new" -i

# Verify Rails patterns
rg "I18n\.t\(" rails-app/

# Verify React patterns
rg "\.t\(|i18n\.t\(" react-app/

# Verify Vue patterns
rg '\$t\(' vue-app/
```

## Expected Tool Behavior

For the search query `cs "add new"`, the tool should output a tree showing:

```
'add new'
├─> config/locales/en.yml:4 (en.invoice.labels.add_new)
│   ├─> app/components/invoices.ts:12 (I18n.t('invoice.labels.add_new'))
├─> src/locales/en.json:4 (invoice.labels.add_new)
│   ├─> src/components/InvoiceManager.tsx:23 (t('invoice.labels.add_new'))
│   ├─> src/components/InvoiceManager.tsx:31 (t('invoice.labels.add_new'))
├─> src/locales/en.json:4 (invoice.labels.add_new)
    ├─> src/components/InvoiceManager.vue:2 ($t('invoice.labels.add_new'))
    ├─> src/components/InvoiceManager.vue:5 ($t('invoice.labels.add_new'))
```

## Coverage Summary

- **Translation files:** 3 (1 YAML, 2 JSON)
- **Code files:** 5 (1 TS, 1 TSX, 3 Vue)
- **Total translation keys:** 13 unique keys across all frameworks
- **Total code references:** 38+ i18n function calls
- **Patterns covered:** 5 distinct i18n patterns (I18n.t, t, i18n.t, $t, this.$t)
