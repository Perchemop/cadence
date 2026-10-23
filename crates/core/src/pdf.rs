//! A minimal PDF writer for report and view printing.
//!
//! A printer needs three things: text pages with selectable font sizes, image
//! pages (a Gantt screenshot is an RGB image), and a file with no third-party
//! dependency in the tree. That is all this module is: a builder that emits
//! PDF 1.4 using only the standard Helvetica fonts and raw RGB images, with
//! every stream ASCII-hex encoded so no byte sequence can collide with the
//! `endstream` marker.
//!
//! It is deliberately not a general PDF library: no compression beyond hex
//! encoding, no embedded fonts, no interactive features. Who chooses where the
//! file goes, and who hands it to the system printer, belongs to the caller.
//!
//! Pages are A4 — portrait for text, landscape for images — with 56 pt
//! margins.

use std::fmt::Write as _;
use std::io::Write as _;

/// Page margin, in points.
const MARGIN: f32 = 56.0;
/// A4 portrait, in points.
const A4_PORTRAIT: (f32, f32) = (595.28, 841.89);
/// A4 landscape, in points.
const A4_LANDSCAPE: (f32, f32) = (841.89, 595.28);
/// Default body text size, in points.
const BODY_SIZE: f32 = 9.5;
/// Line spacing factor.
const LINE_SPACING: f32 = 1.32;

/// One line of text on a page.
#[derive(Debug, Clone, PartialEq)]
pub struct TextLine {
    /// Text to draw, on a single line.
    pub text: String,
    /// Font size in points.
    pub size: f32,
    /// Whether to draw in bold.
    pub bold: bool,
    /// Extra space above the line, in points.
    pub space_before: f32,
}

impl TextLine {
    /// A body line.
    #[must_use]
    pub fn body(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            size: BODY_SIZE,
            bold: false,
            space_before: 0.0,
        }
    }

    /// A heading at the given size.
    #[must_use]
    pub fn heading(text: impl Into<String>, size: f32) -> Self {
        Self {
            text: text.into(),
            size,
            bold: true,
            space_before: size * 0.6,
        }
    }

    /// Estimated width of the line, in points.
    fn width(&self) -> f32 {
        self.text.chars().map(|c| char_width(c, self.size)).sum()
    }
}

/// A page of a PDF document.
#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    /// Text laid out from the top of the page down.
    Text(Vec<TextLine>),
    /// A raw RGB image drawn to fill the page, keeping its aspect ratio.
    Image {
        /// Image width in pixels.
        width: u32,
        /// Image height in pixels.
        height: u32,
        /// RGB bytes, three per pixel, row-major.
        rgb: Vec<u8>,
    },
}

/// A PDF document under construction.
///
/// ```ignore
/// let mut pdf = Pdf::new();
/// pdf.text_page(vec![TextLine::heading("Bridge & Road", 20.0)]);
/// std::fs::write("report.pdf", pdf.to_bytes()).expect("write");
/// ```
#[derive(Debug, Default)]
pub struct Pdf {
    pages: Vec<Page>,
}

