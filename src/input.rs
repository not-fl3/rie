use rustyline::Editor;
use rustyline::error::ReadlineError;

#[derive(Clone)]
pub enum ReplCommand {
    PrintValue(String),
    AddExpression(String),
    PrintCode,
    Exit,
    Nothing,
    RemoveLines(u32),
}

pub struct Input {
    editor: Editor<()>,
    input_buffer: Option<String>,
    greeting: String,
}

impl Input {
    pub fn new() -> Input {
        Input {
            editor: Editor::<()>::new(),
            input_buffer: None,
            greeting: ">> ".to_string(),
        }
    }

    pub fn read(&mut self) -> ReplCommand {
        let readline = self.editor.readline(&self.greeting);

        match readline {
            Ok(line) => if let Some(buffer) = self.input_buffer.clone() {
                if line.chars().nth(0).map_or(false, |c| c == '}')
                    && line.chars().nth(1).map_or(false, |c| c == '}')
                {
                    self.greeting = ">> ".to_string();
                    self.editor.add_history_entry(&buffer);
                    let cmd = ReplCommand::AddExpression(self.input_buffer.take().unwrap());
                    return cmd;
                } else {
                    self.input_buffer.as_mut().unwrap().push_str(&line);
                    return ReplCommand::Nothing;
                }
            } else {
                match (
                    line.chars().nth(0).map_or(None, Some),
                    line.chars().nth(1).map_or(None, Some),
                ) {
                    (Some('%'), Some('d')) => {
                        let mut splitted = line.split(' ');
                        if let Some(number) = splitted
                            .nth(1)
                            .and_then(|number| number.parse::<u32>().ok())
                        {
                            return ReplCommand::RemoveLines(number);
                        }
                        return ReplCommand::Nothing;
                    }

                    (Some('%'), _) => {
                        self.editor.add_history_entry(&line);
                        return ReplCommand::PrintCode;
                    }
                    (Some(':'), Some(_)) => {
                        self.editor.add_history_entry(&line);
                        return ReplCommand::PrintValue(line[1..].to_string());
                    }
                    (Some('{'), Some('{')) => {
                        self.greeting = ">>> ".to_string();
                        self.input_buffer = Some(String::new());
                        return ReplCommand::Nothing;
                    }
                    (None, _) => return ReplCommand::Nothing,
                    _ => {
                        self.editor.add_history_entry(&line);
                        return ReplCommand::AddExpression(line.to_string());
                    }
                }
            },
            Err(ReadlineError::Interrupted) |
            Err(ReadlineError::Eof) |
            Err(_) => {
                ReplCommand::Exit
            }
        }
    }
}
