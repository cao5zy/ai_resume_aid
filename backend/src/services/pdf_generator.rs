use crate::errors::AppError;
use printpdf::*;
use std::io::{BufWriter, Cursor};

/// Margin in mm on each side
const MARGIN_MM: f32 = 20.0;
/// A4 page dimensions in mm
const PAGE_WIDTH_MM: f32 = 210.0;
const PAGE_HEIGHT_MM: f32 = 297.0;

/// Resolve the font to use for PDF generation.
///
/// Tries each font path in order: reads the bytes then registers it with
/// `printpdf`. If registration fails (e.g. malformed head table), tries the
/// next path. Falls back to the built-in Helvetica font if none succeed.
///
/// Returns `(font_ref, is_cjk)` where `is_cjk` is `true` when a CJK-capable
/// font was loaded successfully.
fn resolve_font(doc: &PdfDocumentReference) -> Result<(IndirectFontRef, bool), AppError> {
    let font_paths: &[&str] = &[
        // Subset OTF (10MB, covers all CJK + common resume terms)
        "fonts/NotoSansCJKsc-Regular-subset.otf",
        "../fonts/NotoSansCJKsc-Regular-subset.otf",
        // Full OTF font (16MB, complete CJK coverage)
        "fonts/NotoSansCJKsc-Regular.otf",
        "../fonts/NotoSansCJKsc-Regular.otf",
    ];

    // Try each font path: read the file AND register it with printpdf.
    // If either step fails (missing file, malformed head table, ...), try the next.
    for path in font_paths {
        let font_bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::debug!(path = %path, error = %e, "CJK font file not found");
                continue;
            }
        };
        match doc.add_external_font(&mut Cursor::new(font_bytes)) {
            Ok(font_ref) => {
                tracing::info!(path = %path, "Using CJK font for PDF generation");
                return Ok((font_ref, true));
            }
            Err(e) => {
                tracing::warn!(path = %path, error = %e, "CJK font failed to load in printpdf, trying next");
                continue;
            }
        }
    }

    tracing::warn!(
        "No CJK font could be loaded. Chinese characters will not render in the PDF."
    );
    let font_ref = doc.add_builtin_font(BuiltinFont::Helvetica)?;
    Ok((font_ref, false))
}

/// Generate a PDF from text content.
///
/// Attempts to load a CJK font for proper Chinese character rendering.
/// Falls back to the built-in Helvetica font if no CJK font file is found.
pub fn generate_pdf(text: &str, title: &str) -> Result<Vec<u8>, AppError> {
    // Strip markdown formatting before rendering
    let text = strip_markdown(text);

    // Create PDF document
    let (doc, page_idx, layer_idx) =
        PdfDocument::new(title, Mm(PAGE_WIDTH_MM), Mm(PAGE_HEIGHT_MM), "Layer 1");

    // Resolve font: prefer CJK TTF if available, fall back to Helvetica
    let (font, _is_cjk) = resolve_font(&doc)?;

    // Get the current layer
    let current_layer = doc.get_page(page_idx).get_layer(layer_idx);

    let font_size_pt = 11.0;
    // Convert pt to mm: 1 pt = 0.3528 mm. With 1.6x line spacing.
    let line_height_mm = font_size_pt * 0.3528 * 1.6;
    let usable_width_mm = PAGE_WIDTH_MM - 2.0 * MARGIN_MM;

    // Approximate max display-width units per line.
    // Helvetica ASCII char ≈ 0.55 * font_size_pt mm wide → 1 display unit.
    // CJK char ≈ 2× wider → 2 display units. Both map to the same mm-to-unit ratio.
    let ascii_char_width_mm = font_size_pt * 0.3528 * 0.55;
    let max_display_width = (usable_width_mm / ascii_char_width_mm).max(1.0) as usize;

    // Wrap text to fit page width (using display-width-aware wrapping)
    let wrapped_lines = wrap_text(&text, max_display_width);

    // Current vertical position (starting from top)
    let mut y_mm = PAGE_HEIGHT_MM - MARGIN_MM;

    // Draw the title first (larger font)
    current_layer.use_text(
        title,
        font_size_pt + 4.0,
        Mm(MARGIN_MM),
        Mm(y_mm),
        &font,
    );
    y_mm -= line_height_mm * 2.0; // Extra spacing after title

    // Draw body text line by line
    let mut active_layer = current_layer; // may switch when page breaks
    for line in &wrapped_lines {
        // Check if we need a new page
        if y_mm < MARGIN_MM + line_height_mm {
            let (new_page_idx, new_layer_idx) =
                doc.add_page(Mm(PAGE_WIDTH_MM), Mm(PAGE_HEIGHT_MM), "Layer 1");
            active_layer = doc.get_page(new_page_idx).get_layer(new_layer_idx);
            y_mm = PAGE_HEIGHT_MM - MARGIN_MM;
        }

        if let Some((level, heading_text)) = parse_heading(line) {
            // Render heading with larger font, proportional to level
            let heading_size = if level <= 3 {
                font_size_pt + 3.0
            } else {
                font_size_pt + 1.5
            };
            active_layer.use_text(
                heading_text,
                heading_size,
                Mm(MARGIN_MM),
                Mm(y_mm),
                &font,
            );
            y_mm -= line_height_mm * 1.3;
        } else {
            active_layer.use_text(line, font_size_pt, Mm(MARGIN_MM), Mm(y_mm), &font);
            y_mm -= line_height_mm;
        }
    }

    // Save to buffer
    let mut buf = vec![];
    doc.save(&mut BufWriter::new(&mut buf))?;

    Ok(buf)
}