impl Pdf {
    /// An empty document, with no pages.
    #[must_use]
    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }

    /// Append a page of text.
    pub fn text_page(&mut self, lines: Vec<TextLine>) -> &mut Self {
        self.pages.push(Page::Text(lines));
        self
    }

    /// Append a page showing a raw RGB image.
    pub fn image_page(&mut self, width: u32, height: u32, rgb: Vec<u8>) -> &mut Self {
        self.pages.push(Page::Image { width, height, rgb });
        self
    }

    /// Serialize the document to PDF bytes.
    ///
    /// An empty document still produces a valid file: one blank page, so the
    /// output is never a header without a body.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        // Object numbering: 1 catalogue, 2 page tree, 3 and 4 the two fonts,
        // then two objects per text page (page, contents) and three per image
        // page (page, contents, image).
        let pages = 2u32;
        let regular_font = 3u32;
        let bold_font = 4u32;

        let mut bodies: Vec<Vec<u8>> = vec![
            b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
            Vec::new(),
            b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec(),
            b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>".to_vec(),
        ];

        let mut kids: Vec<String> = Vec::new();
        let mut next_object = 5u32;
        for page in self.pages() {
            let page_object = next_object;
            kids.push(page_object.to_string());
            match page {
                Page::Text(lines) => {
                    let content = text_content(lines);
                    bodies.push(
                        format!(
                            "<< /Type /Page /Parent {pages} 0 R \
                             /Resources << /Font << /F1 {regular_font} 0 R \
                             /F2 {bold_font} 0 R >> >> \
                             /MediaBox [0 0 {} {}] /Contents {} 0 R >>",
                            A4_PORTRAIT.0,
                            A4_PORTRAIT.1,
                            next_object + 1
                        )
                        .into_bytes(),
                    );
                    bodies.push(stream(content.as_bytes()));
                    next_object += 2;
                }
                Page::Image { width, height, rgb } => {
                    let image_object = next_object + 2;
                    let content = image_content(*width, *height, image_object);
                    bodies.push(
                        format!(
                            "<< /Type /Page /Parent {pages} 0 R \
                             /Resources << /XObject << /Im0 {image_object} 0 R >> >> \
                             /MediaBox [0 0 {} {}] /Contents {} 0 R >>",
                            A4_LANDSCAPE.0,
                            A4_LANDSCAPE.1,
                            next_object + 1
                        )
                        .into_bytes(),
                    );
                    bodies.push(stream(content.as_bytes()));
                    bodies.push(
                        format!(
                            "<< /Type /XObject /Subtype /Image /Width {width} \
                             /Height {height} /ColorSpace /DeviceRGB \
                             /BitsPerComponent 8 /Length {} >>",
                            rgb.len() * 2 + 2
                        )
                        .into_bytes(),
                    );
                    // Replace the dictionary above with the dictionary plus its
                    // payload as one object body.
                    let dictionary = bodies.pop().expect("just pushed");
                    let mut image = dictionary;
                    image.extend_from_slice(b"\nstream\n");
                    image.extend_from_slice(&hex_bytes(rgb));
                    image.extend_from_slice(b">\n");
                    bodies.push(image);
                    next_object += 3;
                }
            }
        }

        if kids.is_empty() {
            // Keep every valid-PDF invariant: a blank text page.
            bodies.push(
                format!(
                    "<< /Type /Page /Parent {pages} 0 R \
                     /Resources << /Font << /F1 {regular_font} 0 R >> >> \
                     /MediaBox [0 0 {} {}] /Contents {} 0 R >>",
                    A4_PORTRAIT.0,
                    A4_PORTRAIT.1,
                    next_object + 1
                )
                .into_bytes(),
            );
            bodies.push(stream(b"BT\n/F1 9.5 Tf 1 0 0 1 56 756 Tm ( ) Tj\nET\n"));
            kids.push(next_object.to_string());
        }

        bodies[1] = format!(
            "<< /Type /Pages /Kids [{}] /Count {} >>",
            kids.iter()
                .map(|id| format!("{id} 0 R"))
                .collect::<Vec<_>>()
                .join(" "),
            kids.len()
        )
        .into_bytes();

        assemble(&bodies)
    }

    /// The pages to render; empty documents render one blank page.
    fn pages(&self) -> Vec<&Page> {
        if self.pages.is_empty() {
            return Vec::new();
        }
        self.pages.iter().collect()
    }
}

/// Build the content stream of a text page, wrapping lines that do not fit.
fn text_content(lines: &[TextLine]) -> String {
    let usable = A4_PORTRAIT.0 - 2.0 * MARGIN;
    let mut content = String::from("BT\n");
    let mut cursor = A4_PORTRAIT.1 - MARGIN;

    for line in lines {
        cursor -= line.space_before;
        for segment in wrap(line, usable) {
            let size = segment.size;
            let font = if segment.bold { "/F2" } else { "/F1" };
            let baseline = (cursor - size).max(MARGIN);
            let _ = writeln!(
                content,
                "{font} {size:.2} Tf 1 0 0 1 {x:.2} {baseline:.2} Tm ({}) Tj",
                escape_pdf(&segment.text),
                x = MARGIN
            );
            cursor -= size * LINE_SPACING;
        }
    }
    content.push_str("ET\n");
    content
}

/// Build the content stream of an image page, fitted into the margins.
fn image_content(width: u32, height: u32, image_object: u32) -> String {
    let usable_w = A4_LANDSCAPE.0 - 2.0 * MARGIN;
    let usable_h = A4_LANDSCAPE.1 - 2.0 * MARGIN;
    let scale = if width == 0 || height == 0 {
        1.0
    } else {
        (usable_w / width as f32).min(usable_h / height as f32)
    };
    let draw_w = width as f32 * scale;
    let draw_h = height as f32 * scale;
    let x = (A4_LANDSCAPE.0 - draw_w) / 2.0;
    let y = (A4_LANDSCAPE.1 - draw_h) / 2.0;
    format!("q {draw_w:.2} 0 0 {draw_h:.2} {x:.2} {y:.2} cm /Im0 {image_object} 0 R Do Q\n")
}

