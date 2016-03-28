#![feature(plugin)]
#![plugin(stainless)]

// Note: the algorithm for this tree is based off the algorithm in
// https://en.wikipedia.org/wiki/Red%E2%80%93black_tree

#[macro_use]
#[cfg(not(test))]
mod node;

#[macro_use]
#[cfg(test)]
pub mod node;

use node::{Node, Link, Color, Dir, NodeHelper, Follow};

use std::mem;

pub struct RedBlackTree<T> where T: PartialOrd {
    root: Link<T>,
    count: usize,
}

impl<T> RedBlackTree<T> where T: PartialOrd {
    pub fn new() -> RedBlackTree<T> {
        RedBlackTree {
            root: None,
            count: 0,
        }
    }
    
    pub fn insert(&mut self, value: T) {
        Node::insert_n(value, &mut self.root);
        self.count += 1;
    }

    pub fn remove(&mut self, value: &T) -> Option<T> {
        let ret = Node::remove_n(value, &mut self.root);
        if ret.is_some() {
            self.count -= 1;
        }
        
        ret
    }
    
    pub fn iter(&self) -> RedBlackIterator<T> {
        RedBlackIterator::new(self)
    }
}

pub struct RedBlackIterator<'a, T> where T: PartialOrd + 'a {
    parents: Vec<&'a Link<T>>,
    current: Option<&'a Link<T>>,
}

impl<'a, T> RedBlackIterator<'a, T> where T: PartialOrd {
    fn new(tree: &RedBlackTree<T>) -> RedBlackIterator<T> {
        let mut parents = vec![];
        let mut node = &tree.root;
        
        if node.is_none() {
            return RedBlackIterator {
                parents: parents,
                current: None,
            }
        }
        
        while node.left().is_some() {
            let old_node = node;
            node = node.left();
            parents.push(old_node);
        }
        
        RedBlackIterator {
            parents: parents,
            current: Some(node),
        }
    }
}

impl<'a, T> Iterator for RedBlackIterator<'a, T> where T: PartialOrd {
    type Item = &'a T;
    
    fn next(&mut self) -> Option<&'a T> {
        if self.current.is_none() {
            return None;
        }
        
        let value = self.current.unwrap().value();
        match *self.current.unwrap().right() {
            None => {
                self.current = self.parents.pop();
            },
            Some(_) => {
                self.current = self.current.map(|n| n.right());
                
                while self.current.unwrap().left().is_some() {
                    let old_current = self.current.unwrap();
                    self.current = self.current.map(|n| n.left());
                    self.parents.push(old_current);
                }
            },
        }
        
        Some(value)
    }
}

fn get_dir(is_less: bool) -> Dir {
    if is_less { Dir::Left } else { Dir::Right }
}

impl<T> Node<T> where T: PartialOrd {
    fn insert_n(value: T, node: &mut Link<T>) {
        if node.is_none() {
            *node = Some(Box::new(Node::new(value)));
            Self::ensure_root_black(node);
        } else {
            let dir = get_dir(value < *node.value());
            if let Some(0) = Self::insert_p(value, node, dir) {
                Self::ensure_root_black(node);
            }
        }
    }
    
    fn insert_p(value: T, parent: &mut Link<T>, n_dir: Dir) -> Option<usize> {
        if parent.follow(n_dir).is_none() {
            let mut node = parent.follow_mut(n_dir);
            *node = Some(Box::new(Node::new(value)));
            None
        } else {
            let dir = get_dir(value < *parent.follow(n_dir).value());
            match Self::insert_g(value, parent, n_dir, dir) {
                None | Some(0) => None,
                Some(rest) => Some(rest - 1),
            }
        }
    }
    
    fn insert_g(value: T, grandparent: &mut Link<T>, p_dir: Dir, n_dir: Dir) -> Option<usize> {
        if follow!(grandparent, p_dir, n_dir).is_none() {
            {
                let mut node = follow_mut!(grandparent, p_dir, n_dir);
                *node = Some(Box::new(Node::new(value)));
            }
            match Self::ensure_parent_black(grandparent, p_dir, n_dir) {
                None | Some(0) => None,
                Some(rest) => Some(rest - 1),
            }
        } else {
            let dir = get_dir(value < *follow!(grandparent, p_dir, n_dir).value());
            match Self::insert_g(value, grandparent.follow_mut(p_dir), n_dir, dir) {
                Some(0) => Self::ensure_parent_black(grandparent, p_dir, n_dir),
                Some(rest) => Some(rest - 1),
                None => None,
            }
        }
    }
    
