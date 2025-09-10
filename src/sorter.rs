use anyhow::{anyhow, Result};

pub enum SortState<T> {
	Empty,
	Compare {
		left: Box<T>,
		right: Box<T>,
		unsorted: Vec<Box<T>>,
		sorted: Vec<T>,
	},
	Done {
		sorted: Vec<T>,
	},
}

pub struct Sorter<T> {
	pub state: SortState<T>,
}

trait SwapRemoveRandom<T> {
	fn random_index(&self) -> usize;
	fn swap_remove_random(&mut self) -> Result<T>;
}

impl<T> SwapRemoveRandom<T> for Vec<T> {
	fn random_index(&self) -> usize {
		use rand::Rng;
		let mut rng = rand::rng();
		rng.random_range(0..self.len())
	}

	fn swap_remove_random(&mut self) -> Result<T> {
		if self.len() > 0 {
			Ok(self.swap_remove(self.random_index()))
		} else {
			Err(anyhow!("No more items to pick from"))
		}
	}
}

impl<T: Clone + std::cmp::PartialEq> Sorter<T> {
	pub fn new() -> Self {
		let state = SortState::Empty;
		Sorter { state }
	}

	pub fn start_sorting(&mut self, items: Vec<T>) -> Result<()> {
		let mut unsorted: Vec<_> = items.into_iter().map(|i| Box::new(i)).collect();
		let left = unsorted.swap_remove_random()?;
		let right = unsorted.swap_remove_random()?;
		Ok(self.state = SortState::Compare {
			left,
			right,
			unsorted,
			sorted: vec![],
		})
	}

	pub fn make_choice(&mut self, choice: T) -> Result<()> {
		match &mut self.state {
			SortState::Compare {
				left,
				right,
				unsorted,
				sorted,
			} => {
				if choice == **left {
					sorted.push(*left.clone());
					sorted.push(*right.clone());
				} else {
					sorted.push(*right.clone());
					sorted.push(*left.clone());
				}
				match unsorted.swap_remove_random() {
					Ok(new_left) => match unsorted.swap_remove_random() {
						Ok(new_right) => {
							self.state = SortState::Compare {
								left: new_left,
								right: new_right,
								unsorted: unsorted.clone(),
								sorted: sorted.clone(),
							};
						},
						_ => {
							let new_right = sorted.swap_remove(sorted.len() - 1);
							self.state = SortState::Compare {
								left: new_left,
								right: new_right.into(),
								unsorted: unsorted.clone(),
								sorted: sorted.clone(),
							}
						},
					},
					_ => {
						self.state = SortState::Done {
							sorted: sorted.clone(),
						};
					},
				};
				Ok(())
			},
			SortState::Empty | SortState::Done { .. } => Err(anyhow!("This should not happen")),
		}
	}

	pub fn finish_sorting(&mut self, items: &mut Vec<T>) {
		match &self.state {
			SortState::Done { sorted } => {
				*items = sorted.clone();
			},
			SortState::Empty => {},
			SortState::Compare {
				unsorted, sorted, ..
			} => {
				*items = [
					sorted.clone(),
					unsorted.clone().into_iter().map(|i| *i).collect(),
				]
				.concat()
			},
		}
	}
}
