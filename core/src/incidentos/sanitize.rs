#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanitizedEvidenceText {
    pub content: String,
}

pub fn sanitize_untrusted_log(raw: &str) -> SanitizedEvidenceText {
    // Treat untrusted logs as inert evidence text by preserving content and
    // removing NUL bytes that can break downstream renderers/parsers.
    let content = raw.replace('\0', "");
    SanitizedEvidenceText { content }
}
