use ratatui::{DefaultTerminal, Frame};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Widget};
use dotenvy::dotenv;
use std::{env};

mod api;
use api::ai;
use api::roblox;

use crate::api::roblox::AppState;



#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    dotenv().ok();
    color_eyre::install()?;
    
    let mut terminal = ratatui::init();
    let result = app(&mut terminal).await;
    ratatui::restore();

    result?;
    Ok(())
}

async fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut input_text = String::new();
    let mut view_memory: Vec<String> = Vec::new();
    let mut current_pos : i64 = 0; // pos from right side
    let state = roblox::run_server();


    let api_key = env::var("GEMINI_API_KEY").expect("no gemini key was provided");

    loop {
        terminal.draw(|frame| render(frame, &input_text, &view_memory, &current_pos))?;
        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            if key.kind == crossterm::event::KeyEventKind::Press {
                match key.code {
                    crossterm::event::KeyCode::Esc => break Ok(()),
                    crossterm::event::KeyCode::Char(c) => {
                        input_text.push(c);
                    }

                    crossterm::event::KeyCode::Right => {
                        if current_pos > 0 { current_pos -= 1;}
                    } 
                    crossterm::event::KeyCode::Left => {
                        current_pos += 1;
                    }     
                    crossterm::event::KeyCode::Backspace => {
                        input_text.pop();
                    }
                    crossterm::event::KeyCode::Enter => {
                        if input_text.replace(" ", "") == "" { continue; }
                        view_memory.push(format!("User: {}", input_text.clone()));

                        for line in parse_command(&input_text, &state, &api_key).await.split("<|NL|>") {
                            view_memory.push(line.to_string());
                        }

                        current_pos = 0;
                        input_text.clear();
                    }
                    _ => {}
                }
            }
        }
    }
}


async fn parse_command(current_input: &String, state: &AppState, api_key: &String) -> String {
    // path if we are trying to pass a command
    if let Some(first_char) = current_input.chars().next() {
        if first_char == '!' {
            let command = &current_input[1..];

            let result = state.send(command.to_string()).await;

            if let Some(response) = result {
                return response;
            }
        }
    }

    // path if its question to ai
    match ai::ask_ai(api_key.clone(), current_input.clone()).await {
        Ok(res) => {
            return res;
        },
        Err(e) => eprintln!("Something went wrong!! {}", e),
    }

    return String::new();
}


fn render_string(text: String, current_pos: &i64, inner_area: Rect, y_pos: u16, buf: &mut ratatui::prelude::Buffer) {
    let allowed_text = inner_area.width.saturating_sub(2) as usize;
    let full_len = text.chars().count();
    let current_pos = (*current_pos as usize).min(full_len);
    let end = full_len.saturating_sub(current_pos);
    let start = end.saturating_sub(allowed_text);

    let visible: String = text
        .chars()
        .skip(start)
        .take(allowed_text)
        .collect();

    buf.set_string(
        inner_area.x + 1,
        inner_area.y + y_pos,
        visible,
        ratatui::style::Style::default(),
    );

    if start > 0 {
        buf.set_string(
            inner_area.x + 1,
            inner_area.y + y_pos,
            "...",
            ratatui::style::Style::default(),
        );
    }
    if start + allowed_text < full_len {
        buf.set_string(
            inner_area.x + inner_area.width - 4,
            inner_area.y + y_pos,
            "... │",
            ratatui::style::Style::default(),
        );
    }
}

struct InputBox<'a> {
    text: String,
    current_pos: &'a i64,
}
impl<'a> Widget for InputBox<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let block = Block::bordered();
        let inner_area = block.inner(area);
        block.render(area, buf);

        render_string(self.text, self.current_pos, inner_area, 0, buf);
    }
}

struct ViewBox<'a> {
    memory: &'a Vec<String>,
}
impl<'a> Widget for ViewBox<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let block = Block::bordered().title(" Roslop ai assistant at your service Sir... ");
        let inner_area = block.inner(area);
        block.render(area, buf);
        
        for (i, text) in self.memory.iter().rev().enumerate() {
            if i as u16 > inner_area.height - 1 { break; }

            let line = format!("{}", text);
            let line_length = line.chars().count();
            render_string(line, &(line_length as i64), inner_area, inner_area.height - 1 - i as u16, buf);
        }
    }    
}


fn render(frame: &mut Frame, input_text: &str, view_memory: &Vec<String>, current_pos: &i64) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .split(frame.area());
    
    let view = ViewBox {memory: view_memory};
    frame.render_widget(view, chunks[0]);
    
    let input = InputBox {
        text: input_text.to_string(), 
        current_pos: current_pos,
    };
    frame.render_widget(input, chunks[1]);
}