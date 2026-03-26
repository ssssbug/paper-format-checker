// PDF document parser
// Uses lopdf to extract text content and basic information

use serde::{Deserialize, Serialize};
use std::path::Path;
use lopdf::Document;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfDocument {
    pub pages: Vec<PdfPage>,
    pub metadata: PdfMetadata,
    pub fonts: Vec<PdfFontInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfPage {
    pub page_number: usize,
    pub text: String,
    pub text_with_layout: Vec<PageTextBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageTextBlock {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub font_name: Option<String>,
    pub font_size: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PdfMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub creator: Option<String>,
    pub page_count: usize,
    pub word_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfFontInfo {
    pub name: String,
    pub subtype: Option<String>,
    pub base_font: Option<String>,
}

/// Parse a PDF file and extract text content
pub fn parse_pdf(file_path: &str) -> Result<PdfDocument, String> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    let doc = Document::load(path).map_err(|e| e.to_string())?;

    // Extract metadata
    let mut metadata = extract_metadata(&doc);

    // Extract pages
    let mut pages = Vec::new();
    let page_ids = doc.get_pages();

    for (page_num, page_id) in page_ids.iter() {
        let page_num = *page_num as usize;

        // lopdf's get_pages returns HashMap<u32, (u32, u16)>
        // We need to extract text differently
        let text = doc.extract_text(&[page_num as u32]).unwrap_or_default();

        // Simple text blocks
        let text_with_layout: Vec<PageTextBlock> = text.lines()
            .enumerate()
            .filter(|(_, line)| !line.trim().is_empty())
            .map(|(i, line)| PageTextBlock {
                text: line.to_string(),
                x: 0.0,
                y: (i as f32) * 12.0,
                width: 0.0,
                height: 12.0,
                font_name: None,
                font_size: Some(12.0),
            })
            .collect();

        let word_count = text.split_whitespace().count();
        metadata.word_count += word_count;

        pages.push(PdfPage {
            page_number: page_num,
            text,
            text_with_layout,
        });
    }

    metadata.page_count = pages.len();

    // Extract font information (simplified)
    let fonts = extract_fonts(&doc);

    Ok(PdfDocument {
        pages,
        metadata,
        fonts,
    })
}

fn extract_metadata(doc: &Document) -> PdfMetadata {
    let mut metadata = PdfMetadata::default();

    // Get page count
    let pages = doc.get_pages();
    metadata.page_count = pages.len();

    // Try to get info from document dictionary
    if let Ok(info) = doc.trailer.get(b"Info") {
        if let Ok(info_ref) = info.as_reference() {
            if let Ok(info_dict) = doc.get_dictionary(info_ref) {
                // Title
                if let Ok(title) = info_dict.get(b"Title") {
                    if let Ok(title_str) = title.as_string() {
                        metadata.title = Some(String::from_utf8_lossy(title_str.as_bytes()).to_string());
                    }
                }

                // Author
                if let Ok(author) = info_dict.get(b"Author") {
                    if let Ok(author_str) = author.as_string() {
                        metadata.author = Some(String::from_utf8_lossy(author_str.as_bytes()).to_string());
                    }
                }

                // Subject
                if let Ok(subject) = info_dict.get(b"Subject") {
                    if let Ok(subject_str) = subject.as_string() {
                        metadata.subject = Some(String::from_utf8_lossy(subject_str.as_bytes()).to_string());
                    }
                }

                // Creator
                if let Ok(creator) = info_dict.get(b"Creator") {
                    if let Ok(creator_str) = creator.as_string() {
                        metadata.creator = Some(String::from_utf8_lossy(creator_str.as_bytes()).to_string());
                    }
                }
            }
        }
    }

    metadata
}

fn extract_fonts(_doc: &Document) -> Vec<PdfFontInfo> {
    // Simplified - full implementation would need more complex PDF parsing
    vec![]
}

/// Convert PdfDocument to a simpler format for LLM analysis
pub fn extract_for_llm(doc: &PdfDocument) -> String {
    let mut output = String::new();

    output.push_str("=== PDF 文档分析 ===\n\n");

    // Metadata
    let meta = &doc.metadata;
    if let Some(title) = &meta.title {
        output.push_str(&format!("标题: {}\n", title));
    }
    if let Some(author) = &meta.author {
        output.push_str(&format!("作者: {}\n", author));
    }
    output.push_str(&format!("页数: {}\n", meta.page_count));
    output.push_str(&format!("字数: {}\n\n", meta.word_count));

    // Content preview (first 30 pages)
    output.push_str("=== 内容预览 (前30页) ===\n");
    for page in doc.pages.iter().take(30) {
        output.push_str(&format!("\n--- 第 {} 页 ---\n", page.page_number));

        // Show first few lines of each page
        let lines: Vec<&str> = page.text.lines().take(10).collect();
        for line in lines {
            if !line.trim().is_empty() {
                output.push_str(&format!("{}\n", line));
            }
        }
    }

    if doc.pages.len() > 30 {
        output.push_str(&format!("\n... 共 {} 页，内容已截断 ...\n", doc.pages.len()));
    }

    output
}