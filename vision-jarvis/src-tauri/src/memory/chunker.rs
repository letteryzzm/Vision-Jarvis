/// 文本分块器 - 将Markdown文件分割为适合向量化的文本块
///
/// 核心功能：
/// 1. 检测和跳过YAML frontmatter
/// 2. 智能token估算（CJK字符 + ASCII词）
/// 3. 实现重叠分块（overlap）
/// 4. SHA-256哈希计算

use anyhow::Result;
use sha2::{Sha256, Digest};

/// 分块配置
#[derive(Debug, Clone)]
pub struct ChunkConfig {
    /// 目标token数
    pub target_tokens: usize,
    /// 重叠token数
    pub overlap_tokens: usize,
    /// 最小分块大小
    pub min_tokens: usize,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            target_tokens: 400,
            overlap_tokens: 80,
            min_tokens: 100,
        }
    }
}

/// 文本块
#[derive(Debug, Clone)]
pub struct TextChunk {
    /// 分块内容
    pub text: String,
    /// 起始行号(1-based)
    pub start_line: i32,
    /// 结束行号(1-based)
    pub end_line: i32,
    /// 内容哈希
    pub hash: String,
}

/// 文本分块器
pub struct Chunker {
    config: ChunkConfig,
}

impl Chunker {
    pub fn new(config: ChunkConfig) -> Self {
        Self { config }
    }

    /// 对Markdown内容进行分块
    pub fn chunk_markdown(&self, content: &str) -> Result<Vec<TextChunk>> {
        let lines: Vec<&str> = content.lines().collect();

        // 跳过frontmatter
        let body_start = self.skip_frontmatter(&lines);
        let body_lines = &lines[body_start..];

        if body_lines.is_empty() {
            return Ok(Vec::new());
        }

        // 滑动窗口分块
        let mut chunks = Vec::new();
        let mut current_start = 0;

        while current_start < body_lines.len() {
            // 收集到目标token数
            let (chunk_text, chunk_end) = self.collect_chunk(body_lines, current_start)?;

            // 计算实际行号(加上frontmatter偏移)
            let start_line = (body_start + current_start + 1) as i32;
            let end_line = (body_start + chunk_end) as i32;

            // 生成哈希
            let hash = hash_text(&chunk_text);

            chunks.push(TextChunk {
                text: chunk_text,
                start_line,
                end_line,
                hash,
            });

            // 下一个块的起始位置（考虑overlap）
            let overlap_lines = self.calculate_overlap_lines(body_lines, current_start, chunk_end);
            current_start = chunk_end.saturating_sub(overlap_lines);

            // 如果没有前进，强制移动避免死循环
            if current_start == chunk_end.saturating_sub(overlap_lines) && current_start < body_lines.len() {
                current_start = chunk_end;
            }

            // 已到达末尾
            if chunk_end >= body_lines.len() {
                break;
            }
        }

        Ok(chunks)
    }

    /// 跳过YAML frontmatter，返回正文起始行号
    fn skip_frontmatter(&self, lines: &[&str]) -> usize {
        if lines.is_empty() || lines[0] != "---" {
            return 0;
        }

        // 查找第二个 "---"
        for (i, line) in lines.iter().enumerate().skip(1) {
            if *line == "---" {
                return i + 1; // 返回frontmatter之后的行号
            }
        }

        // 没有找到结束标记，认为整个文件都是frontmatter（异常情况）
        0
    }

    /// 从起始位置收集一个块，返回(文本, 结束行索引)
    fn collect_chunk(&self, lines: &[&str], start: usize) -> Result<(String, usize)> {
        let mut accumulated_text = String::new();
        let mut current_tokens = 0;
        let mut end = start;

        for (i, line) in lines.iter().enumerate().skip(start) {
            let line_tokens = estimate_tokens(line);

            // 如果加上这行超过目标，且已有足够内容
            if current_tokens + line_tokens > self.config.target_tokens
                && current_tokens >= self.config.min_tokens {
                break;
            }

            accumulated_text.push_str(line);
            accumulated_text.push('\n');
            current_tokens += line_tokens;
            end = i + 1;

            // 达到目标token数
            if current_tokens >= self.config.target_tokens {
                break;
            }
        }

        Ok((accumulated_text.trim().to_string(), end))
    }

    /// 计算overlap行数
    fn calculate_overlap_lines(&self, lines: &[&str], start: usize, end: usize) -> usize {
        let mut overlap_tokens = 0;
        let mut overlap_lines = 0;

        // 从结束位置向前扫描
        for i in (start..end).rev() {
            let line_tokens = estimate_tokens(lines[i]);

            if overlap_tokens + line_tokens > self.config.overlap_tokens {
                break;
            }

            overlap_tokens += line_tokens;
            overlap_lines += 1;
        }

        overlap_lines
    }
}

/// 估算文本的token数（简化版）
/// CJK字符: 1字符 ≈ 1 token
/// ASCII词: 1词 ≈ 1 token
pub fn estimate_tokens(text: &str) -> usize {
    let mut tokens = 0;
    let mut in_word = false;

    for ch in text.chars() {
        if is_cjk(ch) {
            // CJK字符，每个算1 token
            tokens += 1;
            in_word = false;
        } else if ch.is_alphanumeric() {
            // ASCII字母数字，按单词计数
            if !in_word {
                tokens += 1;
                in_word = true;
            }
        } else {
            // 空白或标点，结束当前单词
            in_word = false;
        }
    }

    tokens
}

