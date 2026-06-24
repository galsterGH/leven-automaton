/// Represents a state in the Levenshtein automaton.
/// Each state holds a vector of edit distances between prefixes of the pattern
/// and the input consumed so far. Values are clamped to `diffs_allowed + 1`.
#[derive(Clone)]
pub struct State {
    edit_distance: Vec<usize>,
    diffs_allowed : usize,
}

/// A unique identifier for a State, computed by encoding the edit distance vector
/// as a single integer in base `diffs_allowed + 2`.
#[derive(PartialEq,Eq,Hash,Clone,Copy)]
pub struct StateId(pub usize);

impl State {
    /// Creates a new state with all edit distances initialized to zero.
    pub fn new(pattern_size: usize, diffs_allowed : usize) -> Self {
        State {

            edit_distance: vec!(0;pattern_size + 1),
            diffs_allowed,
        }
    }

    /// Creates the initial state of the automaton (before any input is consumed).
    /// edit_distance[i] = min(i, diffs_allowed + 1), representing the cost of
    /// deleting the first i characters of the pattern.
    pub fn initial_state(pattern_size: usize, diffs_allowed : usize) -> Self {
        State {
            edit_distance: (0..=pattern_size)
                .map(|i| std::cmp::min(diffs_allowed + 1, i))
                .collect(),
            diffs_allowed,
        }
    }

    /// Encodes this state's edit distance vector as a unique integer ID.
    /// Uses base `diffs_allowed + 2` since each position holds values in [0, diffs_allowed + 1].
    /// Returns None if the computation would overflow usize.
    pub fn get_state_id(&self)-> Option<StateId>{
        let result = (0..self.edit_distance.len()).zip(self.edit_distance.iter()).fold(Some(0 as usize), |acc, (i,n)|{
            if let None = acc{
                return None;
            }else {
                let a: usize = acc?;

                (self.diffs_allowed + 2).checked_pow(i as u32).and_then(|pow|{
                    let temp_mlt : usize;

                    if usize::MAX / pow > *n {
                        temp_mlt = (*n)*pow;
                    }else {
                        return Option::None;
                    }

                    if usize::MAX - a > temp_mlt {
                        Some(a + temp_mlt)
                    }else{
                        None
                    }
                })
            }
        });

        result.map(|u|{
            StateId(u)
        })
    }

    /// Computes the next state after consuming `new_char` from the input.
    /// Applies the Levenshtein recurrence considering all three edit operations:
    /// - Match/substitution: comparing new_char against each pattern character
    /// - Insertion: cost from new_state[i] (inserting new_char)
    /// - Deletion: cost from self.edit_distance[i + 1] (skipping a pattern character)
    pub fn on_new_char(&self, pattern: &str, new_char: char) -> Self {
        let mut new_state = vec!(0;pattern.chars().count() + 1);

        new_state[0] = std::cmp::min(self.edit_distance[0] + 1, self.diffs_allowed + 1);
        let pattern_chars : Vec<char> = pattern.chars().collect();

        (0..pattern_chars.len()).for_each(|i| {
            if pattern_chars[i] == new_char {
                new_state[i + 1] = self.edit_distance[i];
            } else {
                let min_count = std::cmp::min(std::cmp::min(new_state[i], self.edit_distance[i]), self.edit_distance[i + 1]);

                if min_count >= self.diffs_allowed {
                    new_state[i + 1] = self.diffs_allowed + 1;
                } else {
                    new_state[i + 1] = min_count + 1;
                }
            }
        });

        State { edit_distance: new_state, diffs_allowed: self.diffs_allowed}
    }

