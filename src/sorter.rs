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

#[cfg(test)]
mod tests {
	use super::*;

	use rand::rngs::StdRng;
	use rand::seq::SliceRandom;
	use rand::SeedableRng;

	fn expected_max_comparisons(n: usize) -> usize {
		if n <= 1 {
			return 0;
		}
		let mut total = 0;
		for k in 1..n {
			let x = k + 1;
			let ceil_log2 = (usize::BITS as usize) - (x - 1).leading_zeros() as usize;
			total += ceil_log2;
		}
		total
	}

	fn run_simulated_sort(n: usize, seed: u64) -> (usize, Vec<i32>, Vec<i32>) {
		let mut rng = StdRng::seed_from_u64(seed);
		let mut items: Vec<i32> = (0..n as i32).collect();
		items.shuffle(&mut rng);

		let ground_truth_desc: Vec<i32> = {
			let mut v = items.clone();
			v.sort_by(|a, b| b.cmp(a)); // descending, most important first
			v
		};

		let mut sorter = Sorter::new();
		sorter.start_sorting(items.clone()).unwrap();

		let mut comparisons = 0;
		loop {
			match &sorter.state {
				SortState::Empty => {
					assert_eq!(n, 0);
					break;
				},
				SortState::Done { .. } => break,
				SortState::Compare {
					unsorted,
					sorted,
					lo,
					hi,
				} => {
					// Determine pivot and current x, then choose based on numeric order
					let mid = (*lo + *hi) / 2;
					let x = unsorted.last().expect("there is a current item").as_ref();
					let y = &sorted[mid];
					let choice = if x > y { Choice::Left } else { Choice::Right };
					comparisons += 1;
					sorter.make_choice(choice).unwrap();
				},
			}
		}

		let mut out = items.clone();
		sorter.finish_sorting(&mut out);
		(comparisons, ground_truth_desc, out)
	}

	#[test]
	fn sorts_matches_ground_truth_small_sizes() {
		for &n in &[0, 1, 2, 3, 5, 8, 13] {
			let (comparisons, gt, out) = run_simulated_sort(n, 0xDEADBEEFCAFEBABE);
			println!(
				"small_sizes: n={}, comparisons={}, bound={}",
				n,
				comparisons,
				expected_max_comparisons(n)
			);
			assert_eq!(out, gt, "n={}", n);
			assert!(
				comparisons <= expected_max_comparisons(n),
				"n={}, comparisons={} > bound={}",
				n,
				comparisons,
				expected_max_comparisons(n)
			);
		}
	}

	#[test]
	fn sorts_matches_ground_truth_medium_sizes() {
		for &n in &[10, 16, 20, 32, 40] {
			let (comparisons, gt, out) = run_simulated_sort(n, 0x1234_5678_9ABC_DEF0);
			println!(
				"medium_sizes: n={}, comparisons={}, bound={}",
				n,
				comparisons,
				expected_max_comparisons(n)
			);
			assert_eq!(out, gt, "n={}", n);
			assert!(
				comparisons <= expected_max_comparisons(n),
				"n={}, comparisons={} > bound={}",
				n,
				comparisons,
				expected_max_comparisons(n)
			);
		}
	}

	#[test]
	fn sorts_matches_ground_truth_large_sizes() {
		for &n in &[50, 100, 1000, 10_000, 100_000, 1_000_000] {
			let (comparisons, gt, out) = run_simulated_sort(n, 0xBABABABABABABABA);
			println!(
				"large_sizes: n={}, comparisons={}, bound={}",
				n,
				comparisons,
				expected_max_comparisons(n)
			);
			assert_eq!(out, gt, "n={}", n);
			assert!(
				comparisons <= expected_max_comparisons(n),
				"n={}, comparisons={} > bound={}",
				n,
				comparisons,
				expected_max_comparisons(n)
			);
		}
	}
}
