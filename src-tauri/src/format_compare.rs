// Format comparison engine - compares actual paper format against requirements
// Outputs error locations and modification suggestions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Format requirement specification
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FormatRequirements {
    pub font_body: Option<String>,           // 正文字体
    pub font_title: Option<String>,           // 标题字体
    pub font_size_body: Option<f32>,          // 正文字号
    pub font_size_heading1: Option<f32>,      // 一级标题字号
    pub font_size_heading2: Option<f32>,      // 二级标题字号
    pub font_size_heading3: Option<f32>,      // 三级标题字号
    pub line_spacing: Option<f32>,            // 行距倍数
    pub margin_top: Option<f32>,              // 上边距(cm)
    pub margin_bottom: Option<f32>,           // 下边距(cm)
    pub margin_left: Option<f32>,             // 左边距(cm)
    pub margin_right: Option<f32>,            // 右边距(cm)
    pub indent_body: Option<f32>,             // 正文首行缩进(字符)
    pub indent_first_line: Option<f32>,      // 首行缩进(字符)
    pub alignment_body: Option<String>,       // 正文对齐方式
    pub citation_style: Option<String>,       // 引用格式
    pub bibliography_style: Option<String>,   // 参考文献格式
}

/// Parse format requirements from text
pub fn parse_format_requirements(text: &str) -> FormatRequirements {
    let mut req = FormatRequirements::default();
    let text_lower = text.to_lowercase();

    // Font detection
    if text_lower.contains("宋体") {
        req.font_body = Some("宋体".to_string());
    } else if text_lower.contains("times") || text_lower.contains("times new roman") {
        req.font_body = Some("Times New Roman".to_string());
    }

    if text_lower.contains("黑体") {
        req.font_title = Some("黑体".to_string());
    }

    // Font size detection (multiple patterns)
    if text_lower.contains("五号") || text_lower.contains("10.5") {
        req.font_size_body = Some(10.5);
    } else if text_lower.contains("小五") || text_lower.contains("9") {
        req.font_size_body = Some(9.0);
    } else if text_lower.contains("四号") || text_lower.contains("14") {
        req.font_size_body = Some(14.0);
    }

    // Heading sizes
    if text_lower.contains("三号") || text_lower.contains("15") {
        req.font_size_heading1 = Some(15.0);
    } else if text_lower.contains("四号") || text_lower.contains("14") {
        req.font_size_heading1 = Some(14.0);
    }
    if text_lower.contains("小四") || text_lower.contains("12") {
        req.font_size_heading2 = Some(12.0);
    }
    if text_lower.contains("五号") || text_lower.contains("10.5") {
        req.font_size_heading3 = Some(10.5);
    }

    // Line spacing
    if text_lower.contains("1.5") || text_lower.contains("1.5倍") {
        req.line_spacing = Some(1.5);
    } else if text_lower.contains("2.0") || text_lower.contains("2倍") {
        req.line_spacing = Some(2.0);
    } else if text_lower.contains("1.0") || text_lower.contains("单倍") {
        req.line_spacing = Some(1.0);
    }

    // Margins
    if text_lower.contains("2.5") || text_lower.contains("2.5cm") {
        req.margin_top = Some(2.5);
        req.margin_bottom = Some(2.5);
        req.margin_left = Some(2.5);
        req.margin_right = Some(2.5);
    } else if text_lower.contains("3.0") || text_lower.contains("3cm") {
        req.margin_top = Some(3.0);
        req.margin_bottom = Some(3.0);
        req.margin_left = Some(3.0);
        req.margin_right = Some(3.0);
    }

    // Indentation
    if text_lower.contains("2字符") || text_lower.contains("2个字符") {
        req.indent_body = Some(2.0);
        req.indent_first_line = Some(2.0);
    } else if text_lower.contains("首行缩进") {
        req.indent_first_line = Some(2.0);
    }

    // Alignment
    if text_lower.contains("两端对齐") || text_lower.contains("justify") {
        req.alignment_body = Some("justify".to_string());
    } else if text_lower.contains("左对齐") || text_lower.contains("left") {
        req.alignment_body = Some("left".to_string());
    }

    // Citation style
    if text_lower.contains("gb/t 7714") || text_lower.contains("gb/t7714") {
        req.citation_style = Some("GB/T 7714".to_string());
        req.bibliography_style = Some("GB/T 7714-2015".to_string());
    } else if text_lower.contains("apa") {
        req.citation_style = Some("APA".to_string());
    } else if text_lower.contains("mla") {
        req.citation_style = Some("MLA".to_string());
    }

    req
}

