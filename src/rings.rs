use core::fmt::{self, Display};
use std::collections::HashSet;

pub type Int = u8;
pub type Orders = Vec<Int>;

#[derive(Eq, Hash, PartialEq)]
struct Element(Vec<Int>);

impl Element {
    pub fn with_capacity(capacity: usize) -> Self {
        Element(Vec::with_capacity(capacity))
    }
}

impl Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0.iter().map(|x| x.to_string()).collect::<String>()
        )
    }
}

pub trait Ring: Display {}

struct PlainRing<Orders> {
    elements: HashSet<Element>,
    orders: Orders,
}

impl PlainRing<Orders> {
    pub fn new(orders: Orders) -> Self {
        let mut elements = HashSet::new();
        let mut r = Element::with_capacity(orders.len());
        Self::generate_ring(&mut elements, r, &orders, 0);

        PlainRing { elements, orders }
    }

    fn generate_ring(elements: &mut HashSet<Element>, r: Element, orders: &Orders, index: usize) {
        if index == orders.len() {
            elements.insert(r);
        } else {
            for n in 0..orders[index] {
                let mut q = r.0.
            }
        }
    }
}