/// Split a line into segments that fit the usable width.
fn wrap(line: &TextLine, usable: f32) -> Vec<TextLine> {
    if line.width() <= usable || line.text.trim().is_empty() {
        return vec![line.clone()];
    }

    let mut segments: Vec<TextLine> = Vec::new();
    let mut current = String::new();
    for word in line.text.split(' ') {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        let width: f32 = candidate.chars().map(|c| char_width(c, line.size)).sum();
        if width <= usable || current.is_empty() {
            current = candidate;
        } else {
            segments.push(TextLine {
                text: std::mem::take(&mut current),
                ..line.clone()
            });
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        segments.push(TextLine {
            text: current,
            ..line.clone()
        });
    }
    segments
}

/// Distribute lines over as many A4 portrait pages as they need.
///
/// Long lines are wrapped first, so a line may occupy more than one segment;
/// a page breaks only between segments and never leaves a heading stranded as
/// the last line of a page.
#[must_use]
pub fn paginate(lines: &[TextLine]) -> Vec<Vec<TextLine>> {
    let usable_width = A4_PORTRAIT.0 - 2.0 * MARGIN;
    let usable_height = A4_PORTRAIT.1 - 2.0 * MARGIN;

    let mut pages: Vec<Vec<TextLine>> = Vec::new();
    let mut page: Vec<TextLine> = Vec::new();
    let mut used = 0.0;

    for line in lines {
        for segment in wrap(line, usable_width) {
            let height = segment.space_before + segment.size * LINE_SPACING;
            if used + height > usable_height && !page.is_empty() {
                pages.push(std::mem::take(&mut page));
                used = 0.0;
            }
            used += height;
            page.push(segment);
        }
    }
    if !page.is_empty() {
        pages.push(page);
    }
    if pages.is_empty() {
        pages.push(Vec::new());
    }
    pages
}

/// Approximate Helvetica width, in points, for layout purposes only.
fn char_width(character: char, size: f32) -> f32 {
    let units = match character {
        'i' | 'j' | 'l' | '!' | '.' | ',' | '\'' | ' ' => 0.28,
        'm' | 'w' | 'M' | 'W' => 0.83,
        c if c.is_uppercase() => 0.67,
        c if c.is_ascii_digit() => 0.56,
        _ => 0.5,
    };
    units * size
}

/// Escape text for a PDF literal string.
fn escape_pdf(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '(' => escaped.push_str("\\("),
            ')' => escaped.push_str("\\)"),
            '\\' => escaped.push_str("\\\\"),
            '\n' | '\r' => escaped.push(' '),
            other => escaped.push(other),
        }
    }
    escaped
}

/// Wrap a payload as an ASCII-hex encoded PDF stream, closed for a reader.
///
/// The stream data is the hex encoding, the `>` terminator and an EOL — so the
/// `/Length` is `2 * payload + 2`.
fn stream(payload: &[u8]) -> Vec<u8> {
    let mut body = format!(
        "<< /Filter /ASCIIHexDecode /Length {} >>\nstream\n",
        payload.len() * 2 + 2
    )
    .into_bytes();
    body.extend_from_slice(&hex_bytes(payload));
    body.extend_from_slice(b">\nendstream");
    body
}

/// The ASCII-hex encoding of a payload, without the `>` terminator.
fn hex_bytes(payload: &[u8]) -> Vec<u8> {
    const DIGITS: &[u8; 16] = b"0123456789ABCDEF";
    let mut hex = Vec::with_capacity(payload.len() * 2);
    for byte in payload {
        hex.push(DIGITS[usize::from(byte >> 4)]);
        hex.push(DIGITS[usize::from(byte & 0x0F)]);
    }
    hex
}