/// Paragraph formatting data extracted from document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphFormat {
    pub index: usize,
    pub text: String,
    pub style_name: Option<String>,
    pub is_heading: bool,
    pub heading_level: Option<u8>,  // 1, 2, 3 for heading levels
    pub font_name: Option<String>,
    pub font_size: Option<f32>,
    pub is_bold: bool,
    pub alignment: Option<String>,
    pub line_spacing: Option<f32>,
    pub indent_first_line: Option<f32>,
    pub indent_left: Option<f32>,
    pub space_before: Option<f32>,
    pub space_after: Option<f32>,
}

/// Document formatting data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentFormat {
    pub paragraphs: Vec<ParagraphFormat>,
    pub styles: HashMap<String, StyleFormat>,
    pub page_count: usize,
    pub word_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StyleFormat {
    pub name: Option<String>,
    pub font_name: Option<String>,
    pub font_size: Option<f32>,
    pub is_bold: bool,
    pub is_italic: bool,
    pub alignment: Option<String>,
    pub line_spacing: Option<f32>,
    pub indent_first_line: Option<f32>,
}

/// Comparison result for a single paragraph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub paragraph_index: usize,
    pub issue_type: String,
    pub description: String,
    pub expected: String,
    pub actual: String,
    pub severity: String,  // "critical", "major", "minor"
    pub suggestion: String,
}

/// Compare document format against requirements
pub fn compare_format(
    requirements: &FormatRequirements,
    document: &DocumentFormat,
) -> Vec<ComparisonResult> {
    let mut results = Vec::new();

    // Check each paragraph
    for para in &document.paragraphs {
        // Heading level detection based on style or content
        let heading_level = detect_heading_level(para);

        if heading_level.is_some() {
            // Check heading formatting
            check_heading_format(requirements, para, heading_level.unwrap(), &mut results);
        } else {
            // Check body text formatting
            check_body_format(requirements, para, &mut results);
        }
    }

    // Check overall document properties
    check_overall_format(requirements, document, &mut results);

    results
}

/// Detect if paragraph is a heading and return its level
fn detect_heading_level(para: &ParagraphFormat) -> Option<u8> {
    // Check style name
    if let Some(style) = &para.style_name {
        let style_lower = style.to_lowercase();
        if style_lower.contains("标题 1") || style_lower.contains("heading 1") || style_lower == "1" {
            return Some(1);
        }
        if style_lower.contains("标题 2") || style_lower.contains("heading 2") || style_lower == "2" {
            return Some(2);
        }
        if style_lower.contains("标题 3") || style_lower.contains("heading 3") || style_lower == "3" {
            return Some(3);
        }
        if style_lower.contains("标题") || style_lower.contains("heading") {
            // Check for level number
            for i in (1..=6).rev() {
                if style_lower.contains(&format!("{}", i)) {
                    return Some(i);
                }
            }
        }
    }

    // Check font size - larger fonts might be headings
    if let Some(size) = para.font_size {
        if size >= 15.0 {
            return Some(1);
        }
        if size >= 12.0 {
            return Some(2);
        }
    }

    // Check if bold and large
    if para.is_bold {
        if let Some(size) = para.font_size {
            if size >= 14.0 {
                return Some(2);
            }
        }
    }

    None
}

