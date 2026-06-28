# Levenshtein-automaton

A Levenshtein automaton implementation in Rust. Given a pattern string and a maximum edit distance, it pre-computes a deterministic finite automaton (DFA) that can efficiently test whether any input word is within the allowed Levenshtein distance from the pattern.

## What is a Levenshtein Automaton?

A Levenshtein automaton accepts all strings that are within a given edit distance *k* of a fixed pattern string. The three edit operations counted are:

- **Substitution** - replacing one character with another
- **Insertion** - adding a character
- **Deletion** - removing a character

The automaton is built once for a given pattern and distance threshold, then can match many words in O(n) time per word (where n is the word length), with no additional allocations.

## Single Word Usage

```rust
use leven_automaton::automaton::LevenshteinAutomaton;

fn main() {
    let alphabet: Vec<char> = ('a'..='z').collect();
    let automaton = LevenshteinAutomaton::new("hello", 2, alphabet)
        .expect("failed to build automaton");

    assert!(automaton.match_word("hello"));  // exact match (distance 0)
    assert!(automaton.match_word("hallo"));  // 1 substitution
    assert!(automaton.match_word("helo"));   // 1 deletion
    assert!(automaton.match_word("helloo")); // 1 insertion
    assert!(!automaton.match_word("world")); // distance 4, exceeds threshold
}
```

## Dictionary Fuzzy Search

The `Matcher` combines the automaton with a Trie to efficiently search an entire dictionary for all words within a given edit distance:

```rust
use leven_automaton::matcher::Matcher;

fn main() {
    let dictionary = vec![
        "cat".to_string(), "car".to_string(), "card".to_string(),
        "bat".to_string(), "hat".to_string(), "dog".to_string(),
    ];
    let matcher = Matcher::new(&dictionary).expect("failed to build matcher");

    let results = matcher.match_pattern("cat", 1).unwrap();
    // returns: ["cat", "car", "bat", "hat"] (all within 1 edit)

    let exact = matcher.match_pattern("dog", 0).unwrap();
    // returns: ["dog"]
}
```

## API

### `LevenshteinAutomaton::new(pattern, diffs_allowed, alphabet) -> Option<Self>`

Builds the automaton. Returns `None` if the state space overflows (very long patterns with large edit distances).

- `pattern` - the target string to match against
- `diffs_allowed` - maximum number of edits permitted
- `alphabet` - the set of characters the automaton will recognize. Words containing characters outside this alphabet will not match.

### `automaton.match_word(word) -> bool`

Returns `true` if the Levenshtein distance between `word` and the pattern is at most `diffs_allowed`.

### `Matcher::new(dictionary) -> Option<Self>`

Builds a Matcher from a list of dictionary words. Constructs a Trie internally and collects the alphabet from all characters in the dictionary.

### `matcher.match_pattern(pattern, allowed_diffs) -> Option<Vec<String>>`

Returns all dictionary words within `allowed_diffs` edit distance of `pattern`, or `None` if no matches are found.

## How It Works

1. **State representation**: Each state is a vector of edit distances between prefixes of the pattern and the input consumed so far. Values are clamped to `diffs_allowed + 1` to bound the state space.

2. **State identification**: States are encoded as unique integer IDs using a mixed-radix representation (base `diffs_allowed + 2`), enabling efficient deduplication via a HashMap.

3. **Automaton construction**: A BFS explores all reachable states from the initial state, computing transitions for each character in the alphabet. Equivalent states (same edit distance vector) are merged.

4. **Matching**: The input word is consumed character by character, following transitions in the pre-computed DFA. The final state determines acceptance.

5. **Trie-Automaton intersection**: The `Matcher` walks the Trie and DFA in lockstep via DFS, pruning entire subtrees when a dead state is reached. This avoids checking every word individually.

## Building

```bash
cargo build
```

## Testing

```bash
cargo test
```

The test suite (169 tests) covers:

- State initialization and edit distance computation
- State ID generation and uniqueness
- Character transitions (match, substitution, insertion, deletion)
- Dead state detection and pruning
- Acceptance criteria
- End-to-end automaton matching with exact matches, edits, edge cases (empty strings, single characters), and alphabet restrictions
- Trie insertion, lookup, prefix handling, and traversal
- Matcher dictionary fuzzy search with large word families, shared prefixes, all edit types, and increasing diff thresholds
- Performance tests with dictionaries up to 10,000 words

## Project Structure

```text
src/
  main.rs              - Entry point and module declarations
  state/mod.rs         - State and StateId types, edit distance logic
  automaton/mod.rs     - LevenshteinAutomaton, BFS construction, matching
  utilities/mod.rs     - Trie data structure for dictionary storage
  matcher/mod.rs       - Matcher combining Trie + Automaton for fuzzy dictionary search
```

## License

MIT
