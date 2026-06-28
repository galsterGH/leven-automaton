
use std::collections::HashSet;
use crate::utilities::TrieNode;
use crate::automaton::{LevenshteinAutomaton};
use crate::state::StateId;


/// Combines a Trie dictionary with a Levenshtein automaton to efficiently find
/// all words in the dictionary within a given edit distance of a search pattern.
#[derive(Clone)]
pub struct Matcher{
    /// The trie storing all dictionary words.
    trie : TrieNode,
    /// The union of all characters appearing in the dictionary.
    alphabet : HashSet<char>,
}

impl Matcher {
    /// Builds a Matcher from a list of dictionary words.
    /// Returns None if any word fails to insert into the trie.
    pub fn new(dictionary: &Vec<String>)->Option<Self>{
        let mut matcher:Matcher  = Matcher{
           trie: TrieNode::new(),
           alphabet: HashSet::new()
        };

        let insertion_res  = dictionary.iter()
            .map(|word| matcher.trie.add_new_word(word))
            .fold(true,|acc,b| acc && b);
        
        if insertion_res {
            dictionary.iter()
            .for_each(|w| w.chars().into_iter()
                                            .for_each(|c| {matcher.alphabet.insert(c);}));
            
            Some(matcher)
        }else{
            None
        }
    }

    /// Finds all dictionary words within `allowed_diffs` edit distance of `pattern`.
    /// Returns None if no matches are found or the automaton cannot be constructed.
    pub fn match_pattern(&self, pattern: &str, allowed_diffs : usize) -> Option<Vec<String>> {
        let automaton = LevenshteinAutomaton::new(pattern,allowed_diffs, self.alphabet.iter().cloned().collect())?;
        let mut result : Vec<String> = Vec::new();
        Self::match_pattern_internal(&self.trie, &automaton, automaton.get_initial_state(),& mut result);
        
        if result.is_empty() {
            None
        }else{
            Some(result)
        }
    }

