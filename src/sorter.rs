use anyhow::{anyhow, Result};

pub enum SortState<T> {
	Empty,
	Compare {
		// Items not yet inserted. The current item is always `unsorted.last()`.
		unsorted: Vec<Box<T>>,
		// Current total order built so far (most important first)
		sorted: Vec<T>,
		// Binary search bounds into `sorted` for inserting `left`
		lo: usize,
		hi: usize,
	},
	Done {
		sorted: Vec<T>,
	},
}

pub struct Sorter<T> {
	pub state: SortState<T>,
}

#[derive(Debug, Clone, Copy)]
pub enum Choice {
	Left,
	Right,
}

impl<T: Clone> Sorter<T> {
	pub fn new() -> Self {
		let state = SortState::Empty;
		Sorter { state }
	}

	pub fn start_sorting(&mut self, items: Vec<T>) -> Result<()> {
		// If no items or a single item, we're done immediately
		match items.len() {
			0 => {
				self.state = SortState::Empty;
				return Ok(());
			},
			1 => {
				self.state = SortState::Done { sorted: items };
				return Ok(());
			},
			_ => {},
		}

		let mut iter = items.into_iter();
		// Seed the order with the first item
		let first = iter.next().unwrap();
		let sorted: Vec<T> = vec![first];
		// Remaining items to insert
		let unsorted: Vec<Box<T>> = iter.map(|i| Box::new(i)).collect();

		// Ensure there is a current item to insert at `unsorted.last()`
		if unsorted.is_empty() {
			return Err(anyhow!("Expected at least two items"));
		}
		let lo = 0usize;
		let hi = sorted.len(); // at least 1

		self.state = SortState::Compare {
			unsorted,
			sorted,
			lo,
			hi,
		};
		Ok(())
	}

	pub fn make_choice(&mut self, choice: Choice) -> Result<()> {
		match &mut self.state {
			SortState::Compare {
				unsorted,
				sorted,
				lo,
				hi,
			} => {
				let mid = (*lo + *hi) / 2;
				// If left was chosen, x > pivot -> search upper segment [lo, mid)
				// Else pivot > x -> search lower segment (mid, hi]
				if matches!(choice, Choice::Left) {
					*hi = mid;
				} else {
					*lo = mid + 1;
				}

				if *lo < *hi {
					return Ok(());
				}

				// Insert current item at position `lo` and move to next item (or finish)
				let insert_pos = *lo;
				let x = *unsorted
					.last()
					.expect("unsorted is non-empty when comparing")
					.clone();
				sorted.insert(insert_pos, x);

				// Remove the item we just inserted from the pending stack
				let _removed_current = unsorted.pop();
				if unsorted.is_empty() {
					let final_sorted = sorted.clone();
					self.state = SortState::Done {
						sorted: final_sorted,
					};
					Ok(())
				} else {
					*lo = 0;
					*hi = sorted.len();
					Ok(())
				}
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
