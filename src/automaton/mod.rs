
use crate::state::StateId;
use crate::state::State;
use std::collections::HashMap;
use std::collections::VecDeque;

/// A Levenshtein automaton that pre-computes a DFA for fuzzy string matching.
/// Given a pattern and a maximum edit distance, it builds all reachable states
/// and transitions at construction time, allowing O(n) matching per word.
pub struct LevenshteinAutomaton
{
    /// The pattern string to match against.
    pattern : String,
    /// The set of characters the automaton recognizes.
    alphabet : Vec<char>,
    /// The ID of the initial state (before any input is consumed).
    init_state: StateId,
    /// Maps each StateId to its corresponding State for acceptance checks.
    state_id_to_state : HashMap<StateId,State>,
    /// The transition table: for each state and character, the next state.
    transitions: HashMap<StateId,HashMap<char,StateId>>,
}

impl LevenshteinAutomaton {
    /// Constructs a new Levenshtein automaton for the given pattern.
    /// Returns None if the state space overflows (very long patterns with large edit distances).
    pub fn new(pattern : &str, diffs_allowed : usize, alphabet : Vec<char>)->Option<Self>{
        let mut automaton = LevenshteinAutomaton {
            pattern: pattern.to_string(),
            alphabet,
            init_state : StateId(0xdeadbeef),
            state_id_to_state : HashMap::new(),
            transitions: HashMap::new(),
        };

        return automaton.create_automaton(diffs_allowed).and_then(|_|Some(automaton));
    }

    /// Returns true if the Levenshtein distance between `against` and the pattern
    /// is at most the allowed threshold. Returns false if any transition is missing
    /// (e.g., a character not in the alphabet).
    pub fn match_word(&self, against : &str)-> bool{
        self.match_word_internal(against).unwrap_or(false)
    }

    /// Internal matching logic that returns Option to leverage the ? operator.
    /// Walks the DFA by consuming each character of the input word, then checks
    /// whether the final state is accepting.
    fn match_word_internal(&self, against: &str)->Option<bool> {
        let mut start_state_id = self.init_state;
        for c in against.chars() {
            let next_state_id =
                self.transitions.get(&start_state_id).
                and_then(|inner_hash_map|{
                    inner_hash_map.get(&c)
                })?;

            start_state_id = *next_state_id;
        }

        self.state_id_to_state.get(&start_state_id).and_then(|state| Some(state.is_accepting()))
    }

