use iced::{
	widget::{button, row, text, text_input},
	Alignment, Element, Length,
};

pub type ItemsList = Vec<Item>;
#[derive(Debug, Clone)]
pub struct Item {
	pub description: String,
	state: State,
}
impl PartialEq for Item {
	fn eq(&self, other: &Self) -> bool {
		self.description == other.description
	}
}

#[derive(Debug, Clone)]
pub enum State {
	Idle,
	Editing,
}

#[derive(Debug, Clone)]
pub enum Message {
	Edit,
	DescriptionEdited(String),
	FinishEdition,
	Delete,
}

impl Item {
	pub fn text_input_id(i: &usize) -> text_input::Id {
		text_input::Id::new(format!("item-{}", i))
	}

	pub fn new(description: &str) -> Self {
		Item {
			description: description.into(),
			state: State::Idle,
		}
	}

	pub fn update(&mut self, message: Message) {
		match message {
			Message::Edit => {
				self.state = State::Editing;
			},
			Message::DescriptionEdited(new_description) => {
				self.description = new_description;
			},
			Message::FinishEdition => {
				if !self.description.is_empty() {
					self.state = State::Idle;
				}
			},
			Message::Delete => {},
		}
	}

	pub fn view(&'_ self, i: usize) -> Element<'_, Message> {
		match &self.state {
			State::Idle => row![
				text((i + 1).to_string()),
				text(self.description.as_str()).width(Length::Fill),
				button("Edit")
					.on_press(Message::Edit)
					.padding(10)
					.style(iced::widget::button::text),
			]
			.spacing(20)
			.align_y(Alignment::Center)
			.into(),
			State::Editing => {
				let text_input = text_input("An item to prioritize...", &self.description)
					.id(Self::text_input_id(&i))
					.on_input(Message::DescriptionEdited)
					.on_submit(Message::FinishEdition)
					.padding(10);

				row![
					text_input,
					button("Delete")
						.on_press(Message::Delete)
						.padding(10)
						.style(iced::widget::button::danger)
				]
				.spacing(20)
				.align_y(Alignment::Center)
				.into()
			},
		}
	}
}