/// Check heading paragraph format
fn check_heading_format(
    req: &FormatRequirements,
    para: &ParagraphFormat,
    level: u8,
    results: &mut Vec<ComparisonResult>,
) {
    let expected_font = match level {
        1 => req.font_title.as_ref().or(req.font_body.as_ref()),
        2 => req.font_title.as_ref().or(req.font_body.as_ref()),
        _ => req.font_body.as_ref(),
    };

    let expected_size = match level {
        1 => req.font_size_heading1,
        2 => req.font_size_heading2,
        3 => req.font_size_heading3,
        _ => None,
    };

    // Check font
    if let Some(exp_font) = expected_font {
        if let Some(act_font) = &para.font_name {
            if !fonts_match(exp_font, act_font) {
                results.push(ComparisonResult {
                    paragraph_index: para.index,
                    issue_type: "字体".to_string(),
                    description: format!("第{}级标题字体不符合要求", level),
                    expected: exp_font.clone(),
                    actual: act_font.clone(),
                    severity: "major".to_string(),
                    suggestion: format!("将第{}级标题字体设置为{}", level, exp_font),
                });
            }
        }
    }

    // Check font size
    if let Some(exp_size) = expected_size {
        if let Some(act_size) = para.font_size {
            if (exp_size - act_size).abs() > 0.5 {
                results.push(ComparisonResult {
                    paragraph_index: para.index,
                    issue_type: "字号".to_string(),
                    description: format!("第{}级标题字号不符合要求", level),
                    expected: format!("{}pt", exp_size),
                    actual: format!("{}pt", act_size),
                    severity: "major".to_string(),
                    suggestion: format!("将第{}级标题字号设置为{}pt", level, exp_size),
                });
            }
        }
    }

    // Check bold for headings
    if !para.is_bold && level <= 2 {
        results.push(ComparisonResult {
            paragraph_index: para.index,
            issue_type: "加粗".to_string(),
            description: format!("第{}级标题未加粗", level),
            expected: "加粗".to_string(),
            actual: "未加粗".to_string(),
            severity: "minor".to_string(),
            suggestion: format!("将第{}级标题设置为加粗", level),
        });
    }
}

/// Check body paragraph format
fn check_body_format(
    req: &FormatRequirements,
    para: &ParagraphFormat,
    results: &mut Vec<ComparisonResult>,
) {
    // Skip empty or very short paragraphs
    if para.text.trim().len() < 5 {
        return;
    }

    // Check font
    if let Some(exp_font) = &req.font_body {
        if let Some(act_font) = &para.font_name {
            if !fonts_match(exp_font, act_font) {
                results.push(ComparisonResult {
                    paragraph_index: para.index,
                    issue_type: "字体".to_string(),
                    description: "正文字体不符合要求".to_string(),
                    expected: exp_font.clone(),
                    actual: act_font.clone(),
                    severity: "major".to_string(),
                    suggestion: format!("将正文字体设置为{}", exp_font),
                });
            }
        }
    }

    // Check font size
    if let Some(exp_size) = req.font_size_body {
        if let Some(act_size) = para.font_size {
            if (exp_size - act_size).abs() > 0.5 {
                results.push(ComparisonResult {
                    paragraph_index: para.index,
                    issue_type: "字号".to_string(),
                    description: "正文字号不符合要求".to_string(),
                    expected: format!("{}pt", exp_size),
                    actual: format!("{}pt", act_size),
                    severity: "major".to_string(),
                    suggestion: format!("将正文字号设置为{}pt", exp_size),
                });
            }
        }
    }

    // Check line spacing
    if let Some(exp_spacing) = req.line_spacing {
        if let Some(act_spacing) = para.line_spacing {
            // Allow 10% tolerance
            if (exp_spacing - act_spacing).abs() > 0.15 {
                results.push(ComparisonResult {
                    paragraph_index: para.index,
                    issue_type: "行距".to_string(),
                    description: "行距不符合要求".to_string(),
                    expected: format!("{}倍", exp_spacing),
                    actual: format!("{}倍", act_spacing),
                    severity: "major".to_string(),
                    suggestion: format!("将行距设置为{}倍", exp_spacing),
                });
            }
        }
    }

    // Check first line indent
    if let Some(exp_indent) = req.indent_first_line {
        if let Some(act_indent) = para.indent_first_line {
            if (exp_indent - act_indent).abs() > 0.5 {
                results.push(ComparisonResult {
                    paragraph_index: para.index,
                    issue_type: "缩进".to_string(),
                    description: "首行缩进不符合要求".to_string(),
                    expected: format!("{}字符", exp_indent),
                    actual: format!("{}字符", act_indent),
                    severity: "major".to_string(),
                    suggestion: format!("设置首行缩进{}字符", exp_indent),
                });
            }
        }
    }

    // Check alignment
    if let Some(exp_align) = &req.alignment_body {
        if let Some(act_align) = &para.alignment {
            if !alignments_match(exp_align, act_align) {
                results.push(ComparisonResult {
                    paragraph_index: para.index,
                    issue_type: "对齐".to_string(),
                    description: "段落对齐方式不符合要求".to_string(),
                    expected: exp_align.clone(),
                    actual: act_align.clone(),
                    severity: "minor".to_string(),
                    suggestion: format!("设置对齐方式为{}", exp_align),
                });
            }
        }
    }
}

