// Word (.docx) document parser
// .docx is a ZIP archive containing XML files

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use zip::ZipArchive;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordDocument {
    pub paragraphs: Vec<WordParagraph>,
    pub styles: Vec<WordStyle>,
    pub properties: DocumentProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordParagraph {
    pub index: usize,
    pub text: String,
    pub style_id: Option<String>,
    pub style_name: Option<String>,
    pub runs: Vec<WordRun>,
    pub formatting: ParagraphFormatting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordRun {
    pub text: String,
    pub font_name: Option<String>,
    pub font_size: Option<f32>,
    pub is_bold: bool,
    pub is_italic: bool,
    pub is_underline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParagraphFormatting {
    pub alignment: Option<String>,        // left, center, right, justify
    pub indentation_left: Option<f32>,    // 缩进（字符）
    pub indentation_first_line: Option<f32>, // 首行缩进
    pub line_spacing: Option<f32>,        // 行距倍数
    pub space_before: Option<f32>,        // 段前间距
    pub space_after: Option<f32>,         // 段后间距
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordStyle {
    pub id: String,
    pub name: Option<String>,
    pub style_type: String,               // paragraph or character
    pub based_on: Option<String>,
    pub paragraph_formatting: Option<ParagraphFormatting>,
    pub run_formatting: Option<RunFormatting>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RunFormatting {
    pub font_name: Option<String>,
    pub font_size: Option<f32>,
    pub is_bold: bool,
    pub is_italic: bool,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentProperties {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub creator: Option<String>,
    pub page_count: Option<usize>,
    pub word_count: Option<usize>,
}

/// Parse a .docx file and extract all content and formatting
pub fn parse_docx(file_path: &str) -> Result<WordDocument, String> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader).map_err(|e| e.to_string())?;

    // Parse document.xml for content
    let doc_xml = get_xml_from_zip(&mut archive, "word/document.xml")?;

    // Parse styles.xml for styles
    let styles_xml = get_xml_from_zip(&mut archive, "word/styles.xml").ok();

    // Parse core.xml for document properties
    let props_xml = get_xml_from_zip(&mut archive, "docProps/core.xml").ok();

    // Build style map from styles.xml
    let style_map = parse_styles(styles_xml.as_deref());

    // Parse paragraphs from document.xml
    let paragraphs = parse_document_xml(&doc_xml, &style_map);

    // Count pages (approximate - based on section breaks)
    let page_count = paragraphs.iter().filter(|p| p.text.contains("\x0C")).count() + 1;

    // Count words
    let word_count: usize = paragraphs.iter()
        .map(|p| p.text.split_whitespace().count())
        .sum();

    // Parse document properties
    let mut properties = parse_properties(props_xml.as_deref());
    properties.page_count = Some(page_count);
    properties.word_count = Some(word_count);

    Ok(WordDocument {
        paragraphs,
        styles: style_map.values().cloned().collect(),
        properties,
    })
}

fn get_xml_from_zip(archive: &mut ZipArchive<BufReader<File>>, name: &str) -> Result<String, String> {
    let mut file = archive.by_name(name).map_err(|_| format!("File {} not found in archive", name))?;
    let mut content = String::new();
    file.read_to_string(&mut content).map_err(|e| e.to_string())?;
    Ok(content)
}

fn parse_styles(xml: Option<&str>) -> HashMap<String, WordStyle> {
    let mut styles = HashMap::new();

    if let Some(xml) = xml {
        // Simple XML parsing - look for style elements
        // This is a simplified parser - real implementation would use proper XML parsing

        // Extract style IDs and names using simple string matching
        let style_patterns = ["<w:style ", "<w:style>"];

        for line in xml.split("</w:style>") {
            let mut style_id = String::new();
            let mut style_name = Option::<String>::None;
            let mut style_type = String::from("paragraph");

            // Extract style ID
            if let Some(start) = line.find("w:styleId=\"") {
                let start = start + 12;
                if let Some(end) = line[start..].find('"') {
                    style_id = line[start..start + end].to_string();
                }
            }

            // Extract style name
            if let Some(start) = line.find("<w:name ") {
                let search = &line[start..];
                if let Some(val_start) = search.find("w:val=\"") {
                    let val_start = start + val_start + 7;
                    if let Some(val_end) = line[val_start..].find('"') {
                        style_name = Some(line[val_start..val_start + val_end].to_string());
                    }
                }
            }

            // Extract style type
            if let Some(start) = line.find("w:type=\"") {
                let start = start + 9;
                if let Some(end) = line[start..].find('"') {
                    style_type = line[start..start + end].to_string();
                }
            }

            if !style_id.is_empty() {
                styles.insert(style_id.clone(), WordStyle {
                    id: style_id,
                    name: style_name,
                    style_type,
                    based_on: None,
                    paragraph_formatting: None,
                    run_formatting: None,
                });
            }
        }
    }

    styles
}

fn parse_document_xml(xml: &str, style_map: &HashMap<String, WordStyle>) -> Vec<WordParagraph> {
    let mut paragraphs = Vec::new();
    let mut current_text = String::new();
    let mut current_runs: Vec<WordRun> = Vec::new();
    let mut current_style_id: Option<String> = None;
    let mut current_style_name: Option<String> = None;
    let mut paragraph_index = 0;

    // Simplified XML parsing - look for <w:p> elements (paragraphs)
    for para_block in xml.split("</w:p>") {
        // Extract text content from runs
        let mut runs: Vec<WordRun> = Vec::new();
        let mut text_parts: Vec<String> = Vec::new();

        for run_block in para_block.split("</w:r>") {
            let mut run_text = String::new();
            let mut font_name: Option<String> = None;
            let mut font_size: Option<f32> = None;
            let mut is_bold = false;
            let mut is_italic = false;
            let mut is_underline = false;

            // Extract text content
            for t in run_block.split("</w:t>") {
                if let Some(start) = t.find("<w:t") {
                    let search = &t[start..];
                    if let Some(text_start) = search.find('>') {
                        let text = &search[text_start + 1..];
                        if !text.trim().is_empty() {
                            run_text.push_str(text);
                        }
                    }
                }
            }

            // Check for bold
            if run_block.contains("<w:b ") || run_block.contains("<w:b>") {
                is_bold = true;
            }

            // Check for italic
            if run_block.contains("<w:i ") || run_block.contains("<w:i>") {
                is_italic = true;
            }

            // Check for underline
            if run_block.contains("<w:u ") || run_block.contains("<w:u>") {
                is_underline = true;
            }

            // Extract font name
            if let Some(start) = run_block.find("<w:rFonts ") {
                let search = &run_block[start..];
                if let Some(name_start) = search.find("w:ascii=\"") {
                    let name_start = start + name_start + 10;
                    if let Some(name_end) = search[name_start..].find('"') {
                        font_name = Some(search[name_start..name_start + name_end].to_string());
                    }
                } else if let Some(name_start) = search.find("w:asciiTheme=\"") {
                    let name_start = start + name_start + 14;
                    if let Some(name_end) = search[name_start..].find('"') {
                        font_name = Some(search[name_start..name_start + name_end].to_string());
                    }
                }
            }

            // Extract font size (in half-points, so divide by 2)
            if let Some(start) = run_block.find("<w:sz ") {
                let search = &run_block[start..];
                if let Some(size_start) = search.find("w:val=\"") {
                    let size_start = start + size_start + 8;
                    if let Some(size_end) = search[size_start..].find('"') {
                        let size_str = &search[size_start..size_start + size_end];
                        if let Ok(size) = size_str.parse::<f32>() {
                            font_size = Some(size / 2.0);
                        }
                    }
                }
            }

            if !run_text.is_empty() {
                runs.push(WordRun {
                    text: run_text.clone(),
                    font_name,
                    font_size,
                    is_bold,
                    is_italic,
                    is_underline,
                });
                text_parts.push(run_text);
            }
        }

        let full_text = text_parts.join("");

        // Skip empty paragraphs
        if full_text.trim().is_empty() {
            continue;
        }

        // Extract paragraph style
        if let Some(start) = para_block.find("w:pStyle w:val=\"") {
            let search = &para_block[start..];
            if let Some(style_start) = search.find("w:val=\"") {
                let style_start = start + style_start + 7;
                if let Some(style_end) = search[style_start..].find('"') {
                    let style_id = search[style_start..style_start + style_end].to_string();
                    current_style_id = Some(style_id.clone());
                    if let Some(style) = style_map.get(&style_id) {
                        current_style_name = style.name.clone();
                    }
                }
            }
        }

        // Extract paragraph formatting
        let mut formatting = ParagraphFormatting::default();

        // Alignment
        if let Some(start) = para_block.find("<w:jc ") {
            let search = &para_block[start..];
            if let Some(align_start) = search.find("w:val=\"") {
                let align_start = start + align_start + 7;
                if let Some(align_end) = search[align_start..].find('"') {
                    formatting.alignment = Some(search[align_start..align_start + align_end].to_string());
                }
            }
        }

        // Indentation
        if let Some(start) = para_block.find("<w:ind ") {
            let search = &para_block[start..];

            // First line indent
            if let Some(first_start) = search.find("w:firstLine=\"") {
                let first_start = start + first_start + 13;
                if let Some(first_end) = search[first_start..].find('"') {
                    if let Ok(val) = search[first_start..first_start + first_end].parse::<f32>() {
                        // Convert to characters (approximate)
                        formatting.indentation_first_line = Some(val / 20.0);
                    }
                }
            }

            // Left indent
            if let Some(left_start) = search.find("w:left=\"") {
                let left_start = start + left_start + 8;
                if let Some(left_end) = search[left_start..].find('"') {
                    if let Ok(val) = search[left_start..left_start + left_end].parse::<f32>() {
                        formatting.indentation_left = Some(val / 20.0);
                    }
                }
            }
        }

        // Line spacing
        if let Some(start) = para_block.find("<w:spacing ") {
            let search = &para_block[start..];

            // Line spacing (multiply)
            if let Some(line_start) = search.find("w:line=\"") {
                let line_start = start + line_start + 9;
                if let Some(line_end) = search[line_start..].find('"') {
                    if let Ok(val) = search[line_start..line_start + line_end].parse::<f32>() {
                        formatting.line_spacing = Some(val / 240.0);
                    }
                }
            }

            // Space before
            if let Some(before_start) = search.find("w:before=\"") {
                let before_start = start + before_start + 11;
                if let Some(before_end) = search[before_start..].find('"') {
                    if let Ok(val) = search[before_start..before_start + before_end].parse::<f32>() {
                        formatting.space_before = Some(val / 20.0);
                    }
                }
            }

            // Space after
            if let Some(after_start) = search.find("w:after=\"") {
                let after_start = start + after_start + 10;
                if let Some(after_end) = search[after_start..].find('"') {
                    if let Ok(val) = search[after_start..after_start + after_end].parse::<f32>() {
                        formatting.space_after = Some(val / 20.0);
                    }
                }
            }
        }

        paragraphs.push(WordParagraph {
            index: paragraph_index,
            text: full_text,
            style_id: current_style_id.clone(),
            style_name: current_style_name.clone(),
            runs,
            formatting,
        });

        paragraph_index += 1;
        current_style_id = None;
        current_style_name = None;
    }

    paragraphs
}

fn parse_properties(xml: Option<&str>) -> DocumentProperties {
    let mut props = DocumentProperties::default();

    if let Some(xml) = xml {
        // Extract title
        if let Some(start) = xml.find("<dc:title>") {
            let search = &xml[start + 11..];
            if let Some(end) = search.find("</dc:title>") {
                props.title = Some(search[..end].to_string());
            }
        }

        // Extract author
        if let Some(start) = xml.find("<dc:creator>") {
            let search = &xml[start + 12..];
            if let Some(end) = search.find("</dc:creator>") {
                props.author = Some(search[..end].to_string());
            }
        }

        // Extract subject
        if let Some(start) = xml.find("<dc:subject>") {
            let search = &xml[start + 13..];
            if let Some(end) = search.find("</dc:subject>") {
                props.subject = Some(search[..end].to_string());
            }
        }

        // Extract creator
        if let Some(start) = xml.find("<cp:lastModifiedBy>") {
            let search = &xml[start + 19..];
            if let Some(end) = search.find("</cp:lastModifiedBy>") {
                props.creator = Some(search[..end].to_string());
            }
        }
    }

    props
}

/// Convert WordDocument to a simpler format for LLM analysis
pub fn extract_for_llm(doc: &WordDocument) -> String {
    let mut output = String::new();

    output.push_str("=== 论文格式分析 ===\n\n");

    // Document properties
    if let Some(title) = &doc.properties.title {
        output.push_str(&format!("标题: {}\n", title));
    }
    if let Some(author) = &doc.properties.author {
        output.push_str(&format!("作者: {}\n", author));
    }
    output.push_str(&format!("页数: {:?}\n", doc.properties.page_count));
    output.push_str(&format!("字数: {:?}\n\n", doc.properties.word_count));

    // Styles summary
    output.push_str("=== 样式统计 ===\n");
    for style in &doc.styles {
        if let Some(name) = &style.name {
            output.push_str(&format!("- {}\n", name));
        }
    }
    output.push('\n');

    // Paragraph analysis (first 20 and last 5)
    output.push_str("=== 段落格式详情 (前20段) ===\n");
    for (i, para) in doc.paragraphs.iter().take(20).enumerate() {
        output.push_str(&format!("\n[段落 {}]\n", i + 1));
        output.push_str(&format!("内容: {}\n", &para.text[..para.text.len().min(100)]));

        if let Some(style_name) = &para.style_name {
            output.push_str(&format!("样式: {}\n", style_name));
        }

        let fmt = &para.formatting;
        if let Some(align) = &fmt.alignment {
            output.push_str(&format!("对齐: {}\n", align));
        }
        if let Some(indent) = fmt.indentation_first_line {
            output.push_str(&format!("首行缩进: {:.1} 字符\n", indent));
        }
        if let Some(spacing) = fmt.line_spacing {
            output.push_str(&format!("行距: {:.1} 倍\n", spacing));
        }

        // Font info from runs
        if !para.runs.is_empty() {
            let first_run = &para.runs[0];
            if let Some(font) = &first_run.font_name {
                output.push_str(&format!("字体: {}\n", font));
            }
            if let Some(size) = first_run.font_size {
                output.push_str(&format!("字号: {} pt\n", size));
            }
            if first_run.is_bold {
                output.push_str("加粗: 是\n");
            }
        }
    }

    output.push_str("\n=== 后5段 ===\n");
    for (i, para) in doc.paragraphs.iter().rev().take(5).enumerate() {
        let idx = doc.paragraphs.len() - 5 + i;
        output.push_str(&format!("\n[段落 {}]\n", idx + 1));
        output.push_str(&format!("内容: {}\n", &para.text[..para.text.len().min(100)]));
    }

    output
}