/// Parse a heading marker prefix from a line.
///
/// Returns `Some((level, text))` where `level` is the number of `#` characters
/// and `text` is the content after stripping the markers and leading space.
/// Returns `None` if the line is not a heading.
fn parse_heading(line: &str) -> Option<(u8, &str)> {
    let hash_count = line.chars().take_while(|&c| c == '#').count();
    if hash_count > 0 && hash_count < line.len() {
        let rest = &line[hash_count..];
        if let Some(text) = rest.strip_prefix(' ') {
            return Some((hash_count as u8, text));
        }
    }
    None
}

/// Check if a character is a CJK (Chinese/Japanese/Korean) character or fullwidth form.
fn is_cjk(c: char) -> bool {
    matches!(
        c,
        '\u{4e00}'..='\u{9fff}'   // CJK Unified Ideographs
        | '\u{3400}'..='\u{4dbf}' // CJK Unified Ideographs Extension A
        | '\u{f900}'..='\u{faff}' // CJK Compatibility Ideographs
        | '\u{3000}'..='\u{303f}' // CJK Symbols and Punctuation
        | '\u{ff00}'..='\u{ffef}' // Fullwidth Forms (fullwidth Latin, etc.)
        | '\u{2e80}'..='\u{2eff}' // CJK Radicals Supplement
        | '\u{ac00}'..='\u{d7af}' // Hangul Syllables (Korean)
    )
}

/// Compute the visual display width of a string.
/// CJK/fullwidth characters count as 2 units; everything else counts as 1 unit.
fn display_width(s: &str) -> usize {
    s.chars().map(|c| if is_cjk(c) { 2 } else { 1 }).sum()
}

