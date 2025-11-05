# Ollama Integration Strategy for Handy

## Executive Summary

This document outlines a strategic approach to integrating Ollama with Handy for enhanced transcription post-processing while maintaining Handy's core philosophy of simplicity, privacy, and extensibility.

## Core Principles

1. **No Model Duplication**: Leverage Ollama's existing model infrastructure via API
2. **Privacy First**: All processing remains local (Ollama runs locally)
3. **Optional Enhancement**: Users can use Handy without Ollama
4. **Seamless Integration**: Minimal configuration required

## Why Ollama?

### Advantages
- **Zero Model Duplication**: Uses Ollama's models (GGUF format) without downloading separate copies
- **Local Processing**: Maintains privacy by running everything locally
- **Flexible LLM Access**: Access to hundreds of models (Llama, Mistral, Gemma, etc.)
- **HTTP API**: Simple REST integration, no complex dependencies
- **Community Ecosystem**: Leverage existing Ollama users and model library

### Use Cases
1. **Post-Processing**: Add punctuation, capitalization, grammar correction
2. **Summarization**: Generate summaries of long transcriptions
3. **Command Extraction**: Parse action items, TODO lists from meetings
4. **Formatting**: Convert transcription to different formats (markdown, email, code comments)
5. **Translation Enhancement**: Better translation than Whisper's built-in
6. **Context-Aware Correction**: Use conversation history for better accuracy

## Architecture Design

### High-Level Flow
```
User Speech → Whisper/Parakeet Transcription → [Optional] Ollama Post-Processing → Output
```

