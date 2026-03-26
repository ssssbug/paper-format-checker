use log::info;
use serde::{Deserialize, Serialize};
use std::path::Path;

mod docx_parser;
mod pdf_parser;
mod llm;

use docx_parser::parse_docx;
use pdf_parser::parse_pdf;
use llm::{
    check_format_with_llm, LlmConfig, FormatCheckRequest,
    build_metadata, StyleInfo
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedDocument {
    pub content: String,
    pub paragraphs: Vec<ParsedParagraph>,
    pub metadata: DocumentMetadata,
    pub format_info: FormatInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedParagraph {
    pub index: usize,
    pub text: String,
    pub style_name: Option<String>,
    pub font_name: Option<String>,
    pub font_size: Option<f32>,
    pub line_spacing: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub page_count: usize,
    pub word_count: usize,
    pub title: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatInfo {
    pub file_type: String,
    pub styles: Vec<StyleSummary>,
    pub fonts: Vec<String>,
    pub has_toc: bool,
    pub has_bibliography: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleSummary {
    pub name: String,
    pub font_name: Option<String>,
    pub font_size: Option<f32>,
    pub is_bold: bool,
    pub alignment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatIssue {
    pub issue_type: String,
    pub description: String,
    pub location: IssueLocation,
    pub severity: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueLocation {
    pub page: Option<usize>,
    pub paragraph: Option<usize>,
    pub section: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatCheckResult {
    pub issues: Vec<FormatIssue>,
    pub summary: FormatSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatSummary {
    pub total_issues: usize,
    pub critical: usize,
    pub major: usize,
    pub minor: usize,
    pub overall_assessment: String,
}

// State to store LLM config
pub struct AppState {
    pub llm_config: std::sync::Mutex<LlmConfig>,
}

#[tauri::command]
async fn parse_document(file_path: String) -> Result<ParsedDocument, String> {
    info!("Parsing document: {}", file_path);

    let path = Path::new(&file_path);
    if !path.exists() {
        return Err(format!("文件不存在: {}", file_path));
    }

    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match extension.as_str() {
        "docx" => {
            let doc = parse_docx(&file_path).map_err(|e| e.to_string())?;

            let content = doc.paragraphs.iter()
                .map(|p| p.text.clone())
                .collect::<Vec<_>>()
                .join("\n");

            let paragraphs: Vec<ParsedParagraph> = doc.paragraphs.iter().map(|p| {
                let first_run = p.runs.first();
                ParsedParagraph {
                    index: p.index,
                    text: p.text.clone(),
                    style_name: p.style_name.clone(),
                    font_name: first_run.and_then(|r| r.font_name.clone()),
                    font_size: first_run.and_then(|r| r.font_size),
                    line_spacing: p.formatting.line_spacing,
                }
            }).collect();

            let styles: Vec<StyleSummary> = doc.styles.iter().map(|s| {
                StyleSummary {
                    name: s.name.clone().unwrap_or_else(|| s.id.clone()),
                    font_name: s.run_formatting.as_ref().and_then(|r| r.font_name.clone()),
                    font_size: s.run_formatting.as_ref().and_then(|r| r.font_size),
                    is_bold: s.run_formatting.as_ref().map(|r| r.is_bold).unwrap_or(false),
                    alignment: s.paragraph_formatting.as_ref().and_then(|p| p.alignment.clone()),
                }
            }).collect();

            Ok(ParsedDocument {
                content,
                paragraphs,
                metadata: DocumentMetadata {
                    page_count: doc.properties.page_count.unwrap_or(1),
                    word_count: doc.properties.word_count.unwrap_or(0),
                    title: doc.properties.title,
                    author: doc.properties.author,
                },
                format_info: FormatInfo {
                    file_type: "docx".to_string(),
                    styles,
                    fonts: vec![],
                    has_toc: false,
                    has_bibliography: false,
                },
            })
        }
        "pdf" => {
            let doc = parse_pdf(&file_path).map_err(|e| e.to_string())?;

            let content = doc.pages.iter()
                .map(|p| p.text.clone())
                .collect::<Vec<_>>()
                .join("\n");

            // Create paragraphs from PDF (roughly one per line)
            let paragraphs: Vec<ParsedParagraph> = doc.pages.iter()
                .flat_map(|page| {
                    page.text.lines().enumerate().map(|(i, line)| {
                        ParsedParagraph {
                            index: i,
                            text: line.to_string(),
                            style_name: None,
                            font_name: None,
                            font_size: None,
                            line_spacing: None,
                        }
                    }).collect::<Vec<_>>()
                })
                .collect();

            let fonts: Vec<String> = doc.fonts.iter()
                .filter_map(|f| f.base_font.clone())
                .collect();

            Ok(ParsedDocument {
                content,
                paragraphs,
                metadata: DocumentMetadata {
                    page_count: doc.metadata.page_count,
                    word_count: doc.metadata.word_count,
                    title: doc.metadata.title,
                    author: doc.metadata.author,
                },
                format_info: FormatInfo {
                    file_type: "pdf".to_string(),
                    styles: vec![],
                    fonts,
                    has_toc: false,
                    has_bibliography: false,
                },
            })
        }
        "txt" => {
            let content = std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
            let word_count = content.split_whitespace().count();

            Ok(ParsedDocument {
                content: content.clone(),
                paragraphs: content.lines().enumerate().map(|(i, l)| {
                    ParsedParagraph {
                        index: i,
                        text: l.to_string(),
                        style_name: None,
                        font_name: None,
                        font_size: None,
                        line_spacing: None,
                    }
                }).collect(),
                metadata: DocumentMetadata {
                    page_count: 1,
                    word_count,
                    title: None,
                    author: None,
                },
                format_info: FormatInfo {
                    file_type: "txt".to_string(),
                    styles: vec![],
                    fonts: vec![],
                    has_toc: false,
                    has_bibliography: false,
                },
            })
        }
        _ => Err(format!("不支持的文件格式: {}", extension)),
    }
}

#[tauri::command]
async fn check_format(
    format_requirements: String,
    state: tauri::State<'_, AppState>,
) -> Result<FormatCheckResult, String> {
    info!("Starting format check...");

    // Get LLM config from state - clone it to avoid holding mutex across await
    let llm_config = {
        let config = state.llm_config.lock().map_err(|e| e.to_string())?;
        config.clone()
    };

    // For now, create a simple request with the format requirements
    let request = FormatCheckRequest {
        format_requirements,
        document_content: String::new(),
        document_metadata: llm::DocumentAnalysisMetadata::default(),
    };

    // Call LLM API
    let response = check_format_with_llm(&llm_config, &request).await?;

    // Convert to our format
    let issues: Vec<FormatIssue> = response.issues.into_iter().map(|i| {
        FormatIssue {
            issue_type: i.issue_type,
            description: i.description,
            location: IssueLocation {
                page: i.location.page,
                paragraph: i.location.paragraph,
                section: i.location.section,
            },
            severity: i.severity,
            suggestion: i.suggestion,
        }
    }).collect();

    let summary = FormatSummary {
        total_issues: response.summary.total_issues,
        critical: response.summary.critical,
        major: response.summary.major,
        minor: response.summary.minor,
        overall_assessment: response.summary.overall_assessment,
    };

    Ok(FormatCheckResult { issues, summary })
}

#[tauri::command]
async fn check_format_with_document(
    format_requirements: String,
    parsed_document: ParsedDocument,
    state: tauri::State<'_, AppState>,
) -> Result<FormatCheckResult, String> {
    info!("Checking format with document content...");

    // Get LLM config - clone to avoid holding mutex across await
    let llm_config = {
        let config = state.llm_config.lock().map_err(|e| e.to_string())?;
        config.clone()
    };

    // Build styles for metadata
    let styles: Vec<StyleInfo> = parsed_document.format_info.styles.iter().map(|s| {
        StyleInfo {
            name: s.name.clone(),
            font_name: s.font_name.clone(),
            font_size: s.font_size,
            is_bold: s.is_bold,
            alignment: s.alignment.clone(),
            line_spacing: None,
        }
    }).collect();

    // Build metadata
    let metadata = build_metadata(
        &parsed_document.format_info.file_type,
        parsed_document.metadata.page_count,
        parsed_document.metadata.word_count,
        parsed_document.paragraphs.len(),
        styles,
        parsed_document.format_info.fonts.clone(),
    );

    // Build request
    let request = FormatCheckRequest {
        format_requirements,
        document_content: parsed_document.content,
        document_metadata: metadata,
    };

    // Call LLM
    let response = check_format_with_llm(&llm_config, &request).await?;

    // Convert response
    let issues: Vec<FormatIssue> = response.issues.into_iter().map(|i| {
        FormatIssue {
            issue_type: i.issue_type,
            description: i.description,
            location: IssueLocation {
                page: i.location.page,
                paragraph: i.location.paragraph,
                section: i.location.section,
            },
            severity: i.severity,
            suggestion: i.suggestion,
        }
    }).collect();

    let summary = FormatSummary {
        total_issues: response.summary.total_issues,
        critical: response.summary.critical,
        major: response.summary.major,
        minor: response.summary.minor,
        overall_assessment: response.summary.overall_assessment,
    };

    Ok(FormatCheckResult { issues, summary })
}

#[tauri::command]
async fn set_llm_config(
    provider: String,
    api_key: String,
    model: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    info!("Setting LLM config for provider: {}", provider);

    let mut config = state.llm_config.lock().map_err(|e| e.to_string())?;
    config.provider = provider.clone();
    config.api_key = api_key;
    config.model = model;

    // Set correct base_url based on provider
    match provider.as_str() {
        "openai" => {
            config.base_url = "https://api.openai.com/v1".to_string();
        }
        "minimax" => {
            config.base_url = "https://api.minimax.chat/v1".to_string();
        }
        _ => {}
    }

    info!("LLM base_url updated to: {}", config.base_url);

    Ok(())
}

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("Starting PaperFormatChecker...");

    let app_state = AppState {
        llm_config: std::sync::Mutex::new(LlmConfig::default()),
    };

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            parse_document,
            check_format,
            check_format_with_document,
            set_llm_config,
            get_app_version
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}