/// Check overall document format
fn check_overall_format(
    req: &FormatRequirements,
    document: &DocumentFormat,
    results: &mut Vec<ComparisonResult>,
) {
    // Count paragraphs by type
    let heading_count = document.paragraphs.iter()
        .filter(|p| detect_heading_level(p).is_some())
        .count();

    let body_count = document.paragraphs.iter()
        .filter(|p| detect_heading_level(p).is_none() && p.text.trim().len() >= 10)
        .count();

    // Check if we have headings
    if heading_count == 0 && body_count > 10 {
        results.push(ComparisonResult {
            paragraph_index: 0,
            issue_type: "标题层次".to_string(),
            description: "文档中未检测到标题样式".to_string(),
            expected: "应有标题层次结构".to_string(),
            actual: "无标题".to_string(),
            severity: "major".to_string(),
            suggestion: "请使用规范的标题样式（标题1、标题2、标题3）".to_string(),
        });
    }

    // Check line spacing consistency
    let spacings: Vec<f32> = document.paragraphs.iter()
        .filter_map(|p| p.line_spacing)
        .collect();

    if !spacings.is_empty() {
        let avg: f32 = spacings.iter().sum::<f32>() / spacings.len() as f32;
        if let Some(exp) = req.line_spacing {
            if (avg - exp).abs() > 0.2 {
                results.push(ComparisonResult {
                    paragraph_index: 0,
                    issue_type: "行距".to_string(),
                    description: "行距设置不一致".to_string(),
                    expected: format!("{}倍", exp),
                    actual: format!("平均{}倍", avg),
                    severity: "minor".to_string(),
                    suggestion: "确保全文行距一致".to_string(),
                });
            }
        }
    }
}

/// Check if two font names match (considering Chinese variations)
fn fonts_match(expected: &str, actual: &str) -> bool {
    let exp_lower = expected.to_lowercase();
    let act_lower = actual.to_lowercase();

    // Direct match
    if exp_lower == act_lower {
        return true;
    }

    // Chinese font variations
    let chinese_fonts = vec![
        (vec!["宋体", "songti", "simsun", "song"], "宋体"),
        (vec!["黑体", "heiti", "simhei", "hei"], "黑体"),
        (vec!["楷体", "kaiti", "simkai", "kai"], "楷体"),
        (vec!["times new roman", "times"], "times new roman"),
        (vec!["arial"], "arial"),
    ];

    for (aliases, standard) in chinese_fonts {
        let exp_is = aliases.iter().any(|a| exp_lower.contains(a));
        let act_is = aliases.iter().any(|a| act_lower.contains(a));
        if exp_is && act_is {
            return true;
        }
        if exp_lower == *standard || exp_lower.contains(&standard) {
            if act_lower.contains(&standard) {
                return true;
            }
        }
    }

    false
}

/// Check if two alignment values match
fn alignments_match(expected: &str, actual: &str) -> bool {
    let exp_lower = expected.to_lowercase();
    let act_lower = actual.to_lowercase();

    // Direct match
    if exp_lower == act_lower {
        return true;
    }

    // Common variations
    if exp_lower.contains("justify") || exp_lower.contains("两端") {
        return act_lower.contains("justify") || act_lower.contains("两端") || act_lower == "both";
    }
    if exp_lower.contains("left") || exp_lower.contains("左") {
        return act_lower.contains("left") || act_lower.contains("左") || act_lower == "start";
    }
    if exp_lower.contains("center") || exp_lower.contains("居中") {
        return act_lower.contains("center") || act_lower.contains("居中");
    }
    if exp_lower.contains("right") || exp_lower.contains("右") {
        return act_lower.contains("right") || act_lower.contains("右") || act_lower == "end";
    }

    false
}

/// Convert comparison results to format issues (for lib.rs compatibility)
pub fn to_format_issues(results: Vec<ComparisonResult>) -> Vec<crate::FormatIssue> {
    results
        .into_iter()
        .map(|r| crate::FormatIssue {
            issue_type: r.issue_type,
            description: r.description,
            location: crate::IssueLocation {
                page: None,
                paragraph: Some(r.paragraph_index),
                section: None,
            },
            severity: r.severity,
            suggestion: r.suggestion,
        })
        .collect()
}
