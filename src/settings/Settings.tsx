import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

// ─── Hotkey code mapping ───────────────────────────────────────────────────────

/**
 * Maps a browser `KeyboardEvent.code` value to the string expected by the
 * Rust rdev hotkey parser.  Handles letter keys ("KeyA" → "A"), digit keys
 * ("Digit1" → "Num1"), modifier keys ("ControlRight" → "RightCtrl"), and
 * function keys ("F13" → "F13").  Falls back to the raw code if no specific
 * mapping exists.
 */
function mapKeyCode(code: string): string {
  // Letter keys: "KeyA" → "A"
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  // Digit keys: "Digit1" → "Num1"
  if (/^Digit\d$/.test(code)) return `Num${code.slice(5)}`;
  // Modifiers
  const modMap: Record<string, string> = {
    ControlLeft: "ControlLeft",
    ControlRight: "RightCtrl",
    ShiftLeft: "ShiftLeft",
    ShiftRight: "ShiftRight",
    AltLeft: "Alt",
    AltRight: "AltGr",
    MetaLeft: "MetaLeft",
    MetaRight: "MetaRight",
  };
  if (code in modMap) return modMap[code];
  // Function keys and other named keys are used as-is (e.g. "F13", "Escape")
  return code;
}

// ─── Types ─────────────────────────────────────────────────────────────────────

interface SpeechApiConfig {
  provider: string;
  endpoint: string;
  apiKey: string;
  model: string;
  language: string;
}

interface LlmConfig {
  enabled: boolean;
  endpoint: string;
  apiKey: string;
  model: string;
}

interface DictionaryConfig {
  techVocabEnabled: boolean;
  userVocab: string[];
  maxHistory: number;
}

interface AppSettings {
  language: string;
  hotkey: string;
  speechApi: SpeechApiConfig;
  llm: LlmConfig;
  dictionary: DictionaryConfig;
}

interface LanguageInfo {
  code: string;
  label: string;
}

// ─── Helpers ───────────────────────────────────────────────────────────────────

const defaultSettings: AppSettings = {
  language: "zh-CN",
  hotkey: "RightCtrl",
  speechApi: {
    provider: "mock",
    endpoint: "https://api.openai.com/v1/audio/transcriptions",
    apiKey: "",
    model: "whisper-1",
    language: "zh",
  },
  llm: {
    enabled: false,
    endpoint: "https://api.openai.com/v1/chat/completions",
    apiKey: "",
    model: "gpt-4o-mini",
  },
  dictionary: {
    techVocabEnabled: true,
    userVocab: [],
    maxHistory: 100,
  },
};

// ─── Component ─────────────────────────────────────────────────────────────────

