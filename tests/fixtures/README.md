# Test Fixtures for Code Search CLI

This directory contains test fixtures for verifying the code search CLI tool's ability to:
1. Trace UI text through i18n translation files to implementation code
2. Search for code elements (functions, variables, error messages)
3. Trace function call graphs (forward and backward)

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

### Code Examples (`code-examples/`)

Simulates a TypeScript codebase with function definitions, calls, and error messages for testing code-based searches.

**Files:**
- `utils.ts` - Utility functions with payment processing logic
- `checkout.ts` - CheckoutService class that uses utils functions
- `api.ts` - API handlers that use CheckoutService

**Search Targets:**

1. **Function Names:**
   - `processPayment` - defined in utils.ts, called in checkout.ts
   - `calculateTotal` - defined in utils.ts, called in checkout.ts
   - `validateAmount` - private function in utils.ts, called by processPayment
   - `chargeCard` - private function in utils.ts, called by processPayment
   - `handleCheckoutRequest` - defined in api.ts

2. **Variables/Constants:**
   - `userId` - used across multiple files
   - `ERROR_MESSAGES` - constant object exported from utils.ts, used in checkout.ts and api.ts

3. **Error Messages:**
   - `"Invalid payment amount"` - defined in ERROR_MESSAGES.INVALID_AMOUNT
   - `"User not found"` - defined in ERROR_MESSAGES.USER_NOT_FOUND
   - `"Card was declined"` - defined in ERROR_MESSAGES.CARD_DECLINED

**Call Graph Examples:**
- `processPayment` calls: validateAmount, chargeCard, logTransaction
- `checkout` calls: calculateTotal, processPayment
- `handleCheckoutRequest` calls: CheckoutService.checkout

## Search Targets for Testing

### i18n Text Search Test Cases

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

4. **"edit invoice"** → `invoice.labels.edit`
5. **"Invoice created successfully"** → `invoice.messages.created`
6. **"Invoice not found"** → `invoice.errors.not_found`

### Code Search Test Cases

7. **"processPayment"** (function name)
   - Defined in: `code-examples/utils.ts:5`
   - Called in: `code-examples/checkout.ts:24`

8. **"userId"** (variable name)
   - Used in: `utils.ts`, `checkout.ts`, `api.ts` (multiple occurrences)

9. **"Invalid payment amount"** (error message)
   - Defined in: `code-examples/utils.ts:50` (ERROR_MESSAGES.INVALID_AMOUNT)
   - Used in: `code-examples/utils.ts:9`, `code-examples/checkout.ts:23`

10. **"ERROR_MESSAGES"** (constant)
    - Defined in: `code-examples/utils.ts:49`
    - Imported in: `code-examples/checkout.ts:1`, `code-examples/api.ts:2`
    - Used in: `code-examples/checkout.ts:23`, `code-examples/api.ts:5,14`

## Verification

Run these commands from the `tests/fixtures/` directory to verify the fixtures:

### i18n Search Verification

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

### Code Search Verification

```bash
# Search for function names
rg "processPayment" code-examples/

# Search for error messages
rg "Invalid payment amount" code-examples/

# Search for variable usage
rg "userId" code-examples/

# Search for constant definitions
rg "ERROR_MESSAGES" code-examples/
```

## Expected Tool Behavior

### i18n Text Search Example

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

### Code Search Example

For the search query `cs "processPayment"`, the tool should find:

```
'processPayment'
├─> code-examples/utils.ts:5 (function definition)
├─> code-examples/checkout.ts:1 (import statement)
├─> code-examples/checkout.ts:24 (function call)
```

### Error Message Search Example

For the search query `cs "Invalid payment amount"`, the tool should trace:

```
'Invalid payment amount'
├─> code-examples/utils.ts:50 (ERROR_MESSAGES.INVALID_AMOUNT definition)
├─> code-examples/utils.ts:9 (throw new Error usage)
├─> code-examples/checkout.ts:23 (ERROR_MESSAGES.INVALID_AMOUNT reference)
```

## Coverage Summary

### i18n Fixtures
- **Translation files:** 3 (1 YAML, 2 JSON)
- **i18n code files:** 5 (1 TS, 1 TSX, 3 Vue)
- **Total translation keys:** 13 unique keys across all frameworks
- **Total i18n references:** 38+ function calls
- **i18n patterns covered:** 5 distinct patterns (I18n.t, t, i18n.t, $t, this.$t)

### Code Search Fixtures
- **Code files:** 3 TypeScript files (utils.ts, checkout.ts, api.ts)
- **Function definitions:** 7 functions (processPayment, validateAmount, chargeCard, etc.)
- **Function calls:** 10+ call sites across files
- **Error messages:** 3 distinct error message strings
- **Constants/Variables:** userId, ERROR_MESSAGES, and others
