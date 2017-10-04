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
    binary_path: PathBuf,
}

struct InternalFunction {
    body: Vec<String>,
}

impl InternalFunction {
    fn new() -> InternalFunction {
        InternalFunction { body: vec![] }
    }

    fn file_contents(&self) -> String {
        format!(
            "fn main() {{ {}}}",
            self.body
                .iter()
                .fold(String::new(), |acc, str| { acc + str })
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

            return Err(format!(
                "stdout: {}\nstderr: {}\nerrorcode: {:?}",
                stdout,
                stderr,
                output.status
            ));
        }
        return Ok(CompiledFile {
            _temp_dir: dir,
            binary_path: out_file_path,
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

            return Err(format!(
                "stdout: {}\nstderr: {}\nerrorcode: {:?}",
                stdout,
                stderr,
                output.status
            ));
        } else {
            Ok((String::from_utf8(output.stdout).unwrap()))
        }
    }

    fn append_line(&self, line: String) -> InternalFunction {
        let mut body = self.body.clone();
        body.push(line);
        InternalFunction { body: body }
    }
}

struct Repl {
    function: InternalFunction,
}

impl Repl {
    pub fn process_command(&mut self, command: ReplCommand) -> bool {
        match command {
            ReplCommand::PrintCode => {
                let begin = "fn main() {\n".to_owned();
                let end = "}".to_owned();
                let lines = ::std::iter::once(begin)
                    .chain(self.function.body.iter().map(|s| "    ".to_owned() + s))
                    .chain(::std::iter::once(end));

                println!(
                    "{}",
                    lines.enumerate().fold(
                        String::new(),
                        |acc, (i, str)| acc + &format!("{} {}", i, str)
                    )
                );
                true
            }
            ReplCommand::RemoveLines(line) => {
                if line >= 1 {
                    self.function.body.truncate((line - 1) as usize);
                }
                self.process_command(ReplCommand::PrintCode)
            }
            ReplCommand::Nothing => true,
            ReplCommand::Exit => false,
            ReplCommand::AddExpression(line) => {
                let newfunc = self.function.append_line(format!("{};\n", line));

                match newfunc
                    .try_compile()
                    .and_then(|file| newfunc.try_execute(file))
                {
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
            ReplCommand::PrintValue(line) => {
                let newfunc = self.function
                    .append_line(format!("println!(\"{{:?}}\", {{ {} }});", line));

                let _ = newfunc
                    .try_compile()
                    .and_then(|file| newfunc.try_execute(file))
                    .map(|result| println!("= {}", result))
                    .map_err(|err| println!("ERR {}", err));
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
