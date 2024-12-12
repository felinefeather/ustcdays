// cli_frontend.rs

use crate::frontend::Frontend;
use std::io::{self, Write};

pub struct CLIFrontend;

impl CLIFrontend {
    pub fn new() -> Self {
        CLIFrontend
    }
}

impl Frontend for CLIFrontend {
    fn display_text(&self, text: &str) {
        println!("{}", text);
    }

    fn display_options(&self, options: &[String]) -> usize {
        for (i, option) in options.iter().enumerate() {
            println!("{}: {}", i + 1, option);
        }
        print!("请选择一个选项: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input.trim().parse::<usize>().unwrap_or(1) - 1
    }

    fn display_player_status(&self, descriptions: &[String]) {
        for desc in descriptions {
            println!("{}", desc);
        }
    }

    fn display_error(&self, message: &str) {
        eprintln!("错误: {}", message);
    }
}
