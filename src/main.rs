extern crate rustyline;
extern crate tempdir;

use std::process::Command;

use tempdir::TempDir;
use rustyline::error::ReadlineError;
use rustyline::Editor;

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
        LineType::Value => format!(include_str!("../templates/repl_print_value.rs"), line),
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

    fn clear_buf(&mut self) {
        self.buffer = None
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
            include_str!("../templates/repl_main.rs"),
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

struct Repl {
    function: InternalFunction,
}

impl Repl {
    pub fn process_line(&mut self, line: &str) {
        if line == "%" {
            print!(
                "/** File **/\n{}\n/** Buffer **/\n{}",
                self.function.file_contents(),
                self.function.buffer_contents()
            );
            return;
        }
        let newfunc = self.function.append_line(&line);

        if let Some(_) = newfunc.buffer {
            self.function = newfunc;
            return;
        }

        match newfunc.try_execute() {
            Ok(result) => {
                println!("= {}", result);
                self.function = newfunc
            }
            Err(error) => {
                println!("ERR {}", error);
                self.function.clear_buf()
            }
        }
    }
}

fn main() {
    let mut repl = Repl { function: InternalFunction::new() };
    let mut rl = Editor::<()>::new();

    loop {
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                repl.process_line(&line);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
