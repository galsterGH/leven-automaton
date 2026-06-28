
use std::{collections::HashMap};

/// A node in a Trie (prefix tree) data structure.
/// Each node holds a map of children keyed by character, and optionally stores
/// the full word if this node represents the end of an inserted word.
#[derive(Clone)]
pub struct TrieNode{
    nodes: HashMap<char,TrieNode>,
    word: Option<String>,
}

impl TrieNode {
    /// Creates a new empty trie node with no children and no word.
    pub fn new() -> Self{
        TrieNode {
            nodes : HashMap::new(),
            word : None,
        }
    }

    /// Inserts a word into the trie, creating intermediate nodes as needed.
    /// Returns true upon successful insertion.
    pub fn add_new_word(& mut self, word : &str) -> bool {
        let mut curr_node = self;
        
        for c in word.chars() {
            let temp = curr_node.nodes.entry(c).or_insert(TrieNode { nodes: HashMap::new(), word: None });
            curr_node = temp;
        }

        curr_node.word = Some(word.to_owned());  
        true 
     }

    /// Returns true if the exact word exists in the trie.
    pub fn does_word_exist(&self, word : &str)-> bool{
        self.does_word_exist_internal(word).unwrap_or(false)
     }

    /// Returns a reference to the child node for the given character, or None if no such child exists.
    pub fn get_next_node(&self, c : char) -> Option<&TrieNode>{
        self.nodes.get(&c)
    }

    /// Returns a clone of the word stored at this node, or None if this node is not a word endpoint.
    pub fn get_node_word(&self)->Option<String> {
        self.word.to_owned()
    }

    /// Returns an iterator over all child entries (character, child node) of this node.
    pub fn get_all_children(&self) -> impl Iterator<Item=(&char,&TrieNode)>{
        self.nodes.iter()
    }

    /// Traverses the trie following each character of `word`, returning Some(true) if the
    /// word is found, Some(false) if the path exists but no word is stored, or None if a
    /// character has no matching child.
    fn does_word_exist_internal(&self, word : &str)-> Option<bool> {
        let mut node = self;
        
        for c in word.chars() {
            node = node.nodes.get(&c)?;
        }

        node.word.as_ref().map(|s| *s == word)
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_empty_trie() {
        let trie = TrieNode::new();
        assert!(trie.nodes.is_empty());
        assert!(trie.word.is_none());
    }

    #[test]
    fn add_single_word() {
        let mut trie = TrieNode::new();
        assert!(trie.add_new_word("hello"));
        assert!(trie.does_word_exist("hello"));
    }

    #[test]
    fn search_nonexistent_word() {
        let mut trie = TrieNode::new();
        trie.add_new_word("hello");
        assert!(!trie.does_word_exist("world"));
    }

    #[test]
    fn search_empty_trie() {
        let trie = TrieNode::new();
        assert!(!trie.does_word_exist("hello"));
    }

    #[test]
    fn prefix_is_not_a_word() {
        let mut trie = TrieNode::new();
        trie.add_new_word("hello");
        assert!(!trie.does_word_exist("hel"));
    }

    #[test]
    fn word_is_not_found_by_extension() {
        let mut trie = TrieNode::new();
        trie.add_new_word("hel");
        assert!(!trie.does_word_exist("hello"));
    }

    #[test]
    fn add_multiple_words() {
        let mut trie = TrieNode::new();
        trie.add_new_word("cat");
        trie.add_new_word("car");
        trie.add_new_word("card");
        assert!(trie.does_word_exist("cat"));
        assert!(trie.does_word_exist("car"));
        assert!(trie.does_word_exist("card"));
        assert!(!trie.does_word_exist("ca"));
    }

    #[test]
    fn add_word_that_is_prefix_of_existing() {
        let mut trie = TrieNode::new();
        trie.add_new_word("hello");
        trie.add_new_word("hel");
        assert!(trie.does_word_exist("hello"));
        assert!(trie.does_word_exist("hel"));
    }

    #[test]
    fn add_word_that_extends_existing() {
        let mut trie = TrieNode::new();
        trie.add_new_word("hel");
        trie.add_new_word("hello");
        assert!(trie.does_word_exist("hel"));
        assert!(trie.does_word_exist("hello"));
    }

    #[test]
    fn add_duplicate_word() {
        let mut trie = TrieNode::new();
        trie.add_new_word("hello");
        trie.add_new_word("hello");
        assert!(trie.does_word_exist("hello"));
    }

    #[test]
    fn single_char_word() {
        let mut trie = TrieNode::new();
        trie.add_new_word("a");
        assert!(trie.does_word_exist("a"));
        assert!(!trie.does_word_exist("b"));
    }

    #[test]
    fn empty_word_supported() {
        let mut trie = TrieNode::new();
        trie.add_new_word("");
        assert!(trie.does_word_exist(""));
    }

    #[test]
    fn empty_word_not_added() {
        let trie = TrieNode::new();
        assert!(!trie.does_word_exist(""));
    }

    #[test]
    fn words_with_shared_prefix() {
        let mut trie = TrieNode::new();
        trie.add_new_word("abc");
        trie.add_new_word("abd");
        trie.add_new_word("xyz");
        assert!(trie.does_word_exist("abc"));
        assert!(trie.does_word_exist("abd"));
        assert!(trie.does_word_exist("xyz"));
        assert!(!trie.does_word_exist("ab"));
        assert!(!trie.does_word_exist("xy"));
    }

    #[test]
    fn get_next_node_existing_char() {
        let mut trie = TrieNode::new();
        trie.add_new_word("abc");
        let node = trie.get_next_node('a');
        assert!(node.is_some());
        let node = node.unwrap();
        assert!(node.get_next_node('b').is_some());
    }

    #[test]
    fn get_next_node_missing_char() {
        let mut trie = TrieNode::new();
        trie.add_new_word("abc");
        assert!(trie.get_next_node('x').is_none());
    }

    #[test]
    fn get_next_node_walk_full_word() {
        let mut trie = TrieNode::new();
        trie.add_new_word("cat");
        let c = trie.get_next_node('c').unwrap();
        let a = c.get_next_node('a').unwrap();
        let t = a.get_next_node('t').unwrap();
        assert_eq!(t.word, Some("cat".to_string()));
    }

    #[test]
    fn get_next_node_intermediate_has_no_word() {
        let mut trie = TrieNode::new();
        trie.add_new_word("cat");
        let c = trie.get_next_node('c').unwrap();
        assert!(c.word.is_none());
    }

    #[test]
    fn many_words() {
        let mut trie = TrieNode::new();
        let words = vec!["apple", "app", "application", "bat", "ball", "band", "banana"];
        for w in &words {
            trie.add_new_word(w);
        }
        for w in &words {
            assert!(trie.does_word_exist(w));
        }
        assert!(!trie.does_word_exist("ap"));
        assert!(!trie.does_word_exist("ba"));
        assert!(!trie.does_word_exist("ban"));
        assert!(!trie.does_word_exist("banan"));
    }
}
