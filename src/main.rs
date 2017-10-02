extern crate rustyline;
extern crate tempdir;

mod input;

use std::process::Command;

use tempdir::TempDir;

use input::{Input, ReplCommand};

type ExecutionResult = String;
type CompilationError = String;


struct InternalFunction {
    lines_count: i32,
    body: String,
    buffer: Option<String>,
}

fn format_line(command: ReplCommand) -> String {
    match command {
        ReplCommand::PrintValue(line) => {
            format!(include_str!("../templates/repl_print_value.rs"), line)
        }
        ReplCommand::AddExpression(line) => format!("{};\n", line),
        _ => {
            panic!("Unsupported command");
        }
    }
}

impl InternalFunction {
    fn new() -> InternalFunction {
        InternalFunction {
            lines_count: 0,
            body: String::new(),
            buffer: None,
        }
    }

    fn append_line(&self, command: ReplCommand) -> InternalFunction {
        InternalFunction {
            lines_count: self.lines_count + 1,
            body: self.body.clone() + "\n" + &format_line(command) + "\ncurrent_line += 1;\n",
            buffer: None,
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
    pub fn process_command(&mut self, command: ReplCommand) -> bool {
        match command {
            ReplCommand::PrintCode => {
                print!(
                    "/** File **/\n{}\n/** Buffer **/\n{}",
                    self.function.file_contents(),
                    self.function.buffer_contents()
                );
                true
            }
            ReplCommand::Nothing => true,
            ReplCommand::Exit => false,
            _ => {
                let newfunc = self.function.append_line(command);
                match newfunc.try_execute() {
                    Ok(result) => {
                        println!("= {}", result);
                        self.function = newfunc
                    }
                    Err(error) => {
                        println!("ERR {}", error);
                    }
                }
                true
            }
        }
    }
}

fn main() {
    let mut repl = Repl {
        function: InternalFunction::new(),
    };
    let mut input = Input::new();

    loop {
        let command = input.read();
        if repl.process_command(command) == false {
            break;
        }
    }
}
