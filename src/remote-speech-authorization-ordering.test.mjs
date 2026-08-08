import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

function source(path) {
  return readFileSync(new URL(path, import.meta.url), "utf8");
}

test("top-level remote ASR authorizes before capture and never reauthorizes after drain", () => {
  const handlers = source("../src-tauri/src/command_handlers/voice_handlers.rs");
  const start = handlers.indexOf("fn run_phased_transcribe(");
  const end = handlers.indexOf("\nfn transcription_stop_mode", start);
  assert.notEqual(start, -1);
  assert.notEqual(end, -1);
  const body = handlers.slice(start, end);

  const privacy = body.indexOf("prepare_microphone_transcription(&input)");
  const beginCapture = body.indexOf("begin_transcribe_command(&input)");
  const captureWindow = body.indexOf("thread::sleep");
  const drain = body.indexOf("drain_transcribe_command(plan)");
  const dispatch = body.indexOf("transcribe_captured_audio(");

  assert.ok(privacy >= 0, "remote microphone privacy gate must exist");
  assert.ok(privacy < beginCapture, "privacy must be decided before capture begins");
  assert.ok(beginCapture < captureWindow, "capture must begin before the unlocked window");
  assert.ok(captureWindow < drain, "capture must finish before audio is drained");
  assert.ok(drain < dispatch, "remote ASR dispatch must happen only after drain");
  assert.equal(
    body.match(/prepare_microphone_transcription/g)?.length ?? 0,
    1,
    "the phased path must not re-enter privacy evaluation after capture",
  );
});

test("remote speech provider choke points consume authorization values", () => {
  const asr = source("../src-tauri/src/asr/mod.rs");
  const tts = source("../src-tauri/src/tts/mod.rs");

  assert.match(asr, /remote_authorization: Option<RemoteMicrophoneAuthorization>/);
  assert.doesNotMatch(asr, /remote_authorization: Option<&RemoteMicrophoneAuthorization>/);
  assert.match(tts, /remote_authorization: Option<RemoteNarrationAuthorization>/);
  assert.doesNotMatch(tts, /remote_authorization: Option<&RemoteNarrationAuthorization>/);

  const remoteGuard = asr.indexOf("remote_authorization.is_none()");
  const capture = asr.indexOf("let captured_audio = self.capture_audio", remoteGuard);
  assert.ok(remoteGuard >= 0 && remoteGuard < capture, "remote ASR must reject before capture without authorization");
});