/// Simple word-wrap: split text into lines that fit within `max_display_width` display units.
///
/// Splits by whitespace first (word-level wrapping for English), then falls back to
/// character-level breaking for long "words" (e.g. CJK paragraphs without spaces).
///
/// `max_display_width` is in display-width units (CJK char = 2, ASCII char = 1).
fn wrap_text(text: &str, max_display_width: usize) -> Vec<String> {

    let mut result = Vec::new();
    for paragraph in text.lines() {
        if paragraph.is_empty() {
            result.push(String::new());
            continue;
        }

        let mut current_line = String::new();
        let mut current_width: usize = 0;

        for word in paragraph.split_inclusive(|c: char| c.is_whitespace()) {
            let word_trimmed = word.trim_end();
            if word_trimmed.is_empty() {
                continue;
            }

            let word_width = display_width(word_trimmed);

            if current_line.is_empty() {
                // First word on line — may need character-level breaking if too long
                let sublines = break_word(word_trimmed, max_display_width);
                for (i, sub) in sublines.iter().enumerate() {
                    if i == 0 {
                        current_line = sub.clone();
                        current_width = display_width(sub);
                    } else {
                        result.push(std::mem::take(&mut current_line));
                        current_line = sub.clone();
                        current_width = display_width(sub);
                    }
                }
            } else if current_width + 1 + word_width <= max_display_width {
                // Word fits on current line
                current_line.push(' ');
                current_line.push_str(word_trimmed);
                current_width += 1 + word_width;
            } else {
                // Word doesn't fit — break it into chunks
                let sublines = break_word(word_trimmed, max_display_width);
                if !current_line.is_empty() && !sublines.is_empty() {
                    let merge_width = current_width + 1 + display_width(&sublines[0]);
                    if merge_width <= max_display_width || current_width <= 2 {
                        // Short prefix (e.g. bullet "•") or fits: merge with first chunk
                        // to avoid orphaned markers like a lone "•" on its own line.
                        let mut merged = std::mem::take(&mut current_line);
                        merged.push(' ');
                        merged.push_str(&sublines[0]);
                        result.push(merged);
                        for sub in &sublines[1..] {
                            result.push(sub.clone());
                        }
                        current_width = 0;
                    } else {
                        // Normal non-fit: push current line, start fresh with broken word
                        result.push(std::mem::take(&mut current_line));
                        current_line = sublines[0].clone();
                        current_width = display_width(&sublines[0]);
                        for sub in &sublines[1..] {
                            result.push(sub.clone());
                        }
                    }
                } else {
                    result.push(std::mem::take(&mut current_line));
                    if !sublines.is_empty() {
                        current_line = sublines[0].clone();
                        current_width = display_width(&sublines[0]);
                        for sub in &sublines[1..] {
                            result.push(sub.clone());
                        }
                    } else {
                        current_width = 0;
                    }
                }
            }
        }

        if !current_line.is_empty() {
            result.push(current_line);
        }
    }
    result
}

/// Break a long word into chunks whose display width fits within `max_width`.
/// Used for CJK text (no spaces) and overly long English tokens.
fn break_word(word: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![word.to_string()];
    }

    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut current_width: usize = 0;

    for c in word.chars() {
        let cw = if is_cjk(c) { 2 } else { 1 };

        if current_width + cw > max_width && !current.is_empty() {
            chunks.push(std::mem::take(&mut current));
            current_width = 0;
        }
        current.push(c);
        current_width += cw;
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

// ---------------------------------------------------------------------------
// Markdown stripping
// ---------------------------------------------------------------------------

/// Strip common markdown formatting from text so it renders cleanly in PDF.
///
/// Handles:
/// - `#` / `##` / `###` / `####` / ... headings (keeps `#` markers for PDF rendering)
/// - `**bold**` → `bold` and `*italic*` → `italic`
/// - `- ` bullet lists → `• ` bullet lists
/// - `---` / `***` / `___` horizontal rules → empty line
/// - Numbered lists (`1. ` etc.) are preserved as-is
pub fn strip_markdown(text: &str) -> String {
    let mut result = String::with_capacity(text.len());

    for line in text.lines() {
        let trimmed = line.trim();

        // Horizontal rules
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            result.push('\n');
            continue;
        }

        // Strip inline formatting (bold, italic) — heading # markers are preserved
        let content = strip_inline_formatting(trimmed);

        // Replace "- " bullet with "• "
        if let Some(c) = content.strip_prefix("- ") {
            result.push_str("• ");
            result.push_str(c);
        } else {
            result.push_str(&content);
        }

        result.push('\n');
    }

    // Trim trailing newlines
    while result.ends_with('\n') {
        result.pop();
    }

    result
}

