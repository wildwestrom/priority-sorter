use iced::{
	event::{self, Event},
	keyboard::{self, key::Named, Key, Modifiers},
	widget::{
		self, button, column, container, row, scrollable, text, text_input, Scrollable, Text,
	},
	window, Alignment, Element, Length, Subscription, Task,
};
use once_cell::sync::Lazy;

use crate::{
	item::{Item, ItemsList, Message as ItemMessage},
	sorter::{Choice, SortState, Sorter},
};

static INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

pub struct AppState {
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
	ToggleFullscreen(window::Id, window::Mode),
	ChooseLeft,
	ChooseRight,
}

pub fn init() -> (AppState, Task<Message>) {
	let items = vec![
		Item::new("10"),
		Item::new("20"),
		Item::new("30"),
		Item::new("40"),
		Item::new("50"),
		Item::new("60"),
		Item::new("70"),
		Item::new("80"),
		Item::new("90"),
		Item::new("100"),
	];
	let sorter = Sorter::<Item>::new();
	let state = State {
		input_value: "".into(),
		items,
		sorter,
	};
	(
		AppState {
			state,
			mode: AppMode::List,
		},
		Task::none(),
	)
}

pub fn title(_state: &AppState) -> String {
	"Priority Sorter".into()
}

pub fn update(app: &mut AppState, message: Message) -> Task<Message> {
	let state = &mut app.state;
	let mode = &mut app.mode;
	match message {
		Message::InputChanged(value) => {
			state.input_value = value;
			Task::none()
		},
		Message::CreateItem => {
			if !state.input_value.is_empty() {
				state
					.items
					.insert(state.items.len(), Item::new(&state.input_value.clone()));
				state.input_value.clear();
			}

			Task::none()
		},
		Message::ItemMessage(i, ItemMessage::Delete) => {
			state.items.remove(i);

			Task::none()
		},
		Message::ItemMessage(i, item_message) => {
			if let Some(item) = state.items.get_mut(i) {
				let should_focus = matches!(item_message, ItemMessage::Edit);

				item.update(item_message);

				if should_focus {
					let id = Item::text_input_id(&i);
					Task::batch(vec![
						text_input::focus(id.clone()),
						text_input::select_all(id),
					])
				} else {
					Task::none()
				}
			} else {
				Task::none()
			}
		},
		Message::TabPressed { shift } => {
			if shift {
				widget::focus_previous()
			} else {
				widget::focus_next()
			}
		},
		Message::ToggleFullscreen(id, mode) => window::change_mode(id, mode),
		Message::ListView => {
			state.sorter.finish_sorting(&mut state.items);
			*mode = AppMode::List;
			Task::none()
		},
		Message::SortItems => {
			match state.sorter.start_sorting(state.items.clone()) {
				Ok(_) => *mode = AppMode::Choose,
				Err(_) => *mode = AppMode::List,
			}
			Task::none()
		},
		Message::ChooseLeft => {
			match &state.sorter.make_choice(Choice::Left) {
				Ok(_) => {},
				Err(_) => {},
			}
			Task::none()
		},
		Message::ChooseRight => {
			match &state.sorter.make_choice(Choice::Right) {
				Ok(_) => {},
				Err(_) => {},
			}
			Task::none()
		},
	}
}

pub fn view(app: &'_ AppState) -> Element<'_, Message> {
	let input_value = &app.state.input_value;
	let items = &app.state.items;
	let sort_state = &app.state.sorter.state;

	match app.mode {
		AppMode::List => list_view(items, input_value),
		AppMode::Choose => choose_view(sort_state),
	}
}

pub fn subscription(_app: &AppState) -> Subscription<Message> {
	event::listen_with(|event, status, window| match (event, status) {
		(
			Event::Keyboard(keyboard::Event::KeyPressed {
				key: Key::Named(Named::Tab),
				modifiers,
				..
			}),
			event::Status::Ignored,
		) => Some(Message::TabPressed {
			shift: modifiers.shift(),
		}),
		(
			Event::Keyboard(keyboard::Event::KeyPressed {
				key,
				modifiers: Modifiers::SHIFT,
				..
			}),
			event::Status::Ignored,
		) => match key {
			Key::Named(Named::ArrowUp) => {
				Some(Message::ToggleFullscreen(window, window::Mode::Fullscreen))
			},
			Key::Named(Named::ArrowDown) => {
				Some(Message::ToggleFullscreen(window, window::Mode::Windowed))
			},
			_ => None,
		},
		_ => None,
	})
}

fn make_container(content: Element<Message>) -> Scrollable<Message> {
	scrollable(
		container(content)
			.width(Length::Fill)
			.padding(40)
			.center_x(Length::Shrink),
	)
}

fn title_text(title: &'_ str, size: impl Into<iced_core::Pixels>) -> Text<'_> {
	text(title).width(Length::Fill).size(size)
}

fn list_view<'a>(items: &'a ItemsList, input_value: &'a str) -> Element<'a, Message> {
	let title = title_text("Priorities", 100);

	let input = text_input("What would you like to prioritize?", input_value)
		.id(INPUT_ID.clone())
		.on_input(Message::InputChanged)
		.on_submit(Message::CreateItem)
		.padding(15)
		.size(30);

	let items_list = column(items.iter().enumerate().map(|(i, item)| {
		item.view(i)
			.map(move |message| Message::ItemMessage(i, message))
	}))
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

fn choose_view(sorter_state: &'_ SortState<Item>) -> Element<'_, Message> {
	let prompt_text = title_text(
		match sorter_state {
			SortState::Empty => "There is nothing to compare.",
			SortState::Done { .. } => "Done comparing!",
			SortState::Compare { .. } => "Which one is higer priority?",
		},
		48,
	);

	let prompt = match sorter_state {
		SortState::Compare {
			unsorted,
			sorted,
			lo,
			hi,
			..
		} => {
			let mid = (*lo + *hi) / 2;
			let left_desc = unsorted
				.last()
				.map(|b| b.description.as_str())
				.unwrap_or("");
			let right_desc = sorted[mid].description.as_str();
			let left_btn = button(left_desc).on_press(Message::ChooseLeft);
			let right_btn = button(right_desc).on_press(Message::ChooseRight);
			container(row![left_btn, right_btn].spacing(40))
		},
		SortState::Done { .. } | SortState::Empty => {
			container(button("Back To Items View").on_press(Message::ListView))
		},
	}
	.width(Length::Fill)
	.center_x(Length::Shrink);

	scrollable(
		container(
			column![prompt_text, prompt]
				.align_x(Alignment::Center)
				.spacing(60)
				.width(Length::Fill)
				.max_width(800),
		)
		.width(Length::Fill)
		.padding(40)
		.center_x(Length::Shrink),
	)
	.into()
}
