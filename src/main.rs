mod app;
mod item;
mod sorter;

use iced::window;

pub fn main() -> iced::Result {
	iced::application(app::title, app::update, app::view)
		.subscription(app::subscription)
		.window(window::Settings {
			size: iced::Size::new(500.0, 800.0),
			..window::Settings::default()
		})
		.run_with(app::init)
}
