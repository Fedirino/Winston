const { onRequest } = require('firebase-functions/v2/https');
const { defineSecret } = require('firebase-functions/params');
const { initializeApp } = require('firebase-admin/app');
const { getAuth } = require('firebase-admin/auth');

initializeApp();

const openRouterKey = defineSecret('OPENROUTER_API_KEY');
const hermesApiKey = defineSecret('HERMES_API_KEY');
const hermesApiUrl = defineSecret('HERMES_API_URL');
const elevenLabsKey = defineSecret('ELEVENLABS_API_KEY');
const allowedEmail = defineSecret('WINSTON_ALLOWED_EMAIL');

const ALLOWED_ORIGINS = new Set([
  'https://kyo-os.web.app',
  'https://kyo-os.firebaseapp.com'
]);

const ALLOWED_MODELS = new Set([
  'xiaomi/mimo-v2.5',
  'google/gemini-2.5-flash-lite',
  'openai/gpt-5.4-nano',
  'qwen/qwen3.5-flash-02-23',
  'deepseek/deepseek-chat',
  'deepseek/deepseek-reasoner',
  'openrouter/auto'
]);

const requestWindows = new Map();

function json(res, status, body) {
  res.status(status).set('Cache-Control', 'no-store').json(body);
}

function rateLimit(uid) {
  const now = Date.now();
  const cutoff = now - 60_000;
  const recent = (requestWindows.get(uid) || []).filter((time) => time > cutoff);
  if (recent.length >= 30) return false;
  recent.push(now);
  requestWindows.set(uid, recent);
  return true;
}

async function authenticate(req) {
  const header = req.get('authorization') || '';
  const match = header.match(/^Bearer\s+(.+)$/i);
  if (!match) return null;
  try {
    const decoded = await getAuth().verifyIdToken(match[1]);
    const expected = allowedEmail.value().trim().toLowerCase();
    const actual = String(decoded.email || '').trim().toLowerCase();
    if (!expected || !decoded.email_verified || actual !== expected) return null;
    return decoded;
  } catch (error) {
    return null;
  }
}

function cleanMessages(messages) {
  if (!Array.isArray(messages)) return null;
  const cleaned = messages.slice(-32).map((message) => ({
    role: ['system', 'user', 'assistant'].includes(message && message.role) ? message.role : '',
    content: typeof (message && message.content) === 'string' ? message.content.slice(0, 12_000) : ''
  })).filter((message) => message.role && message.content);
  const total = cleaned.reduce((sum, message) => sum + message.content.length, 0);
  return cleaned.length && total <= 60_000 ? cleaned : null;
}

async function handleChat(req, res) {
  const messages = cleanMessages(req.body && req.body.messages);
  if (!messages) return json(res, 400, { error: 'Invalid messages.' });

  const hermesUrl = (hermesApiUrl.value() || '').replace(/\/+$/, '');
  const hermesKey = hermesApiKey.value();
  if (!hermesUrl || !hermesKey) {
    return json(res, 503, { error: 'Hermes relay is not configured.' });
  }

  const upstream = await fetch(hermesUrl + '/v1/chat/completions', {
    method: 'POST',
    headers: {
      Authorization: 'Bearer ' + hermesKey,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      model: 'hermes-agent',
      messages,
      max_tokens: Math.min(1000, Math.max(80, Number(req.body.max_tokens) || 220)),
      temperature: Math.min(1, Math.max(0, Number(req.body.temperature) || 0.6))
    })
  });
  if (!upstream.ok) {
    const detail = (await upstream.text().catch(() => '')).slice(0, 180);
    console.error('Hermes relay error', upstream.status, detail);
    return json(res, 502, { error: 'The Hermes relay is unavailable.' });
  }
  const data = await upstream.json();
  const reply = data && data.choices && data.choices[0] && data.choices[0].message
    ? String(data.choices[0].message.content || '').trim() : '';
  return json(res, 200, { reply: reply || '…' });
}

async function handleSpeech(req, res) {
  const text = String((req.body && req.body.text) || '').trim();
  const voiceId = String((req.body && req.body.voiceId) || '').trim();
  if (!text || text.length > 2500) return json(res, 400, { error: 'Invalid speech text.' });
  if (!/^[A-Za-z0-9_-]{5,80}$/.test(voiceId)) return json(res, 400, { error: 'Invalid voice ID.' });

  const upstream = await fetch('https://api.elevenlabs.io/v1/text-to-speech/' + encodeURIComponent(voiceId), {
    method: 'POST',
    headers: {
      'xi-api-key': elevenLabsKey.value(),
      'Content-Type': 'application/json',
      Accept: 'audio/mpeg'
    },
    body: JSON.stringify({
      text,
      model_id: 'eleven_multilingual_v2',
      voice_settings: { stability: 0.45, similarity_boost: 0.8, style: 0.25, use_speaker_boost: true }
    })
  });
  if (!upstream.ok) {
    console.error('ElevenLabs error', upstream.status, (await upstream.text().catch(() => '')).slice(0, 180));
    return json(res, 502, { error: 'The speech provider is unavailable.' });
  }
  const audio = Buffer.from(await upstream.arrayBuffer());
  res.status(200)
    .set('Cache-Control', 'no-store')
    .set('Content-Type', upstream.headers.get('content-type') || 'audio/mpeg')
    .send(audio);
}

exports.api = onRequest({
  region: 'us-central1',
  timeoutSeconds: 120,
  memory: '256MiB',
  maxInstances: 3,
  secrets: [openRouterKey, hermesApiKey, hermesApiUrl, elevenLabsKey, allowedEmail]
}, async (req, res) => {
  const origin = req.get('origin');
  res.set('Vary', 'Origin');
  res.set('X-Content-Type-Options', 'nosniff');
  if (origin && !ALLOWED_ORIGINS.has(origin)) return json(res, 403, { error: 'Origin not allowed.' });

  const path = String(req.path || req.url || '').split('?')[0].replace(/\/+$/, '');
  if (req.method === 'GET' && path.endsWith('/health')) {
    return json(res, 200, { ready: true, authentication: 'google' });
  }
  if (req.method !== 'POST') return json(res, 405, { error: 'Method not allowed.' });
  const contentLength = Number(req.get('content-length') || 0);
  const parsedLength = req.body ? JSON.stringify(req.body).length : 0;
  if (contentLength > 100_000 || parsedLength > 100_000) {
    return json(res, 413, { error: 'Request is too large.' });
  }

  const user = await authenticate(req);
  if (!user) return json(res, 401, { error: 'Sign in with the authorized Google account.' });
  if (!rateLimit(user.uid)) return json(res, 429, { error: 'Too many requests. Try again shortly.' });

  try {
    if (path.endsWith('/chat')) return await handleChat(req, res);
    if (path.endsWith('/tts')) return await handleSpeech(req, res);
    return json(res, 404, { error: 'Unknown endpoint.' });
  } catch (error) {
    console.error('Winston API error', error);
    return json(res, 500, { error: 'Winston could not complete that request.' });
  }
});
