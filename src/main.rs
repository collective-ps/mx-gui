use std::path::PathBuf;

use iced::{
    button, executor, scrollable, Align, Application, Button, Color, Column, Command, Container,
    Element, Length, Row, Scrollable, Settings, Subscription, Text, VerticalAlignment,
};
use iced_native::window::Event as WindowEvent;
use iced_native::Event;
use walkdir::WalkDir;

mod api;
mod message;
mod scenes;
mod styles;
mod widgets;

use api::{Config, User};
use message::{Filter, Message};
use scenes::{Scenes, WelcomeScene};
use widgets::file::{self, File, FileMessage, FileState};

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
    id_counter: u64,
    hovering_with_files: bool,
    files: Vec<File>,
    file_scrollable: scrollable::State,

    current_filter: Filter,

    pending_button: button::State,
    duplicate_button: button::State,
    completed_button: button::State,
    failed_button: button::State,

    // API
    current_user: Option<User>,
    current_config: Option<Config>,

    // Scenes
    current_scene: Scenes,

    // Scenes::Welcome
    welcome_scene: WelcomeScene,
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

    pub fn pending(&self) -> Vec<&File> {
        self.files
            .iter()
            .filter(|file| {
                vec![
                    FileState::Analyzing,
                    FileState::Analyzed,
                    FileState::Pending,
                    FileState::CheckingDuplicate,
                    FileState::Uploading,
                ]
                .contains(&file.state)
            })
            .collect()
    }

    pub fn duplicate(&self) -> Vec<&File> {
        self.files
            .iter()
            .filter(|file| file.state == FileState::Duplicate)
            .collect()
    }

    pub fn completed(&self) -> Vec<&File> {
        self.files
            .iter()
            .filter(|file| file.state == FileState::Completed)
            .collect()
    }

    pub fn failed(&self) -> Vec<&File> {
        self.files
            .iter()
            .filter(|file| file.state == FileState::Failed)
            .collect()
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
                    if self.current_scene == Scenes::FileIndex {
                        self.hovering_with_files = false;
                        let commands = self.add_path(path);

                        return Command::batch(commands);
                    }
                }
                _ => {}
            },
            Message::FileAnalyzed(id, result) => match result {
                Ok(analysis) => {
                    if let Some(file) = self.files.iter_mut().find(|file| file.id == id) {
                        file.update(FileMessage::Analyzed(analysis));
                    }

                    let all_files_analyzed = self
                        .files
                        .iter()
                        .all(|file| file.state != FileState::Analyzing);

                    if all_files_analyzed {
                        let checksums: Vec<String> = self
                            .files
                            .iter()
                            .filter(|file| file.state == FileState::Analyzed)
                            .map(|file| file.get_md5())
                            .collect();
                        let requested_checksums = checksums.clone();
                        let config = self.current_config.clone().unwrap();

                        return Command::perform(
                            async move { api::Checksums::check(&requested_checksums, &config).await },
                            move |response| match response {
                                Ok(response) => Message::DuplicateCheckResponse(
                                    checksums.clone(),
                                    response.checksums,
                                ),
                                Err(_) => Message::Noop,
                            },
                        );
                    }
                }
                Err(_) => {}
            },
            Message::Noop => {}
            Message::FileMessage(id, message) => {
                if let Some(file) = self.files.iter_mut().find(|file| file.id == id) {
                    file.update(message);
                }
            }
            Message::WelcomeMessage(msg) => {
                return self.welcome_scene.update(msg);
            }
            Message::SetConfigAndUser(config, user) => {
                self.current_config = Some(config);
                self.current_user = Some(user);
                self.current_scene = Scenes::FileIndex;
            }
            Message::DuplicateCheckResponse(checksums, duplicate_checksums) => {
                for file in self.files.iter_mut() {
                    let file_checksum = file.get_md5();
                    let part_of_original_request = checksums
                        .iter()
                        .find(|checksum| file_checksum == **checksum)
                        .is_some();
                    let is_duplicate = duplicate_checksums
                        .iter()
                        .find(|checksum| file_checksum == **checksum)
                        .is_some();

                    if part_of_original_request {
                        if is_duplicate {
                            file.state = FileState::Duplicate;
                        } else {
                            file.state = FileState::Pending;
                        }
                    }
                }
            }
            Message::SetFilter(filter) => {
                self.current_filter = filter;
            }
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_native::subscription::events().map(Message::EventOccurred)
    }

    fn view(&mut self) -> Element<Message> {
        match self.current_scene {
            Scenes::Welcome => self.welcome_scene.view().map(Message::WelcomeMessage),
            Scenes::FileIndex => {
                let pending = self.pending();
                let duplicate = self.duplicate();
                let completed = self.completed();
                let failed = self.failed();
                let pending_count = pending.len();
                let duplicate_count = duplicate.len();
                let completed_count = completed.len();
                let failed_count = failed.len();
                let is_empty = self.files.is_empty();

                let files = match self.current_filter {
                    Filter::Pending => pending,
                    Filter::Duplicate => duplicate,
                    Filter::Completed => completed,
                    Filter::Failed => failed,
                };

                let file_index = file::file_index(files);

                let file_scroll_view = Scrollable::new(&mut self.file_scrollable)
                    .width(Length::Fill)
                    .height(Length::FillPortion(5))
                    .push(file_index);

                let bottom_bar = Container::new(
                    styles::text(format!(
                        "Logged in as: {}",
                        self.current_user.as_ref().unwrap().username
                    ))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .vertical_alignment(VerticalAlignment::Center),
                )
                .height(Length::Units(30))
                .width(Length::Fill)
                .padding(6)
                .style(styles::Container::Secondary);

                let filter_bar = Row::new()
                    .width(Length::Fill)
                    .height(Length::Units(30))
                    .spacing(24)
                    .push(
                        Button::new(
                            &mut self.pending_button,
                            styles::text(format!("Pending ({})", pending_count)),
                        )
                        .on_press(Message::SetFilter(Filter::Pending))
                        .style(styles::Button::Transparent),
                    )
                    .push(
                        Button::new(
                            &mut self.duplicate_button,
                            styles::text(format!("Duplicate ({})", duplicate_count)),
                        )
                        .on_press(Message::SetFilter(Filter::Duplicate))
                        .style(styles::Button::Transparent),
                    )
                    .push(
                        Button::new(
                            &mut self.completed_button,
                            styles::text(format!("Completed ({})", completed_count)),
                        )
                        .on_press(Message::SetFilter(Filter::Completed))
                        .style(styles::Button::Transparent),
                    )
                    .push(
                        Button::new(
                            &mut self.failed_button,
                            styles::text(format!("Failed ({})", failed_count)),
                        )
                        .on_press(Message::SetFilter(Filter::Failed))
                        .style(styles::Button::Transparent),
                    );

                let top_view = Row::new()
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(6)
                    .push(file_scroll_view);

                let content = if is_empty {
                    Column::new()
                        .push(
                            Container::new(
                                Text::new("Drag and drop files here").color(Color::WHITE),
                            )
                            .height(Length::FillPortion(5))
                            .width(Length::Fill)
                            .center_x()
                            .center_y(),
                        )
                        .push(bottom_bar)
                        .align_items(Align::Center)
                } else {
                    Column::new()
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .push(filter_bar)
                        .push(top_view)
                        .push(bottom_bar)
                };

                Container::new(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(styles::HoveredContainer::new(self.hovering_with_files))
                    .center_x()
                    .center_y()
                    .into()
            }
        }
    }
}