    fn ensure_root_black(node: &mut Link<T>) {
        node.set_color(Color::Black);
    }
    
    fn ensure_parent_black(grandparent: &mut Link<T>, p_dir: Dir, n_dir: Dir) -> Option<usize> {
        match *grandparent.follow(p_dir).color() {
            Color::Red => Self::ensure_balanced_when_uncle_red(grandparent, p_dir, n_dir),
            Color::Black => None,
        }
    }
    
    fn ensure_balanced_when_uncle_red(grandparent: &mut Link<T>, p_dir: Dir, n_dir: Dir) -> Option<usize> {
        let uncle_red = {
            let uncle = grandparent.follow_mut(p_dir.opposite());
            match *uncle {
                None => false,
                _ => match *uncle.color() {
                    Color::Black => false,
                    Color::Red => {
                        uncle.set_color(Color::Black);
                        true
                    },
                },
            }
        };

        if uncle_red {
            grandparent.follow_mut(p_dir).set_color(Color::Black);
            grandparent.set_color(Color::Red);
            Some(2)
        } else {
            Self::ensure_balanced_when_uncle_black(grandparent, p_dir, n_dir);
            None
        }
    }
    
    fn ensure_balanced_when_uncle_black(grandparent: &mut Link<T>, p_dir: Dir, n_dir: Dir) {
        match (p_dir, n_dir) {
            (Dir::Left, Dir::Right) => {
                Self::rotate_left(grandparent.follow_mut(p_dir));
            },
            (Dir::Right, Dir::Left) => {
                Self::rotate_right(grandparent.follow_mut(p_dir));
            },
            _ => (),
        }
        
        grandparent.set_color(Color::Red);
        grandparent.follow_mut(p_dir).set_color(Color::Black);
        match n_dir {
            Dir::Left => Self::rotate_right(grandparent),
            Dir::Right => Self::rotate_left(grandparent),
        }
    }
    
    fn rotate_left(parent: &mut Link<T>) {
        let mut parent = parent;
        let mut node = parent.right_mut().take();
        let node_left = node.left_mut().take();
        parent.set_right(node_left);
        mem::swap(parent, &mut node);
        
        // so as to avoid confusion
        let new_node = parent;
        let new_parent = node;
        new_node.set_left(new_parent);
    }
    
    fn rotate_right(parent: &mut Link<T>) {
        let mut parent = parent;
        let mut node = parent.left_mut().take();
        let node_right = node.right_mut().take();
        parent.set_left(node_right);
        mem::swap(parent, &mut node);
        
        // so as to avoid confusion
        let new_node = parent;
        let new_parent = node;
        new_node.set_right(new_parent);
    }
    
    fn remove_n(value: &T, node: &mut Link<T>) -> Option<T> {
        if node.is_none() {
            return None;
        }
        
        if *node.value() == *value {
            Some(Self::find_child_to_delete(node).0)
        } else {
            let dir = get_dir(*node.value() < *value);
            Self::remove_p(value, node, dir).0
        }
    }
    
    fn remove_p(value: &T, parent: &mut Link<T>, n_dir: Dir) -> (Option<T>, bool) {
        if parent.follow(n_dir).is_none() {
            return (None, false);
        }
        
        if *parent.follow(n_dir).value() == *value {
            let (value, should_fix_parent) = Self::find_child_to_delete(parent.follow_mut(n_dir));
            (Some(value), should_fix_parent)
        } else {
            let dir = get_dir(*parent.follow(n_dir).value() < *value);
            let (ret, mut should_fix_parent) = Self::remove_p(value, parent.follow_mut(n_dir), dir);
            if should_fix_parent {
                should_fix_parent = Self::delete_case2(parent, n_dir);
            }
            
            (ret, should_fix_parent)
        }
    }
    
    fn find_child_to_delete(node: &mut Link<T>) -> (T, bool) {
        if node.left().is_none() || node.right().is_none() {
            return Self::delete_one_child(node);
        }
        
        let (mut value, mut should_fix_parent) = {
            let left = node.left_mut();
            Self::find_largest_child_to_delete(left)
        };
        
        mem::swap(&mut value, node.value_mut());
        
        if should_fix_parent {
            should_fix_parent = Self::delete_case2(node, Dir::Left);
        }
        
        (value, should_fix_parent)
    }
    
