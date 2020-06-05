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

use file::{AnalyzeResult, File, FileMessage};

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

#[derive(Debug, Clone)]
enum Message {
    EventOccurred(iced_native::Event),
    FileMessage(u64, FileMessage),
    FileAnalyzed(u64, AnalyzeResult),
}

#[derive(Debug, Default)]
struct App {
    id_counter: u64,
    hovering_with_files: bool,
    files: Vec<File>,
    file_scrollable: scrollable::State,
}

impl App {
    fn get_id(&mut self) -> u64 {
        let id = self.id_counter;
        self.id_counter += 1;
        id
    }

    pub fn contains_path(&self, path: &PathBuf) -> bool {
        self.files.iter().find(|file| &file.path == path).is_some()
    }

    pub fn add_path(&mut self, path: PathBuf) -> Vec<Command<Message>> {
        let mut commands = Vec::new();

        if path.is_dir() {
            for entry in WalkDir::new(path) {
                let file_path = entry.unwrap().path().to_owned();

                if is_video(&file_path) && !self.contains_path(&file_path) {
                    let id = self.get_id();

                    commands.push(Command::perform(
                        File::analyze_file(id, file_path.clone()),
                        move |result| Message::FileAnalyzed(id, result),
                    ));

                    self.files.push(File {
                        id,
                        path: file_path,
                        ..Default::default()
                    });
                }
            }
        } else if is_video(&path) && !self.contains_path(&path) {
            let id = self.get_id();

            commands.push(Command::perform(
                File::analyze_file(id.clone(), path.clone()),
                move |result| Message::FileAnalyzed(id, result),
            ));

            self.files.push(File {
                id,
                path,
                ..Default::default()
            });
        }

        commands
    }
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
                    let commands = self.add_path(path);

                    return Command::batch(commands);
                }
                _ => {}
            },
            Message::FileAnalyzed(id, result) => match result {
                Ok(analysis) => {
                    if let Some(file) = self.files.iter_mut().find(|file| file.id == id) {
                        file.update(FileMessage::Analyzed(analysis));
                    }
                }
                Err(_) => {}
            },
            Message::FileMessage(_idx, _msg) => todo!(),
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_native::subscription::events().map(Message::EventOccurred)
    }

    fn view(&mut self) -> Element<Message> {
        let files = self
            .files
            .iter_mut()
            .fold(Column::new().spacing(10), |column, file| {
                let id = file.id;

                column.push(
                    file.view()
                        .map(move |message| Message::FileMessage(id, message)),
                )
            });

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