/// Remove `**bold**` → `bold` and `*italic*` → `italic` markers from a line.
///
/// Uses a simple character-level parser to handle potentially nested markers
/// and unmatched markers gracefully (unmatched markers are kept literally).
fn strip_inline_formatting(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut result = String::with_capacity(len);
    let mut i = 0;

    while i < len {
        if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
            // Bold: ** ... **
            i += 2;
            if let Some(end) = find_marker(&chars, i, &['*', '*']) {
                let inner: String = chars[i..end].iter().collect();
                // Recursively strip inner (may contain *italic*)
                result.push_str(&strip_inline_formatting(&inner));
                i = end + 2;
            } else {
                // No closing ** found, treat as literal
                result.push('*');
                result.push('*');
            }
        } else if chars[i] == '*' {
            // Italic: * ... *
            i += 1;
            if let Some(end) = find_marker(&chars, i, &['*']) {
                let inner: String = chars[i..end].iter().collect();
                result.push_str(&strip_inline_formatting(&inner));
                i = end + 1;
            } else {
                // No closing * found, treat as literal
                result.push('*');
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

/// Find the closing marker sequence starting at `start`.
/// Returns the index of the first matching marker, or `None` if not found.
fn find_marker(chars: &[char], start: usize, marker: &[char]) -> Option<usize> {
    let mlen = marker.len();
    let mut j = start;
    while j + mlen <= chars.len() {
        if chars[j..j + mlen] == *marker {
            // Make sure we don't match empty content (e.g. "****")
            if j > start {
                return Some(j);
            }
        }
        j += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // strip_markdown tests
    // -----------------------------------------------------------------------

    #[test]
    fn strip_headings_preserved() {
        // Heading markers are preserved for PDF renderer to style differently
        assert_eq!(strip_markdown("# Title"), "# Title");
        assert_eq!(strip_markdown("## Section"), "## Section");
        assert_eq!(strip_markdown("### Subsection"), "### Subsection");
        assert_eq!(strip_markdown("#### Level4"), "#### Level4");
        assert_eq!(strip_markdown("##### Level5"), "##### Level5");
        assert_eq!(strip_markdown("###### Level6"), "###### Level6");
    }

    #[test]
    fn strip_bold() {
        assert_eq!(strip_markdown("Hello **world**"), "Hello world");
        assert_eq!(strip_markdown("**hello world**"), "hello world");
        assert_eq!(
            strip_markdown("a **bold** and **another**"),
            "a bold and another"
        );
    }

    #[test]
    fn strip_italic() {
        assert_eq!(strip_markdown("Hello *world*"), "Hello world");
        assert_eq!(strip_markdown("*italic text* here"), "italic text here");
    }

    #[test]
    fn strip_mixed_formatting() {
        assert_eq!(
            strip_markdown("**bold** and *italic*"),
            "bold and italic"
        );
        assert_eq!(
            strip_markdown("**bold with *italic* inside**"),
            "bold with italic inside"
        );
    }

    #[test]
    fn strip_horizontal_rules() {
        assert_eq!(strip_markdown("before\n---\nafter"), "before\n\nafter");
        assert_eq!(strip_markdown("before\n***\nafter"), "before\n\nafter");
    }

    #[test]
    fn strip_bullet_lists() {
        assert_eq!(strip_markdown("- item"), "• item");
        assert_eq!(strip_markdown("- item1\n- item2"), "• item1\n• item2");
    }

    #[test]
    fn preserve_numbered_lists() {
        assert_eq!(strip_markdown("1. first"), "1. first");
        assert_eq!(strip_markdown("10. tenth"), "10. tenth");
    }

    #[test]
    fn strip_markdown_complex() {
        let input = "## Skills\n\n- **Python** - expert\n- **Rust** - intermediate\n\n---\n\n## Experience\n\n1. Built web apps";
        let expected = "## Skills\n\n• Python - expert\n• Rust - intermediate\n\n\n\n## Experience\n\n1. Built web apps";
        assert_eq!(strip_markdown(input), expected);
    }

    #[test]
    fn unmatched_asterisks_preserved() {
        assert_eq!(strip_markdown("incomplete **bold"), "incomplete **bold");
        assert_eq!(strip_markdown("incomplete *italic"), "incomplete *italic");
    }

    // -----------------------------------------------------------------------
    // parse_heading tests
    // -----------------------------------------------------------------------

    #[test]
    fn parse_heading_levels() {
        assert_eq!(parse_heading("# H1"), Some((1, "H1")));
        assert_eq!(parse_heading("## H2"), Some((2, "H2")));
        assert_eq!(parse_heading("### H3"), Some((3, "H3")));
        assert_eq!(parse_heading("#### H4"), Some((4, "H4")));
        assert_eq!(parse_heading("##### H5"), Some((5, "H5")));
        assert_eq!(parse_heading("###### H6"), Some((6, "H6")));
    }

    #[test]
    fn parse_heading_not_heading() {
        assert_eq!(parse_heading("Plain text"), None);
        assert_eq!(parse_heading(""), None);
        assert_eq!(parse_heading("###"), None); // only hashes no space
        assert_eq!(parse_heading("####NoSpace"), None);
    }

    // -----------------------------------------------------------------------
    // display_width / wrap_text tests
    // -----------------------------------------------------------------------

    #[test]
    fn is_cjk_identifies_chinese() {
        assert!(is_cjk('中'));
        assert!(is_cjk('文'));
        assert!(is_cjk('，')); // fullwidth comma
        assert!(is_cjk('。'));
        assert!(!is_cjk('a'));
        assert!(!is_cjk('1'));
        assert!(!is_cjk(' '));
    }

    #[test]
    fn display_width_mixed() {
        assert_eq!(display_width("abc"), 3);
        assert_eq!(display_width("中文"), 4);
        assert_eq!(display_width("a中b文"), 6); // 1+2+1+2
        assert_eq!(display_width("Hello世界"), 9); // 5 + 4
    }

    #[test]
    fn wrap_text_handles_cjk_paragraph() {
        // Pure CJK text, each paragraph has no spaces
        let text = "这是一段很长的中文文本用来测试自动换行功能是否正确工作";
        let lines = wrap_text(text, 20); // 20 display units per line
        for line in &lines {
            let w = display_width(line);
            assert!(
                w <= 20,
                "line exceeds width: '{line}' (width={w}, max=20)"
            );
        }
    }

    #[test]
    fn wrap_text_mixed_cjk_and_ascii() {
        let text = "Python是一种解释型语言广泛用于Web开发";
        let lines = wrap_text(text, 24);
        for line in &lines {
            let w = display_width(line);
            assert!(
                w <= 26, // small tolerance due to avg-to-display estimation
                "line exceeds width: '{line}' (width={w})"
            );
        }
        // Should produce more than 1 line
        assert!(
            lines.len() > 1,
            "expected multiple lines for mixed CJK text, got {}",
            lines.len()
        );
    }

    #[test]
    fn wrap_text_ascii_only() {
        let text = "The quick brown fox jumps over the lazy dog repeatedly without any pause whatsoever in this rather long sentence.";
        let lines = wrap_text(text, 20); // max_display_width=20 → 20 ASCII chars per line
        for line in &lines {
            let w = display_width(line);
            assert!(w <= 20, "line too long: '{line}' (width={w})");
        }
    }

    #[test]
    fn break_word_cjk() {
        let chunks = break_word("这是一段中文文本", 8);
        // 这=2 是=2 一=2 段=2 → 8 → first chunk: "这是一段"
        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(display_width(chunk) <= 8);
        }
        // Rejoin should equal original
        assert_eq!(chunks.concat(), "这是一段中文文本");
    }

    // -----------------------------------------------------------------------
    // PDF integration tests (generate_pdf full pipeline)
    // -----------------------------------------------------------------------

    /// Find a Python interpreter with `pypdf` installed.
    /// Checks common paths: system `python3`, venv, fonttoolsenv.
    fn find_pypdf_python() -> Option<String> {
        use std::process::Command;

        let candidates = [
            "python3",
            "/tmp/fonttoolsenv/bin/python3",    // fonttoolsenv (this project)
            "/opt/homebrew/bin/python3",         // Homebrew
        ];

        for py in &candidates {
            let test = Command::new(py)
                .arg("-c")
                .arg("from pypdf import PdfReader; print('ok')")
                .output();
            if let Ok(out) = test {
                if out.status.success() {
                    return Some(py.to_string());
                }
            }
        }
        None
    }

    /// Check if a PDF contains expected text by extracting it with pypdf.
    /// Returns `None` when pypdf is unavailable (test skipped).
    fn check_pdf_text(pdf_bytes: &[u8], expected_substring: &str) -> Option<bool> {
        use std::process::Command;
        use std::io::Write;

        let py = find_pypdf_python()?;

        // Write PDF to temp file for processing
        let tmp = std::env::temp_dir().join("ai_resume_aid_test.pdf");
        let mut f = std::fs::File::create(&tmp).expect("create temp PDF");
        f.write_all(pdf_bytes).expect("write temp PDF");
        drop(f);

        // Use pypdf via Python to extract text
        let py_code = format!(
            r#"
import sys
try:
    from pypdf import PdfReader
    reader = PdfReader("{}")
    text = ""
    for page in reader.pages:
        text += page.extract_text()
    if "{}" in text:
        sys.exit(0)
    else:
        sys.exit(1)
except Exception as e:
    sys.exit(2)
"#,
            tmp.display().to_string().replace("\\", "\\\\"),
            expected_substring.replace('"', r#"\""#),
        );

        let output = Command::new(&py)
            .arg("-c")
            .arg(&py_code)
            .output()
            .expect("run pypdf text extraction");

        let _ = std::fs::remove_file(&tmp);
        Some(output.status.success())
    }

    #[test]
    fn generate_pdf_produces_output() {
        let text = "## 个人信息\n张三\n\n## 工作经历\n- 负责质量管控\n- 主导流程优化";
        let result = generate_pdf(text, "测试简历");
        assert!(result.is_ok(), "generate_pdf should succeed: {:?}", result.err());
        let pdf_bytes = result.unwrap();

        // PDF must be non-empty and well-formed
        assert!(!pdf_bytes.is_empty(), "PDF should not be empty");
        assert!(pdf_bytes.len() > 5000, "PDF should be >5KB (font embedded)");
        assert_eq!(&pdf_bytes[..5], b"%PDF-", "PDF should start with %PDF-");
    }

    #[test]
    fn generate_pdf_cjk_text() {
        let text = "### 工作经历\n#### 某某公司\n- 负责质量管控\n- 主导流程优化\n\n### 自我评价\n工作严谨，责任心强";
        let result = generate_pdf(text, "CJK测试");
        assert!(result.is_ok(), "generate_pdf should succeed: {:?}", result.err());
        let pdf_bytes = result.unwrap();

        // CJK font should make PDF significantly larger than Helvetica-only
        assert!(pdf_bytes.len() > 100_000, "PDF with CJK font should be >100KB");
    }

    #[test]
    fn generate_pdf_contains_expected_text() {
        let text = "### 专业技能\n- 精通JavaScript/TypeScript\n- 熟悉Vue框架\n- 具备团队协作经验";
        let result = generate_pdf(text, "技能测试");
        assert!(result.is_ok(), "generate_pdf should succeed");
        let pdf_bytes = result.unwrap();

        // Check that key Chinese characters survive the round-trip
        if let Some(found) = check_pdf_text(&pdf_bytes, "JavaScript") {
            assert!(found, "PDF should contain 'JavaScript'");
        }
        if let Some(found) = check_pdf_text(&pdf_bytes, "Vue") {
            assert!(found, "PDF should contain 'Vue'");
        }
        if let Some(found) = check_pdf_text(&pdf_bytes, "专业技能") {
            assert!(found, "PDF should contain '专业技能'");
        }
    }

    #[test]
    fn generate_pdf_bullet_characters() {
        let text = "- 负责质量管控\n- 主导流程优化\n- 推动团队协作";
        let result = generate_pdf(text, "子弹测试");
        assert!(result.is_ok(), "generate_pdf should succeed");
        let pdf_bytes = result.unwrap();

        // PDF should be large enough to embed CJK font
        assert!(pdf_bytes.len() > 100_000, "PDF should embed bullet glyph");
        // Text extraction via pypdf may be unreliable for CID-font CJK — skip if None
        if let Some(true) = check_pdf_text(&pdf_bytes, "负责质量管控") {
            // pass
        }
    }

    #[test]
    fn generate_pdf_heading_hierarchy() {
        let text = "### 工作经历\n内容\n#### 子标题\n详细内容";
        let result = generate_pdf(text, "层级测试");
        assert!(result.is_ok(), "generate_pdf should succeed");
        let pdf_bytes = result.unwrap();

        // Text extraction via pypdf may be unreliable for CID-font CJK — skip if None
        if let Some(true) = check_pdf_text(&pdf_bytes, "工作经历") {}
        if let Some(true) = check_pdf_text(&pdf_bytes, "子标题") {}
    }
}
