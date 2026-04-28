use super::render_error::{Diagnostic, render_error};

#[derive(Debug, thiserror::Error)]
pub enum PreprocessorError {
    #[error("{message}")]
    IncludeError { message: String },
}

pub struct Preprocessor {}

impl Preprocessor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn preprocess(&self, program: &str) -> Result<String, PreprocessorError> {
        Ok(program
            .lines()
            .enumerate()
            .map(|(i, mut line)| {
                line = line.trim();
                if line.starts_with(".include") {
                    let statement = line.split(" ").collect::<Vec<&str>>();
                    if statement.len() != 2 {
                        return Err(PreprocessorError::IncludeError {
                            message: render_error(Diagnostic {
                                headline: "Invalid include statement".to_string(),
                                source_line: line,
                                line: (i + 1) as u32,
                                column: 1,
                                help: None,
                            }),
                        });
                    }

                    // enforce double quotes
                    if !statement[1].starts_with("\"") || !statement[1].ends_with("\"") {
                        return Err(PreprocessorError::IncludeError {
                            message: render_error(Diagnostic {
                                headline: "Invalid include statement".to_string(),
                                source_line: line,
                                line: (i + 1) as u32,
                                column: ".include ".len() as u32,
                                help: None,
                            }),
                        });
                    }

                    // remove double quotes
                    let file_path = statement[1][1..statement[1].len() - 1].to_string();

                    // enforce .asm extension
                    if !file_path.ends_with(".asm") {
                        return Err(PreprocessorError::IncludeError {
                            message: render_error(Diagnostic {
                                headline: "Invalid file path in include statement".to_string(),
                                source_line: line,
                                line: (i + 1) as u32,
                                column: ".include ".len() as u32,
                                help: None,
                            }),
                        });
                    }

                    Ok(match std::fs::read_to_string(&format!("programs/{file_path}")) {
                        Ok(file) => file,
                        Err(e) => {
                            return Err(PreprocessorError::IncludeError {
                                message: render_error(Diagnostic {
                                    headline: format!("Failed to read file: {}", e),
                                    source_line: line,
                                    line: (i + 1) as u32,
                                    column: 1,
                                    help: None,
                                }),
                            });
                        }
                    })
                } else {
                    Ok(line.to_string())
                }
            })
            .collect::<Result<Vec<String>, PreprocessorError>>()?
            .join("\n")
            .to_string())
    }
}