    fn find_largest_child_to_delete(node: &mut Link<T>) -> (T, bool) {
        if node.right().is_none() {
            return Self::delete_one_child(node);
        }
        
        let (value, mut should_fix_parent) = {
            let right = node.right_mut();
            Self::find_largest_child_to_delete(right)
        };
        
        if should_fix_parent {
            should_fix_parent = Self::delete_case2(node, Dir::Right);
        }
        
        (value, should_fix_parent)
    }
    
    fn delete_one_child(node: &mut Link<T>) -> (T, bool) {
        let child = (if node.right().is_none() {
            node.left_mut()
        } else {
            node.right_mut()
        }).take();
        
        let mut new_is_black = false;
        let original_node = mem::replace(node, child);
        if original_node.is_black() {
            new_is_black = node.is_black();
            if !new_is_black {
                node.set_color(Color::Black);
            }
        }
        
        (original_node.take_value(), new_is_black)
    }
    
    fn delete_case2(parent: &mut Link<T>, n_dir: Dir) -> bool {
        let mut is_red = false;
        {
            let sibling = parent.follow_mut(n_dir.opposite());
            if sibling.is_red() {
                is_red = true;
                sibling.set_color(Color::Black);
            }
        }
        
        if is_red {
            parent.set_color(Color::Red);
            match n_dir {
                Dir::Left => Self::rotate_left(parent),
                Dir::Right => Self::rotate_right(parent),
            }
        }
        
        Self::delete_case3(parent, n_dir)
    }
    
    fn delete_case3(parent: &mut Link<T>, n_dir: Dir) -> bool{
        if parent.is_red() {
           Self::delete_case4(parent, n_dir);
           return false;
        }
        
        let sibling_needs_change: bool;
        {
            let sibling = parent.follow_mut(n_dir.opposite());
            sibling_needs_change = sibling.is_black()
                && sibling.left().is_black()
                && sibling.right().is_black();
            
            if sibling_needs_change {
                sibling.set_color(Color::Red);
            }
        }
        
        if !sibling_needs_change {
            Self::delete_case4(parent, n_dir);
        }
        
        sibling_needs_change
    }
    
    fn delete_case4(parent: &mut Link<T>, n_dir: Dir) {
        if parent.is_red()
            && parent.follow(n_dir.opposite()).is_black()
            && parent.follow(n_dir.opposite()).left().is_black()
            && parent.follow(n_dir.opposite()).right().is_black() {
            parent.follow_mut(n_dir.opposite()).set_color(Color::Red);
            parent.set_color(Color::Black);
        } else {
            Self::delete_case5(parent, n_dir);
            Self::delete_case6(parent, n_dir);
        }
    }
    
    fn delete_case5(parent: &mut Link<T>, n_dir: Dir) {
        if parent.follow(n_dir.opposite()).is_black() {
            let mut rotate_right = false;
            let mut rotate_left = false;
            
            {
                let sibling = parent.follow_mut(n_dir.opposite());
                match n_dir {
                    Dir::Left => {
                        if sibling.right().is_black() && sibling.left().is_red() {
                            sibling.set_color(Color::Red);
                            sibling.left_mut().set_color(Color::Black);
                            rotate_right = true;
                        }
                    },
                    Dir::Right => {
                        if sibling.left().is_black() && sibling.right().is_red() {
                            sibling.set_color(Color::Red);
                            sibling.right_mut().set_color(Color::Black);
                            rotate_left = true;
                        }
                    },
                }
            }
            
            if rotate_right {
                Self::rotate_right(parent);
            } else if rotate_left {
                Self::rotate_left(parent);
            }
        }
    }
    
    fn delete_case6(parent: &mut Link<T>, n_dir: Dir) {
        let parent_color = *parent.color();
        parent.set_color(Color::Black);
        let mut rotate_left = false;
        let mut rotate_right = false;
        
        {
            let sibling = parent.follow_mut(n_dir.opposite());
            sibling.set_color(parent_color);
        
            match n_dir {
                Dir::Left => {
                    sibling.right_mut().set_color(Color::Black);
                    rotate_left = true;
                },
                Dir::Right => {
                    sibling.left_mut().set_color(Color::Black);
                    rotate_right = true;
                }
            }
        }
        
        if rotate_left {
            Self::rotate_left(parent);
        } else if rotate_right {
            Self::rotate_right(parent);
        }
    }
}

