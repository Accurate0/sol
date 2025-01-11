use codespan_reporting::diagnostic::Diagnostic;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ParserError {
    #[error("diagnostic")]
    Diagnostic(Diagnostic<usize>),
}