### Component Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Handy Application                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐      ┌──────────────┐                   │
│  │   Audio      │──────▶│Transcription │                   │
│  │   Manager    │      │   Manager    │                   │
│  └──────────────┘      └──────┬───────┘                   │
│                                │                           │
│                                ▼                           │
│                    ┌───────────────────────┐               │
│                    │  Post-Processing      │               │
│                    │     Pipeline          │               │
│                    └───────────┬───────────┘               │
│                                │                           │
│                    ┌───────────▼───────────┐               │
│                    │   Ollama Manager      │               │
│                    │  (HTTP API Client)    │               │
│                    └───────────┬───────────┘               │
└────────────────────────────────┼───────────────────────────┘
                                 │ HTTP (localhost:11434)
                                 ▼
                    ┌─────────────────────────┐
                    │   Ollama Service        │
                    │  (User's Installation)  │
                    └─────────────────────────┘
```

### Data Flow

**Without Ollama (Current)**
```
Audio → VAD → Whisper/Parakeet → Custom Words → Paste
```

**With Ollama (New)**
```
Audio → VAD → Whisper/Parakeet → Custom Words → Ollama Processing → Paste
```

## Implementation Design

### 1. OllamaManager (Rust)

**Responsibilities:**
- Detect if Ollama is running (health check)
- List available models via Ollama API
- Send prompts and receive responses
- Handle streaming responses
- Error handling and fallback

**Key Methods:**
```rust
pub struct OllamaManager {
    base_url: String,
    client: reqwest::Client,
    is_available: Arc<AtomicBool>,
}

impl OllamaManager {
    pub async fn new(base_url: Option<String>) -> Self;
    pub async fn check_availability() -> bool;
    pub async fn list_models() -> Result<Vec<OllamaModel>>;
    pub async fn process_text(
        text: &str,
        model: &str,
        prompt_template: &str
    ) -> Result<String>;
    pub async fn process_text_streaming(
        text: &str,
        model: &str,
        prompt_template: &str,
        callback: impl Fn(String)
    ) -> Result<String>;
}
```

### 2. Post-Processing Pipeline

**Design:**
- Chain of processors: CustomWords → Ollama → OutputFormatter
- Each processor is optional and configurable
- Processors implement a common trait

```rust
pub trait TextProcessor: Send + Sync {
    fn process(&self, text: String) -> Result<String>;
    fn name(&self) -> &str;
}

pub struct PostProcessingPipeline {
    processors: Vec<Box<dyn TextProcessor>>,
}

impl PostProcessingPipeline {
    pub fn new() -> Self;
    pub fn add_processor(&mut self, processor: Box<dyn TextProcessor>);
    pub fn process(&self, text: String) -> Result<String>;
}
```

### 3. Settings Integration

**New Settings:**
```rust
pub struct OllamaSettings {
    pub enabled: bool,
    pub base_url: String,  // Default: "http://localhost:11434"
    pub selected_model: String,  // e.g., "llama3.2:3b"
    pub processing_mode: ProcessingMode,
    pub custom_prompt: Option<String>,
    pub timeout_seconds: u64,
}

pub enum ProcessingMode {
    Punctuation,      // Add punctuation and capitalization
    Summarize,        // Summarize the transcription
    CommandExtract,   // Extract action items
    Custom,           // Use custom prompt
    Disabled,
}
```

**Prompt Templates:**
```
Punctuation: "Add proper punctuation and capitalization to this text, preserving the exact words: {text}"

Summarize: "Provide a concise summary of this transcription: {text}"

CommandExtract: "Extract action items and TODO items from this transcription as a bulleted list: {text}"
```

### 4. UI Components

**Ollama Settings Panel:**
- Enable/disable toggle
- Ollama connection status indicator
- Model selector (populated from Ollama API)
- Processing mode dropdown
- Custom prompt text area
- Test button (transcribe sample → process → show result)

### 5. Model Sharing Considerations

**Current State:**
- Handy uses GGML format (whisper.cpp)
- Ollama uses GGUF format (llama.cpp)

**No Direct Sharing Possible:**
- Different formats, cannot share model files
- Whisper models in Ollama are for different purposes (chat/completion, not STT)

**Solution:**
- Keep transcription models (Whisper/Parakeet) in Handy
- Use Ollama's LLM models for post-processing only
- No duplication because they serve different purposes:
  - Handy's models: Speech-to-Text
  - Ollama's models: Text-to-Text enhancement

**Future Optimization:**
- If `transcribe-rs` adds GGUF support, could potentially use Ollama's Whisper models
- For now, separate concerns is cleaner architecture

## Implementation Phases

### Phase 1: Foundation (Week 1-2)
- [ ] Create OllamaManager with basic HTTP client
- [ ] Implement health check and model listing
- [ ] Add Ollama settings to settings struct
- [ ] Create UI for Ollama settings panel

### Phase 2: Core Integration (Week 3-4)
- [ ] Implement text processing with Ollama API
- [ ] Create post-processing pipeline architecture
- [ ] Add prompt templates for common modes
- [ ] Integrate pipeline into transcription flow

### Phase 3: User Experience (Week 5-6)
- [ ] Add streaming support for real-time feedback
- [ ] Implement timeout and error handling
- [ ] Create test interface in settings
- [ ] Add usage examples and documentation

### Phase 4: Advanced Features (Week 7-8)
- [ ] Custom prompt templates
- [ ] Conversation history context
- [ ] Batch processing of history entries
- [ ] Performance metrics and optimization

## API Integration Details

### Ollama REST API Endpoints

**Check Health:**
```bash
GET http://localhost:11434/api/tags
# Returns list of available models
```

**Generate Completion:**
```bash
POST http://localhost:11434/api/generate
{
  "model": "llama3.2:3b",
  "prompt": "Add punctuation: hello world how are you",
  "stream": false
}
```

**Streaming Response:**
```bash
POST http://localhost:11434/api/generate
{
  "model": "llama3.2:3b",
  "prompt": "Add punctuation: hello world how are you",
  "stream": true
}
# Returns newline-delimited JSON
```

### Rust Implementation

```rust
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

#[derive(Deserialize)]
struct OllamaResponse {
    model: String,
    response: String,
    done: bool,
}

pub async fn process_with_ollama(
    text: &str,
    model: &str,
    prompt_template: &str,
) -> Result<String> {
    let client = reqwest::Client::new();
    let prompt = prompt_template.replace("{text}", text);

    let request = OllamaRequest {
        model: model.to_string(),
        prompt,
        stream: false,
        system: Some("You are a helpful assistant that improves transcription quality.".to_string()),
    };

    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&request)
        .send()
        .await?;

    let result: OllamaResponse = response.json().await?;
    Ok(result.response)
}
```

## Error Handling & Fallbacks

### Error Scenarios
1. **Ollama Not Running**: Gracefully skip post-processing, use raw transcription
2. **Model Not Available**: Show error, suggest downloading model
3. **Timeout**: Cancel processing after N seconds, use raw transcription
4. **Network Error**: Fallback to raw transcription
5. **Invalid Response**: Log error, use raw transcription

### Fallback Strategy
```rust
async fn transcribe_with_ollama(&self, audio: Vec<f32>) -> Result<String> {
    // Always do transcription first
    let raw_text = self.transcription_manager.transcribe(audio)?;

    // Try Ollama post-processing if enabled
    if let Ok(settings) = get_ollama_settings() {
        if settings.enabled {
            match self.ollama_manager.process_text(&raw_text, &settings).await {
                Ok(enhanced) => return Ok(enhanced),
                Err(e) => {
                    eprintln!("Ollama processing failed: {}, using raw transcription", e);
                    // Fall through to return raw_text
                }
            }
        }
    }

    Ok(raw_text)
}
```

## Testing Strategy

### Unit Tests
- OllamaManager HTTP client (mock server)
- Prompt template rendering
- Error handling and fallbacks
- Settings validation

### Integration Tests
- End-to-end transcription with Ollama
- Streaming response handling
- Timeout behavior
- Model availability detection

### Manual Testing Checklist
- [ ] Test with Ollama running
- [ ] Test without Ollama running (graceful fallback)
- [ ] Test with different models (small vs large)
- [ ] Test different processing modes
- [ ] Test custom prompts
- [ ] Test timeout scenarios
- [ ] Test with network issues
- [ ] Performance testing (latency impact)

## Performance Considerations

### Latency
- **Whisper Transcription**: 0.5-3s (depending on model and audio length)
- **Ollama Processing**: 0.5-5s (depending on model size and prompt)
- **Total**: 1-8s end-to-end

### Optimization Strategies
1. **Model Selection**: Recommend smaller models (3B params) for speed
2. **Streaming**: Show partial results as they arrive
3. **Caching**: Cache common corrections
4. **Async Processing**: Don't block UI during Ollama processing
5. **Skip Short Text**: Don't process very short transcriptions (<5 words)

### Resource Usage
- **Memory**: Ollama runs separately, minimal impact
- **CPU**: Shared with Ollama service
- **Network**: Localhost only, negligible

## Security & Privacy

### Privacy Guarantees
✅ **All processing is local** - No data leaves the user's machine
✅ **No telemetry** - No usage tracking or analytics
✅ **User control** - Can be completely disabled
✅ **Open source** - Transparent implementation

### Security Considerations
- Validate Ollama responses (don't execute code)
- Sanitize custom prompts (prevent injection)
- Use localhost only by default
- HTTPS if connecting to remote Ollama (advanced users)

## Documentation Plan

### User Documentation
1. **Getting Started with Ollama Integration**
   - Installing Ollama
   - Recommended models
   - Basic configuration

2. **Processing Modes Explained**
   - When to use each mode
   - Example outputs
   - Best practices

3. **Custom Prompts Guide**
   - Prompt engineering basics
   - Examples for common tasks
   - Variables and templating

### Developer Documentation
1. **Architecture Overview**
2. **API Reference** (OllamaManager)
3. **Adding New Processing Modes**
4. **Troubleshooting Guide**

## Success Metrics

### Technical Metrics
- Latency: <2s for small models on average hardware
- Reliability: >99% successful processing when Ollama is running
- Fallback: 100% graceful degradation when Ollama unavailable

### User Experience Metrics
- Setup time: <5 minutes from start to first enhanced transcription
- Configuration complexity: <3 steps to enable
- Error rate: <1% user-reported issues

## Future Enhancements

### Short-term (3-6 months)
- Conversation context (multi-turn awareness)
- Batch processing of history
- Advanced prompt templates library
- Model performance benchmarking

### Long-term (6-12 months)
- Plugin system for custom processors
- Multi-model pipelines (use different models for different tasks)
- RAG integration (use external knowledge)
- Voice command system (e.g., "Handy, summarize")

## Migration Path for Existing Users

### Non-Breaking Changes
- Ollama integration is **opt-in**
- No impact on existing workflows
- No required dependencies
- Existing transcriptions work as before

### Recommended Adoption
1. Install Ollama separately
2. Enable Ollama in Handy settings
3. Test with sample transcriptions
4. Choose preferred processing mode
5. Adjust settings as needed

## Conclusion

This integration strategy:
- ✅ Avoids model duplication (leverages Ollama's infrastructure)
- ✅ Maintains privacy (all local processing)
- ✅ Stays simple (optional, minimal configuration)
- ✅ Extends capabilities (post-processing, summarization, etc.)
- ✅ Remains forkable (clean architecture, well-documented)

**Next Steps:**
1. Get community feedback on this strategy
2. Create detailed implementation roadmap
3. Build Phase 1 prototype
4. Iterate based on testing
