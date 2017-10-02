extern crate rustyline;
extern crate tempdir;

use std::process::Command;

use tempdir::TempDir;
use rustyline::error::ReadlineError;
use rustyline::Editor;

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
            body: "".to_string(),
        }
    }

    fn append_line(&self, line: &str, line_type: LineType) -> InternalFunction {
        InternalFunction {
            lines_count: self.lines_count + 1,
            body: self.body.clone() + "\n" + &format_line(line, line_type)
                + "\ncurrent_line += 1;\n",
        }
    }

    fn filecontents(&self) -> String {
        format!(
            include_str!("../templates/repl_main.rs"),
            self.lines_count,
            self.body
        )
    }

    fn try_execute(&self) -> Result<ExecutionResult, CompilationError> {
        use std::io::Write;
        use std::fs::File;

        let dir = TempDir::new("rustci").unwrap();
        let file_path = dir.path().join("tmp.rs");
        let out_file_path = dir.path().join("tmp_binary");
        let mut file = File::create(&file_path).unwrap();

        write!(&mut file, "{}", self.filecontents()).unwrap();

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
    function : InternalFunction
}

impl Repl {
    pub fn process_line(&mut self, line : &str) {
        let (line, line_type) = match line.chars().next().unwrap() {
            ':' => (line[1..].to_string(), LineType::Value),
            '%' => {
                println!("{}", self.function.filecontents());
                return;
            }
            _ => (line.to_string(), LineType::Expression),
        };
        let newfunc = self.function.append_line(&line, line_type);

        match newfunc.try_execute() {
            Ok(result) => {
                println!("= {}", result);
                self.function = newfunc;
            }
            Err(error) => {
                println!("ERR {}", error);
            }
        }

    }
}

fn main() {
    let mut repl = Repl { function : InternalFunction::new() };
    let mut rl = Editor::<()>::new();

    loop {
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                repl.process_line(&line);
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
}