    /// Recursive DFS that walks the trie and automaton in lockstep.
    /// Collects words at accepting states and prunes branches at dead states.
    fn match_pattern_internal(trie: &TrieNode, automaton: &LevenshteinAutomaton, state_id : StateId, result : & mut Vec<String>) {
        if automaton.is_accepting_state(state_id) {

            if let Some(word) = trie.get_node_word(){
                result.push(word);
            }
        }

        if automaton.is_dead_state(state_id){
            return;
        }

        for pair in trie.get_all_children(){
            if let Some(next_state_id) = automaton.get_next_state(state_id, *pair.0) {
                 Self::match_pattern_internal(pair.1, automaton,next_state_id, result);
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn dict(words: &[&str]) -> Vec<String> {
        words.iter().map(|s| s.to_string()).collect()
    }

    // --- Matcher::new ---

    #[test]
    fn new_returns_some_for_valid_dictionary() {
        let matcher = Matcher::new(&dict(&["hello", "world"]));
        assert!(matcher.is_some());
    }

    #[test]
    fn new_empty_dictionary() {
        let matcher = Matcher::new(&dict(&[]));
        assert!(matcher.is_some());
    }

    #[test]
    fn new_single_word_dictionary() {
        let matcher = Matcher::new(&dict(&["cat"]));
        assert!(matcher.is_some());
    }

    #[test]
    fn new_collects_alphabet_from_all_words() {
        let matcher = Matcher::new(&dict(&["ab", "cd"])).unwrap();
        assert!(matcher.alphabet.contains(&'a'));
        assert!(matcher.alphabet.contains(&'b'));
        assert!(matcher.alphabet.contains(&'c'));
        assert!(matcher.alphabet.contains(&'d'));
    }

    #[test]
    fn new_alphabet_has_no_extra_chars() {
        let matcher = Matcher::new(&dict(&["ab"])).unwrap();
        assert_eq!(matcher.alphabet.len(), 2);
        assert!(!matcher.alphabet.contains(&'z'));
    }

    #[test]
    fn new_duplicate_words_in_dictionary() {
        let matcher = Matcher::new(&dict(&["cat", "cat", "cat"]));
        assert!(matcher.is_some());
    }

    // --- match_pattern: exact matches (0 diffs) ---

    #[test]
    fn exact_match_found() {
        let matcher = Matcher::new(&dict(&["cat", "car", "bat"])).unwrap();
        let result = matcher.match_pattern("cat", 0).unwrap();
        assert_eq!(result, vec!["cat"]);
    }

    #[test]
    fn exact_match_not_found() {
        let matcher = Matcher::new(&dict(&["cat", "car", "bat"])).unwrap();
        let result = matcher.match_pattern("dog", 0);
        assert!(result.is_none());
    }

    // --- match_pattern: substitution ---

    #[test]
    fn one_substitution() {
        let matcher = Matcher::new(&dict(&["cat", "car", "bat"])).unwrap();
        let result = matcher.match_pattern("cat", 1).unwrap();
        assert!(result.contains(&"cat".to_string()));
        assert!(result.contains(&"car".to_string()));
        assert!(result.contains(&"bat".to_string()));
    }

    #[test]
    fn two_substitutions_not_matched_with_one_allowed() {
        let matcher = Matcher::new(&dict(&["cat", "dog"])).unwrap();
        let result = matcher.match_pattern("cat", 1).unwrap();
        assert!(result.contains(&"cat".to_string()));
        assert!(!result.contains(&"dog".to_string()));
    }

    // --- match_pattern: insertion ---

    #[test]
    fn one_insertion() {
        let matcher = Matcher::new(&dict(&["cat", "cats"])).unwrap();
        let result = matcher.match_pattern("cat", 1).unwrap();
        assert!(result.contains(&"cat".to_string()));
        assert!(result.contains(&"cats".to_string()));
    }

    // --- match_pattern: deletion ---

    #[test]
    fn one_deletion() {
        let matcher = Matcher::new(&dict(&["cat", "ca"])).unwrap();
        let result = matcher.match_pattern("cat", 1).unwrap();
        assert!(result.contains(&"cat".to_string()));
        assert!(result.contains(&"ca".to_string()));
    }

    // --- match_pattern: mixed edits ---

    #[test]
    fn two_edits_with_two_allowed() {
        let matcher = Matcher::new(&dict(&["kitten", "sitting"])).unwrap();
        let result = matcher.match_pattern("kitten", 3).unwrap();
        assert!(result.contains(&"kitten".to_string()));
        assert!(result.contains(&"sitting".to_string()));
    }

    #[test]
    fn two_edits_with_one_allowed_no_match() {
        let matcher = Matcher::new(&dict(&["abc", "xyz"])).unwrap();
        let result = matcher.match_pattern("abc", 1).unwrap();
        assert!(result.contains(&"abc".to_string()));
        assert!(!result.contains(&"xyz".to_string()));
    }

    // --- match_pattern: empty pattern ---

    #[test]
    fn empty_pattern_zero_diffs_matches_empty_word() {
        let matcher = Matcher::new(&dict(&["", "a", "ab"])).unwrap();
        let result = matcher.match_pattern("", 0).unwrap();
        assert_eq!(result, vec!["".to_string()]);
    }

    #[test]
    fn empty_pattern_with_diffs_matches_short_words() {
        let matcher = Matcher::new(&dict(&["", "a", "ab", "abc"])).unwrap();
        let result = matcher.match_pattern("", 2).unwrap();
        assert!(result.contains(&"".to_string()));
        assert!(result.contains(&"a".to_string()));
        assert!(result.contains(&"ab".to_string()));
        assert!(!result.contains(&"abc".to_string()));
    }

    // --- match_pattern: empty dictionary ---

    #[test]
    fn empty_dictionary_returns_none() {
        let matcher = Matcher::new(&dict(&[])).unwrap();
        let result = matcher.match_pattern("cat", 2);
        assert!(result.is_none());
    }

    // --- match_pattern: no matches at all ---

    #[test]
    fn no_matches_returns_none() {
        let matcher = Matcher::new(&dict(&["aaa", "bbb"])).unwrap();
        let result = matcher.match_pattern("zzz", 0);
        assert!(result.is_none());
    }

    // --- match_pattern: all words match ---

    #[test]
    fn large_diffs_matches_everything() {
        let matcher = Matcher::new(&dict(&["a", "b", "c"])).unwrap();
        let result = matcher.match_pattern("a", 1).unwrap();
        assert!(result.contains(&"a".to_string()));
        assert!(result.contains(&"b".to_string()));
        assert!(result.contains(&"c".to_string()));
    }

    // --- match_pattern: words sharing prefixes ---

    #[test]
    fn shared_prefix_words() {
        let matcher = Matcher::new(&dict(&["app", "apple", "application"])).unwrap();
        let result = matcher.match_pattern("app", 0).unwrap();
        assert_eq!(result, vec!["app".to_string()]);
    }

    #[test]
    fn shared_prefix_words_with_diffs() {
        let matcher = Matcher::new(&dict(&["app", "apple", "application"])).unwrap();
        let result = matcher.match_pattern("app", 2).unwrap();
        assert!(result.contains(&"app".to_string()));
        assert!(result.contains(&"apple".to_string()));
        assert!(!result.contains(&"application".to_string()));
    }

    // --- match_pattern: single character words ---

    #[test]
    fn single_char_exact() {
        let matcher = Matcher::new(&dict(&["a", "b", "c"])).unwrap();
        let result = matcher.match_pattern("a", 0).unwrap();
        assert_eq!(result, vec!["a".to_string()]);
    }

    #[test]
    fn single_char_pattern_with_diffs() {
        let matcher = Matcher::new(&dict(&["a", "ab", "abc"])).unwrap();
        let result = matcher.match_pattern("a", 1).unwrap();
        assert!(result.contains(&"a".to_string()));
        assert!(result.contains(&"ab".to_string()));
        assert!(!result.contains(&"abc".to_string()));
    }

    // --- match_pattern: longer patterns ---

    #[test]
    fn longer_pattern_exact_match() {
        let matcher = Matcher::new(&dict(&["abcdefgh"])).unwrap();
        let result = matcher.match_pattern("abcdefgh", 0).unwrap();
        assert_eq!(result, vec!["abcdefgh".to_string()]);
    }

    #[test]
    fn longer_pattern_one_diff() {
        let matcher = Matcher::new(&dict(&["abcdefgh", "abcxefgh"])).unwrap();
        let result = matcher.match_pattern("abcdefgh", 1).unwrap();
        assert!(result.contains(&"abcdefgh".to_string()));
        assert!(result.contains(&"abcxefgh".to_string()));
    }

    // --- match_pattern: pattern not in dictionary ---

    #[test]
    fn pattern_not_in_dictionary_but_close_words_match() {
        let matcher = Matcher::new(&dict(&["bat", "cat", "hat"])).unwrap();
        let result = matcher.match_pattern("mat", 1).unwrap();
        assert!(result.contains(&"bat".to_string()));
        assert!(result.contains(&"cat".to_string()));
        assert!(result.contains(&"hat".to_string()));
    }

    // --- match_pattern: different lengths ---

    #[test]
    fn word_shorter_than_pattern() {
        let matcher = Matcher::new(&dict(&["ab"])).unwrap();
        let result = matcher.match_pattern("abc", 1).unwrap();
        assert!(result.contains(&"ab".to_string()));
    }

    #[test]
    fn word_longer_than_pattern() {
        let matcher = Matcher::new(&dict(&["abcd"])).unwrap();
        let result = matcher.match_pattern("abc", 1).unwrap();
        assert!(result.contains(&"abcd".to_string()));
    }

    #[test]
    fn word_much_longer_than_pattern_no_match() {
        let matcher = Matcher::new(&dict(&["abcdef"])).unwrap();
        let result = matcher.match_pattern("abc", 1);
        assert!(result.is_none());
    }

    // --- match_pattern: large dictionary ---

    #[test]
    fn large_dictionary() {
        let words: Vec<String> = (b'a'..=b'z')
            .map(|c| String::from(c as char))
            .collect();
        let matcher = Matcher::new(&words).unwrap();
        let result = matcher.match_pattern("a", 0).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "a");
    }

    #[test]
    fn large_dictionary_with_diffs() {
        let words: Vec<String> = (b'a'..=b'z')
            .map(|c| String::from(c as char))
            .collect();
        let matcher = Matcher::new(&words).unwrap();
        let result = matcher.match_pattern("a", 1).unwrap();
        assert_eq!(result.len(), 26);
    }

    // --- match_pattern: multiple calls reuse the same trie ---

    #[test]
    fn multiple_patterns_on_same_matcher() {
        let matcher = Matcher::new(&dict(&["cat", "car", "bat", "bar"])).unwrap();

        let r1 = matcher.match_pattern("cat", 0).unwrap();
        assert_eq!(r1, vec!["cat".to_string()]);

        let r2 = matcher.match_pattern("bar", 0).unwrap();
        assert_eq!(r2, vec!["bar".to_string()]);

        let r3 = matcher.match_pattern("car", 1).unwrap();
        assert!(r3.contains(&"car".to_string()));
        assert!(r3.contains(&"cat".to_string()));
        assert!(r3.contains(&"bar".to_string()));
    }

    // --- match_pattern: prefix in trie is not a word ---

    #[test]
    fn prefix_not_returned_as_match() {
        let matcher = Matcher::new(&dict(&["card"])).unwrap();
        let result = matcher.match_pattern("car", 0);
        assert!(result.is_none());
    }

    #[test]
    fn prefix_returned_only_if_within_diffs() {
        let matcher = Matcher::new(&dict(&["card"])).unwrap();
        let result = matcher.match_pattern("car", 1).unwrap();
        assert!(result.contains(&"card".to_string()));
    }

    // --- comprehensive: large word families with shared prefixes ---

    fn animal_dict() -> Vec<String> {
        dict(&[
            "cat", "cats", "catch", "catcher", "catching",
            "car", "card", "cards", "care", "cares", "caring", "careful",
            "bat", "bats", "batch", "batcher",
            "hat", "hats", "hatch", "hatcher", "hatching",
            "rat", "rats", "ratch", "rattle", "rattled",
            "mat", "mats", "match", "matcher", "matching",
            "sat", "sit", "set", "sot", "sut",
            "dog", "dogs", "dodge", "dodger",
            "dot", "dots", "dote", "doted",
            "log", "logs", "lodge", "lodger",
            "fog", "fogs",
            "bog", "bogs",
            "hog", "hogs",
        ])
    }

    #[test]
    fn animal_dict_exact_match_cat() {
        let matcher = Matcher::new(&animal_dict()).unwrap();
        let result = matcher.match_pattern("cat", 0).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains(&"cat".to_string()));
    }

    #[test]
    fn animal_dict_one_diff_from_cat() {
        let matcher = Matcher::new(&animal_dict()).unwrap();
        let result = matcher.match_pattern("cat", 1).unwrap();
        let expected_in = ["cat", "cats", "car", "bat", "hat", "rat", "mat", "sat"];
        for w in &expected_in {
            assert!(result.contains(&w.to_string()), "{} should match cat with 1 diff", w);
        }
        let expected_out = ["catch", "catcher", "card", "cards", "care", "batch", "dog", "dogs"];
        for w in &expected_out {
            assert!(!result.contains(&w.to_string()), "{} should NOT match cat with 1 diff", w);
        }
    }

    #[test]
    fn animal_dict_two_diffs_from_cat() {
        let matcher = Matcher::new(&animal_dict()).unwrap();
        let result = matcher.match_pattern("cat", 2).unwrap();
        let expected_in = [
            "cat", "cats", "catch", "car", "card", "care",
            "bat", "bats", "hat", "hats", "rat", "rats",
            "mat", "mats", "sat", "sit", "set", "sot", "sut",
            "dot",
        ];
        for w in &expected_in {
            assert!(result.contains(&w.to_string()), "{} should match cat with 2 diffs", w);
        }
        let expected_out = ["catcher", "catching", "careful", "caring", "rattle", "rattled", "matching", "lodger"];
        for w in &expected_out {
            assert!(!result.contains(&w.to_string()), "{} should NOT match cat with 2 diffs", w);
        }
    }

    #[test]
    fn animal_dict_exact_match_dog() {
        let matcher = Matcher::new(&animal_dict()).unwrap();
        let result = matcher.match_pattern("dog", 0).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains(&"dog".to_string()));
    }