    /// Builds the automaton via BFS over the state space.
    /// Starting from the initial state, it computes transitions for every character
    /// in the alphabet, discovers new states, and continues until all reachable
    /// states have been explored.
    fn create_automaton(& mut self, diffs_allowed: usize)->Option<bool>{
        let mut queue : VecDeque<StateId> = VecDeque::new();

        let init_state = State::initial_state(self.pattern.chars().count(), diffs_allowed);
        let init_state_id = init_state.get_state_id()?;

        self.init_state = init_state_id;

        self.state_id_to_state.insert(init_state_id,init_state);
        queue.push_back(init_state_id);

        while !queue.is_empty(){
            let state_id  = queue.pop_front()?;

            let popped_state = self.state_id_to_state.get(&state_id).cloned()?;

            for c in &self.alphabet{
                let new_state = popped_state.on_new_char(&self.pattern, *c);
                let new_state_id =  new_state.get_state_id()?;

                self.transitions.entry(state_id)
                .or_insert_with(HashMap::new)
                .insert(*c,new_state_id);

                if !self.state_id_to_state.contains_key(&new_state_id){
                    self.state_id_to_state.entry(new_state_id)
                    .or_insert(new_state);

                    queue.push_back(new_state_id);
                }else {
                    continue
                }
            }
        }

        Some(true)

    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn ascii_alphabet() -> Vec<char> {
        ('a'..='z').collect()
    }

    #[test]
    fn new_returns_some_for_valid_input() {
        let automaton = LevenshteinAutomaton::new("abc", 1, ascii_alphabet());
        assert!(automaton.is_some());
    }

    #[test]
    fn new_empty_pattern() {
        let automaton = LevenshteinAutomaton::new("", 2, ascii_alphabet());
        assert!(automaton.is_some());
    }

    #[test]
    fn new_zero_diffs() {
        let automaton = LevenshteinAutomaton::new("hello", 0, ascii_alphabet());
        assert!(automaton.is_some());
    }

    #[test]
    fn match_exact_word() {
        let automaton = LevenshteinAutomaton::new("abc", 1, ascii_alphabet()).unwrap();
        assert!(automaton.match_word("abc"));
    }

    #[test]
    fn match_exact_zero_diffs() {
        let automaton = LevenshteinAutomaton::new("hello", 0, ascii_alphabet()).unwrap();
        assert!(automaton.match_word("hello"));
    }

    #[test]
    fn no_match_zero_diffs() {
        let automaton = LevenshteinAutomaton::new("hello", 0, ascii_alphabet()).unwrap();
        assert!(!automaton.match_word("hallo"));
    }

    #[test]
    fn match_one_substitution() {
        let automaton = LevenshteinAutomaton::new("abc", 1, ascii_alphabet()).unwrap();
        assert!(automaton.match_word("axc"));
    }

    #[test]
    fn match_one_insertion() {
        let automaton = LevenshteinAutomaton::new("abc", 1, ascii_alphabet()).unwrap();
        assert!(automaton.match_word("abbc"));
    }

    #[test]
    fn match_one_deletion() {
        let automaton = LevenshteinAutomaton::new("abc", 1, ascii_alphabet()).unwrap();
        assert!(automaton.match_word("ac"));
    }

    #[test]
    fn no_match_two_edits_with_one_allowed() {
        let automaton = LevenshteinAutomaton::new("abc", 1, ascii_alphabet()).unwrap();
        assert!(!automaton.match_word("axx"));
    }

    #[test]
    fn match_two_edits_with_two_allowed() {
        let automaton = LevenshteinAutomaton::new("abc", 2, ascii_alphabet()).unwrap();
        assert!(automaton.match_word("axx"));
    }

    #[test]
    fn match_empty_pattern_empty_word() {
        let automaton = LevenshteinAutomaton::new("", 0, ascii_alphabet()).unwrap();
        assert!(automaton.match_word(""));
    }

    #[test]
    fn match_empty_pattern_short_word() {
        let automaton = LevenshteinAutomaton::new("", 2, ascii_alphabet()).unwrap();
        assert!(automaton.match_word("ab"));
    }

    #[test]
    fn no_match_empty_pattern_long_word() {
        let automaton: LevenshteinAutomaton = LevenshteinAutomaton::new("", 1, ascii_alphabet()).unwrap();
        assert!(!automaton.match_word("ab"));
    }

    #[test]
    fn match_empty_word_short_pattern() {
        let automaton = LevenshteinAutomaton::new("ab", 2, ascii_alphabet()).unwrap();
        assert!(automaton.match_word(""));
    }

    #[test]
    fn no_match_empty_word_long_pattern() {
        let automaton = LevenshteinAutomaton::new("abc", 2, ascii_alphabet()).unwrap();
        assert!(!automaton.match_word(""));
    }

    #[test]
    fn match_completely_different_within_threshold() {
        let automaton = LevenshteinAutomaton::new("ab", 2, ascii_alphabet()).unwrap();
        assert!(automaton.match_word("cd"));
    }

    #[test]
    fn no_match_completely_different_over_threshold() {
        let automaton = LevenshteinAutomaton::new("abc", 2, ascii_alphabet()).unwrap();
        assert!(!automaton.match_word("xyz"));
    }

    #[test]
    fn match_single_char_pattern() {
        let automaton = LevenshteinAutomaton::new("a", 1, ascii_alphabet()).unwrap();
        assert!(automaton.match_word("a"));
        assert!(automaton.match_word("b"));
        assert!(automaton.match_word(""));
        assert!(!automaton.match_word("bc"));
    }

    #[test]
    fn match_word_not_in_alphabet_returns_false() {
        let automaton = LevenshteinAutomaton::new("abc", 1, vec!['a', 'b', 'c']).unwrap();
        assert!(!automaton.match_word("axc"));
    }

    #[test]
    fn match_prefix_deletion() {
        let automaton = LevenshteinAutomaton::new("abc", 1, ascii_alphabet()).unwrap();
        assert!(automaton.match_word("bc"));
    }

    #[test]
    fn match_suffix_deletion() {
        let automaton = LevenshteinAutomaton::new("abc", 1, ascii_alphabet()).unwrap();
        assert!(automaton.match_word("ab"));
    }

    #[test]
    fn no_match_too_long() {
        let automaton = LevenshteinAutomaton::new("abc", 1, ascii_alphabet()).unwrap();
        assert!(!automaton.match_word("abcde"));
    }

    #[test]
    fn no_match_too_short() {
        let automaton = LevenshteinAutomaton::new("abcde", 1, ascii_alphabet()).unwrap();
        assert!(!automaton.match_word("abc"));
    }

    #[test]
    fn match_same_length_multiple_edits() {
        let automaton = LevenshteinAutomaton::new("kitten", 3, ascii_alphabet()).unwrap();
        assert!(automaton.match_word("sitting"));
    }
}
