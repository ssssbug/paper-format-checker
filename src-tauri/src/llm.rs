// LLM API integration for format checking and guideline parsing

use serde::{Deserialize, Serialize};
use serde_json::Value;
use reqwest::Client;

/// Format check request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatCheckRequest {
    pub format_requirements: String,
    pub document_content: String,
    pub document_metadata: DocumentAnalysisMetadata,
}

/// Document metadata extracted from parsing
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentAnalysisMetadata {
    pub file_type: String,  // "docx" or "pdf"
    pub page_count: usize,
    pub word_count: usize,
    pub styles_used: Vec<StyleInfo>,
    pub paragraph_count: usize,
    pub has_toc: bool,
    pub has_bibliography: bool,
    pub fonts_detected: Vec<String>,
    pub average_line_spacing: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleInfo {
    pub name: String,
    pub font_name: Option<String>,
    pub font_size: Option<f32>,
    pub is_bold: bool,
    pub alignment: Option<String>,
    pub line_spacing: Option<f32>,
}

/// LLM API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatCheckResponse {
    pub issues: Vec<FormatIssue>,
    pub summary: CheckSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatIssue {
    pub issue_type: String,
    pub description: String,
    pub location: IssueLocation,
    pub severity: String,  // "critical", "major", "minor"
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueLocation {
    pub page: Option<usize>,
    pub paragraph: Option<usize>,
    pub section: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckSummary {
    pub total_issues: usize,
    pub critical: usize,
    pub major: usize,
    pub minor: usize,
    pub overall_assessment: String,
}

/// LLM API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,  // "minimax" or "openai"
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: "minimax".to_string(),
            api_key: String::new(),
            model: "MiniMax-Text-01".to_string(),
            base_url: "https://api.minimax.chat/v1".to_string(),
        }
    }
}

/// Check format using LLM API
pub async fn check_format_with_llm(
    config: &LlmConfig,
    request: &FormatCheckRequest,
) -> Result<FormatCheckResponse, String> {
    let client = Client::new();

    // Build prompt
    let prompt = build_format_check_prompt(request);

    // Call API based on provider
    match config.provider.as_str() {
        "minimax" => call_minimax_api(&client, config, &prompt).await,
        "openai" => call_openai_api(&client, config, &prompt).await,
        _ => Err(format!("Unknown provider: {}", config.provider)),
    }
}

