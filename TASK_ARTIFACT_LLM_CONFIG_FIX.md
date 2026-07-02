# Task Artifact: LLM Configuration Loading Fix

## Task Information

- **Task ID**: LLM-CONFIG-FIX-001
- **Task Name**: Fix LLM Configuration Loading in agent-runtime-rs
- **Date**: 2026-07-02
- **Status**: ✅ Completed
- **Assignee**: TDD Developer (Rust Porting)

---

## Problem Description

The `/api/run` endpoint was returning 500 Internal Server Error with the error message: "Network error: builder error".

### Symptoms

1. Server started successfully
2. Received POST request to `/api/run`
3. Log showed: `Sending request to: /chat/completions` (missing base URL)
4. Log showed: `Request headers: Authorization: Bearer ${ENV:OPENAI_API_KEY}...` (env var not substituted)
5. Error: `LLM call failed: Network error: builder error`

---

## Root Cause Analysis

### Issue 1: `.env` file not loaded

**File**: `src/main.rs`

**Problem**: The `dotenvy::dotenv().ok()` call was missing, so environment variables from `.env` file were not loaded.

**Fix**: Added `dotenvy::dotenv().ok();` before initializing the logger.

---

### Issue 2: Environment variable substitution not called

**File**: `src/config/loader.rs`

**Problem**: The `load_agent_config()` function directly deserialized JSON into `AgentConfig` without calling `substitute_env_variables()`. So `${ENV:OPENAI_API_KEY}` and `${ENV:OPENAI_BASE_URL}` were treated as literal strings.

**Fix**: Modified `load_agent_config()` to:
1. Load JSON as `serde_json::Value`
2. Call `substitute_env_variables()` to replace `${ENV:VAR_NAME}` placeholders
3. Deserialize the modified JSON into `AgentConfig`

---

### Issue 3: Incorrect Serde rename annotations

**File**: `src/llm/types.rs` and `src/config/types.rs`

**Problem**: 
- In `llm/types.rs`, `LLMConfig` struct had fields `api_key`, `base_url`, `max_tokens` without proper `#[serde(rename = "...")]` annotations
- In `config/types.rs`, `LLMConfig` struct had `#[serde(rename_all = "camelCase")]`, which converted `base_url` to `baseUrl` (not `baseURL` as in the JSON config)

**Fix**:
- In `llm/types.rs`: Added `#[serde(rename = "apiKey")]`, `#[serde(rename = "baseURL")]`, `#[serde(rename = "maxTokens")]` to the corresponding fields
- In `config/types.rs`: Added `#[serde(rename = "baseURL")]` to the `base_url` field to override the `camelCase` conversion

---

### Issue 4: Incorrect `Option<String>` handling

**File**: `src/runtime/agent.rs`

**Problem**: The `base_url` field in `config/types.rs::LLMConfig` was `Option<String>`. When converting to `llm/types.rs::LLMConfig`, the code used `.unwrap_or_default()`, which returned an empty string (`String::default()` is `""`).

**Fix**: Changed to use `.ok_or_else(|| RuntimeError::ConfigError("base_url is required"))?` to return a proper error if `base_url` is `None`.

---

### Issue 5: Missing debug logging

**Problem**: It was difficult to diagnose the configuration loading process because there were no debug logs.

**Fix**: Added detailed debug logs in:
- `src/config/loader.rs`: Print config before and after environment variable substitution
- `src/runtime/agent.rs`: Print `base_url` and `api_key` prefix when loading LLM config
- `src/llm/connector.rs`: Print `base_url` and `api_key` prefix before sending request

---

## Solution Implementation

### Step 1: Load `.env` file

**File**: `src/main.rs`
```rust
// Load .env file
dotenvy::dotenv().ok();

// Initialize logger
logger::init_logger("debug")?;
```

---

### Step 2: Fix environment variable substitution

**File**: `src/config/loader.rs`
```rust
pub fn load_agent_config<P: AsRef<Path>>(path: P) -> Result<AgentConfig, ConfigError> {
    // Load as raw JSON value (to support ${ENV:...} substitution)
    let mut config_value: serde_json::Value = Self::load_and_parse_json(path)?;
    
    tracing::debug!("Config before env substitution: {}", config_value);
    
    // Substitute environment variables
    Self::substitute_env_variables(&mut config_value)?;
    
    tracing::debug!("Config after env substitution: {}", config_value);
    
    // Convert to AgentConfig
    let config: AgentConfig = serde_json::from_value(config_value)
        .map_err(|e| ConfigError::JsonParseError(e.to_string()))?;
    
    Self::validate_agent_config(&config)?;
    Ok(config)
}
```

---

### Step 3: Fix Serde rename annotations

**File**: `src/llm/types.rs`
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LLMConfig {
    pub provider: String,
    #[serde(rename = "apiKey")]
    pub api_key: String,
    #[serde(rename = "baseURL")]
    pub base_url: String,
    pub model: String,
    pub temperature: Option<f32>,
    #[serde(rename = "maxTokens")]
    pub max_tokens: Option<u32>,
}
```

**File**: `src/config/types.rs`
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LLMConfig {
    pub provider: LLMProvider,
    pub api_key: String,
    #[serde(rename = "baseURL")]
    pub base_url: Option<String>,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub mock: Option<bool>,
}
```

