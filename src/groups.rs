use std::collections::HashSet;

pub type Int = u8;
pub type Orders = Vec<Int>;

pub trait Element: Display {}

struct Group<T: Element> {
    elements: Vec<T>,
}

pub fn generate_group(orders: &Orders) -> Group {
    let g_ = Element::new();
    let mut g = Group::new();

    generate_group_helper(orders, 0, g_, &mut g);
    return g;
}

fn generate_group_helper(orders: &Orders, index: usize, g_: Element, g: &mut Group) {
    if index == orders.len() {
        g.insert(g_);
    } else {
        for n in 0..orders[index] {
            let mut h_ = g_.clone();
            h_.push(n as Int);
            generate_group_helper(orders, index + 1, h_, g);
        }
    }
}

//nie widzę bardziej eleganckiego rozwiązania
pub fn string_element(g_: &Element) -> String {
    return g_.iter().map(|x| x.to_string()).collect::<String>();
}

//nie widzę bardziej eleganckiego rozwiązania
pub fn string_group(g: &Group) -> String {
    let mut iter = g.iter().peekable();
    let mut output = String::new();

    while let Some(g_) = iter.next() {
        output += &string_element(g_);
        if iter.clone().last().is_some() {
            output += " ";
        }
    }

    output
}