fn build_format_check_prompt(request: &FormatCheckRequest) -> String {
    let mut prompt = String::new();

    prompt.push_str("你是论文格式检查专家。请根据用户提供的格式要求和论文格式信息，检查所有格式是否符合要求。\n\n");

    // 完整的格式要求
    prompt.push_str("## 论文格式要求（请严格遵守）\n");
    prompt.push_str(&request.format_requirements);
    prompt.push_str("\n\n");

    // 论文格式信息
    prompt.push_str("## 论文格式信息\n");
    prompt.push_str(&format!("- 总页数: {}\n", request.document_metadata.page_count));
    prompt.push_str(&format!("- 总字数: {}\n", request.document_metadata.word_count));
    prompt.push_str(&format!("- 总段落数: {}\n", request.document_metadata.paragraph_count));

    // 检测到的字体
    if !request.document_metadata.fonts_detected.is_empty() {
        prompt.push_str("\n### 使用的字体\n");
        for font in &request.document_metadata.fonts_detected {
            prompt.push_str(&format!("- {}\n", font));
        }
    }

    // 样式统计
    if !request.document_metadata.styles_used.is_empty() {
        prompt.push_str(&format!("\n### 样式统计（共 {} 种样式）\n", request.document_metadata.styles_used.len()));
        for style in &request.document_metadata.styles_used {
            prompt.push_str(&format!("- {}: ", style.name));
            let mut attrs = Vec::new();
            if let Some(font) = &style.font_name {
                attrs.push(format!("字体: {}", font));
            }
            if let Some(size) = style.font_size {
                attrs.push(format!("字号: {}pt", size));
            }
            if style.is_bold {
                attrs.push("加粗".to_string());
            }
            if let Some(align) = &style.alignment {
                attrs.push(format!("对齐: {}", align));
            }
            if let Some(spacing) = style.line_spacing {
                attrs.push(format!("行距: {:.1}倍", spacing));
            }
            if attrs.is_empty() {
                prompt.push_str("(默认样式)\n");
            } else {
                prompt.push_str(&attrs.join(", "));
                prompt.push_str("\n");
            }
        }
    }

    // 行距信息
    if let Some(avg_spacing) = request.document_metadata.average_line_spacing {
        prompt.push_str(&format!("\n### 行距\n- 平均行距: {:.1} 倍\n", avg_spacing));
    }

    // 章节检测
    prompt.push_str("\n### 章节结构\n");
    if request.document_metadata.has_toc {
        prompt.push_str("- 有目录\n");
    } else {
        prompt.push_str("- 无目录\n");
    }
    if request.document_metadata.has_bibliography {
        prompt.push_str("- 有参考文献\n");
    } else {
        prompt.push_str("- 无参考文献\n");
    }

    prompt.push_str("\n## 检测要求\n");
    prompt.push_str("请仔细对比\"论文格式要求\"和\"论文格式信息\"，检查以下方面：\n");
    prompt.push_str("1. 字体：要求使用什么字体，论文实际使用了什么字体\n");
    prompt.push_str("2. 字号：各级标题、正文的字号是否正确\n");
    prompt.push_str("3. 行距：行距设置是否符合要求\n");
    prompt.push_str("4. 页边距：检查页面设置\n");
    prompt.push_str("5. 对齐方式：段落对齐是否正确\n");
    prompt.push_str("6. 缩进：段落缩进是否正确\n");
    prompt.push_str("7. 标题层次：标题层级是否规范\n");
    prompt.push_str("8. 页眉页脚：是否有且格式正确\n");
    prompt.push_str("9. 参考文献：格式是否符合要求\n");
    prompt.push_str("\n## 输出要求\n");
    prompt.push_str("请以JSON格式返回检查结果，必须包含所有发现的问题：\n");
    prompt.push_str(r#"{
  "issues": [
    {
      "issue_type": "字体/字号/行距/页边距/对齐/缩进/标题层次/页眉页脚/参考文献/其他",
      "description": "具体问题描述，明确指出哪里不符合要求",
      "location": {"page": null, "paragraph": null, "section": "具体位置"},
      "severity": "critical/major/minor",
      "suggestion": "具体修复建议"
    }
  ],
  "summary": {
    "total_issues": 5,
    "critical": 1,
    "major": 3,
    "minor": 1,
    "overall_assessment": "整体评估"
  }
}"#);
    prompt.push_str("\n\n请直接返回JSON，不要有其他文字。确保返回所有发现的问题，不要遗漏。");

    prompt
}

async fn call_minimax_api(
    client: &Client,
    config: &LlmConfig,
    prompt: &str,
) -> Result<FormatCheckResponse, String> {
    let response = client
        .post(format!("{}/text/chatcompletion_v2", config.base_url))
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": config.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.3
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let response_text = response.text().await.map_err(|e| e.to_string())?;

    // Parse response
    let json: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse API response: {} - {}", e, response_text))?;

    // Extract content from response
    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No content in response")?;

    // Parse the JSON in the content
    parse_llm_response(content)
}

async fn call_openai_api(
    client: &Client,
    config: &LlmConfig,
    prompt: &str,
) -> Result<FormatCheckResponse, String> {
    let response = client
        .post(format!("{}/chat/completions", config.base_url))
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": config.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.3
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let response_text = response.text().await.map_err(|e| e.to_string())?;

    let json: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse API response: {} - {}", e, response_text))?;

    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No content in response")?;

    parse_llm_response(content)
}

fn parse_llm_response(content: &str) -> Result<FormatCheckResponse, String> {
    // Try to extract JSON from the response
    // The response might contain markdown code blocks
    let json_str = if content.contains("```json") {
        content
            .split("```json")
            .nth(1)
            .unwrap_or(content)
            .split("```")
            .next()
            .unwrap_or(content)
            .trim()
    } else if content.contains("```") {
        content
            .split("```")
            .nth(1)
            .unwrap_or(content)
            .trim()
    } else {
        content.trim()
    };

    // Try to find JSON in the string
    let json_str = if let Some(start) = json_str.find('{') {
        if let Some(end) = json_str.rfind('}') {
            &json_str[start..=end]
        } else {
            json_str
        }
    } else {
        json_str
    };

    serde_json::from_str(json_str).map_err(|e| format!("Failed to parse LLM response: {}", e))
}