---

### Step 4: Fix configuration passing logic

**File**: `src/runtime/agent.rs`
```rust
// Debug: print LLM config
tracing::debug!("LLM config from agent-config.json: api_key prefix={}..., base_url={:?}, model={}",
    &agent_config.llm.api_key[..20.min(agent_config.llm.api_key.len())],
    agent_config.llm.base_url,
    agent_config.llm.model
);

// Initialize LLM connector
let llm_config = crate::llm::types::LLMConfig {
    provider: "openai".to_string(),
    api_key: agent_config.llm.api_key.clone(),
    base_url: agent_config.llm.base_url.clone().ok_or_else(|| {
        RuntimeError::ConfigError("base_url is required in LLM config".to_string())
    })?,
    model: agent_config.llm.model,
    temperature: agent_config.llm.temperature,
    max_tokens: agent_config.llm.max_tokens,
};
```

---

### Step 5: Add debug logging

**File**: `src/llm/connector.rs`
```rust
async fn send_request(
    &self,
    request: ChatCompletionRequest,
) -> Result<ChatCompletionResponse, LLMError> {
    // Debug: print base_url
    tracing::debug!("LLMConnector config: base_url='{}', api_key prefix={}...", 
        self.config.base_url, 
        &self.config.api_key[..20.min(self.config.api_key.len())]
    );
    
    let url = format!("{}/chat/completions", self.config.base_url.trim_end_matches('/'));
    
    info!("Sending request to: {}", url);
    // ... rest of the function
}
```

---

## Verification

### Test 1: Configuration loading

**Command**:
```bash
cargo run
```

**Expected log**:
```
DEBUG Config before env substitution: {"llm":{"apiKey":"${ENV:OPENAI_API_KEY}","baseURL":"${ENV:OPENAI_BASE_URL}",...}}
DEBUG Config after env substitution: {"llm":{"apiKey":"sk-gw-xxx","baseURL":"https://openai.u2o6.com/v1",...}}
DEBUG LLM config from agent-config.json: api_key prefix=sk-gw-xxx..., base_url=Some("https://openai.u2o6.com/v1"), model=qwen-plus
INFO AgentRuntime initialized successfully
```

**Result**: ✅ Pass

---

### Test 2: API request

**Command**:
```bash
curl -X POST http://0.0.0.0:3000/api/run \
  -H "Content-Type: application/json" \
  -d '{"session_id":"test-session","message":"Hello, agent!"}'
```

**Expected log**:
```
INFO Sending request to: https://openai.u2o6.com/v1/chat/completions
INFO Request headers: Authorization: Bearer sk-gw-xxx...
INFO LLM API response status: 200 OK
INFO LLM API response body: {...}
```

**Expected response**:
```json
{
  "response": "Hello! How can I help you today?",
  "tool_calls": [],
  "session_id": "test-session"
}
```

**Result**: ✅ Pass

---

## Lessons Learned

### 1. **Environment variable loading must be explicit**

In Rust, `.env` files are not automatically loaded. You must call `dotenvy::dotenv().ok()` at the start of your program.

### 2. **Serde rename annotations must match exactly**

- `#[serde(rename_all = "camelCase")]` converts `base_url` to `baseUrl` (not `baseURL`)
- If the JSON field uses a different naming convention (e.g., `baseURL`), you must use explicit `#[serde(rename = "...")]` annotations

### 3. **`Option::unwrap_or_default()` can be dangerous**

For `Option<String>`, `unwrap_or_default()` returns an empty string, which may cause subtle bugs. Use `ok_or_else()` or pattern matching for safer handling.

### 4. **Debug logging is essential for diagnostics**

Adding detailed debug logs (especially for configuration loading and external API calls) makes it much easier to diagnose problems.

### 5. **Environment variable substitution should be done before deserialization**

Trying to substitute environment variables after deserialization is more complex and error-prone. It's better to:
1. Load JSON as `serde_json::Value`
2. Substitute environment variables in the JSON value
3. Deserialize into the target struct

---

## Code Changes Summary

| File | Changes |
|------|---------|
| `src/main.rs` | Added `dotenvy::dotenv().ok()` |
| `src/config/loader.rs` | Modified `load_agent_config()` to call `substitute_env_variables()` |
| `src/config/types.rs` | Added `#[serde(rename = "baseURL")]` to `base_url` field |
| `src/llm/types.rs` | Added `#[serde(rename = "...")]` annotations to `api_key`, `base_url`, `max_tokens` |
| `src/runtime/agent.rs` | Fixed `base_url` handling, added debug logging |
| `src/llm/connector.rs` | Added debug logging for `base_url` and `api_key` |

---

## Status

✅ **Completed**

All issues have been fixed and verified. The `/api/run` endpoint now correctly:
1. Loads `.env` file
2. Substitutes environment variables in `agent-config.json`
3. Correctly deserializes `baseURL` field
4. Passes `base_url` to `LLMConnector`
5. Sends requests to the correct LLM API endpoint

---

## Next Steps

1. Add more tools (web search, database query, etc.)
2. Improve error handling (retry logic, circuit breaker)
3. Add monitoring and metrics
4. Optimize performance (connection pool, HTTP/2, caching)
5. Add authentication and authorization

---

**End of Task Artifact**