#[macro_export]
macro_rules! rb_tree [
    ($($item:expr),*) => ({
        let mut _tree = RedBlackTree::new();
        $(_tree.insert($item);)*
        _tree
    });
];


#[cfg(test)]
#[macro_use(expect)]
extern crate expectest;

#[cfg(test)]
mod unit_tests {
    pub use expectest::prelude::*;
    
    use std::fmt::Debug;
    pub use super::*;
    pub use super::node::*;
    
    // print before each insert so we can see how the tree
    // is being constructed
    macro_rules! rb_tree_print [
        ($($item:expr),*) => ({
            let mut _tree = RedBlackTree::new();
            $(
                println!("--\n");
                print_tree(&_tree);
                _tree.insert($item);
            )*
            _tree
        })
    ];
    
    impl PartialEq for Color {
        fn eq(&self, other: &Self) -> bool {
            match *self {
                Color::Red => match *other {
                    Color::Red => true,
                    _ => false,
                },
                Color::Black => match *other {
                    Color::Black => true,
                    _ => false,
                }
            }
        }
    }
    
    // Allows you to verify a tree's structure
    // by using the same format as the output
    // of the print_tree function:
    // verify!{ tree =>
    //         < B.2 >
    //     B.1   < R.4 >
    //         B.3   < B.6 >
    //             R.5     R.7
    // };
    macro_rules! verify {
        // helper rule
        (@expr $e:expr) => {$e};
        
        (@color R) => { Color::Red };
        (@color B) => { Color::Black };
        
        (@expect $node:expr => $c:tt.$v:tt) => {{
            expect!($node.as_ref()).to(be_some()
                .value(&Box::new(node(
                    verify!(@color $c),
                    verify!(@expr $v)))));
        }};
        (@expect $node:expr => None) => {{
            expect!($node.as_ref()).to(be_none());
        }};
        
        // edge case where root is None
        (@match_node [$node:expr] => None) => {
            verify!(@expect $node => None);
        };
        // two children
        (@match_node [$node:expr, $($stack:expr),+] => <$c:tt.$v:tt> $($nodes:tt)+) => {{
            verify!(@expect $node => $c.$v);
            verify!(@match_node [$($stack),+, $node.left(), $node.right()] => $($nodes)+);
        }};
        // two children and one stack item
        (@match_node [$node:expr] => <$c:tt.$v:tt> $($nodes:tt)+) => {{
            verify!(@expect $node => $c.$v);
            verify!(@match_node [$node.left(), $node.right()] => $($nodes)+);
        }};
        // only right child
        (@match_node [$node:expr, $($stack:expr),+] => $c:tt.$v:tt> $($nodes:tt)+) => {{
            verify!(@expect $node => $c.$v);
            verify!(@expect $node.left() => None);
            verify!(@match_node [$($stack),+, $node.right()] => $($nodes)+);
        }};
        // only right child and one stack item
        (@match_node [$node:expr] => $c:tt.$v:tt> $($nodes:tt)+) => {{
            verify!(@expect $node => $c.$v);
            verify!(@expect $node.left() => None);
            verify!(@match_node [$node.right()] => $($nodes)+);
        }};
        // only left child
        (@match_node [$node:expr, $($stack:expr),+] => <$c:tt.$v:tt $($nodes:tt)+) => {{
            verify!(@expect $node => $c.$v);
            verify!(@expect $node.right() => None);
            verify!(@match_node [$($stack),+, $node.left()] => $($nodes)+);
        }};
        // only left child and one stack item
        (@match_node [$node:expr] => <$c:tt.$v:tt $($nodes:tt)+) => {{
            verify!(@expect $node => $c.$v);
            verify!(@expect $node.right() => None);
            verify!(@match_node [$node.left()] => $($nodes)+);
        }};
        // no children
        (@match_node [$node:expr, $($stack:expr),+] => $c:tt.$v:tt $($nodes:tt)+) => {{
            verify!(@expect $node => $c.$v);
            verify!(@expect $node.right() => None);
            verify!(@expect $node.left() => None);
            verify!(@match_node [$($stack),+] => $($nodes)+);
        }};
        // no children and one stack item
        (@match_node [$node:expr] => $c:tt.$v:tt) => {{
            verify!(@expect $node => $c.$v);
            verify!(@expect $node.left() => None);
            verify!(@expect $node.right() => None);
        }};
    
        // initial macro call
        ($tree:expr => $($nodes:tt)+) => {
            verify!(@match_node [$tree.root] => $($nodes)+);
        };
    }
    
