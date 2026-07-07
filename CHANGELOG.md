# Changelog

## 0.5.1
- Added a desktop/large-viewport breakpoint (≥820px wide, ≥640px tall): wider shell, bigger brand mark, core, mic, and text, scrollable body instead of clipped. Same single file, no separate app — the mobile layout is untouched below that size.

## 0.5.0
- Ember Core now drifts subtly with the real local time of day: warm bright orange near 2pm, cooling toward a deeper, dimmer ember-red around 2am, smoothly in between. Idle state also breathes gently instead of holding a static spike.
- Kyo now gets a live system note before every message with the real current date/time, and — with location permission — current local weather pulled from Open-Meteo (free, no API key), refreshed every 15 minutes. Keeps answers about "what time is it" / "what's it like out" grounded in reality instead of guessed.
- Persona now addresses the user as Fedirino or Sir (never Nick) and is written to feel like it already has rapport — established assistant, not a fresh introduction.
- Conversation history is still session-only for now (not persisted across reloads) — holding off on that by request.

## 0.4.1
- Killed the default mobile blue tap-highlight on buttons/links; added a proper focus-visible ring for keyboard/accessibility use instead.
- Mic button now clearly shows "on": filled amber background + brighter border while listening, plus an expanding pulse ring, on top of the existing corona and icon glow.
- Map action now opens the real Google Maps app first (`comgooglemaps://`) and only falls back to the Maps website if the app doesn't actually open (desktop, or app not installed).

## 0.4.0
- Kyo can now actually do things: open a website, run a web search, or pull up a map/directions, triggered straight from the conversation.
- The model signals an action with a trailing tag it never speaks aloud (`[ACTION:OPEN_URL|...]`, `[ACTION:SEARCH|...]`, `[ACTION:MAP|...]`); Kyo strips it before displaying/speaking and opens the real destination (Google Search / Google Maps / direct URL) in a new tab.
- Added a small amber HUD toast ("opening map — ...") that confirms what just launched, matching the Ember Core look.
- Version and service worker cache bumped to 0.4.0.

## 0.3.1
- Mic button redesign: replaced the glossy gradient sphere with a dense ember-particle corona (kin to the hero core's spike engine) behind the mic glyph. Flares brighter/faster while listening.
- Design exploration for this pass archived under mockups/ (mic-options.html through mic-options-v6.html).
- Version and service worker cache bumped to 0.3.1.

## 0.3.0
- Full visual redesign: EMBER MINIMAL. Living particle-burst core (canvas) replaces the orb — flares when listening, contracts and swirls when thinking, pulses while speaking.
- New warm ember palette (orange on graphite) replacing neon green; glossy mic button; hairline-only UI.
- Kyo wordmark rebuilt: fully legible gradient base with periodic glitch bursts.
- New persona: calm, precise "warm machine" assistant (replaces AI-scared hillbilly).
- Conversation shown as floating lines with a time-aware greeting; status reads "Kyo is listening/thinking/speaking".
- Design exploration archive added under mockups/.
- Manifest and service worker cache bumped for clean rollout.

## 0.2.3
- Pushed Kyo into a much more drastic AI-scared hillbilly comedy persona.
- Tuned the voice to feel jumpy, suspicious of automation, and funny without losing usefulness.
- Updated the greeting and visible version/cache so the new shell is obvious after refresh.

## 0.2.2
- Switched Kyo into Hillbilly Skeptic personality mode.
- Added user context to the system prompt so Kyo knows Nick works in management in retail produce and prefers concise, direct help.
- Tuned the default tone to be folksy, skeptical, and useful without becoming cartoonish.

## 0.2.1
- Added proper home-screen icons and Apple touch icon assets.
- Updated manifest and service worker cache to avoid stale installed PWA shells.
- Bumped app shell version for cleaner refresh links.

## 0.2.0
- Added OpenRouter chat integration.
- Added Settings drawer for API key + model selection.
- Added Web Speech voice input and TTS voice output.
- Avatar reacts to listening / thinking / speaking states.
- Default model set to a cheap OpenRouter option.

## 0.1.0
- Initial Kyo scaffold.
