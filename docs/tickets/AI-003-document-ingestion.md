# Ticket: AI-003 — Document Ingestion Pipeline

## Objective
Implement PDF, email, and text document parsing and indexing for the AI assistant, incorporating all document types and edge cases from the research.

## Target Files
- `crates/assistant/src/lib.rs` — Document indexer, text extraction pipeline (PDF via `pdf-extract`/`minimac`, `.eml` headers/body, plain text, CSV)
- `crates/ui-egui/src/lib.rs` — Settings view: "Upload Document" button, document list view, remove document
- `crates/core/src/model.rs` — `Document` struct (path, text, last_modified, source_type), `DocumentIndex`
- `crates/persistence/src/lib.rs` — Persist document list in project file, schema migration

## Document Struct (from research)
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document {
    /// File system path
    pub path: PathBuf,
    
    /// Source type: PDF, email, text, CSV
    pub source_type: DocumentSource,
    
    /// File modification timestamp
    pub last_modified: SystemTime,
    
    /// Extracted text excerpt (sample for context)
    pub text_excerpt: String,
    
    /// Token count for indexing purposes
    pub token_count: usize,
}

/// Document source types from research
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DocumentSource {
    Pdf,
    Email,    // .eml or .msg
    Text,     // plain text
    Csv,      // structured data
}
```

## Text Extraction Pipeline (from research)
- **PDF files**: Text extraction via `pdf-extract` or `minimac` crates
  - Scanned/image PDFs: explicitly report "no text layer" rather than inventing content
  - Size cap: typically 2-4MB per document
  - Chunking: documents split into manageable chunks for indexing
- **`.eml` and `.msg` email files**: Header and body extraction
  - Extract: sender, recipients, subject, date, body text
- **Plain text files**: Direct text extraction
- **CSV files**: Structured data extraction with column headers

## Document Index (from research)
```rust
#[derive(Clone, Debug)]
pub struct DocumentIndex {
    /// All indexed documents
    pub documents: Vec<Document>,
    
    /// Total token count across all documents
    pub total_tokens: usize,
    
    /// Whether indexing is active
    pub active: bool,
}
```

## Upload/Remove Workflow (from research)
- [ ] "Upload Document" button in Assistant Settings → opens native file picker
- [ ] Supported formats: PDF (.pdf), Email (.eml, .msg), Plain text (.txt), CSV (.csv)
- [ ] Extracted text stored with document metadata (path, source_type, last_modified)
- [ ] Index appears in Assistant view context; questions can cite documents
- [ ] User can remove individual documents; index updates accordingly
- [ ] Scanned/image PDFs handled: system reports "no text layer" rather than inventing content
- [ ] Documents listed in Settings → Assistant → Documents with remove controls

## Blocking Dependencies
- AI-001 (provider configuration)

## Acceptance Criteria
- [ ] "Upload Document" button in Assistant Settings → opens native file picker
- [ ] Supported formats: PDF (.pdf), Email (.eml, .msg), Plain text (.txt), CSV (.csv)
- [ ] Extracted text stored with document metadata (path, source_type, last_modified)
- [ ] Index appears in Assistant view context; questions can cite documents
- [ ] User can remove individual documents; index updates accordingly
- [ ] Scanned/image PDFs handled: "no text layer detected" message (not invented content)
- [ ] Documents listed in Settings → Assistant → Documents with remove controls
- [ ] Size cap enforced (2-4MB per document)
- [ ] Chunking implemented for large documents

## Verification
- Headless test: Upload PDF → query assistant → answer cites document page
- UI test: File picker opens, document appears in list, remove works
- Edge case: Scanned PDF → "no text layer detected" message
- Edge case: CSV → structured data extracted and queryable
- Edge case: .eml email → headers and body extracted
- Security test: No document content written to git/project file beyond explicit user control