/// 判断是否为CJK字符
fn is_cjk(ch: char) -> bool {
    matches!(ch,
        '\u{4E00}'..='\u{9FFF}' |  // CJK统一表意文字
        '\u{3400}'..='\u{4DBF}' |  // CJK扩展A
        '\u{20000}'..='\u{2A6DF}' | // CJK扩展B
        '\u{2A700}'..='\u{2B73F}' | // CJK扩展C
        '\u{2B740}'..='\u{2B81F}' | // CJK扩展D
        '\u{2B820}'..='\u{2CEAF}' | // CJK扩展E
        '\u{F900}'..='\u{FAFF}' |   // CJK兼容表意文字
        '\u{2F800}'..='\u{2FA1F}'   // CJK兼容扩展
    )
}

/// 计算文本的SHA-256哈希（取前16字符）
pub fn hash_text(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)[..16].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_frontmatter() {
        let chunker = Chunker::new(ChunkConfig::default());

        let content = "---\nid: test\n---\nContent here";
        let lines: Vec<&str> = content.lines().collect();
        let start = chunker.skip_frontmatter(&lines);
        assert_eq!(start, 3);
    }

    #[test]
    fn test_skip_frontmatter_no_frontmatter() {
        let chunker = Chunker::new(ChunkConfig::default());

        let content = "Direct content\nNo frontmatter";
        let lines: Vec<&str> = content.lines().collect();
        let start = chunker.skip_frontmatter(&lines);
        assert_eq!(start, 0);
    }

    #[test]
    fn test_estimate_tokens_cjk() {
        assert_eq!(estimate_tokens("你好世界"), 4);
        assert_eq!(estimate_tokens("��文测试文本"), 5);
    }

    #[test]
    fn test_estimate_tokens_ascii() {
        assert_eq!(estimate_tokens("hello world"), 2);
        assert_eq!(estimate_tokens("test multiple words here"), 4);
    }

    #[test]
    fn test_estimate_tokens_mixed() {
        assert_eq!(estimate_tokens("你好 world 测试"), 5); // 你好(2) + world(1) + 测试(2)
    }

    #[test]
    fn test_hash_text() {
        let hash1 = hash_text("test content");
        let hash2 = hash_text("test content");
        let hash3 = hash_text("different content");

        assert_eq!(hash1, hash2); // 相同内容哈希相同
        assert_ne!(hash1, hash3); // 不同内容哈希不同
        assert_eq!(hash1.len(), 16); // 取前16字符
    }

    #[test]
    fn test_chunk_markdown_simple() {
        let chunker = Chunker::new(ChunkConfig {
            target_tokens: 10,
            overlap_tokens: 2,
            min_tokens: 5,
        });

        let content = "---\nid: test\n---\n这是第一段内容。\n这是第二段内容。\n这是第三段内容。";

        let chunks = chunker.chunk_markdown(content).unwrap();
        assert!(!chunks.is_empty());

        // 验证起始行号正确（跳过了frontmatter）
        assert!(chunks[0].start_line >= 4);
    }

    #[test]
    fn test_chunk_markdown_overlap() {
        let chunker = Chunker::new(ChunkConfig {
            target_tokens: 15,
            overlap_tokens: 5,
            min_tokens: 5,
        });

        let content = "Line one has some content.\nLine two has more content.\nLine three continues.\nLine four ends.";

        let chunks = chunker.chunk_markdown(content).unwrap();

        // 验证至少生成了chunks
        assert!(!chunks.is_empty());

        // 如果有多个块，验证行号递增
        if chunks.len() > 1 {
            assert!(chunks[1].start_line >= chunks[0].start_line);
        }
    }

    #[test]
    fn test_chunk_empty_content() {
        let chunker = Chunker::new(ChunkConfig::default());
        let chunks = chunker.chunk_markdown("").unwrap();
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_chunk_only_frontmatter() {
        let chunker = Chunker::new(ChunkConfig::default());
        let content = "---\nid: test\ntitle: test\n---";
        let chunks = chunker.chunk_markdown(content).unwrap();
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_is_cjk() {
        assert!(is_cjk('中'));
        assert!(is_cjk('文'));
        assert!(is_cjk('测'));
        assert!(!is_cjk('a'));
        assert!(!is_cjk('1'));
        assert!(!is_cjk(' '));
    }

    #[test]
    fn test_collect_chunk_respects_min_tokens() {
        let chunker = Chunker::new(ChunkConfig {
            target_tokens: 10,
            overlap_tokens: 2,
            min_tokens: 8,
        });

        let lines = vec!["短行", "另一短行", "第三行", "第四行"];
        let (text, end) = chunker.collect_chunk(&lines, 0).unwrap();

        // 应该收集足够的行以满足min_tokens
        assert!(!text.is_empty());
        assert!(end > 1);
    }
}
