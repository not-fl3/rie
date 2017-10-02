extern crate tempfile;

use std::process::Command;

type ExecutionResult = String;
type CompilationError = String;

enum LineType {
    Value,
    Expression,
}

struct InternalFunction {
    lines_count: i32,
    body: String,
}

fn format_line(line : &str, line_type: LineType) -> String  {
    match line_type {
        LineType::Value =>  {
            format!("if current_line == lines_count - 1 {{ println!(\"{{:?}}\", {{ {} }}); }}\n", line)
        },
        LineType::Expression => {
            format!("{};\n", line)
        }
    }
}

impl InternalFunction {
    fn new() -> InternalFunction {
        InternalFunction {
            lines_count: 0,
            body: "".to_string(),
        }
    }

    fn append_line(&self, line: &str, line_type: LineType) -> InternalFunction {
        InternalFunction {
            lines_count: self.lines_count + 1,
            body: self.body.clone() + "\n" + &format_line(line, line_type) + "\ncurrent_line += 1;\n",
        }
    }

    fn try_execute(&self) -> Result<ExecutionResult, CompilationError> {
        use std::io::Write;

        let mut file = tempfile::NamedTempFileOptions::new()
            .prefix("tmp")
            .suffix(".rs")
            .rand_bytes(5)
            .create()
            .unwrap();

        write!(
            &mut file,
            "fn main() {{ let lines_count = {}; let mut current_line = 0; {} }}",
            self.lines_count,
            self.body
        ).unwrap();

        println!(
            "fn main() {{ let lines_count = {}; let mut current_line = 0; {} }}",
            self.lines_count,
            self.body
        );

        let output = Command::new("rustc").arg(&file.path()).output().unwrap();

        if output.status.success() == false {
            let stdout = String::from_utf8(output.stdout).unwrap();
            let stderr = String::from_utf8(output.stderr).unwrap();

            return Err(format!("stdout: {}, stderr: {}", stdout, stderr));
        }

        let output = Command::new(format!(
            "./{}",
            file.path().file_stem().unwrap().to_str().unwrap()
        )).output()
            .unwrap();

        Ok((String::from_utf8(output.stdout).unwrap()))
    }
}

fn main() {
    use std::io;
    use std::io::prelude::*;

    let stdin = io::stdin();

    stdin
        .lock()
        .lines()
        .fold(InternalFunction::new(), |func, line| {
            let line = line.unwrap();
            let (line, line_type) = match line.chars().next().unwrap() {
                ':' => {
                    (line[2..].to_string(), LineType::Value)
                },
                _ => (line.clone(), LineType::Expression)
            };
            let newfunc = func.append_line(&line, line_type);

            match newfunc.try_execute() {
                Ok(result) => {
                    println!("= {}", result);
                    newfunc
                }
                Err(error) => {
                    println!("ERR {}", error);
                    func
                }
            }
        });
}