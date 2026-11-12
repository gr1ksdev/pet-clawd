// Cliente de IA — mesmos provedores do plugin (Gemini / OpenAI-compatível
// com fallback ai2: OpenRouter, Groq, Ollama local, etc).
use serde::{Deserialize, Serialize};

const GEMINI_URL: &str = "https://generativelanguage.googleapis.com/v1beta";
const OPENAI_URL: &str = "https://api.openai.com/v1";
const AI_ERR: &str = "IA: verifique a chave e o modelo";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AiConfig {
    /// 0 = Gemini, 1 = OpenAI-compatível
    pub provider: u8,
    pub key: String,
    pub model: String,
    /// Fallback (usado quando a principal falha), OpenAI-compatível
    pub ai2_url: String,
    pub ai2_key: String,
    pub ai2_model: String,
}

async fn post_json(
    url: &str,
    body: &serde_json::Value,
    bearer: Option<&str>,
) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(25))
        .build()
        .map_err(|e| e.to_string())?;
    let mut req = client.post(url).json(body);
    if let Some(b) = bearer {
        req = req.bearer_auth(b);
    }
    let resp = req
        .send()
        .await
        .map_err(|e| format!("IA: rede falhou ({})", e))?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!(
            "IA: Erro HTTP {} (verifique chave/modelo)",
            status.as_u16()
        ));
    }
    Ok(text)
}

fn gemini_reply(raw: &str) -> Result<String, String> {
    let v: serde_json::Value = serde_json::from_str(raw).map_err(|_| AI_ERR.to_string())?;
    v["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AI_ERR.to_string())
}

fn openai_reply(raw: &str) -> Result<String, String> {
    let v: serde_json::Value = serde_json::from_str(raw).map_err(|_| AI_ERR.to_string())?;
    v["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AI_ERR.to_string())
}

const SYSTEM: &str = "Você é o Clawd, um gatinho pixelado espirituoso e carinhoso \
que vive na tela do seu humano. Responda em UMA frase curta (máx ~90 caracteres), \
em português do Brasil, fofa e direta. No máximo um emoji.";

async fn gemini_talk(cfg: &AiConfig, user: &str) -> Result<String, String> {
    let key = cfg.key.trim();
    if key.is_empty() {
        return Err(AI_ERR.to_string());
    }
    let model = if cfg.model.trim().is_empty() {
        "gemini-2.0-flash"
    } else {
        cfg.model.trim()
    };
    let url = format!("{}/models/{}:generateContent?key={}", GEMINI_URL, model, key);
    let body = serde_json::json!({
        "systemInstruction": {"parts": [{"text": SYSTEM}]},
        "contents": [{"role": "user", "parts": [{"text": user}]}],
        "generationConfig": {"maxOutputTokens": 120, "temperature": 1.0}
    });
    let raw = post_json(&url, &body, None).await?;
    gemini_reply(&raw)
}

async fn openai_talk(
    base: &str,
    key: &str,
    model: &str,
    user: &str,
) -> Result<String, String> {
    if key.is_empty() {
        return Err(AI_ERR.to_string());
    }
    let base = base.trim().trim_end_matches('/');
    let base = if base.is_empty() { OPENAI_URL } else { base };
    let url = format!("{}/chat/completions", base);
    let body = serde_json::json!({
        "model": model,
        "max_tokens": 120,
        "temperature": 1.0,
        "messages": [
            {"role": "system", "content": SYSTEM},
            {"role": "user", "content": user}
        ]
    });
    let raw = post_json(&url, &body, Some(key)).await?;
    openai_reply(&raw)
}

/// Fala do Clawd: provedor principal → fallback ai2 (quando configurado).
pub async fn talk(cfg: &AiConfig, user: &str) -> Result<String, String> {
    let primary = if cfg.provider == 0 {
        gemini_talk(cfg, user).await
    } else {
        let model = if cfg.model.trim().is_empty() {
            "gpt-4o-mini"
        } else {
            cfg.model.trim()
        };
        openai_talk(OPENAI_URL, cfg.key.trim(), model, user).await
    };
    match primary {
        Ok(s) => Ok(s),
        Err(e) => {
            let m2 = cfg.ai2_model.trim();
            if m2.is_empty() {
                Err(e)
            } else {
                openai_talk(cfg.ai2_url.trim(), cfg.ai2_key.trim(), m2, user).await
            }
        }
    }
}
