use iced::{
	theme,
	widget::{button, row, text, text_input},
	Alignment, Element, Length,
};

#[derive(Debug, Clone)]
pub struct Item {
	pub description: String,
	state: State,
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

	pub fn new(description: String) -> Self {
		Item {
			description,
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

	pub fn view(&self, i: usize) -> Element<Message> {
		match &self.state {
			State::Idle => row![
				text((i + 1).to_string()),
				text(self.description.as_str()).width(Length::Fill),
				button("Edit")
					.on_press(Message::Edit)
					.padding(10)
					.style(theme::Button::Text),
			]
			.spacing(20)
			.align_items(Alignment::Center)
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
						.style(theme::Button::Destructive)
				]
				.spacing(20)
				.align_items(Alignment::Center)
				.into()
			},
		}
	}
}