    #[test]
    fn animal_dict_one_diff_from_dog() {
        let matcher = Matcher::new(&animal_dict()).unwrap();
        let result = matcher.match_pattern("dog", 1).unwrap();
        let expected_in = ["dog", "dogs", "dot", "log", "fog", "bog", "hog"];
        for w in &expected_in {
            assert!(result.contains(&w.to_string()), "{} should match dog with 1 diff", w);
        }
        let expected_out = ["dodge", "dodger", "lodge", "lodger", "dots", "dote", "doted"];
        for w in &expected_out {
            assert!(!result.contains(&w.to_string()), "{} should NOT match dog with 1 diff", w);
        }
    }

    #[test]
    fn animal_dict_two_diffs_from_dog() {
        let matcher = Matcher::new(&animal_dict()).unwrap();
        let result = matcher.match_pattern("dog", 2).unwrap();
        let expected_in = ["dog", "dogs", "dodge", "dot", "dots", "dote",
                           "log", "logs", "fog", "fogs", "bog", "bogs", "hog", "hogs"];
        for w in &expected_in {
            assert!(result.contains(&w.to_string()), "{} should match dog with 2 diffs", w);
        }
        let expected_out = ["dodger", "lodger", "doted", "catching", "catcher"];
        for w in &expected_out {
            assert!(!result.contains(&w.to_string()), "{} should NOT match dog with 2 diffs", w);
        }
    }

