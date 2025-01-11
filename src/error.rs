// Marker type for when diagnostic was already emitted but we want to return with Error
#[derive(Debug)]
pub struct DiagnosticEmitted;

impl std::error::Error for DiagnosticEmitted {}

impl std::fmt::Display for DiagnosticEmitted {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
