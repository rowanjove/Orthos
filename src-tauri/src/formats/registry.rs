use super::adapter::FormatAdapter;
use super::capabilities::FormatDescriptor;
use super::csv::CsvAdapter;
use super::env::EnvAdapter;
use super::extensions::{
    EditorConfigAdapter, GitConfigAdapter, HclAdapter, Json5Adapter, JsoncAdapter, JsonlAdapter,
    PropertiesAdapter, TsvAdapter,
};
use super::ini::IniAdapter;
use super::json::JsonAdapter;
use super::toml::TomlAdapter;
use super::xml::XmlAdapter;
use super::yaml::YamlAdapter;
use std::sync::{Arc, OnceLock};

pub struct FormatRegistry {
    adapters: Vec<Arc<dyn FormatAdapter>>,
}

impl Default for FormatRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FormatRegistry {
    pub fn new() -> Self {
        let adapters: Vec<Arc<dyn FormatAdapter>> = vec![
            Arc::new(JsonAdapter),
            Arc::new(YamlAdapter),
            Arc::new(TomlAdapter),
            Arc::new(XmlAdapter),
            Arc::new(CsvAdapter),
            Arc::new(IniAdapter),
            Arc::new(EnvAdapter),
            Arc::new(JsoncAdapter),
            Arc::new(Json5Adapter),
            Arc::new(JsonlAdapter),
            Arc::new(TsvAdapter),
            Arc::new(PropertiesAdapter),
            Arc::new(EditorConfigAdapter),
            Arc::new(GitConfigAdapter),
            Arc::new(HclAdapter),
        ];
        Self { adapters }
    }

    pub fn get(&self, id: &str) -> Option<&Arc<dyn FormatAdapter>> {
        let target = id.to_ascii_lowercase();
        self.adapters.iter().find(|a| a.id() == target)
    }

    pub fn list(&self) -> Vec<FormatDescriptor> {
        self.adapters
            .iter()
            .map(|a| FormatDescriptor {
                id: a.id().to_string(),
                name: a.name().to_string(),
                extensions: a.extensions().iter().map(|e| e.to_string()).collect(),
                capabilities: a.capabilities(),
            })
            .collect()
    }

    pub fn detect(&self, filename: Option<&str>, content: Option<&str>) -> Option<String> {
        // 1. 优先根据文件扩展名精确匹配
        if let Some(fname) = filename {
            let lower = fname.to_ascii_lowercase();
            for adapter in &self.adapters {
                for ext in adapter.extensions() {
                    if lower.ends_with(ext) {
                        return Some(adapter.id().to_string());
                    }
                }
            }
        }

        // 2. 根据内容嗅探评分选出最匹配的格式
        if let Some(text) = content {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return None;
            }

            let mut best_score = 0u8;
            let mut best_id = None;

            for adapter in &self.adapters {
                let score = adapter.sniff(trimmed);
                if score > best_score {
                    best_score = score;
                    best_id = Some(adapter.id().to_string());
                }
            }

            if best_score > 50 {
                return best_id;
            }
        }

        None
    }
}

static GLOBAL_REGISTRY: OnceLock<FormatRegistry> = OnceLock::new();

pub fn get_registry() -> &'static FormatRegistry {
    GLOBAL_REGISTRY.get_or_init(FormatRegistry::new)
}