    #[test]
    fn animal_dict_pattern_not_in_dict_zero_diffs() {
        let matcher = Matcher::new(&animal_dict()).unwrap();
        let result = matcher.match_pattern("caz", 0);
        assert!(result.is_none());
    }

    #[test]
    fn animal_dict_pattern_not_in_dict_one_diff() {
        let matcher = Matcher::new(&animal_dict()).unwrap();
        let result = matcher.match_pattern("caz", 1).unwrap();
        let expected_in = ["cat", "car"];
        for w in &expected_in {
            assert!(result.contains(&w.to_string()), "{} should match caz with 1 diff", w);
        }
        assert!(!result.contains(&"care".to_string()));
    }

    // --- comprehensive: prefix chain at various thresholds ---

    fn prefix_chain_dict() -> Vec<String> {
        dict(&["a", "ab", "abc", "abcd", "abcde", "abcdef", "abcdefg"])
    }

    #[test]
    fn prefix_chain_zero_diffs() {
        let matcher = Matcher::new(&prefix_chain_dict()).unwrap();
        for word in &["a", "ab", "abc", "abcd", "abcde", "abcdef", "abcdefg"] {
            let result = matcher.match_pattern(word, 0).unwrap();
            assert_eq!(result.len(), 1, "exact match for '{}' should return 1 result", word);
            assert_eq!(result[0], word.to_string());
        }
    }

    #[test]
    fn prefix_chain_one_diff_from_abc() {
        let matcher = Matcher::new(&prefix_chain_dict()).unwrap();
        let result = matcher.match_pattern("abc", 1).unwrap();
        assert!(result.contains(&"ab".to_string()));
        assert!(result.contains(&"abc".to_string()));
        assert!(result.contains(&"abcd".to_string()));
        assert!(!result.contains(&"a".to_string()));
        assert!(!result.contains(&"abcde".to_string()));
    }

