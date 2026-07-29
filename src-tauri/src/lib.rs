use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

// ── App state ──────────────────────────────────────────────────────
struct AppConfig {
    hermes_url: String,
    hermes_key: String,
    eleven_voice: String,
}

struct AppState {
    config: Mutex<AppConfig>,
    client: reqwest::Client,
}

// ── API types ──────────────────────────────────────────────────────

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    input: String,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Deserialize)]
struct RunResponse {
    run_id: String,
}

#[derive(Deserialize)]
struct SSEMessage {
    event: String,
    #[serde(default)]
    delta: Option<String>,
    #[serde(default)]
    output: Option<String>,
    #[serde(default)]
    tool: Option<String>,
    #[serde(default)]
    preview: Option<String>,
    #[serde(default)]
    duration: Option<f64>,
    #[serde(default)]
    error: Option<bool>,
}

#[derive(Serialize)]
struct ToolCallInfo {
    tool: String,
    arguments: String,
    status: String,
    latency: Option<u64>,
}

#[derive(Serialize)]
struct ChatResult {
    reply: String,
    tool_calls: Vec<ToolCallInfo>,
}

#[derive(Serialize)]
struct TTSRequest {
    text: String,
    voice_id: String,
}

#[derive(Serialize)]
struct TranscribeRequest {
    audio: String,
    mime: String,
}

#[derive(Deserialize)]
struct TranscribeResponse {
    text: String,
}

// ── Commands ───────────────────────────────────────────────────────

#[tauri::command]
async fn chat(input: String, state: State<'_, AppState>) -> Result<ChatResult, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let client = &state.client;

    // 1. Create run
    let resp = client
        .post(format!("{}/v1/runs", config.hermes_url))
        .header("Authorization", format!("Bearer {}", config.hermes_key))
        .json(&ChatRequest {
            model: "hermes-agent".into(),
            input,
            max_tokens: 500,
            temperature: 0.6,
        })
        .send()
        .await
        .map_err(|e| format!("Failed to create run: {}", e))?;

    let run: RunResponse = resp.json().await.map_err(|e| format!("Bad run response: {}", e))?;

    // 2. Poll events
    let mut full_output = String::new();
    let mut tool_calls: Vec<ToolCallInfo> = Vec::new();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(110);

    loop {
        if std::time::Instant::now() > deadline {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(600)).await;

        let events_resp = client
            .get(format!("{}/v1/runs/{}/events", config.hermes_url, run.run_id))
            .header("Authorization", format!("Bearer {}", config.hermes_key))
            .send()
            .await;

        match events_resp {
            Ok(resp) => {
                if !resp.status().is_success() {
                    // 404 means run is done — fetch final status
                    if resp.status().as_u16() == 404 {
                        if let Ok(status_resp) = client
                            .get(format!("{}/v1/runs/{}", config.hermes_url, run.run_id))
                            .header("Authorization", format!("Bearer {}", config.hermes_key))
                            .send()
                            .await
                        {
                            if let Ok(status) = status_resp.json::<serde_json::Value>().await {
                                if let Some(out) = status.get("output").and_then(|v| v.as_str()) {
                                    full_output = out.to_string();
                                }
                            }
                        }
                    }
                    break;
                }

                let raw = resp.text().await.unwrap_or_default();
                for line in raw.lines() {
                    if !line.starts_with("data: ") {
                        continue;
                    }
                    let payload = &line[6..].trim();
                    if payload.is_empty() {
                        continue;
                    }
                    if let Ok(evt) = serde_json::from_str::<SSEMessage>(payload) {
                        match evt.event.as_str() {
                            "run.completed" => {
                                if let Some(out) = evt.output {
                                    full_output = out;
                                }
                                break;
                            }
                            "message.delta" => {
                                if let Some(d) = evt.delta {
                                    full_output.push_str(&d);
                                }
                            }
                            "tool.started" => {
                                tool_calls.push(ToolCallInfo {
                                    tool: evt.tool.unwrap_or_else(|| "tool".into()),
                                    arguments: evt.preview.unwrap_or_default(),
                                    status: "running".into(),
                                    latency: None,
                                });
                            }
                            "tool.completed" => {
                                let is_error = evt.error.unwrap_or(false);
                                let dur = evt.duration.map(|d| (d * 1000.0) as u64);
                                if let Some(tc) = tool_calls.iter_mut().rev().find(|t| t.status == "running") {
                                    tc.status = if is_error { "error".into() } else { "success".into() };
                                    tc.latency = dur;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("Events poll error: {}", e);
                break;
            }
        }
    }

    Ok(ChatResult {
        reply: if full_output.is_empty() { "…".into() } else { full_output },
        tool_calls,
    })
}

#[tauri::command]
async fn speak(text: String, state: State<'_, AppState>) -> Result<Vec<u8>, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let client = &state.client;

    let voice_id = &config.eleven_voice;
    let resp = client
        .post(format!("https://api.elevenlabs.io/v1/text-to-speech/{}", voice_id))
        .header("xi-api-key", "TODO") // Will need ElevenLabs key in config
        .header("Content-Type", "application/json")
        .header("Accept", "audio/mpeg")
        .json(&serde_json::json!({
            "text": text,
            "model_id": "eleven_multilingual_v2",
            "voice_settings": {
                "stability": 0.45,
                "similarity_boost": 0.8,
                "style": 0.25,
                "use_speaker_boost": true
            }
        }))
        .send()
        .await
        .map_err(|e| format!("TTS failed: {}", e))?;

    let audio = resp.bytes().await.map_err(|e| format!("TTS read failed: {}", e))?;
    Ok(audio.to_vec())
}

#[tauri::command]
async fn transcribe(audio: String, mime: String, state: State<'_, AppState>) -> Result<String, String> {
    let client = &state.client;

    let resp = client
        .post("https://openrouter.ai/api/v1/audio/transcriptions")
        .header("Authorization", "Bearer TODO") // Will need OpenRouter key in config
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "openai/whisper-large-v3",
            "audio": format!("data:{};base64,{}", mime, audio),
            "response_format": "json"
        }))
        .send()
        .await
        .map_err(|e| format!("Transcribe failed: {}", e))?;

    let data = resp.json::<TranscribeResponse>().await.map_err(|e| format!("Transcribe parse failed: {}", e))?;
    Ok(data.text)
}

#[tauri::command]
async fn get_health(state: State<'_, AppState>) -> Result<bool, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let client = &state.client;

    match client
        .get(format!("{}/health", config.hermes_url))
        .send()
        .await
    {
        Ok(resp) => Ok(resp.status().is_success()),
        Err(_) => Ok(false),
    }
}

// ── App entry ──────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Read config from environment or Hermes .env
    let hermes_url = std::env::var("HERMES_URL").unwrap_or_else(|_| "http://127.0.0.1:8642".into());
    let hermes_key = std::env::var("HERMES_KEY").unwrap_or_default();
    let eleven_voice = std::env::var("ELEVEN_VOICE").unwrap_or_else(|_| "21m00Tcm4TlvDq8ikWAM".into());

    tauri::Builder::default()
        .manage(AppState {
            config: Mutex::new(AppConfig {
                hermes_url,
                hermes_key,
                eleven_voice,
            }),
            client: reqwest::Client::new(),
        })
        .invoke_handler(tauri::generate_handler![
            chat,
            speak,
            transcribe,
            get_health,
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}