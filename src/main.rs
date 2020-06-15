use std::path::PathBuf;

use iced::{
    button, executor, scrollable, text_input, Align, Application, Button, Color, Column, Command,
    Container, Element, Length, Row, Scrollable, Settings, Subscription, Text, TextInput,
    VerticalAlignment,
};
use iced_native::input::keyboard::{Event as KeyboardEvent, KeyCode};
use iced_native::input::ButtonState;
use iced_native::window::Event as WindowEvent;
use iced_native::Event;
use walkdir::WalkDir;

mod api;
mod config;
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

#[derive(Debug, PartialEq)]
pub enum FileSelection {
    None,
    Single(usize),
    Multiple(Vec<usize>),
}

impl Default for FileSelection {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Default)]
struct App {
    id_counter: u64,
    hovering_with_files: bool,
    files: Vec<File>,
    file_scrollable: scrollable::State,

    current_filter: Filter,

    pending_button: button::State,
    queued_button: button::State,
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

    // Is left shift pressed?
    left_shift: bool,

    // Is left control pressed?
    left_control: bool,

    file_selection: FileSelection,
    enqueue_button: button::State,
    tag_input: text_input::State,
    tags: String,
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

    pub fn pending(&mut self) -> Vec<&mut File> {
        self.files
            .iter_mut()
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

    pub fn duplicate(&mut self) -> Vec<&mut File> {
        self.files
            .iter_mut()
            .filter(|file| file.state == FileState::Duplicate)
            .collect()
    }

    pub fn completed(&mut self) -> Vec<&mut File> {
        self.files
            .iter_mut()
            .filter(|file| file.state == FileState::Completed)
            .collect()
    }

    pub fn failed(&mut self) -> Vec<&mut File> {
        self.files
            .iter_mut()
            .filter(|file| file.state == FileState::Failed)
            .collect()
    }

    pub fn queued(&mut self) -> Vec<&mut File> {
        self.files
            .iter_mut()
            .filter(|file| file.state == FileState::Queued)
            .collect()
    }
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (App, Command<Message>) {
        let api_key = config::read_api_key().ok();

        let cmd = match api_key {
            Some(api_key) => {
                let config = api::Config::new(api_key.clone());

                Command::perform(
                    async move {
                        let config = api::Config::new(api_key.clone());
                        let response = api::User::get(&config).await;
                        response
                    },
                    move |resp| match resp {
                        Ok(user) => Message::SetConfigAndUser(config.clone(), user),
                        Err(e) => Message::WelcomeMessage(scenes::WelcomeMessage::SetDisplayError(
                            e.to_string(),
                        )),
                    },
                )
            }
            None => Command::none(),
        };

        (App::default(), cmd)
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
                Event::Keyboard(KeyboardEvent::Input {
                    state, key_code, ..
                }) => {
                    if key_code == KeyCode::LShift {
                        match state {
                            ButtonState::Pressed => {
                                self.left_shift = true;
                            }
                            ButtonState::Released => {
                                self.left_shift = false;
                            }
                        }
                    } else if key_code == KeyCode::LControl {
                        match state {
                            ButtonState::Pressed => {
                                self.left_control = true;
                            }
                            ButtonState::Released => {
                                self.left_control = false;
                            }
                        }
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
                let _ = config::write_api_key(&config.api_token);

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
                self.file_selection = FileSelection::None;
            }
            Message::SelectFile(selected_idx) => match &self.file_selection {
                FileSelection::None => {
                    self.file_selection = FileSelection::Single(selected_idx);
                }
                FileSelection::Single(first_idx) => {
                    if *first_idx == selected_idx {
                        self.file_selection = FileSelection::None;
                        return Command::none();
                    }

                    if self.left_shift {
                        let min = std::cmp::min(first_idx, &selected_idx);
                        let max = std::cmp::max(first_idx, &selected_idx) + 1;
                        let selection = (*min..max).collect();
                        self.file_selection = FileSelection::Multiple(selection);
                    } else if self.left_control {
                        if *first_idx != selected_idx {
                            self.file_selection =
                                FileSelection::Multiple(vec![*first_idx, selected_idx]);
                        }
                    } else {
                        self.file_selection = FileSelection::Single(selected_idx);
                    }
                }
                FileSelection::Multiple(indices) => {
                    if self.left_shift {
                        let first_idx = indices.first().unwrap();
                        let min = std::cmp::min(first_idx, &selected_idx);
                        let max = std::cmp::max(first_idx, &selected_idx) + 1;
                        let selection = (*min..max).collect();
                        self.file_selection = FileSelection::Multiple(selection);
                    } else if self.left_control {
                        if !indices.contains(&selected_idx) {
                            self.file_selection = FileSelection::Multiple(
                                [indices.as_slice(), &[selected_idx]].concat(),
                            );
                        } else {
                            self.file_selection = FileSelection::Multiple(
                                indices
                                    .into_iter()
                                    .filter(|idx| **idx != selected_idx)
                                    .map(|idx| *idx)
                                    .collect(),
                            );
                        }
                    } else {
                        self.file_selection = FileSelection::Single(selected_idx);
                    }
                }
            },
            Message::SetTags(tags) => {
                self.tags = tags;
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
                let pending_count = self.pending().len();
                let queued_count = self.queued().len();
                let completed_count = self.completed().len();
                let duplicate_count = self.duplicate().len();
                let failed_count = self.failed().len();

                let is_empty = self.files.is_empty();

                let current_filter = self.current_filter;
                let files = self
                    .files
                    .iter_mut()
                    .filter(|file| current_filter.states().contains(&file.state))
                    .collect();

                let file_index = file::file_index(&self.file_selection, files);

                let file_scroll_view = Scrollable::new(&mut self.file_scrollable)
                    .width(Length::Fill)
                    .height(Length::FillPortion(5))
                    .push(file_index);

                let mut bottom_bar = Row::new()
                    .push(
                        styles::text(format!(
                            "Logged in as: {}",
                            self.current_user.as_ref().unwrap().username
                        ))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .vertical_alignment(VerticalAlignment::Center),
                    )
                    .spacing(3);

                let file_form = Row::new()
                    .push(TextInput::new(
                        &mut self.tag_input,
                        "Enter in tags",
                        &self.tags,
                        Message::SetTags,
                    ))
                    .push(
                        Button::new(&mut self.enqueue_button, styles::text("Add to Queue"))
                            .padding(0),
                    );

                bottom_bar = match self.file_selection {
                    FileSelection::Single(_) => bottom_bar
                        .push(file_form)
                        .push(styles::text("1 file selected")),
                    FileSelection::Multiple(ref indices) => bottom_bar
                        .push(file_form)
                        .push(styles::text(format!("{} files selected", indices.len()))),
                    FileSelection::None => bottom_bar,
                };

                let bottom_bar_container = Container::new(bottom_bar)
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
                            &mut self.queued_button,
                            styles::text(format!("Queued ({})", queued_count)),
                        )
                        .on_press(Message::SetFilter(Filter::Queued))
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
                            &mut self.duplicate_button,
                            styles::text(format!("Duplicate ({})", duplicate_count)),
                        )
                        .on_press(Message::SetFilter(Filter::Duplicate))
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
                        .push(bottom_bar_container)
                        .align_items(Align::Center)
                } else {
                    Column::new()
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .push(filter_bar)
                        .push(top_view)
                        .push(bottom_bar_container)
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
