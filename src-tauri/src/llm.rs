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
            model: "abab6.5s-chat".to_string(),
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

    prompt.push_str("你是论文格式检查专家。请根据用户提供的格式要求和论文元数据，分析并列出所有格式问题。\n\n");

    prompt.push_str("## 格式要求\n");
    prompt.push_str(&request.format_requirements);
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

    prompt.push_str("\n## 样式列表\n");
    for style in &request.document_metadata.styles_used {
        prompt.push_str(&format!("- {}: ", style.name));
        let mut attrs = Vec::new();
        if let Some(font) = &style.font_name {
            attrs.push(format!("字体: {}", font));
        }
        if let Some(size) = style.font_size {
            attrs.push(format!("{}pt", size));
        }
        if style.is_bold {
            attrs.push("加粗".to_string());
        }
        if let Some(align) = &style.alignment {
            attrs.push(format!("对齐: {}", align));
        }
        if attrs.is_empty() {
            prompt.push_str("(默认样式)\n");
        } else {
            prompt.push_str(&attrs.join(", "));
            prompt.push_str("\n");
        }
    }

    prompt.push_str("\n## 论文内容预览（前2000字）\n");
    let content_preview = &request.document_content[..request.document_content.len().min(2000)];
    prompt.push_str(content_preview);
    if request.document_content.len() > 2000 {
        prompt.push_str("\n...(内容已截断)");
    }

    prompt.push_str("\n\n## 输出要求\n");
    prompt.push_str("请以JSON格式返回检查结果，格式如下：\n");
    prompt.push_str(r#"{
  "issues": [
    {
      "issue_type": "字体",
      "description": "问题描述",
      "location": {"page": 1, "paragraph": 5, "section": "第一章"},
      "severity": "major",
      "suggestion": "修复建议"
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
    prompt.push_str("\n\n请直接返回JSON，不要有其他文字。");

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