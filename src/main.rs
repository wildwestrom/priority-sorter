mod item;

use iced::{
	alignment::{self},
	event::{self, Event},
	keyboard::{self, KeyCode, Modifiers},
	subscription,
	theme::Theme,
	widget::{self, button, column, container, scrollable, text, text_input},
	window, Application, Color, Command, Element, Length, Settings, Subscription,
};
use once_cell::sync::Lazy;

use crate::item::{Item, Message as ItemMessage};

static INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

pub fn main() -> iced::Result {
	TopLevel::run(Settings {
		window: window::Settings {
			size: (500, 800),
			..window::Settings::default()
		},
		..Settings::default()
	})
}

#[derive(Debug)]
struct TopLevel {
	state: State,
}

#[derive(Debug, Default)]
struct State {
	input_value: String,
	items: Vec<Item>,
}

#[derive(Debug, Clone)]
enum Message {
	InputChanged(String),
	CreateTask,
	ItemMessage(usize, ItemMessage),
	TabPressed { shift: bool },
	ToggleFullscreen(window::Mode),
}

impl Application for TopLevel {
	type Executor = iced::executor::Default;
	type Flags = ();
	type Message = Message;
	type Theme = Theme;

	fn new(_flags: ()) -> (TopLevel, Command<Message>) {
		let state = State {
			input_value: "".into(),
			items: Vec::new(),
		};
		(TopLevel { state }, Command::none())
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
		}
	}

	fn view(&self) -> Element<Message> {
		let input_value = &self.state.input_value;

		let items = &self.state.items;
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

		let content = if self.state.items.len() > 1 {
			column![title, input, button("Sort items?"), items_list]
		} else {
			column![title, input, items_list]
		}
		.spacing(20)
		.max_width(800);

		scrollable(
			container(content)
				.width(Length::Fill)
				.padding(40)
				.center_x(),
		)
		.into()
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