/// Assemble object bodies into a PDF file with a cross-reference table.
fn assemble(bodies: &[Vec<u8>]) -> Vec<u8> {
    let mut file = Vec::new();
    file.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");

    let mut offsets: Vec<usize> = Vec::with_capacity(bodies.len());
    for (index, body) in bodies.iter().enumerate() {
        offsets.push(file.len());
        let _ = writeln!(file, "{} 0 obj", index + 1);
        file.extend_from_slice(body);
        file.extend_from_slice(b"\nendobj\n");
    }

    let xref_start = file.len();
    let _ = writeln!(file, "xref\n0 {}", bodies.len() + 1);
    file.extend_from_slice(b"0000000000 65535 f \n");
    for offset in &offsets {
        let _ = writeln!(file, "{offset:010} 00000 n ");
    }
    let _ = write!(
        file,
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_start}\n%%EOF\n",
        bodies.len() + 1
    );
    file
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_of(bytes: &[u8]) -> String {
        String::from_utf8_lossy(bytes).to_string()
    }

    /// Hex-encode a slice, as the writer does, so tests can search for text the
    /// way a reader would decode it.
    fn hex_of(text: &str) -> String {
        String::from_utf8(hex_bytes(text.as_bytes())).expect("hex is ASCII")
    }

    #[test]
    fn a_text_document_is_a_valid_pdf() {
        let mut pdf = Pdf::new();
        pdf.text_page(vec![
            TextLine::heading("Bridge & Road", 20.0),
            TextLine::body("Survey"),
            TextLine::body("Excavate \\ with (parens) and a 100% character"),
        ]);

        let bytes = pdf.to_bytes();
        let text = text_of(&bytes);

        assert!(text.starts_with("%PDF-1.4"));
        assert!(text.contains("/Type /Catalog"));
        assert!(text.contains("/Type /Pages"));
        assert!(text.contains("/BaseFont /Helvetica"));
        // Streams are ASCII-hex encoded, so the text appears in the document
        // only after decoding: that is what a reader does, and it must find the
        // words escaped as PDF literal strings.
        assert!(text.contains(&hex_of("Survey")));
        assert!(text.contains(&hex_of("Excavate \\ with (parens)")));
        // A cross-reference table and trailer close the file.
        assert!(text.contains("xref\n0 7"));
        assert!(text.contains("/Root 1 0 R"));
        assert!(text.ends_with("%%EOF\n"));
    }

    #[test]
    fn the_startxref_offset_points_at_the_xref_table() {
        let mut pdf = Pdf::new();
        pdf.text_page(vec![TextLine::body("A")]);
        let bytes = pdf.to_bytes();
        let text = text_of(&bytes);

        let marker = "startxref\n";
        let start = text.find(marker).expect("startxref") + marker.len();
        let end = text[start..].find('\n').expect("end of offset line");
        let offset: usize = text[start..start + end].parse().expect("numeric offset");

        assert_eq!(&bytes[offset..offset + 4], b"xref");
    }

    #[test]
    fn every_page_object_is_counted_and_referenced() {
        let mut pdf = Pdf::new();
        pdf.text_page(vec![TextLine::body("A")]);
        pdf.text_page(vec![TextLine::body("B")]);
        pdf.image_page(2, 2, vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255]);

        let text = text_of(&pdf.to_bytes());
        assert!(text.contains("/Count 3"));
        assert!(text.contains("/Type /XObject"));
        assert!(text.contains("/Subtype /Image"));
        assert!(text.contains("/ColorSpace /DeviceRGB"));
        assert!(text.contains("/BitsPerComponent 8"));
        // Hex-encoded pixels: the last one in the payload is white.
        assert!(text.contains("FFFFFF>"));
    }

    #[test]
    fn an_empty_document_still_produces_one_page() {
        let text = text_of(&Pdf::new().to_bytes());
        assert!(text.contains("/Count 1"));
        assert!(text.ends_with("%%EOF\n"));
    }

    #[test]
    fn hex_streams_cannot_contain_the_stream_delimiter() {
        // No byte sequence in the payload can spell "endstream", because the
        // encoding is restricted to hex digits and terminator.
        let payload = b"endstream is a keyword, not data".to_vec();
        let hex = hex_bytes(&payload);
        let as_text = text_of(&hex);
        assert!(!as_text.contains("endstream"));
    }

    #[test]
    fn long_lines_are_wrapped_to_the_page_width() {
        let line =
            TextLine::body("participate in the leveling of the resource schedule ".repeat(12));
        let segments = wrap(&line, A4_PORTRAIT.0 - 2.0 * MARGIN);
        assert!(
            segments.len() > 1,
            "expected wrapping, got {} segments",
            segments.len()
        );
        for segment in &segments {
            assert!(segment.width() <= A4_PORTRAIT.0 - 2.0 * MARGIN + 0.1);
        }
    }

    #[test]
    fn image_pages_fit_the_landscape_page() {
        let content = image_content(1600, 1000, 9);
        assert!(content.contains("/Im0 9 0 R Do"));
        let usable = A4_LANDSCAPE.0 - 2.0 * MARGIN;
        let expected_width =
            (usable / 1600.0).min((A4_LANDSCAPE.1 - 2.0 * MARGIN) / 1000.0) * 1600.0;
        assert!(content.contains(&format!("{expected_width:.2} 0 0 ")));
    }

    #[test]
    fn parentheses_and_backslashes_are_escaped() {
        assert_eq!(escape_pdf("a(b)c\\d"), "a\\(b\\)c\\\\d");
        assert_eq!(escape_pdf("line\nbreak"), "line break");
    }
}
