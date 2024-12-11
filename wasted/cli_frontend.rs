// cli_frontend.rs
// WASTED

use crate::frontend::Frontend;
use std::io::{self, Write};

#[derive(Clone)]
pub struct CLIFrontend;

impl CLIFrontend {
    
}

impl Frontend for CLIFrontend {

    fn new() -> Self {
        CLIFrontend
    }

    fn display_text(&mut self, text: &str) {
        println!("{}", text);
    }

    fn display_options(&mut self, options: &[String]) -> impl std::future::Future<Output = usize> {
        for (i, option) in options.iter().enumerate() {
            println!("{}: {}", i + 1, option);
        }
        print!("请选择一个选项: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        move {input.trim().parse::<usize>().unwrap_or(1) - 1 as usize}
    }

    fn display_player_status(&mut self, descriptions: &[String]) {
        for desc in descriptions {
            println!("{}", desc);
        }
    }

    fn display_error(&mut self, message: &str) {
        eprintln!("错误: {}", message);
    }
}


// #[derive(Clone)]
// struct EguiFrontend {
//     main_area: LayoutJob,
//     option_area: Vec<LayoutJob>,
//     notify: Notify,
//     selected: Option<usize>,
// }

// impl EguiFrontend {
//     fn new() -> Self { Self {
//         main_area: LayoutJob::default(),
//         option_area: vec![],
//         notify: Notify::new(),
//         selected: None,
//     } }

//     fn select(&mut self,id: usize) {
//         self.selected = Some(id);
//         self.notify.notify_one();
//     }
// }

// impl Frontend for EguiFrontend {
//     fn display_text(&mut self, text: &str) {
//         self.main_area.append(text, 0., TextFormat::default());
//     }

//     fn display_options(&mut self, options: &[String]) -> usize {
//         self.option_area.iter_mut().zip(options).for_each(|(l,r)| {
//             l.append(r, 0., TextFormat { color: Color32::BLUE,..Default::default()});
//         });
//         self.notify.notified().await;
//         self.selected.unwrap()
//     }

//     fn display_player_status(&mut self, descriptions: &[String]) {
//         // todo!()
//     }

//     fn display_error(&mut self, message: &str) {
//         // todo!()
//     }
// }