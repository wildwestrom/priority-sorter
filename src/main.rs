mod app;
mod sorter;
mod item;

use app::App;
use iced::{window, Application, Settings};

pub fn main() -> iced::Result {
	App::run(Settings {
		window: window::Settings {
			size: (500, 800),
			..window::Settings::default()
		},
		..Settings::default()
	})
}
