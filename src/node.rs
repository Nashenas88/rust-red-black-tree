use std::fmt::{self, Debug};

#[derive(Copy, Clone)]
pub enum Color {
    Red,
    Black,
}

impl Color {
    pub fn is_red(self) -> bool {
        match self {
            Color::Red => true,
            Color::Black => false,
        }
    }
    
    pub fn is_black(self) -> bool {
        match self {
            Color::Red => false,
            Color::Black => true,
        }
    }
}

impl Debug for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match *self {
            Color::Red => "R",
            Color::Black => "B",
        };
        
        write!(f, "{}", string)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Dir {
    Left,
    Right
}

impl Dir {
    pub fn opposite(self) -> Self {
        match self {
            Dir::Left => Dir::Right,
            Dir::Right => Dir::Left,
        }
    }
}

pub struct Node<T> where T: PartialOrd {
    color: Color,
    value: T,
    left: Link<T>,
    right: Link<T>,
}

pub type Link<T> = Option<Box<Node<T>>>;

impl<T> Node<T> where T: PartialOrd {
    pub fn new(value: T) -> Node<T> {
        Node::<T> {
            value: value,
            color: Color::Red,
            left: None,
            right: None,
        }
    }
}

impl<T> Debug for Node<T> where T: PartialOrd + Debug {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}.{:?}", self.color, self.value)
    }
}

pub trait NodeHelper<T> where T: PartialOrd {
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

pub trait Follow<T> where T: PartialOrd {
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
#[macro_export]
macro_rules! follow {
    ($node:expr, $dir:ident) => {
        $node.follow($dir)
    };
    ($node:expr, $dir:ident, $($dirs:ident),+) => {
        follow!($node.follow($dir), $($dirs),+)
    };
}

#[macro_export]
macro_rules! follow_mut {
    ($node:expr, $dir:ident) => {
        $node.follow_mut($dir)
    };
    ($node:expr, $dir:ident, $($dirs:ident),+) => {
        follow_mut!($node.follow_mut($dir), $($dirs),+)
    };
}

#[cfg(test)]
impl<T: PartialOrd> PartialEq for Node<T> {
    fn eq(&self, other: &Self) -> bool {
        self.color == other.color
            && self.value == other.value
    }
}

#[cfg(test)]
pub fn node<T: PartialOrd>(color: Color, value: T) -> Node<T> {
    Node::<T> {
        color: color,
        value: value,
        left: None,
        right: None,
    }
}