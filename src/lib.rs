#![feature(plugin)]
#![plugin(stainless)]


use std::mem;

#[derive(Debug, Copy, Clone)]
pub enum Color {
    Red,
    Black,
}

impl Color {
    fn is_red(self) -> bool {
        match self {
            Color::Red => true,
            Color::Black => false,
        }
    }
    
    fn is_black(self) -> bool {
        match self {
            Color::Red => false,
            Color::Black => true,
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Dir {
    Left,
    Right
}

impl Dir {
    fn opposite(self) -> Self {
        match self {
            Dir::Left => Dir::Right,
            Dir::Right => Dir::Left,
        }
    }
}

#[derive(Debug)]
struct Node<T> where T: PartialOrd {
    color: Color,
    value: T,
    left: Link<T>,
    right: Link<T>,
}

type Link<T> = Option<Box<Node<T>>>;

impl<T> Node<T> where T: PartialOrd {
    fn new(value: T) -> Node<T> {
        Node::<T> {
            value: value,
            color: Color::Red,
            left: None,
            right: None,
        }
    }
}

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

trait NodeHelper<T> where T: PartialOrd {
    fn color(&self) -> &Color;
    fn set_color(&mut self, color: Color);
    
    fn left(&self) -> &Link<T>;
    fn left_mut(&mut self) -> &mut Link<T>;
    fn set_left(&mut self, left: Link<T>);
    
    fn right(&self) -> &Link<T>;
    fn right_mut(&mut self) -> &mut Link<T>;
    fn set_right(&mut self, right: Link<T>);
    
    fn value(&self) -> &T;
    fn value_mut(&mut self) -> &mut T;
    fn set_value(&mut self, value: T);
    fn take_value(self) -> T;
    
    fn is_black(&self) -> bool;
    fn is_red(&self) -> bool;
}

impl<T> NodeHelper<T> for Link<T> where T: PartialOrd {
    fn color(&self) -> &Color {
        &self.as_ref().unwrap().color
    }
    fn set_color(&mut self, color: Color) {
        self.as_mut().map(|n| n.color = color);
    }
    
    fn left(&self) -> &Link<T> {
        &self.as_ref().unwrap().left
    }
    fn left_mut(&mut self) -> &mut Link<T> {
        &mut self.as_mut().unwrap().left
    }
    fn set_left(&mut self, left: Link<T>) {
        self.as_mut().map(|n| n.left = left);
    }
    
    fn right(&self) -> &Link<T> {
        &self.as_ref().unwrap().right
    }
    fn right_mut(&mut self) -> &mut Link<T> {
        &mut self.as_mut().unwrap().right
    }
    fn set_right(&mut self, right: Link<T>) {
        self.as_mut().map(|n| n.right = right);
    }
    
    fn value(&self) -> &T {
        &self.as_ref().unwrap().value
    }
    fn value_mut(&mut self) -> &mut T {
        &mut self.as_mut().unwrap().value
    }
    fn set_value(&mut self, value: T) {
        self.as_mut().map(|n| n.value = value);
    }
    fn take_value(self) -> T {
        self.unwrap().value
    }
    
    fn is_black(&self) -> bool {
        self.as_ref().map_or(true, |n| n.color.is_black())
    }
    fn is_red(&self) -> bool {
        self.as_ref().map_or(false, |n| n.color.is_red())
    }
}

impl<'a, T> NodeHelper<T> for Option<&'a mut Link<T>> where T: PartialOrd {
    fn color(&self) -> &Color {
        self.as_ref().unwrap().color()
    }
    fn set_color(&mut self, color: Color) {
        self.as_mut().map(|n| n.set_color(color));
    }
    
    fn left(&self) -> &Link<T> {
        self.as_ref().unwrap().left()
    }
    fn left_mut(&mut self) -> &mut Link<T> {
        (**self.as_mut().unwrap()).left_mut()
    }
    fn set_left(&mut self, left: Link<T>) {
        self.as_mut().map(|n| n.set_left(left));
    }
    
    fn right(&self) -> &Link<T> {
        self.as_ref().unwrap().right()
    }
    fn right_mut(&mut self) -> &mut Link<T> {
        (**self.as_mut().unwrap()).right_mut()
    }
    fn set_right(&mut self, right: Link<T>) {
        self.as_mut().map(|n| n.set_right(right));
    }
    
    fn value(&self) -> &T {
        self.as_ref().unwrap().value()
    }
    fn value_mut(&mut self) -> &mut T {
        self.as_mut().unwrap().value_mut()
    }
    fn set_value(&mut self, value: T) {
        self.as_mut().map(|n| n.set_value(value));
    }
    fn take_value(self) -> T {
        unimplemented!()
    }
    
    fn is_black(&self) -> bool {
        self.as_ref().unwrap().is_black()
    }
    fn is_red(&self) -> bool {
        self.as_ref().unwrap().is_red()
    }
}

trait Follow<T> where T: PartialOrd {
    fn follow(&self, direction: Dir) -> &Link<T>;
    fn follow_mut(&mut self, direction: Dir) -> &mut Link<T>;
}

impl<T> Follow<T> for Node<T> where T: PartialOrd {
    fn follow(&self, direction: Dir) -> &Link<T> {
        match direction {
            Dir::Left => &self.left,
            Dir::Right => &self.right,
        }
    }
    fn follow_mut(&mut self, direction: Dir) -> &mut Link<T> {
        match direction {
            Dir::Left => &mut self.left,
            Dir::Right => &mut self.right,
        }
    }
}

impl<T> Follow<T> for Link<T> where T: PartialOrd {
    fn follow(&self, direction: Dir) -> &Link<T> {
        self.as_ref().unwrap().follow(direction)
    }
    fn follow_mut(&mut self, direction: Dir) -> &mut Link<T> {
        self.as_mut().unwrap().follow_mut(direction)
    }
}

impl<'a, T> Follow<T> for Option<&'a mut Link<T>> where T: PartialOrd {
    fn follow(&self, direction: Dir) -> &Link<T> {
        self.as_ref().unwrap().follow(direction)
    }
    fn follow_mut(&mut self, direction: Dir) -> &mut Link<T> {
        self.as_mut().unwrap().follow_mut(direction)
    }
}

// variadic versions of follow
macro_rules! follow {
    ($node:expr, $dir:ident) => {
        $node.follow($dir)
    };
    ($node:expr, $dir:ident, $($dirs:ident),+) => {
        follow!($node.follow($dir), $($dirs),+)
    };
}

macro_rules! follow_mut {
    ($node:expr, $dir:ident) => {
        $node.follow_mut($dir)
    };
    ($node:expr, $dir:ident, $($dirs:ident),+) => {
        follow_mut!($node.follow_mut($dir), $($dirs),+)
    };
}

fn get_dir(is_less: bool) -> Dir {
    if is_less { Dir::Left } else { Dir::Right }
}

impl<T> Node<T> where T: PartialOrd {
    fn insert_g(value: T, grandparent: &mut Link<T>, p_dir: Dir, n_dir: Dir) -> Option<usize> {
        if follow!(grandparent, p_dir, n_dir).is_none() {
            {
                let mut node = follow_mut!(grandparent, p_dir, n_dir);
                *node = Some(Box::new(Node::new(value)));
            }
            Self::insert_2_g(grandparent, p_dir, n_dir)
        } else {
            let dir = get_dir(value < *follow!(grandparent, p_dir, n_dir).value());
            match Self::insert_g(value, grandparent.follow_mut(p_dir), n_dir, dir) {
                Some(0) => Self::insert_2_g(grandparent, p_dir, n_dir),
                Some(rest) => Some(rest - 1),
                None => None,
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
            Self::insert_g(value, parent, n_dir, dir)
                .map(|rest| rest - 1)
        }
    }
    
    fn insert_n(value: T, node: &mut Link<T>) {
        if node.is_none() {
            *node = Some(Box::new(Node::new(value)));
            Self::insert_1_n(node);
        } else {
            let dir = get_dir(value < *node.value());
            if let Some(0) = Self::insert_p(value, node, dir) {
                Self::insert_1_n(node);
            }
        }
    }
    
    fn insert_1_n(node: &mut Link<T>) {
        node.set_color(Color::Black);
    }
    
    fn insert_2_g(grandparent: &mut Link<T>, p_dir: Dir, n_dir: Dir) -> Option<usize> {
        match *grandparent.follow(p_dir).color() {
            Color::Red => Self::insert_3_g(grandparent, p_dir, n_dir),
            Color::Black => None,
        }
    }
    
    fn insert_3_g(grandparent: &mut Link<T>, p_dir: Dir, n_dir: Dir) -> Option<usize> {
        let uncle_red = grandparent.follow_mut(p_dir.opposite()).as_mut()
            .map_or(false, |uncle| match uncle.color {
                Color::Black => false,
                Color::Red => {
                    uncle.color = Color::Black;
                    true
                },
            });

        if uncle_red {
            grandparent.follow_mut(p_dir).set_color(Color::Black);
            grandparent.set_color(Color::Black);
            Some(2)
        } else {
            Self::insert_4_g(grandparent, p_dir, n_dir);
            None
        }
    }
    
    fn insert_4_g(grandparent: &mut Link<T>, p_dir: Dir, n_dir: Dir) {
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
        let parent = grandparent.follow_mut(p_dir);
        parent.set_color(Color::Black);
        match n_dir {
            Dir::Left => Self::rotate_right(parent),
            Dir::Right => Self::rotate_left(parent),
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
            Some(Self::delete_one_child(node).0)
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
            let (value, should_fix_parent) = Self::delete_one_child(parent.follow_mut(n_dir));
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
    })
];


#[cfg(test)]
#[macro_use(expect)]
extern crate expectest;

#[cfg(test)]
mod test {
    pub use expectest::prelude::{be_some, be_none, be_equal_to};
    use NodeHelper;
    use Link;
    pub use super::*;
    
    // node based iterator for verifying structure of tree during testing
    struct RedBlackNodeIterator<'a, T> where T: PartialOrd + 'a {
        stack: Vec<&'a Link<T>>,
        current: Option<&'a Link<T>>,
    }
    
    impl<'a, T> RedBlackNodeIterator<'a, T> where T: PartialOrd + 'a {
        fn new(tree: &RedBlackTree<T>) -> RedBlackNodeIterator<T> {
            RedBlackNodeIterator {
                stack: vec![],
                current: Some(&tree.root),
            }
        }
    }
    
    impl<'a, T> Iterator for RedBlackNodeIterator<'a, T> where T: PartialOrd {
        type Item = &'a Link<T>;
        
        // traverses depth first
        fn next(&mut self) -> Option<&'a Link<T>> {
            // todo
            unimplemented!()
        }
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
                expect!(tree.root.color().is_black()).to(be_equal_to(true));
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
                
                expect!(tree.count).to(be_equal_to(1));
                expect!(tree.remove(&value)).to(be_some().value(1));
                expect!(tree.count).to(be_equal_to(0));
                expect!(tree.root).to(be_none());
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