    #[test]
    fn prefix_chain_two_diffs_from_abc() {
        let matcher = Matcher::new(&prefix_chain_dict()).unwrap();
        let result = matcher.match_pattern("abc", 2).unwrap();
        assert!(result.contains(&"a".to_string()));
        assert!(result.contains(&"ab".to_string()));
        assert!(result.contains(&"abc".to_string()));
        assert!(result.contains(&"abcd".to_string()));
        assert!(result.contains(&"abcde".to_string()));
        assert!(!result.contains(&"abcdef".to_string()));
    }

    #[test]
    fn prefix_chain_three_diffs_from_abc() {
        let matcher = Matcher::new(&prefix_chain_dict()).unwrap();
        let result = matcher.match_pattern("abc", 3).unwrap();
        assert!(result.contains(&"a".to_string()));
        assert!(result.contains(&"ab".to_string()));
        assert!(result.contains(&"abc".to_string()));
        assert!(result.contains(&"abcd".to_string()));
        assert!(result.contains(&"abcde".to_string()));
        assert!(result.contains(&"abcdef".to_string()));
        assert!(!result.contains(&"abcdefg".to_string()));
    }

    #[test]
    fn prefix_chain_one_diff_from_abcdefg() {
        let matcher = Matcher::new(&prefix_chain_dict()).unwrap();
        let result = matcher.match_pattern("abcdefg", 1).unwrap();
        assert!(result.contains(&"abcdefg".to_string()));
        assert!(result.contains(&"abcdef".to_string()));
        assert!(!result.contains(&"abcde".to_string()));
    }

    // --- comprehensive: words differing in all positions ---

    fn similar_words_dict() -> Vec<String> {
        dict(&[
            "abcde",
            "xbcde", "axcde", "abxde", "abcxe", "abcdx",
            "xxcde", "axxde", "abxxe", "abcxx",
            "xxxde", "axxxe", "abxxx",
            "xxxxe", "axxxx",
            "xxxxx",
        ])
    }

