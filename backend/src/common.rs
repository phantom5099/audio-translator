use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::CoreError;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LanguageTag(#[doc = "语言标签的实际字符串值，例如 zh-CN、en-US 或 ja-JP。"] pub String);
impl LanguageTag {
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(CoreError::InvalidInput(
                "language tag must not be empty".to_owned(),
            ));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for LanguageTag {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for LanguageTag {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    pub start_ms: u64,
    pub end_ms: u64,
}

impl TimeRange {
    pub fn new(start_ms: u64, end_ms: u64) -> Result<Self, CoreError> {
        if start_ms >= end_ms {
            return Err(CoreError::InvalidInput(format!(
                "time range must satisfy start_ms < end_ms, got {start_ms}..{end_ms}"
            )));
        }
        Ok(Self { start_ms, end_ms })
    }

    pub fn duration_ms(&self) -> u64 {
        self.end_ms.saturating_sub(self.start_ms)
    }

    pub fn validate(&self) -> Result<(), CoreError> {
        if self.start_ms >= self.end_ms {
            return Err(CoreError::InvalidInput(format!(
                "time range must satisfy start_ms < end_ms, got {}..{}",
                self.start_ms, self.end_ms
            )));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProviderOptions {
    /// provider 名称到 provider 私有配置 JSON 值的映射。
    pub values: BTreeMap<String, serde_json::Value>,
}
