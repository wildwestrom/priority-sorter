use iced::{
	alignment,
	event::{self, Event},
	keyboard::{self, KeyCode, Modifiers},
	subscription,
	theme::Theme,
	widget::{
		self, button, column, container, row, scrollable, text, text_input, Scrollable, Text,
	},
	window, Alignment, Application, Color, Command, Element, Length, Subscription,
};
use once_cell::sync::Lazy;

use crate::{
	item::{Item, ItemsList, Message as ItemMessage},
	sorter::{SortState, Sorter},
};

static INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

pub struct App {
	state: State,
	mode: AppMode,
}
struct State {
	input_value: String,
	items: ItemsList,
	sorter: Sorter<Item>,
}

#[derive(Debug, Clone)]
enum AppMode {
	List,
	Choose,
}

#[derive(Debug, Clone)]
pub enum Message {
	SortItems,
	ListView,
	InputChanged(String),
	CreateItem,
	ItemMessage(usize, ItemMessage),
	TabPressed { shift: bool },
	ToggleFullscreen(window::Mode),
	ChooseItem(Box<Item>),
}

impl Application for App {
	type Executor = iced::executor::Default;
	type Flags = ();
	type Message = Message;
	type Theme = Theme;

	fn new(_flags: ()) -> (App, Command<Message>) {
		let items = vec![
			Item::new("fuckballs"),
			Item::new("fuckass"),
			Item::new("fuckeroni"),
			Item::new("shitballs"),
			Item::new("fucknugget"),
		];
		let sorter = Sorter::<Item>::new();
		let state = State {
			input_value: "".into(),
			items,
			sorter,
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
		let mode = &mut self.mode;
		match message {
			Message::InputChanged(value) => {
				state.input_value = value;
				Command::none()
			},
			Message::CreateItem => {
				if !state.input_value.is_empty() {
					state
						.items
						.insert(state.items.len(), Item::new(&state.input_value.clone()));
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
			Message::ListView => {
				self.state.sorter.finish_sorting(&mut self.state.items);
				*mode = AppMode::List;
				Command::none()
			},
			Message::SortItems => {
				match state.sorter.start_sorting(state.items.clone()) {
					Ok(_) => *mode = AppMode::Choose,
					Err(_) => *mode = AppMode::List,
				}
				Command::none()
			},
			Message::ChooseItem(choice) => {
				match &state.sorter.make_choice(*choice) {
					Ok(_) => {},
					Err(_) => {},
				}
				Command::none()
			},
		}
	}

	fn view(&self) -> Element<Message> {
		let input_value = &self.state.input_value;
		let items = &self.state.items;
		let sort_state = &self.state.sorter.state;

		match self.mode {
			AppMode::List => list_view(items, input_value),
			AppMode::Choose => choose_view(sort_state),
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

fn make_container(content: Element<Message>) -> Scrollable<Message> {
	scrollable(
		container(content)
			.width(Length::Fill)
			.padding(40)
			.center_x(),
	)
}

fn title_text(title: &str, size: impl Into<iced_core::Pixels>) -> Text {
	text(title)
		.width(Length::Fill)
		.size(size)
		.style(Color::from([0.5, 0.5, 0.5]))
		.horizontal_alignment(alignment::Horizontal::Center)
}

fn list_view<'a>(items: &'a ItemsList, input_value: &str) -> Element<'a, Message> {
	let title = title_text("Priorities", 100);

	let input = text_input("What would you like to prioritize?", input_value)
		.id(INPUT_ID.clone())
		.on_input(Message::InputChanged)
		.on_submit(Message::CreateItem)
		.padding(15)
		.size(30);

	let items_list = column(
		items
			.iter()
			.enumerate()
			.map(|(i, item)| {
				item.view(i)
					.map(move |message| Message::ItemMessage(i, message))
			})
			.collect(),
	)
	.spacing(10);

	make_container(
		match items.len() {
			2..=usize::MAX => {
				column![
					title,
					input,
					button("Sort Items").on_press(Message::SortItems),
					items_list
				]
			},
			_ => column![title, input, items_list],
		}
		.spacing(20)
		.max_width(800)
		.into(),
	)
	.into()
}

fn choose_view<'a>(sorter_state: &'a SortState<Item>) -> Element<'a, Message> {
	let prompt_text = title_text(
		match sorter_state {
			SortState::Empty => "There is nothing to compare.",
			SortState::Done { .. } => "Done comparing!",
			SortState::Compare { .. } => "Which one is higer priority?",
		},
		48,
	);

	let prompt = match sorter_state {
		SortState::Compare { left, right, .. } => {
			let left_desc = left.description.as_str();
			let right_desc = right.description.as_str();
			let left_btn = button(left_desc).on_press(Message::ChooseItem(left.clone()));
			let right_btn = button(right_desc).on_press(Message::ChooseItem(right.clone()));
			container(row![left_btn, right_btn].spacing(40))
		},
		SortState::Done { .. } | SortState::Empty => {
			container(button("Back To Items View").on_press(Message::ListView))
		},
	}
	.width(Length::Fill)
	.center_x();

	scrollable(
		container(
			column![prompt_text, prompt]
				.align_items(Alignment::Center)
				.spacing(60)
				.width(Length::Fill)
				.max_width(800),
		)
		.width(Length::Fill)
		.padding(40)
		.center_x(),
	)
	.into()
}
