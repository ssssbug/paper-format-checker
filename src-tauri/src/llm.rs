// LLM API integration for format checking

use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::collections::HashMap;

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

    prompt.push_str("你是论文格式检查专家。请根据用户提供的格式要求和论文内容，详细检查所有格式问题，包括但不限于：字体、字号、行距、页边距、对齐方式、段落缩进、页眉页脚、图表格式、参考文献格式等。\n\n");

    // 完整的格式要求
    prompt.push_str("## 格式要求（完整内容）\n");
    prompt.push_str(&request.format_requirements);
    prompt.push_str("\n\n");

    // 完整的论文内容
    prompt.push_str("## 论文内容（完整）\n");
    prompt.push_str(&request.document_content);
    prompt.push_str("\n\n");

    prompt.push_str("## 论文元数据\n");
    prompt.push_str(&format!("- 文件类型: {}\n", request.document_metadata.file_type));
    prompt.push_str(&format!("- 页数: {}\n", request.document_metadata.page_count));
    prompt.push_str(&format!("- 字数: {}\n", request.document_metadata.word_count));
    prompt.push_str(&format!("- 段落数: {}\n", request.document_metadata.paragraph_count));

    if !request.document_metadata.fonts_detected.is_empty() {
        prompt.push_str("- 检测到的字体: ");
        prompt.push_str(&request.document_metadata.fonts_detected.join(", "));
        prompt.push_str("\n");
    }

    if let Some(spacing) = request.document_metadata.average_line_spacing {
        prompt.push_str(&format!("- 平均行距: {:.1} 倍\n", spacing));
    }

    prompt.push_str("\n## 检测要求\n");
    prompt.push_str("1. 仔细对比格式要求和论文内容，找出所有不符合的地方\n");
    prompt.push_str("2. 对于每种格式要求（字体、字号、行距、页边距等），都要逐一检查\n");
    prompt.push_str("3. 如果格式要求中明确规定了某项格式，论文中未满足的都要报告\n");
    prompt.push_str("4. 检查页面设置（页边距、页眉页脚、页码等）\n");
    prompt.push_str("5. 检查标题层次是否正确\n");
    prompt.push_str("6. 检查参考文献格式是否符合要求\n\n");

    prompt.push_str("## 输出要求\n");
    prompt.push_str("请以JSON格式返回检查结果，必须包含所有发现的问题：\n");
    prompt.push_str(r#"{
  "issues": [
    {
      "issue_type": "字体/字号/行距/页边距/对齐/缩进/页眉页脚/标题层次/参考文献/其他",
      "description": "具体问题描述，明确指出哪里不符合要求",
      "location": {"page": 1, "paragraph": 5, "section": "具体章节"},
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