    // Will print out a tree like the following:
    //   <B.2>
    // B.1     <R.4>
    //       B.3   B.5>
    //                R.6
    #[allow(dead_code)]
    pub fn print_tree<T: PartialOrd + Debug> (tree: &RedBlackTree<T>) {
        let link = &tree.root;
        if link.is_none() {
            println!("None");
            return;
        }
        
        let mut print_queue = vec![(0, link)];
        while !print_queue.is_empty() {
            let mut new_print_queue = vec![];
            let mut printed_offset = 0;
            for (link_offset, link) in print_queue {
                printed_offset += print_link(
                    link,
                    link_offset,
                    link_offset - printed_offset,
                    &mut new_print_queue);
            }
            
            print_queue = new_print_queue;
            println!("\n");
        }
    }
    
    fn tree_width<T: PartialOrd + Debug>(link: &Link<T>) -> usize {
        match *link {
            None => 0,
            _ => tree_width(link.left())
                + link_str(link).len()
                + tree_width(link.right())
        }
    }
    
    fn link_str<T: PartialOrd + Debug> (link: &Link<T>) -> String {
        format!("{:?}", link.as_ref().unwrap())
    }
    
    fn print_link<'a, 'b, T>(link: &'a Link<T>, offset: usize, printed_offset: usize, print_queue: &'b mut Vec<(usize, &'a Link<T>)>) -> usize
    where T: PartialOrd + Debug {
        let right = link.right();
        let left = link.left();
        
        if left.is_some() {
            print_queue.push((offset, left));
        }
        
        let link_str = link_str(link);
        let link_str_len = link_str.len();
        let link_offset = tree_width(left) + link_str_len;
        if right.is_some() {
            print_queue.push((offset + link_offset, right));
        }
        let link_with_arrows = format!("{}{}{}",
            left.as_ref().map_or("", |_| "<"),
            link_str,
            right.as_ref().map_or("", |_| ">")
        );
        let right_offset = right.as_ref().map_or(0, |_| 1);
        let arrow_offset = left.as_ref().map_or(0, |_| 1) + right_offset;
        
        print!("{0:>1$}", link_with_arrows, link_offset + printed_offset + right_offset);
        link_offset + offset + arrow_offset
    }
    
    describe! the_red_black_tree {
        describe! new_constructor {
            it "creates a tree with no root and with a count of 0" {
                let tree = RedBlackTree::<usize>::new();
                expect!(tree.root).to(be_none());
                expect!(tree.count).to(be_equal_to(0));
            }
        }
        
        describe! insert {
            it "creates a black root when the first item is inserted" {
                let tree = rb_tree![1];
                assert!(tree.root.is_some());
                
                expect!(tree.root.value()).to(be_equal_to(&1));
                expect!(tree.count).to(be_equal_to(1));
                expect!(tree.root.color().is_black()).to(be_true());
            }
            
            it "correctly rotates the tree when it becomes unbalanced on the third insert" {
                // right heavy
                let tree = rb_tree![1, 2, 3];
                verify!{ tree =>
                      < B.2 >
                    R.1     R.3
                };
                
                // left heavy
                let tree = rb_tree![4, 3, 2];
                verify!{ tree =>
                      < B.3 >
                    R.2     R.4
                };
            }
            
            it "fixes uncles when an insert leaves an imbalance in the number of black nodes" {
                // right-right
                let tree = rb_tree![1, 2, 3, 4];
                verify!{ tree =>
                      < B.2 >
                    B.1     B.3 >
                                R.4
                };
                
                // left-left
                let tree = rb_tree![4, 3, 2, 1];
                verify!{ tree =>
                          < B.3 >
                      < B.2     B.4
                    R.1
                };
                
                // right-left
                let tree = rb_tree![1, 2, 4, 3];
                verify!{ tree =>
                      < B.2 >
                    B.1   < B.4
                        R.3
                };
                
                // left-right
                let tree = rb_tree![4, 3, 1, 2];
                verify!{ tree =>
                      < B.3 >
                    B.1 >   B.4
                        R.2
                };
            }
            
            it "makes grandparents red when parents and uncles were red" {
                let mut tree = rb_tree_print![1, 2, 3, 4, 5, 6];
                verify!{ tree =>
                      < B.2 >
                    B.1   < R.4 >
                        B.3     B.5 >
                                    R.6
                };
                print_tree(&tree);
                tree.insert(7);
                verify!{ tree =>
                      < B.2 >
                    B.1   < R.4 >
                        B.3   < B.6 >
                            R.5     R.7
                };
            }
        }
        
        describe! remove {
            it "returns None when the tree is empty" {
                let mut tree: RedBlackTree<usize> = rb_tree![];
                let value = 1;
                expect!(tree.remove(&value)).to(be_none());
            }
            
            it "returns Some item matching value when a match is found, and removes the item from the tree" {
                let mut tree = rb_tree![1];
                let value = 1;
                
                verify!{ tree => B.1 };
                expect!(tree.remove(&value)).to(be_some().value(1));
                verify!{ tree => None };
            }
            
            it "can remove a node that has two children" {
                let mut tree = rb_tree![1, 2, 3];
                verify!{tree =>
                      < B.2 >
                    R.1     R.3
                };
                
                let value = 2;
                expect!(tree.remove(&value)).to(be_some().value(2));
                verify!{tree =>
                    B.1 >
                        R.3
                };
            }
            
            it "rebalances when a deep removal is made" {
                let mut tree = rb_tree_print![1, 2, 3, 4, 5, 6];
                verify!{ tree =>
                      < B.2 >
                    B.1   < R.4 >
                        B.3     B.5 >
                                    R.6
                };
                
                let value = 4;
                expect!(tree.remove(&value)).to(be_some().value(4));
                verify!{tree =>
                      < B.2 >
                    B.1     B.3 >
                                B.5 >
                                    R.6
                };
            }
        }
        
        describe! tree_iterator {
            it "iterates in ascending order" {
                let mut tree = rb_tree![3];
                {
                    let mut iter = tree.iter();
                    expect!(iter.next()).to(be_some().value(&3));
                    expect!(iter.next()).to(be_none());
                }
                
                tree.insert(9);
                {
                    let mut iter = tree.iter();
                    expect!(iter.next()).to(be_some().value(&3));
                    expect!(iter.next()).to(be_some().value(&9));
                    expect!(iter.next()).to(be_none());
                }
                
                tree.insert(1);
                {
                    let mut iter = tree.iter();
                    expect!(iter.next()).to(be_some().value(&1));
                    expect!(iter.next()).to(be_some().value(&3));
                    expect!(iter.next()).to(be_some().value(&9));
                    expect!(iter.next()).to(be_none());
                }
                
                tree.insert(10);
                {
                    let mut iter = tree.iter();
                    expect!(iter.next()).to(be_some().value(&1));
                    expect!(iter.next()).to(be_some().value(&3));
                    expect!(iter.next()).to(be_some().value(&9));
                    expect!(iter.next()).to(be_some().value(&10));
                    expect!(iter.next()).to(be_none());
                }
                
                tree.insert(2);
                {
                    let mut iter = tree.iter();
                    expect!(iter.next()).to(be_some().value(&1));
                    expect!(iter.next()).to(be_some().value(&2));
                    expect!(iter.next()).to(be_some().value(&3));
                    expect!(iter.next()).to(be_some().value(&9));
                    expect!(iter.next()).to(be_some().value(&10));
                    expect!(iter.next()).to(be_none());
                }
                
                tree.insert(4);
                {
                    let mut iter = tree.iter();
                    expect!(iter.next()).to(be_some().value(&1));
                    expect!(iter.next()).to(be_some().value(&2));
                    expect!(iter.next()).to(be_some().value(&3));
                    expect!(iter.next()).to(be_some().value(&4));
                    expect!(iter.next()).to(be_some().value(&9));
                    expect!(iter.next()).to(be_some().value(&10));
                    expect!(iter.next()).to(be_none());
                    expect!(iter.next()).to(be_none());
                }
            }
        }
    }
}
