use std::path::PathBuf;

use iced::{
    executor, scrollable, Application, Column, Command, Container, Element, Length, Scrollable,
    Settings, Subscription,
};
use iced_native::window::Event as WindowEvent;
use iced_native::Event;
use walkdir::WalkDir;

mod file;
mod styles;

use file::{File, FileMessage, FileState};

fn is_video(path: &PathBuf) -> bool {
    let guess = mime_guess::from_path(path);

    match guess.first() {
        Some(guess) => guess.to_string().starts_with("video/"),
        None => false,
    }
}

pub fn main() {
    let mut settings = Settings::default();

    settings.default_font = Some(include_bytes!("../fonts/SourceCodePro-Regular.ttf"));

    App::run(settings)
}

#[derive(Debug, Default)]
struct App {
    hovering_with_files: bool,
    files: Vec<File>,
    file_scrollable: scrollable::State,
}

impl App {
    pub fn add_path(&mut self, path: PathBuf) {
        if path.is_dir() {
            for entry in WalkDir::new(path) {
                let file_path = entry.unwrap().path().to_owned();

                if is_video(&file_path) {
                    self.files.push(File {
                        path: file_path,
                        state: FileState::default(),
                    });
                }
            }
        } else if is_video(&path) {
            self.files.push(File {
                state: FileState::default(),
                path,
            });
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    EventOccurred(iced_native::Event),
    FileMessage(usize, FileMessage),
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (App, Command<Message>) {
        (App::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("mx")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::EventOccurred(event) => match event {
                Event::Window(WindowEvent::FileHovered(_)) => {
                    self.hovering_with_files = true;
                }
                Event::Window(WindowEvent::FilesHoveredLeft) => {
                    self.hovering_with_files = false;
                }
                Event::Window(WindowEvent::FileDropped(path)) => {
                    self.hovering_with_files = false;
                    self.add_path(path);
                }
                _ => {}
            },
            Message::FileMessage(_idx, _msg) => todo!(),
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_native::subscription::events().map(Message::EventOccurred)
    }

    fn view(&mut self) -> Element<Message> {
        let files = self.files.iter_mut().enumerate().fold(
            Column::new().spacing(10),
            |column, (i, file)| {
                column.push(
                    file.view()
                        .map(move |message| Message::FileMessage(i, message)),
                )
            },
        );

        let content = Scrollable::new(&mut self.file_scrollable)
            .width(Length::Fill)
            .push(files);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(styles::Container {
                hovered: self.hovering_with_files,
            })
            .into()
    }
}