/// Build metadata from parsed document
pub fn build_metadata(
    file_type: &str,
    page_count: usize,
    word_count: usize,
    paragraphs: usize,
    styles: Vec<StyleInfo>,
    fonts: Vec<String>,
) -> DocumentAnalysisMetadata {
    let mut metadata = DocumentAnalysisMetadata {
        file_type: file_type.to_string(),
        page_count,
        word_count,
        paragraph_count: paragraphs,
        styles_used: styles,
        fonts_detected: fonts,
        ..Default::default()
    };

    // Calculate average line spacing from styles
    let spacings: Vec<f32> = metadata.styles_used.iter()
        .filter_map(|s| s.line_spacing)
        .collect();

    if !spacings.is_empty() {
        let sum: f32 = spacings.iter().sum();
        metadata.average_line_spacing = Some(sum / spacings.len() as f32);
    }

    // Check for common sections
    metadata.has_toc = false; // Would need more detailed parsing
    metadata.has_bibliography = false;

    metadata
}

// ============================================================================
// Guideline Parsing Module
// Extracts structured format requirements from paper writing guidelines
// ============================================================================

/// Structured format requirements extracted from guidelines
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GuidelineFormatRequirements {
    pub font: Option<FontRequirement>,
    pub spacing: Option<SpacingRequirement>,
    pub margins: Option<MarginRequirement>,
    pub citations: Option<CitationRequirement>,
    pub headings: Option<HeadingRequirement>,
    pub abstract_spec: Option<AbstractRequirement>,
    pub references: Option<ReferencesRequirement>,
    pub figures: Option<FigureRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontRequirement {
    pub font_family: Option<String>,
    pub font_family_alternative: Option<String>,
    pub font_size_body: Option<f32>,
    pub font_size_title: Option<f32>,
    pub font_size_heading1: Option<f32>,
    pub font_size_heading2: Option<f32>,
    pub font_size_heading3: Option<f32>,
    pub font_style_body: Option<String>, // "normal", "italic", etc.
    pub bold_required: bool,
    pub confidence: String, // "high", "medium", "low"
    pub source_snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacingRequirement {
    pub line_spacing: Option<f32>,
    pub line_spacing_unit: Option<String>, // "倍", "pt", "cm"
    pub paragraph_spacing_before: Option<f32>,
    pub paragraph_spacing_after: Option<f32>,
    pub paragraph_indent: Option<f32>,
    pub paragraph_indent_unit: Option<String>,
    pub confidence: String,
    pub source_snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginRequirement {
    pub top: Option<f32>,
    pub bottom: Option<f32>,
    pub left: Option<f32>,
    pub right: Option<f32>,
    pub unit: Option<String>, // "cm", "inch", "in"
    pub header_position: Option<f32>,
    pub footer_position: Option<f32>,
    pub confidence: String,
    pub source_snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationRequirement {
    pub citation_style: Option<String>, // "APA", "MLA", "Chicago", "IEEE", "Harvard"
    pub in_text_format: Option<String>,
    pub reference_format: Option<String>,
    pub reference_numbering: Option<String>, // "alphabetical", "numerical", "order of appearance"
    pub confidence: String,
    pub source_snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadingRequirement {
    pub heading_level1_numbering: Option<String>,
    pub heading_level2_numbering: Option<String>,
    pub heading_level3_numbering: Option<String>,
    pub heading_level1_format: Option<String>,
    pub heading_level2_format: Option<String>,
    pub heading_level3_format: Option<String>,
    pub heading_style: Option<String>, // "居中", "左对齐", "加粗"
    pub confidence: String,
    pub source_snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbstractRequirement {
    pub abstract_keyword: Option<String>, // "摘要" / "Abstract"
    pub abstract_length: Option<String>, // e.g., "200-300字"
    pub abstract_font_size: Option<f32>,
    pub abstract_alignment: Option<String>,
    pub abstract_keywords_required: bool,
    pub abstract_keywords_label: Option<String>,
    pub confidence: String,
    pub source_snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferencesRequirement {
    pub references_title: Option<String>, // "参考文献" / "References"
    pub references_placement: Option<String>, // "文末", "章末"
    pub references_ordering: Option<String>, // "字母顺序", "引用顺序"
    pub references_format: Option<String>,
    pub references_indentation: Option<String>, // "悬挂缩进"
    pub confidence: String,
    pub source_snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FigureRequirement {
    pub figure_caption_position: Option<String>, // "图下方" / "图上方"
    pub figure_caption_format: Option<String>,
    pub table_caption_position: Option<String>,
    pub table_caption_format: Option<String>,
    pub confidence: String,
    pub source_snippet: Option<String>,
}

/// Parsing result with detailed provenance and ambiguity flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedGuidelineResult {
    pub requirements: GuidelineFormatRequirements,
    pub raw_json: String,
    pub parsing_errors: Vec<String>,
    pub ambiguous_items: Vec<AmbiguousItem>,
    pub unparsed_segments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbiguousItem {
    pub field_name: String,
    pub raw_text: String,
    pub interpretation: String,
    pub confidence: String,
    pub suggestion: String,
}

/// Request for parsing guidelines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuidelineParseRequest {
    pub guideline_text: String,
    pub guideline_source: Option<String>, // e.g., "学校官网", "期刊要求"
    pub expected_style: Option<String>, // e.g., "APA", "GB/T 7714"
}

/// Parse paper writing guidelines into structured format requirements
pub async fn parse_guideline_with_llm(
    config: &LlmConfig,
    request: &GuidelineParseRequest,
) -> Result<ParsedGuidelineResult, String> {
    let client = Client::new();
    let prompt = build_guideline_parsing_prompt(request);

    // Call API based on provider
    let response_text = match config.provider.as_str() {
        "minimax" => call_llm_api_raw(&client, config, &prompt).await,
        "openai" => call_llm_api_raw(&client, config, &prompt).await,
        _ => Err(format!("Unknown provider: {}", config.provider)),
    }?;

    // Parse the JSON response
    parse_guideline_response(&response_text)
}

fn build_guideline_parsing_prompt(request: &GuidelineParseRequest) -> String {
    let mut prompt = String::new();

    prompt.push_str("你是论文格式规范解析专家。请从以下论文撰写规范中提取所有格式要求，返回结构化的JSON数据。\n\n");

    if let Some(source) = &request.guideline_source {
        prompt.push_str(&format!("## 规范来源\n{}\n\n", source));
    }

    if let Some(style) = &request.expected_style {
        prompt.push_str(&format!("## 预期格式标准\n{}\n\n", style));
    }

    prompt.push_str("## 论文撰写规范原文\n");
    prompt.push_str("```\n");
    prompt.push_str(&request.guideline_text);
    prompt.push_str("\n```\n\n");

    prompt.push_str(r#"## 提取要求

请仔细阅读上述规范，提取所有格式相关的要求。即使某些信息没有明确说明，也可以根据上下文推断，但要标记为"inferred"。

## 输出格式

请严格以以下JSON格式返回，不要包含任何其他文字：

{
  "font": {
    "font_family": "字体名称，如 Times New Roman, 宋体, 黑体",
    "font_family_alternative": "备选字体（如有）",
    "font_size_body": 正文字号（数字，单位pt）,
    "font_size_title": 标题字号,
    "font_size_heading1": 一级标题字号,
    "font_size_heading2": 二级标题字号,
    "font_size_heading3": 三级标题字号,
    "font_style_body": "normal/italic/加粗",
    "bold_required": true或false,
    "confidence": "high/medium/low",
    "source_snippet": "对应的原文片段"
  },
  "spacing": {
    "line_spacing": 行距数值（如1.5表示1.5倍行距）,
    "line_spacing_unit": "倍",
    "paragraph_spacing_before": 段落前间距（单位pt）,
    "paragraph_spacing_after": 段落后间距（单位pt）,
    "paragraph_indent": 段落首行缩进（如2表示2字符或2cm）,
    "paragraph_indent_unit": "字符/cm",
    "confidence": "high/medium/low",
    "source_snippet": "对应的原文片段"
  },
  "margins": {
    "top": 上边距数值,
    "bottom": 下边距数值,
    "left": 左边距数值,
    "right": 右边距数值,
    "unit": "cm/英寸/inch",
    "header_position": 页眉到顶边距离,
    "footer_position": 页脚到底边距离,
    "confidence": "high/medium/low",
    "source_snippet": "对应的原文片段"
  },
  "citations": {
    "citation_style": "APA/MLA/Chicago/IEEE/Harvard/GB/T 7714/其他",
    "in_text_format": "文中引用格式描述",
    "reference_format": "参考文献格式描述",
    "reference_numbering": "字母顺序/数字顺序/引用顺序",
    "confidence": "high/medium/low",
    "source_snippet": "对应的原文片段"
  },
  "headings": {
    "heading_level1_numbering": "一级标题编号格式，如 1, 1.1",
    "heading_level2_numbering": "二级标题编号格式",
    "heading_level3_numbering": "三级标题编号格式",
    "heading_level1_format": "一级标题样式，如 居中、加粗",
    "heading_level2_format": "二级标题样式",
    "heading_level3_format": "三级标题样式",
    "heading_style": "整体标题风格描述",
    "confidence": "high/medium/low",
    "source_snippet": "对应的原文片段"
  },
  "abstract_spec": {
    "abstract_keyword": "摘要标题，如 摘要/Abstract",
    "abstract_length": "摘要长度要求，如 200-300字",
    "abstract_font_size": 摘要字号,
    "abstract_alignment": "对齐方式",
    "abstract_keywords_required": true或false,
    "abstract_keywords_label": "关键词标题，如 关键词/Keywords",
    "confidence": "high/medium/low",
    "source_snippet": "对应的原文片段"
  },
  "references": {
    "references_title": "参考文献标题，如 参考文献/References",
    "references_placement": "位置，如 文末、章节末尾",
    "references_ordering": "排序方式，如 按字母顺序、按引用顺序",
    "references_format": "格式描述",
    "references_indentation": "缩进方式，如 悬挂缩进2字符",
    "confidence": "high/medium/low",
    "source_snippet": "对应的原文片段"
  },
  "figures": {
    "figure_caption_position": "图注位置，如 图下方、图上方",
    "figure_caption_format": "图注格式，如 图1 XXX",
    "table_caption_position": "表注位置",
    "table_caption_format": "表注格式",
    "confidence": "high/medium/low",
    "source_snippet": "对应的原文片段"
  },
  "ambiguous_items": [
    {
      "field_name": "字段名",
      "raw_text": "原始文本",
      "interpretation": "可能的解释",
      "confidence": "medium或low",
      "suggestion": "建议如何处理"
    }
  ],
  "unparsed_segments": ["无法解析的原文片段（如果有）"],
  "parsing_errors": ["解析错误说明（如果有）"]
}

## 重要说明

1. 所有字段都是可选的，如果没有找到相关信息，请使用 null
2. 如果文本使用了"标准"、"常规"等模糊表述，请标注 confidence 为 "low" 并说明是默认值
3. "source_snippet" 应截取最相关的原文片段（不超过50字）
4. 如果发现矛盾的要求（如先说12pt后说14pt），请在 ambiguous_items 中标注
5. 对于隐含的要求（如"双倍行距"隐含 line_spacing=2.0），请提取并标注

请直接返回JSON："#);

    prompt
}

async fn call_llm_api_raw(
    client: &Client,
    config: &LlmConfig,
    prompt: &str,
) -> Result<String, String> {
    let response = client
        .post(format!("{}/text/chatcompletion_v2", config.base_url))
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": config.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.3
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let response_text = response.text().await.map_err(|e| e.to_string())?;

    let json: Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse API response: {} - Raw: {}", e, response_text))?;

    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No content in response")?
        .to_string();

    Ok(content)
}

fn parse_guideline_response(response_text: &str) -> Result<ParsedGuidelineResult, String> {
    // Extract JSON from response (handle markdown code blocks)
    let json_str = extract_json(response_text);

    // Parse the JSON
    let json: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse guideline JSON: {} - Input: {}", e, json_str))?;

    let raw_json = serde_json::to_string_pretty(&json)
        .unwrap_or_else(|_| response_text.to_string());

    // Extract structured requirements
    let requirements = GuidelineFormatRequirements {
        font: parse_section::<FontRequirement>(&json, "font"),
        spacing: parse_section::<SpacingRequirement>(&json, "spacing"),
        margins: parse_section::<MarginRequirement>(&json, "margins"),
        citations: parse_section::<CitationRequirement>(&json, "citations"),
        headings: parse_section::<HeadingRequirement>(&json, "headings"),
        abstract_spec: parse_section::<AbstractRequirement>(&json, "abstract_spec"),
        references: parse_section::<ReferencesRequirement>(&json, "references"),
        figures: parse_section::<FigureRequirement>(&json, "figures"),
    };

    // Extract ambiguous items
    let ambiguous_items: Vec<AmbiguousItem> = json["ambiguous_items"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    serde_json::from_value(item.clone()).ok()
                })
                .collect()
        })
        .unwrap_or_default();

    // Extract unparsed segments
    let unparsed_segments: Vec<String> = json["unparsed_segments"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // Extract parsing errors
    let parsing_errors: Vec<String> = json["parsing_errors"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Ok(ParsedGuidelineResult {
        requirements,
        raw_json,
        parsing_errors,
        ambiguous_items,
        unparsed_segments,
    })
}

fn extract_json(content: &str) -> &str {
    let trimmed = content.trim();
    if trimmed.contains("```json") {
        trimmed
            .split("```json")
            .nth(1)
            .unwrap_or(trimmed)
            .split("```")
            .next()
            .unwrap_or(trimmed)
            .trim()
    } else if trimmed.contains("```") {
        trimmed
            .split("```")
            .nth(1)
            .unwrap_or(trimmed)
            .trim()
    } else if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            &trimmed[start..=end]
        } else {
            trimmed
        }
    } else {
        trimmed
    }
}

fn parse_section<T: for<'de> Deserialize<'de>>(json: &Value, key: &str) -> Option<T> {
    json.get(key).and_then(|v| {
        if v.is_null() || (v.is_object() && v.as_object().map(|o| o.is_empty()).unwrap_or(false)) {
            None
        } else {
            serde_json::from_value(v.clone()).ok()
        }
    })
}

/// Build a user-friendly format requirements description from parsed result
pub fn format_requirements_to_text(requirements: &GuidelineFormatRequirements) -> String {
    let mut text = String::new();

    text.push_str("## 论文格式要求\n\n");

    if let Some(font) = &requirements.font {
        text.push_str("### 字体要求\n");
        if let Some(family) = &font.font_family {
            text.push_str(&format!("- 正文字体：{}\n", family));
        }
        if let Some(size) = font.font_size_body {
            text.push_str(&format!("- 正文字号：{}pt\n", size));
        }
        if let Some(size) = font.font_size_title {
            text.push_str(&format!("- 题目字号：{}pt\n", size));
        }
        if let Some(size) = font.font_size_heading1 {
            text.push_str(&format!("- 一级标题字号：{}pt\n", size));
        }
        text.push_str("\n");
    }

    if let Some(spacing) = &requirements.spacing {
        text.push_str("### 间距要求\n");
        if let Some(ls) = spacing.line_spacing {
            text.push_str(&format!("- 行距：{} 倍\n", ls));
        }
        if let Some(indent) = spacing.paragraph_indent {
            let unit = spacing.paragraph_indent_unit.as_deref().unwrap_or("字符");
            text.push_str(&format!("- 段落缩进：{}{}\n", indent, unit));
        }
        text.push_str("\n");
    }

    if let Some(margins) = &requirements.margins {
        text.push_str("### 页边距要求\n");
        let unit = margins.unit.as_deref().unwrap_or("cm");
        if let (Some(t), Some(b), Some(l), Some(r)) = (margins.top, margins.bottom, margins.left, margins.right) {
            text.push_str(&format!("- 上：{}{}\n", t, unit));
            text.push_str(&format!("- 下：{}{}\n", b, unit));
            text.push_str(&format!("- 左：{}{}\n", l, unit));
            text.push_str(&format!("- 右：{}{}\n", r, unit));
        }
        text.push_str("\n");
    }

    if let Some(citations) = &requirements.citations {
        text.push_str("### 引用格式\n");
        if let Some(style) = &citations.citation_style {
            text.push_str(&format!("- 引用风格：{}\n", style));
        }
        if let Some(format) = &citations.in_text_format {
            text.push_str(&format!("- 文中引用：{}\n", format));
        }
        text.push_str("\n");
    }

    if let Some(abstract_spec) = &requirements.abstract_spec {
        text.push_str("### 摘要要求\n");
        if let Some(length) = &abstract_spec.abstract_length {
            text.push_str(&format!("- 摘要长度：{}\n", length));
        }
        if abstract_spec.abstract_keywords_required {
            text.push_str("- 需要关键词\n");
        }
        text.push_str("\n");
    }

    if let Some(refs) = &requirements.references {
        text.push_str("### 参考文献要求\n");
        if let Some(title) = &refs.references_title {
            text.push_str(&format!("- 参考文献标题：{}\n", title));
        }
        if let Some(order) = &refs.references_ordering {
            text.push_str(&format!("- 排序方式：{}\n", order));
        }
        text.push_str("\n");
    }

    text.trim().to_string()
}