export default function Settings() {
  const [settings, setSettings] = useState<AppSettings>(defaultSettings);
  const [languages, setLanguages] = useState<LanguageInfo[]>([]);
  const [activeTab, setActiveTab] = useState<string>("general");
  const [status, setStatus] = useState<string>("");
  const [recording, setRecording] = useState<boolean>(false);

  useEffect(() => {
    invoke<AppSettings>("get_settings")
      .then(setSettings)
      .catch((e) => console.error("get_settings error:", e));
    invoke<LanguageInfo[]>("get_supported_languages")
      .then(setLanguages)
      .catch((e) => console.error("get_supported_languages error:", e));
  }, []);

  function update<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
    setSettings((prev) => ({ ...prev, [key]: value }));
  }

  function updateSpeech<K extends keyof SpeechApiConfig>(
    key: K,
    value: SpeechApiConfig[K]
  ) {
    setSettings((prev) => ({
      ...prev,
      speechApi: { ...prev.speechApi, [key]: value },
    }));
  }

  function updateLlm<K extends keyof LlmConfig>(key: K, value: LlmConfig[K]) {
    setSettings((prev) => ({
      ...prev,
      llm: { ...prev.llm, [key]: value },
    }));
  }

  function updateDict<K extends keyof DictionaryConfig>(
    key: K,
    value: DictionaryConfig[K]
  ) {
    setSettings((prev) => ({
      ...prev,
      dictionary: { ...prev.dictionary, [key]: value },
    }));
  }

  async function handleSave() {
    try {
      await invoke("save_settings", { settings });
      setStatus("✓ Settings saved.");
      setTimeout(() => setStatus(""), 3000);
    } catch (e) {
      setStatus(`Error: ${e}`);
    }
  }

  function startHotkeyRecord() {
    setRecording(true);
    setStatus("Press the key you want to use as hotkey…");
  }

  useEffect(() => {
    if (!recording) return;
    const handler = (e: KeyboardEvent) => {
      e.preventDefault();
      // Map e.code to the key name expected by the Rust hotkey manager
      const keyName = mapKeyCode(e.code);
      update("hotkey", keyName);
      setRecording(false);
      setStatus("");
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [recording]);

  const tabs = ["general", "speech", "llm", "dictionary"];

  return (
    <div style={styles.container}>
      <h1 style={styles.title}>⚡ Typeless Settings</h1>

      {/* Tab bar */}
      <div style={styles.tabBar}>
        {tabs.map((t) => (
          <button
            key={t}
            style={{
              ...styles.tab,
              ...(activeTab === t ? styles.tabActive : {}),
            }}
            onClick={() => setActiveTab(t)}
          >
            {t.charAt(0).toUpperCase() + t.slice(1)}
          </button>
        ))}
      </div>

      <div style={styles.panel}>
        {/* ── General ── */}
        {activeTab === "general" && (
          <section>
            <h2 style={styles.sectionTitle}>General</h2>
            <Field label="Display Language">
              <select
                style={styles.input}
                value={settings.language}
                onChange={(e) => update("language", e.target.value)}
              >
                {languages.map((l) => (
                  <option key={l.code} value={l.code}>
                    {l.label}
                  </option>
                ))}
              </select>
            </Field>
            <Field label="Global Hotkey">
              <div style={{ display: "flex", gap: 8 }}>
                <input
                  style={{ ...styles.input, flex: 1 }}
                  value={settings.hotkey}
                  readOnly
                  placeholder="e.g. RightCtrl"
                />
                <button
                  style={{ ...styles.button, background: recording ? "#e55" : "#555" }}
                  onClick={startHotkeyRecord}
                >
                  {recording ? "Listening…" : "Record"}
                </button>
              </div>
            </Field>
          </section>
        )}

        {/* ── Speech API ── */}
        {activeTab === "speech" && (
          <section>
            <h2 style={styles.sectionTitle}>Speech Recognition</h2>
            <Field label="Provider">
              <select
                style={styles.input}
                value={settings.speechApi.provider}
                onChange={(e) => updateSpeech("provider", e.target.value)}
              >
                <option value="mock">Mock (testing)</option>
                <option value="openai_http">OpenAI Whisper (HTTP)</option>
                <option value="websocket">WebSocket Stream</option>
              </select>
            </Field>
            <Field label="Endpoint URL">
              <input
                style={styles.input}
                value={settings.speechApi.endpoint}
                onChange={(e) => updateSpeech("endpoint", e.target.value)}
              />
            </Field>
            <Field label="API Key">
              <input
                style={styles.input}
                type="password"
                value={settings.speechApi.apiKey}
                onChange={(e) => updateSpeech("apiKey", e.target.value)}
                placeholder="sk-…"
              />
            </Field>
            <Field label="Model">
              <input
                style={styles.input}
                value={settings.speechApi.model}
                onChange={(e) => updateSpeech("model", e.target.value)}
              />
            </Field>
            <Field label="Recognition Language">
              <input
                style={styles.input}
                value={settings.speechApi.language}
                onChange={(e) => updateSpeech("language", e.target.value)}
                placeholder="zh / en / ja"
              />
            </Field>
          </section>
        )}

        {/* ── LLM ── */}
        {activeTab === "llm" && (
          <section>
            <h2 style={styles.sectionTitle}>LLM Post-processing</h2>
            <Field label="Enable LLM Refinement">
              <label style={styles.toggle}>
                <input
                  type="checkbox"
                  checked={settings.llm.enabled}
                  onChange={(e) => updateLlm("enabled", e.target.checked)}
                />
                <span style={{ marginLeft: 8 }}>
                  {settings.llm.enabled ? "Enabled" : "Disabled"}
                </span>
              </label>
            </Field>
            <Field label="Endpoint URL">
              <input
                style={styles.input}
                value={settings.llm.endpoint}
                onChange={(e) => updateLlm("endpoint", e.target.value)}
                disabled={!settings.llm.enabled}
              />
            </Field>
            <Field label="API Key">
              <input
                style={styles.input}
                type="password"
                value={settings.llm.apiKey}
                onChange={(e) => updateLlm("apiKey", e.target.value)}
                placeholder="sk-…"
                disabled={!settings.llm.enabled}
              />
            </Field>
            <Field label="Model">
              <input
                style={styles.input}
                value={settings.llm.model}
                onChange={(e) => updateLlm("model", e.target.value)}
                disabled={!settings.llm.enabled}
              />
            </Field>
          </section>
        )}

        {/* ── Dictionary ── */}
        {activeTab === "dictionary" && (
          <section>
            <h2 style={styles.sectionTitle}>Dictionary</h2>
            <Field label="Tech Vocabulary">
              <label style={styles.toggle}>
                <input
                  type="checkbox"
                  checked={settings.dictionary.techVocabEnabled}
                  onChange={(e) =>
                    updateDict("techVocabEnabled", e.target.checked)
                  }
                />
                <span style={{ marginLeft: 8 }}>
                  Enable built-in tech vocab hints
                </span>
              </label>
            </Field>
            <Field label="Custom Vocabulary (one per line)">
              <textarea
                style={{ ...styles.input, height: 120, resize: "vertical" }}
                value={settings.dictionary.userVocab.join("\n")}
                onChange={(e) =>
                  updateDict(
                    "userVocab",
                    e.target.value
                      .split("\n")
                      .map((s) => s.trim())
                      .filter(Boolean)
                  )
                }
              />
            </Field>
            <Field label="Max History Entries">
              <input
                style={{ ...styles.input, width: 100 }}
                type="number"
                min={10}
                max={10000}
                value={settings.dictionary.maxHistory}
                onChange={(e) =>
                  updateDict("maxHistory", parseInt(e.target.value, 10) || 100)
                }
              />
            </Field>
          </section>
        )}
      </div>

      {/* Footer */}
      <div style={styles.footer}>
        {status && <span style={styles.statusMsg}>{status}</span>}
        <button style={styles.saveButton} onClick={handleSave}>
          Save Settings
        </button>
      </div>
    </div>
  );
}

// ─── Sub-component ─────────────────────────────────────────────────────────────

function Field({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div style={styles.field}>
      <label style={styles.label}>{label}</label>
      {children}
    </div>
  );
}

// ─── Styles ────────────────────────────────────────────────────────────────────

const styles: Record<string, React.CSSProperties> = {
  container: {
    fontFamily: "'Inter', '-apple-system', 'Segoe UI', sans-serif",
    background: "#1a1b1e",
    color: "#e0e0e0",
    minHeight: "100vh",
    display: "flex",
    flexDirection: "column",
    padding: "0 0 24px 0",
  },
  title: {
    fontSize: 20,
    fontWeight: 700,
    padding: "20px 24px 12px",
    margin: 0,
    borderBottom: "1px solid #2d2e32",
    color: "#fff",
  },
  tabBar: {
    display: "flex",
    gap: 2,
    padding: "8px 24px 0",
    borderBottom: "1px solid #2d2e32",
  },
  tab: {
    background: "none",
    border: "none",
    color: "#888",
    cursor: "pointer",
    padding: "8px 16px",
    fontSize: 14,
    borderBottom: "2px solid transparent",
    marginBottom: -1,
  },
  tabActive: {
    color: "#fff",
    borderBottomColor: "#5865f2",
  },
  panel: {
    flex: 1,
    padding: "20px 24px",
    overflowY: "auto",
  },
  sectionTitle: {
    fontSize: 15,
    fontWeight: 600,
    color: "#aaa",
    margin: "0 0 16px",
    textTransform: "uppercase",
    letterSpacing: 1,
  },
  field: {
    marginBottom: 16,
  },
  label: {
    display: "block",
    fontSize: 13,
    color: "#aaa",
    marginBottom: 6,
  },
  input: {
    background: "#2d2e32",
    border: "1px solid #3a3b40",
    borderRadius: 6,
    color: "#e0e0e0",
    padding: "7px 10px",
    fontSize: 14,
    width: "100%",
    boxSizing: "border-box",
    outline: "none",
  },
  toggle: {
    display: "flex",
    alignItems: "center",
    cursor: "pointer",
    fontSize: 14,
  },
  button: {
    background: "#3a3b40",
    border: "1px solid #555",
    borderRadius: 6,
    color: "#e0e0e0",
    padding: "7px 14px",
    fontSize: 13,
    cursor: "pointer",
    whiteSpace: "nowrap",
  },
  footer: {
    display: "flex",
    alignItems: "center",
    justifyContent: "flex-end",
    gap: 16,
    padding: "0 24px",
  },
  statusMsg: {
    fontSize: 13,
    color: "#7ec87e",
  },
  saveButton: {
    background: "#5865f2",
    border: "none",
    borderRadius: 8,
    color: "#fff",
    padding: "10px 24px",
    fontSize: 14,
    fontWeight: 600,
    cursor: "pointer",
  },
};