    #[test]
    fn similar_words_zero_diffs() {
        let matcher = Matcher::new(&similar_words_dict()).unwrap();
        let result = matcher.match_pattern("abcde", 0).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains(&"abcde".to_string()));
    }

    #[test]
    fn similar_words_one_diff() {
        let matcher = Matcher::new(&similar_words_dict()).unwrap();
        let result = matcher.match_pattern("abcde", 1).unwrap();
        let expected_in = ["abcde", "xbcde", "axcde", "abxde", "abcxe", "abcdx"];
        for w in &expected_in {
            assert!(result.contains(&w.to_string()), "{} should match with 1 diff", w);
        }
        let expected_out = ["xxcde", "axxde", "abxxe", "abcxx", "xxxxx"];
        for w in &expected_out {
            assert!(!result.contains(&w.to_string()), "{} should NOT match with 1 diff", w);
        }
    }

    #[test]
    fn similar_words_two_diffs() {
        let matcher = Matcher::new(&similar_words_dict()).unwrap();
        let result = matcher.match_pattern("abcde", 2).unwrap();
        let expected_in = [
            "abcde", "xbcde", "axcde", "abxde", "abcxe", "abcdx",
            "xxcde", "axxde", "abxxe", "abcxx",
        ];
        for w in &expected_in {
            assert!(result.contains(&w.to_string()), "{} should match with 2 diffs", w);
        }
        let expected_out = ["xxxde", "axxxe", "abxxx", "xxxxe", "axxxx", "xxxxx"];
        for w in &expected_out {
            assert!(!result.contains(&w.to_string()), "{} should NOT match with 2 diffs", w);
        }
    }

    #[test]
    fn similar_words_three_diffs() {
        let matcher = Matcher::new(&similar_words_dict()).unwrap();
        let result = matcher.match_pattern("abcde", 3).unwrap();
        let expected_in = [
            "abcde", "xbcde", "axcde", "abxde", "abcxe", "abcdx",
            "xxcde", "axxde", "abxxe", "abcxx",
            "xxxde", "axxxe", "abxxx",
        ];
        for w in &expected_in {
            assert!(result.contains(&w.to_string()), "{} should match with 3 diffs", w);
        }
        let expected_out = ["xxxxe", "axxxx", "xxxxx"];
        for w in &expected_out {
            assert!(!result.contains(&w.to_string()), "{} should NOT match with 3 diffs", w);
        }
    }

    #[test]
    fn similar_words_five_diffs_matches_all() {
        let matcher = Matcher::new(&similar_words_dict()).unwrap();
        let result = matcher.match_pattern("abcde", 5).unwrap();
        assert_eq!(result.len(), 16);
    }

    // --- comprehensive: mixed edit types (insertion, deletion, substitution) ---

    fn edit_types_dict() -> Vec<String> {
        dict(&[
            "hello",
            "hllo", "helo", "hell",
            "hhello", "heello", "helllo", "helloo",
            "jello", "hullo", "hella", "helms",
            "hllo", "heo", "hel",
            "help", "held", "helm",
            "yellow", "fellow", "bellow", "mellow",
        ])
    }

    #[test]
    fn edit_types_zero_diffs() {
        let matcher = Matcher::new(&edit_types_dict()).unwrap();
        let result = matcher.match_pattern("hello", 0).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains(&"hello".to_string()));
    }

    #[test]
    fn edit_types_one_diff_deletions() {
        let matcher = Matcher::new(&edit_types_dict()).unwrap();
        let result = matcher.match_pattern("hello", 1).unwrap();
        for w in &["hllo", "helo", "hell"] {
            assert!(result.contains(&w.to_string()), "'{}' (1 deletion) should match", w);
        }
    }

    #[test]
    fn edit_types_one_diff_insertions() {
        let matcher = Matcher::new(&edit_types_dict()).unwrap();
        let result = matcher.match_pattern("hello", 1).unwrap();
        for w in &["hhello", "heello", "helllo", "helloo"] {
            assert!(result.contains(&w.to_string()), "'{}' (1 insertion) should match", w);
        }
    }

    #[test]
    fn edit_types_one_diff_substitutions() {
        let matcher = Matcher::new(&edit_types_dict()).unwrap();
        let result = matcher.match_pattern("hello", 1).unwrap();
        for w in &["jello", "hullo", "hella"] {
            assert!(result.contains(&w.to_string()), "'{}' (1 substitution) should match", w);
        }
    }

    #[test]
    fn edit_types_one_diff_excludes_two_edits() {
        let matcher = Matcher::new(&edit_types_dict()).unwrap();
        let result = matcher.match_pattern("hello", 1).unwrap();
        for w in &["heo", "hel", "yellow", "fellow", "bellow", "mellow"] {
            assert!(!result.contains(&w.to_string()), "'{}' should NOT match with 1 diff", w);
        }
    }

    #[test]
    fn edit_types_two_diffs_from_hello() {
        let matcher = Matcher::new(&edit_types_dict()).unwrap();
        let result = matcher.match_pattern("hello", 2).unwrap();
        for w in &["heo", "hel", "help", "held", "helm", "helms"] {
            assert!(result.contains(&w.to_string()), "'{}' should match with 2 diffs", w);
        }
        for w in &["yellow", "fellow", "bellow", "mellow"] {
            assert!(result.contains(&w.to_string()), "'{}' should match with 2 diffs", w);
        }
    }

    // --- comprehensive: increasing allowed_diffs on same pattern ---

    #[test]
    fn increasing_diffs_returns_monotonically_more_results() {
        let matcher = Matcher::new(&animal_dict()).unwrap();
        let mut prev_count = 0;
        for d in 0..=5 {
            let count = matcher.match_pattern("cat", d)
                .map(|v| v.len())
                .unwrap_or(0);
            assert!(count >= prev_count,
                "diffs={} returned {} results, but diffs={} returned {}",
                d, count, d - 1, prev_count);
            prev_count = count;
        }
    }

    #[test]
    fn increasing_diffs_on_dog() {
        let matcher = Matcher::new(&animal_dict()).unwrap();
        let mut prev_count = 0;
        for d in 0..=5 {
            let count = matcher.match_pattern("dog", d)
                .map(|v| v.len())
                .unwrap_or(0);
            assert!(count >= prev_count,
                "diffs={} returned {} results, but diffs={} returned {}",
                d, count, d - 1, prev_count);
            prev_count = count;
        }
    }

    // --- comprehensive: pattern longer than all dictionary words ---

    #[test]
    fn pattern_longer_than_all_words_zero_diffs() {
        let matcher = Matcher::new(&dict(&["a", "ab", "abc"])).unwrap();
        let result = matcher.match_pattern("abcdefgh", 0);
        assert!(result.is_none());
    }

    #[test]
    fn pattern_longer_than_all_words_large_diffs() {
        let matcher = Matcher::new(&dict(&["a", "ab", "abc"])).unwrap();
        let result = matcher.match_pattern("abcdefgh", 7).unwrap();
        assert!(result.contains(&"a".to_string()));
        assert!(result.contains(&"ab".to_string()));
        assert!(result.contains(&"abc".to_string()));
    }

    #[test]
    fn pattern_longer_than_all_words_just_enough_diffs() {
        let matcher = Matcher::new(&dict(&["a", "ab", "abc"])).unwrap();
        let result = matcher.match_pattern("abcde", 4).unwrap();
        assert!(result.contains(&"a".to_string()));
        let result2 = matcher.match_pattern("abcde", 3);
        assert!(result2.is_some());
        let r2 = result2.unwrap();
        assert!(!r2.contains(&"a".to_string()));
        assert!(r2.contains(&"ab".to_string()));
    }

    // --- comprehensive: all same-length words ---

    #[test]
    fn same_length_words_gradual_diffs() {
        let matcher = Matcher::new(&dict(&[
            "aaa", "aab", "aba", "baa",
            "abb", "bab", "bba",
            "bbb",
        ])).unwrap();

        let r0 = matcher.match_pattern("aaa", 0).unwrap();
        assert_eq!(r0.len(), 1);
        assert!(r0.contains(&"aaa".to_string()));

        let r1 = matcher.match_pattern("aaa", 1).unwrap();
        assert!(r1.contains(&"aaa".to_string()));
        assert!(r1.contains(&"aab".to_string()));
        assert!(r1.contains(&"aba".to_string()));
        assert!(r1.contains(&"baa".to_string()));
        assert!(!r1.contains(&"abb".to_string()));
        assert!(!r1.contains(&"bbb".to_string()));

        let r2 = matcher.match_pattern("aaa", 2).unwrap();
        assert!(r2.contains(&"abb".to_string()));
        assert!(r2.contains(&"bab".to_string()));
        assert!(r2.contains(&"bba".to_string()));
        assert!(!r2.contains(&"bbb".to_string()));

        let r3 = matcher.match_pattern("aaa", 3).unwrap();
        assert_eq!(r3.len(), 8);
        assert!(r3.contains(&"bbb".to_string()));
    }

    // --- comprehensive: words with repeated characters ---

    #[test]
    fn repeated_chars_exact() {
        let matcher = Matcher::new(&dict(&["aaa", "bbb", "aab", "abb"])).unwrap();
        let result = matcher.match_pattern("aaa", 0).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn repeated_chars_one_diff() {
        let matcher = Matcher::new(&dict(&["aaa", "bbb", "aab", "abb"])).unwrap();
        let result = matcher.match_pattern("aaa", 1).unwrap();
        assert!(result.contains(&"aaa".to_string()));
        assert!(result.contains(&"aab".to_string()));
        assert!(!result.contains(&"abb".to_string()));
        assert!(!result.contains(&"bbb".to_string()));
    }

    // --- comprehensive: verifying no false positives with unrelated words ---

    #[test]
    fn unrelated_words_never_leak_in() {
        let matcher = Matcher::new(&dict(&[
            "alpha", "bravo", "charlie", "delta", "echo",
            "foxtrot", "golf", "hotel", "india", "juliet",
        ])).unwrap();

        let r = matcher.match_pattern("alpha", 0).unwrap();
        assert_eq!(r.len(), 1);
        assert!(r.contains(&"alpha".to_string()));

        let r1 = matcher.match_pattern("alpha", 1).unwrap();
        for w in &["bravo", "charlie", "foxtrot", "golf", "hotel", "india", "juliet"] {
            assert!(!r1.contains(&w.to_string()), "'{}' should not match alpha with 1 diff", w);
        }

        let r2 = matcher.match_pattern("echo", 1).unwrap();
        assert!(r2.contains(&"echo".to_string()));
        for w in &["alpha", "bravo", "charlie", "foxtrot", "golf", "hotel", "india", "juliet"] {
            assert!(!r2.contains(&w.to_string()), "'{}' should not match echo with 1 diff", w);
        }
    }

    // --- performance tests ---

    use std::time::Instant;

    fn generate_words(count: usize, word_len: usize) -> Vec<String> {
        let chars: Vec<char> = ('a'..='z').collect();
        (0..count).map(|i| {
            (0..word_len).map(|j| chars[(i + j * 7) % 26]).collect()
        }).collect()
    }

    fn generate_similar_words(base: &str, count: usize) -> Vec<String> {
        let chars: Vec<char> = ('a'..='z').collect();
        let base_chars: Vec<char> = base.chars().collect();
        let mut words = vec![base.to_string()];
        for i in 0..count {
            let mut w = base_chars.clone();
            w[i % base_chars.len()] = chars[i % 26];
            words.push(w.into_iter().collect());
        }
        words
    }

    #[test]
    fn perf_build_matcher_1000_words() {
        let words = generate_words(1000, 5);
        let start = Instant::now();
        let matcher = Matcher::new(&words);
        let elapsed = start.elapsed();
        assert!(matcher.is_some());
        eprintln!("Build matcher (1000 words, len 5): {:?}", elapsed);
    }

    #[test]
    fn perf_build_matcher_5000_words() {
        let words = generate_words(5000, 6);
        let start = Instant::now();
        let matcher = Matcher::new(&words);
        let elapsed = start.elapsed();
        assert!(matcher.is_some());
        eprintln!("Build matcher (5000 words, len 6): {:?}", elapsed);
    }

    #[test]
    fn perf_build_matcher_10000_words() {
        let words = generate_words(10000, 8);
        let start = Instant::now();
        let matcher = Matcher::new(&words);
        let elapsed = start.elapsed();
        assert!(matcher.is_some());
        eprintln!("Build matcher (10000 words, len 8): {:?}", elapsed);
    }

    #[test]
    fn perf_query_zero_diffs_1000_words() {
        let words = generate_words(1000, 5);
        let matcher = Matcher::new(&words).unwrap();
        let start = Instant::now();
        for w in &words[..100] {
            matcher.match_pattern(w, 0);
        }
        let elapsed = start.elapsed();
        eprintln!("100 queries (0 diffs, 1000 words): {:?}", elapsed);
    }

    #[test]
    fn perf_query_one_diff_1000_words() {
        let words = generate_words(1000, 5);
        let matcher = Matcher::new(&words).unwrap();
        let start = Instant::now();
        for w in &words[..100] {
            matcher.match_pattern(w, 1);
        }
        let elapsed = start.elapsed();
        eprintln!("100 queries (1 diff, 1000 words): {:?}", elapsed);
    }

    #[test]
    fn perf_query_two_diffs_1000_words() {
        let words = generate_words(1000, 5);
        let matcher = Matcher::new(&words).unwrap();
        let start = Instant::now();
        for w in &words[..100] {
            matcher.match_pattern(w, 2);
        }
        let elapsed = start.elapsed();
        eprintln!("100 queries (2 diffs, 1000 words): {:?}", elapsed);
    }

    #[test]
    fn perf_query_zero_diffs_5000_words() {
        let words = generate_words(5000, 6);
        let matcher = Matcher::new(&words).unwrap();
        let start = Instant::now();
        for w in &words[..100] {
            matcher.match_pattern(w, 0);
        }
        let elapsed = start.elapsed();
        eprintln!("100 queries (0 diffs, 5000 words): {:?}", elapsed);
    }

    #[test]
    fn perf_query_one_diff_5000_words() {
        let words = generate_words(5000, 6);
        let matcher = Matcher::new(&words).unwrap();
        let start = Instant::now();
        for w in &words[..100] {
            matcher.match_pattern(w, 1);
        }
        let elapsed = start.elapsed();
        eprintln!("100 queries (1 diff, 5000 words): {:?}", elapsed);
    }

    #[test]
    fn perf_query_two_diffs_5000_words() {
        let words = generate_words(5000, 6);
        let matcher = Matcher::new(&words).unwrap();
        let start = Instant::now();
        for w in &words[..100] {
            matcher.match_pattern(w, 2);
        }
        let elapsed = start.elapsed();
        eprintln!("100 queries (2 diffs, 5000 words): {:?}", elapsed);
    }

    #[test]
    fn perf_long_words() {
        let words = generate_words(500, 15);
        let matcher = Matcher::new(&words).unwrap();
        let start = Instant::now();
        for w in &words[..50] {
            matcher.match_pattern(w, 1);
        }
        let elapsed = start.elapsed();
        eprintln!("50 queries (1 diff, 500 words, len 15): {:?}", elapsed);
    }

    #[test]
    fn perf_many_similar_words() {
        let words = generate_similar_words("hello", 200);
        let matcher = Matcher::new(&words).unwrap();
        let start = Instant::now();
        let result = matcher.match_pattern("hello", 1);
        let elapsed = start.elapsed();
        assert!(result.is_some());
        let matches = result.unwrap();
        assert!(matches.len() > 1);
        eprintln!("1 query (1 diff, 200 similar words): {:?}, {} matches", elapsed, matches.len());
    }

    #[test]
    fn perf_repeated_queries_same_pattern() {
        let words = generate_words(1000, 5);
        let matcher = Matcher::new(&words).unwrap();
        let pattern = &words[0];
        let start = Instant::now();
        for _ in 0..500 {
            matcher.match_pattern(pattern, 1);
        }
        let elapsed = start.elapsed();
        eprintln!("500 repeated queries (same pattern, 1 diff, 1000 words): {:?}", elapsed);
    }

    #[test]
    fn perf_increasing_diffs_scaling() {
        let words = generate_words(1000, 5);
        let matcher = Matcher::new(&words).unwrap();
        let pattern = &words[0];
        for d in 0..=3 {
            let start = Instant::now();
            let result = matcher.match_pattern(pattern, d);
            let elapsed = start.elapsed();
            let count = result.map(|v| v.len()).unwrap_or(0);
            eprintln!("  diffs={}: {:?}, {} matches", d, elapsed, count);
        }
    }

    #[test]
    fn perf_worst_case_high_diffs_short_words() {
        let words: Vec<String> = {
            let chars = ['a', 'b', 'c', 'd'];
            let mut result = Vec::new();
            for &a in &chars {
                for &b in &chars {
                    for &c in &chars {
                        result.push(format!("{}{}{}", a, b, c));
                    }
                }
            }
            result
        };
        let matcher = Matcher::new(&words).unwrap();
        let start = Instant::now();
        let result = matcher.match_pattern("abc", 3).unwrap();
        let elapsed = start.elapsed();
        assert_eq!(result.len(), 64);
        eprintln!("All 3-char combos over {{a,b,c,d}} (64 words), 3 diffs: {:?}", elapsed);
    }

    #[test]
    fn perf_large_alphabet() {
        let chars: Vec<char> = ('a'..='z').chain('A'..='Z').collect();
        let words: Vec<String> = (0..2000).map(|i| {
            (0..6).map(|j| chars[(i * 3 + j * 11) % chars.len()]).collect()
        }).collect();
        let matcher = Matcher::new(&words).unwrap();
        let start = Instant::now();
        for w in &words[..50] {
            matcher.match_pattern(w, 1);
        }
        let elapsed = start.elapsed();
        eprintln!("50 queries (1 diff, 2000 words, 52-char alphabet): {:?}", elapsed);
    }
}
