extern crate tempdir;

use std::process::Command;

use tempdir::TempDir;

type ExecutionResult = String;
type CompilationError = String;

#[derive(Copy, Clone)]
enum LineType {
    Value,
    Expression,
}

struct InternalFunction {
    lines_count: i32,
    body: String,
    buffer: Option<String>,
    typ: LineType,
}

fn format_line(line: &str, line_type: LineType) -> String {
    match line_type {
        LineType::Value => {
            format!(
                "if current_line == lines_count - 1 {{ println!(\"{{:?}}\", {{ {} }}); }}\n",
                line
            )
        }
        LineType::Expression => format!("{};\n", line),
    }
}

impl InternalFunction {
    fn new() -> InternalFunction {
        InternalFunction {
            lines_count: 0,
            body: String::new(),
            buffer: None,
            typ: LineType::Value,
        }
    }

    fn clear_buf(mut self) -> InternalFunction {
        self.buffer = None;
        self
    }

    fn match_line_type<'a>(&self, line: &'a str) -> (LineType, &'a str) {
        match self.buffer {
            Some(_) => (self.typ, line),
            None => {
                if let Some(':') = line.chars().next() {
                    (LineType::Value, &line[1..])
                } else {
                    (LineType::Expression, line)
                }
            }
        }
    }

    fn append_line(&self, line: &str) -> InternalFunction {
        let lines = self.buffer.clone().unwrap_or(String::new());
        let (typ, line) = self.match_line_type(line);

        if let Some('.') = line.chars().last() {
            println!("Waiting for more...");
            InternalFunction {
                lines_count: self.lines_count,
                body: self.body.clone(),
                buffer: Some(lines + &line[0..line.len() - 1] + "\n"),
                typ: typ,
            }
        } else {
            InternalFunction {
                lines_count: self.lines_count + 1,
                body: self.body.clone() + "\n" + &format_line((lines + line).as_str(), typ) +
                    "\ncurrent_line += 1;\n",
                buffer: None,
                typ: typ,
            }
        }
    }

    fn file_contents(&self) -> String {
        format!(
            "fn main() {{ let lines_count = {}; let mut current_line = 0; {} }}",
            self.lines_count,
            self.body
        )
    }

    fn buffer_contents(&self) -> String {
        self.buffer.clone().unwrap_or(String::new())
    }

    fn try_execute(&self) -> Result<ExecutionResult, CompilationError> {
        use std::io::Write;
        use std::fs::File;

        let dir = TempDir::new("rustci").unwrap();
        let file_path = dir.path().join("tmp.rs");
        let out_file_path = dir.path().join("tmp_binary");
        let mut file = File::create(&file_path).unwrap();

        write!(&mut file, "{}", self.file_contents()).unwrap();

        let output = Command::new("rustc")
            .arg(&file_path)
            .arg("-o")
            .arg(&out_file_path)
            .output()
            .unwrap();

        if output.status.success() == false {
            let stdout = String::from_utf8(output.stdout).unwrap();
            let stderr = String::from_utf8(output.stderr).unwrap();

            return Err(format!("stdout: {}, stderr: {}", stdout, stderr));
        }

        let output = Command::new(out_file_path).output().unwrap();

        Ok((String::from_utf8(output.stdout).unwrap()))
    }
}

fn main() {
    use std::io;
    use std::io::prelude::*;

    let stdin = io::stdin();

    stdin.lock().lines().fold(
        InternalFunction::new(),
        |func, line| {
            let line = line.unwrap();
            if line == "%" {
                print!(
                    "/** File **/\n{}\n/** Buffer **/\n{}",
                    func.file_contents(),
                    func.buffer_contents()
                );
                return func;
            }
            let newfunc = func.append_line(&line);

            if let Some(_) = newfunc.buffer {
                return newfunc;
            }

            match newfunc.try_execute() {
                Ok(result) => {
                    println!("= {}", result);
                    newfunc
                }
                Err(error) => {
                    println!("ERR {}", error);
                    func.clear_buf()
                }
            }
        },
    );
}
