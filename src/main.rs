mod item;

use iced::{
	alignment,
	event::{self, Event},
	keyboard::{self, KeyCode, Modifiers},
	subscription,
	theme::Theme,
	widget::{self, button, column, container, row, scrollable, text, text_input},
	window, Alignment, Application, Color, Command, Element, Length, Settings, Subscription,
};
use once_cell::sync::Lazy;

use crate::item::{Item, Message as ItemMessage};

static INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

pub fn main() -> iced::Result {
	App::run(Settings {
		window: window::Settings {
			size: (500, 800),
			..window::Settings::default()
		},
		..Settings::default()
	})
}

#[derive(Debug)]
struct App {
	state: State,
	mode: AppMode,
}

#[derive(Debug)]
enum AppMode {
	List,
	Choose,
}

type ItemsList = Vec<Item>;

#[derive(Debug, Default)]
struct State {
	input_value: String,
	items: Vec<Item>,
}

trait CanCompare {
	fn can_compare(&self) -> bool;
}

impl CanCompare for ItemsList {
	fn can_compare(&self) -> bool {
		self.len() >= 2
	}
}

#[derive(Debug, Clone)]
enum Message {
	ToggleView,
	InputChanged(String),
	CreateTask,
	ItemMessage(usize, ItemMessage),
	TabPressed { shift: bool },
	ToggleFullscreen(window::Mode),
}

impl Application for App {
	type Executor = iced::executor::Default;
	type Flags = ();
	type Message = Message;
	type Theme = Theme;

	fn new(_flags: ()) -> (App, Command<Message>) {
		let state = State {
			input_value: "".into(),
			items: Vec::new(),
		};
		(
			App {
				state,
				mode: AppMode::List,
			},
			Command::none(),
		)
	}

	fn title(&self) -> String {
		"Priority Sorter".into()
	}

	fn update(&mut self, message: Message) -> Command<Message> {
		let state = &mut self.state;
		match message {
			Message::InputChanged(value) => {
				state.input_value = value;

				Command::none()
			},
			Message::CreateTask => {
				if !state.input_value.is_empty() {
					state
						.items
						.insert(state.items.len(), Item::new(state.input_value.clone()));
					state.input_value.clear();
				}

				Command::none()
			},
			Message::ItemMessage(i, ItemMessage::Delete) => {
				state.items.remove(i);

				Command::none()
			},
			Message::ItemMessage(i, item_message) => {
				if let Some(item) = state.items.get_mut(i) {
					let should_focus = matches!(item_message, ItemMessage::Edit);

					item.update(item_message);

					if should_focus {
						let id = Item::text_input_id(&i);
						Command::batch(vec![
							text_input::focus(id.clone()),
							text_input::select_all(id),
						])
					} else {
						Command::none()
					}
				} else {
					Command::none()
				}
			},
			Message::TabPressed { shift } => {
				if shift {
					widget::focus_previous()
				} else {
					widget::focus_next()
				}
			},
			Message::ToggleFullscreen(mode) => window::change_mode(mode),
			Message::ToggleView => {
				match self.mode {
					AppMode::List => {
						if self.state.items.can_compare() {
							self.mode = AppMode::Choose
						}
					},
					AppMode::Choose => self.mode = AppMode::List,
				}
				Command::none()
			},
		}
	}

	fn view(&self) -> Element<Message> {
		let input_value = &self.state.input_value;

		let items = &self.state.items;

		let make_container = |content| {
			scrollable(
				container(content)
					.width(Length::Fill)
					.padding(40)
					.center_x(),
			)
		};

		match self.mode {
			AppMode::List => {
				let title = text("Priorities")
					.width(Length::Fill)
					.size(100)
					.style(Color::from([0.5, 0.5, 0.5]))
					.horizontal_alignment(alignment::Horizontal::Center);

				let input = text_input("What would you like to prioritize?", input_value)
					.id(INPUT_ID.clone())
					.on_input(Message::InputChanged)
					.on_submit(Message::CreateTask)
					.padding(15)
					.size(30);

				let items_list: Element<_> = column(
					items
						.iter()
						.enumerate()
						.map(|(i, item)| {
							item.view(i)
								.map(move |message| Message::ItemMessage(i, message))
						})
						.collect(),
				)
				.spacing(10)
				.into();

				let content = if self.state.items.can_compare() {
					column![
						title,
						input,
						button("Sort Items").on_press(Message::ToggleView),
						items_list
					]
				} else {
					column![title, input, items_list]
				}
				.spacing(20)
				.max_width(800);

				make_container(content).into()
			},
			AppMode::Choose => {
				let prompt_text = text("Which one is higher priority?")
					.width(Length::Fill)
					.size(48)
					.style(Color::from([0.5, 0.5, 0.5]))
					.horizontal_alignment(alignment::Horizontal::Center);

				let choices = container(
					row![button("choice a"), button("choice b")]
						.spacing(40)
						.align_items(Alignment::Start)
						.width(Length::Fill),
				)
				.center_x();

				let content = column![prompt_text, choices]
					.align_items(Alignment::Center)
					.spacing(60)
					.width(Length::Fill)
					.max_width(800);

				make_container(content).into()
			},
		}
	}

	fn subscription(&self) -> Subscription<Message> {
		subscription::events_with(|event, status| match (event, status) {
			(
				Event::Keyboard(keyboard::Event::KeyPressed {
					key_code: keyboard::KeyCode::Tab,
					modifiers,
					..
				}),
				event::Status::Ignored,
			) => Some(Message::TabPressed {
				shift: modifiers.shift(),
			}),
			(
				Event::Keyboard(keyboard::Event::KeyPressed {
					key_code,
					modifiers: Modifiers::SHIFT,
				}),
				event::Status::Ignored,
			) => match key_code {
				KeyCode::Up => Some(Message::ToggleFullscreen(window::Mode::Fullscreen)),
				KeyCode::Down => Some(Message::ToggleFullscreen(window::Mode::Windowed)),
				_ => None,
			},
			_ => None,
		})
	}
}
