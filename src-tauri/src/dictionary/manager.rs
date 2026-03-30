use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Built-in technical vocabulary for ASR hint augmentation.
#[allow(dead_code)]
const TECH_VOCAB: &[&str] = &[
    // Rust
    "Rust",
    "Cargo",
    "crate",
    "trait",
    "impl",
    "lifetime",
    "borrow",
    "ownership",
    "async",
    "await",
    "tokio",
    "serde",
    "anyhow",
    "thiserror",
    // Kubernetes / cloud
    "Kubernetes",
    "kubectl",
    "Pod",
    "Deployment",
    "Service",
    "ConfigMap",
    "Ingress",
    "Helm",
    "Namespace",
    "ReplicaSet",
    "StatefulSet",
    "DaemonSet",
    "CronJob",
    "YAML",
    "kubeconfig",
    // TypeScript / JS
    "TypeScript",
    "JavaScript",
    "React",
    "Vue",
    "Node.js",
    "npm",
    "pnpm",
    "Vite",
    "webpack",
    "ESLint",
    "Prettier",
    "interface",
    "generic",
    "Promise",
    "async/await",
    "JSX",
    "TSX",
    // Databases
    "PostgreSQL",
    "MySQL",
    "Redis",
    "MongoDB",
    "SQLite",
    "Elasticsearch",
    "Kafka",
    // General tech
    "API",
    "REST",
    "GraphQL",
    "gRPC",
    "WebSocket",
    "JSON",
    "OAuth",
    "JWT",
    "Docker",
    "GitHub",
    "GitLab",
    "CI/CD",
    "DevOps",
    "Linux",
    "macOS",
    "Windows",
    // AI/ML
    "LLM",
    "GPT",
    "Whisper",
    "OpenAI",
    "embedding",
    "tokenizer",
    "fine-tuning",
    "inference",
];

fn history_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("typeless")
        .join("history.json")
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct History {
    /// word -> usage count
    frequency: HashMap<String, usize>,
    entries: Vec<String>,
}

pub struct DictionaryManager {
    #[allow(dead_code)]
    user_vocab: Vec<String>,
    #[allow(dead_code)]
    history: History,
    max_history: usize,
    #[allow(dead_code)]
    tech_vocab_enabled: bool,
}

impl DictionaryManager {
    pub fn new(tech_vocab_enabled: bool, user_vocab: Vec<String>, max_history: usize) -> Self {
        let history = Self::load_history().unwrap_or_default();
        Self {
            user_vocab,
            history,
            max_history,
            tech_vocab_enabled,
        }
    }

    fn load_history() -> Result<History> {
        let path = history_path();
        if !path.exists() {
            return Ok(History::default());
        }
        let data = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read history from {}", path.display()))?;
        serde_json::from_str(&data).context("Failed to parse history JSON")
    }

    pub fn save_history(&self) -> Result<()> {
        let path = history_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&self.history)?;
        fs::write(&path, json)?;
        Ok(())
    }

    /// Return vocabulary hints relevant to the partial text.
    #[allow(dead_code)]
    pub fn get_hints(&self, partial: &str) -> Vec<String> {
        let lower = partial.to_lowercase();
        let mut hints: Vec<String> = Vec::new();

        if self.tech_vocab_enabled {
            for &word in TECH_VOCAB {
                if word.to_lowercase().contains(&lower) {
                    hints.push(word.to_string());
                }
            }
        }

        for word in &self.user_vocab {
            if word.to_lowercase().contains(&lower) && !hints.contains(word) {
                hints.push(word.clone());
            }
        }

        // Sort by frequency descending
        hints.sort_by(|a, b| {
            let fa = self.history.frequency.get(a).copied().unwrap_or(0);
            let fb = self.history.frequency.get(b).copied().unwrap_or(0);
            fb.cmp(&fa)
        });

        hints.truncate(20);
        hints
    }

    /// Record a completed transcription for frequency tracking.
    pub fn record_input(&mut self, text: &str) {
        self.history.entries.push(text.to_string());
        if self.history.entries.len() > self.max_history {
            self.history.entries.remove(0);
        }

        // Update word frequency
        for word in text.split_whitespace() {
            let clean: String = word
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                .collect();
            if !clean.is_empty() {
                *self.history.frequency.entry(clean).or_insert(0) += 1;
            }
        }

        if let Err(e) = self.save_history() {
            log::warn!("Failed to save history: {e}");
        }
    }
}
