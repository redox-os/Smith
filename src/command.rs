use state::{Editable, Undoable, Saveable, Movement};
use termion::event::Key;
use view::View;
use clipboard::ClipboardContext;

enum State {
    Insert,
    Message(String),
    Prompt(String, String),
    Exit,
}

pub struct Command {
    state: State,
}

impl Command {
    pub fn new() -> Command {
        Command { state: State::Insert }
    }

    pub fn treat_event<T>(&mut self, content: &mut T, view: &mut View, key: Key) -> bool
        where T: Editable + Saveable + Undoable
    {
        match self.state {
            State::Insert => treat_insert_event(content, view, key, &mut self.state),
            State::Prompt(_, _) => treat_prompt_event(content, view, key, &mut self.state),
            State::Message(_) => treat_message_event(content, view, key, &mut self.state),
            State::Exit => panic!("continued after an Exit state"),
        };
        if let State::Exit = self.state {
            return true;
        }
        false
    }
}

fn treat_message_event<T>(content: &mut T, view: &mut View, key: Key, state: &mut State)
    where T: Editable + Saveable + Undoable
{
    view.quiet();
    treat_insert_event(content, view, key, state)
}

fn treat_insert_event<T>(content: &mut T, view: &mut View, key: Key, state: &mut State)
    where T: Editable + Saveable + Undoable
{
    match key {
        Key::Ctrl('q') => {
            *state = State::Exit;
        }
        Key::Ctrl('s') => {
            let prompt = "Save to: ".to_string();
            let message = content.name().clone();
            view.prompt(&prompt, &message);
            *state = State::Prompt(prompt, message);
        }
        key => {
            match key {
                Key::Ctrl('z') => content.undo(),
                Key::Ctrl('y') => content.redo(),
                Key::Ctrl('v') => {
                    let ctx = ClipboardContext::new().unwrap();
                    for c in ctx.get_contents().unwrap().chars() {
                        content.insert(c);
                    }
                }
                Key::Up => content.step(Movement::Up),
                Key::Down => content.step(Movement::Down),
                Key::Left => content.step(Movement::Left),
                Key::Right => content.step(Movement::Right),
                Key::PageUp => content.step(Movement::PageUp(view.lines_height() as usize)),
                Key::PageDown => content.step(Movement::PageDown(view.lines_height() as usize)),
                Key::Home => content.step(Movement::LineStart),
                Key::End => content.step(Movement::LineEnd),
                Key::Backspace => {
                    content.delete();
                }
                Key::Char(c) => content.insert(c),
                _ => {}
            }
            *state = State::Insert;
        }
    }
}
fn treat_prompt_event<T>(content: &mut T, view: &mut View, key: Key, state: &mut State)
    where T: Editable + Saveable + Undoable
{
    match key {
        Key::Char('\n') => {
            let msg: String;
            if let State::Prompt(_, ref mut message) = *state {
                let old_name = content.name().clone();
                content.set_name(message.clone());
                msg = match content.save() {
                    Err(e) => {
                        content.set_name(old_name);
                        e.to_string()
                    }
                    Ok(_) => format!("Saved file {}", content.name()),
                };
                view.message(&msg);
            } else {
                panic!("Treating prompt event when event is not a Prompt");
            }
            *state = State::Message(msg);
        }
        Key::Char(c) => {
            if let State::Prompt(ref prompt, ref mut message) = *state {
                message.push(c);
                view.prompt(prompt, message);
            } else {
                panic!("Treating prompt event when event is not a Prompt");
            }
        }
        Key::Backspace => {
            if let State::Prompt(ref prompt, ref mut message) = *state {
                message.pop();
                view.prompt(prompt, message);
            } else {
                panic!("Treating prompt event when event is not a Prompt");
            }
        }
        Key::Ctrl('q') => {
            *state = State::Exit;
        }
        _ => {}
    }
}
