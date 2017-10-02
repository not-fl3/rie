extern crate rustyline;
extern crate tempdir;

mod input;

use std::path::PathBuf;
use std::process::Command;

use tempdir::TempDir;

use input::{Input, ReplCommand};

type ExecutionResult = String;
type CompilationError = String;
type RuntimeError = String;

struct CompiledFile {
    _temp_dir: TempDir,
    binary_path: PathBuf
}

struct InternalFunction {
    lines_count: i32,
    body: String,
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
        }
    }

    fn append_line(&self, command: ReplCommand) -> InternalFunction {
        InternalFunction {
            lines_count: self.lines_count + 1,
            body: self.body.clone() + "\n" + &format_line(command) + "\ncurrent_line += 1;\n",
        }
    }

    fn file_contents(&self) -> String {
        format!(
            include_str!("../templates/repl_main.rs"),
            self.lines_count,
            self.body
        )
    }

    fn try_compile(&self) -> Result<CompiledFile, RuntimeError> {
        use std::io::Write;
        use std::fs::File;

        let source_filename = "tmp.rs";
        let binary_filename = "tmp_binary";
        let dir = TempDir::new("rustci").unwrap();
        let file_path = dir.path().join(source_filename);
        let out_file_path = dir.path().join(binary_filename);
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

            return Err(format!("stdout: {}\n, stderr: {}\n, errorcode: {:?}", stdout, stderr, output.status));
        }
        return Ok(CompiledFile {
            _temp_dir: dir,
            binary_path: out_file_path
        });
    }

    fn try_execute(
        &self,
        compiled_file: CompiledFile,
    ) -> Result<ExecutionResult, CompilationError> {
        let output = Command::new(compiled_file.binary_path).output().unwrap();

        if output.status.success() == false {
            let stdout = String::from_utf8(output.stdout).unwrap();
            let stderr = String::from_utf8(output.stderr).unwrap();

            return Err(format!("stdout: {}\n, stderr: {}\n, errorcode: {:?}", stdout, stderr, output.status));
        } else {
            Ok((String::from_utf8(output.stdout).unwrap()))
        }
    }
}

struct Repl {
    function: InternalFunction,
}

impl Repl {
    pub fn process_command(&mut self, command: ReplCommand) -> bool {
        match command {
            ReplCommand::PrintCode => {
                print!("/** File **/\n{}", self.function.file_contents(),);
                true
            }
            ReplCommand::Nothing => true,
            ReplCommand::Exit => false,
            _ => {
                let newfunc = self.function.append_line(command);
                match newfunc.try_compile().and_then(|file| newfunc.try_execute(file)) {
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
