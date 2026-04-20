pub struct Diagnostic<'a> {
    pub headline: String,
    pub line: u32,
    pub source_line: &'a str,
    pub column: u32,
    pub help: Option<&'a str>,
}
pub fn render_error(diagnostic: Diagnostic) -> String {
    let digit_count = diagnostic.line.to_string().len();
    format!(
        "{} at:\n{} |\n{} | {}\n{} | {}\n{}",
        diagnostic.headline,
        " ".repeat(digit_count),
        diagnostic.line,
        diagnostic.source_line,
        " ".repeat(digit_count),
        " ".repeat(diagnostic.column as usize - 1) + "^",
        match diagnostic.help {
            Some(help) => format!("help: {}", help),
            None => "".to_string(),
        }
    )
}