    /// Returns true if this state represents a successful match, i.e., the edit distance
    /// for the full pattern is within the allowed threshold.
    pub fn is_accepting(&self)->bool{
        return self.edit_distance[self.edit_distance.len() - 1] <= self.diffs_allowed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_zero_vector() {
        let state = State::new(3, 1);
        assert_eq!(state.edit_distance, vec![0, 0, 0, 0]);
        assert_eq!(state.diffs_allowed, 1);
    }

    #[test]
    fn new_empty_pattern() {
        let state = State::new(0, 2);
        assert_eq!(state.edit_distance, vec![0]);
    }

    #[test]
    fn initial_state_values() {
        let state = State::initial_state(5, 2);
        assert_eq!(state.edit_distance, vec![0, 1, 2, 3, 3, 3]);
    }

    #[test]
    fn initial_state_small_pattern() {
        let state = State::initial_state(2, 5);
        assert_eq!(state.edit_distance, vec![0, 1, 2]);
    }

    #[test]
    fn initial_state_zero_diffs() {
        let state = State::initial_state(3, 0);
        assert_eq!(state.edit_distance, vec![0, 1, 1, 1]);
    }

    #[test]
    fn initial_state_empty_pattern() {
        let state = State::initial_state(0, 2);
        assert_eq!(state.edit_distance, vec![0]);
    }

    #[test]
    fn get_state_id_returns_some() {
        let state = State::initial_state(3, 1);
        assert!(state.get_state_id().is_some());
    }

    #[test]
    fn get_state_id_unique_for_different_states() {
        let s1 = State::initial_state(3, 2);
        let s2 = s1.on_new_char("abc", 'a');
        let id1 = s1.get_state_id().unwrap();
        let id2 = s2.get_state_id().unwrap();
        assert!(id1 != id2);
    }

    #[test]
    fn get_state_id_same_for_equal_states() {
        let s1 = State::initial_state(3, 2);
        let s2 = State::initial_state(3, 2);
        assert!(s1.get_state_id().unwrap() == s2.get_state_id().unwrap());
    }

    #[test]
    fn get_state_id_zero_diffs() {
        let state = State::initial_state(2, 0);
        assert!(state.get_state_id().is_some());
    }

    #[test]
    fn on_new_char_matching_first_char() {
        let state = State::initial_state(3, 1);
        let next = state.on_new_char("abc", 'a');
        assert_eq!(next.edit_distance[0], 1);
        assert_eq!(next.edit_distance[1], 0);
    }

    #[test]
    fn on_new_char_no_match() {
        let state = State::initial_state(3, 1);
        let next = state.on_new_char("abc", 'z');
        assert_eq!(next.edit_distance[0], 1);
        assert!(next.edit_distance[1] >= 1);
    }

    #[test]
    fn on_new_char_preserves_diffs_allowed() {
        let state = State::initial_state(3, 2);
        let next = state.on_new_char("abc", 'x');
        assert_eq!(next.diffs_allowed, 2);
    }

    #[test]
    fn on_new_char_vector_length() {
        let state = State::initial_state(4, 1);
        let next = state.on_new_char("abcd", 'a');
        assert_eq!(next.edit_distance.len(), 5);
    }

    #[test]
    fn on_new_char_sequence_builds_exact_match() {
        let state = State::initial_state(3, 0);
        let s1 = state.on_new_char("abc", 'a');
        let s2 = s1.on_new_char("abc", 'b');
        let s3 = s2.on_new_char("abc", 'c');
        assert!(s3.is_accepting());
    }

    #[test]
    fn on_new_char_sequence_wrong_chars_not_accepting() {
        let state = State::initial_state(3, 0);
        let s1 = state.on_new_char("abc", 'x');
        let s2 = s1.on_new_char("abc", 'y');
        let s3 = s2.on_new_char("abc", 'z');
        assert!(!s3.is_accepting());
    }

    #[test]
    fn is_accepting_exact_match() {
        let state = State { edit_distance: vec![1, 0], diffs_allowed: 1 };
        assert!(state.is_accepting());
    }

    #[test]
    fn is_accepting_at_threshold() {
        let state = State { edit_distance: vec![0, 1, 2], diffs_allowed: 2 };
        assert!(state.is_accepting());
    }

    #[test]
    fn is_accepting_over_threshold() {
        let state = State { edit_distance: vec![0, 1, 3], diffs_allowed: 2 };
        assert!(!state.is_accepting());
    }

    #[test]
    fn is_accepting_zero_diffs_exact() {
        let state = State { edit_distance: vec![0, 0], diffs_allowed: 0 };
        assert!(state.is_accepting());
    }

    #[test]
    fn is_accepting_zero_diffs_nonzero() {
        let state = State { edit_distance: vec![0, 1], diffs_allowed: 0 };
        assert!(!state.is_accepting());
    }

    #[test]
    fn initial_state_empty_pattern_is_accepting() {
        let state = State::initial_state(0, 2);
        assert!(state.is_accepting());
    }

    #[test]
    fn state_id_equality() {
        let a = StateId(42);
        let b = StateId(42);
        let c = StateId(99);
        assert!(a == b);
        assert!(a != c);
    }

    #[test]
    fn state_id_copy() {
        let a = StateId(10);
        let b = a;
        assert!(a == b);
    }
}
