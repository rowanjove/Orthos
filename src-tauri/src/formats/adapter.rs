use super::capabilities::FormatCapabilities;
use crate::document::DocumentNode;
use crate::FormatError;

pub trait FormatAdapter: Send + Sync {
    fn id(&self) -> &'static str;

    fn name(&self) -> &'static str;

    fn extensions(&self) -> &'static [&'static str];

    /// 嗅探内容属于该格式的可能性评分 (0 ~ 100)
    fn sniff(&self, content: &str) -> u8;

    /// 校验文本内容
    fn validate(&self, content: &str) -> Vec<FormatError>;

    /// 尝试自动修复
    fn repair(&self, content: &str) -> Option<String>;

    /// 格式化排版文本
    fn format(&self, content: &str) -> Result<String, FormatError>;

    /// 解析为 DocumentNode
    fn parse(&self, content: &str) -> Result<DocumentNode, FormatError>;

    /// 从 DocumentNode 序列化为文本
    fn serialize(&self, document: &DocumentNode) -> Result<String, FormatError>;

    /// 描述该格式拥有的能力
    fn capabilities(&self) -> FormatCapabilities;
}
