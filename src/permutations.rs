use num::Integer;

/// Implemented as a modified version of [Heap's Algorithm](https://en.m.wikipedia.org/wiki/Heap%27s_algorithm#Details_of_the_algorithm)
pub struct PermutationsIterator<T> {
    items: Vec<T>,
    counters: Vec<usize>,
    current_index: usize,
    output_iniital: bool,
}

impl<T: Clone> PermutationsIterator<T> {
    pub fn from(items: Vec<T>) -> Self {
        let len = items.len();
        let mut counters = Vec::new();
        counters.resize(len, 0);
        Self {
            items,
            counters,
            current_index: 0,
            output_iniital: false,
        }
    }
}

impl<T: Clone> Iterator for PermutationsIterator<T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.output_iniital {
            self.output_iniital = true;
            return Some(self.items.clone());
        }
        while self.current_index < self.items.len() {
            let i = self.current_index;
            if self.counters[i] < i {
                if i.is_even() {
                    self.items.swap(0, i);
                } else {
                    self.items.swap(self.counters[i], i);
                }

                self.counters[i] += 1;
                self.current_index = 0;
                return Some(self.items.clone());
            } else {
                self.counters[i] = 0;
                self.current_index += 1;
            }
        }
        return None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_permute_a_collection() {
        let collection = vec![1, 2, 3];
        let mut permutations: Vec<Vec<u32>> = PermutationsIterator::from(collection).collect();
        permutations.sort();

        let mut expected = vec![
            vec![1, 2, 3],
            vec![1, 3, 2],
            vec![2, 1, 3],
            vec![2, 3, 1],
            vec![3, 1, 2],
            vec![3, 2, 1],
        ];
        expected.sort();

        assert_eq!(expected, permutations);
    }
}
