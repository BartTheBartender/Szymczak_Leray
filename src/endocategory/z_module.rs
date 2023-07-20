//imports from external sources
use std::collections::{HashMap, HashSet};
use std::io::{Error, ErrorKind};

//imports from the crate
use super::torsion_coeff;
use crate::{Int, TorsionCoeff};

type Element = Vec<Int>;

#[derive(Debug, Hash, PartialEq)]
pub struct ZModule {
    pub torsion_coeff: TorsionCoeff,
    pub elements: Vec<Element>,
}

impl Eq for ZModule {}

impl ZModule {
    //I don't really see the way to use this iterator :(
    pub fn new(source: TorsionCoeff, target: TorsionCoeff) -> Self {
        let mut elements = Vec::<Element>::new();
        let mut element: Element = Vec::with_capacity(source.len() + target.len());

        let mut torsion_coeff = TorsionCoeff::new();
        torsion_coeff.extend(source);
        torsion_coeff.extend(target);
        let torsion_coeff = torsion_coeff;

        Self::new_help(&torsion_coeff, &mut elements, element, 0);

        ZModule {
            torsion_coeff,
            elements,
        }
    }

    fn new_help(
        torsion_coeff: &TorsionCoeff,
        elements: &mut Vec<Element>,
        element: Element,
        index: usize,
    ) {
        if index != torsion_coeff.len() {
            for x in 0..torsion_coeff[index] {
                let mut element_ = element.clone();
                element_.push(x);
                Self::new_help(torsion_coeff, elements, element_, index + 1);
            }
        } else {
            elements.push(element);
        }
    }
    //GENERAL REMARK - i dont know how to use iterator in the situation as below, secondly this module will be private (only used to generate relations)
    pub fn add(&self, left: &Element, right: &Element) -> Result<Element, Error> {
        if left.len() == self.torsion_coeff.len() && right.len() == self.torsion_coeff.len() {
            let mut output: Element = Vec::with_capacity(self.torsion_coeff.len());

            for index in 0..self.torsion_coeff.len() {
                output.push((left[index] + right[index]) % self.torsion_coeff[index]);
            }

            Ok(output)
        } else {
            Err(Error::new(
                ErrorKind::InvalidInput,
                "Invaid dimensions for elements!",
            ))
        }
    }

    pub fn increment_left<'a>(
        &'a self,
        left: &'a mut Element,
        right: &'a Element,
    ) -> Result<&'a mut Element, Error> {
        if left.len() == self.torsion_coeff.len() && right.len() == self.torsion_coeff.len() {
            for index in 0..self.torsion_coeff.len() {
                left[index] = (left[index] + right[index]) % self.torsion_coeff[index];
            }
            Ok(left)
        } else {
            Err(Error::new(
                ErrorKind::InvalidInput,
                "Invalid dimensions for elements!",
            ))
        }
    }

    pub fn increment_right<'a>(
        &'a self,
        left: &'a Element,
        right: &'a mut Element,
    ) -> Result<&'a mut Element, Error> {
        self.increment_left(right, left)
    }

    pub fn multiply_by_scalar(&self, scalar: Int, element: &Element) -> Result<Element, Error> {
        if element.len() == self.torsion_coeff.len() {
            let mut output = Element::with_capacity(element.len());
            for index in 0..self.torsion_coeff.len() {
                output.push((element[index] * scalar) % self.torsion_coeff[index]);
            }
            Ok(output)
        } else {
            Err(Error::new(
                ErrorKind::InvalidInput,
                "Invalid dimension for element!",
            ))
        }
    }

    pub fn submodules(self) -> HashSet<Self> {
        let a = self.maximal_cyclic_submodule();

        if a.elements.len() == self.elements.len() {
            a.submodules_of_cyclic_module()
        } else {
            let b = Self::split(self, &a);
            Self::goursat(a, b)
        }
    }

    pub fn maximal_cyclic_submodule(&self) -> Self {
        let max_element = self
            .elements
            .iter()
            .max_by_key(|&element| self.order(element))
            .unwrap();
        let max_order = self.order(max_element);

        let mut elements = Vec::<Element>::with_capacity(max_order.into());

        for scalar in 0..max_order {
            elements.push(self.multiply_by_scalar(scalar, &max_element).unwrap());
        }

        ZModule {
            torsion_coeff: self.torsion_coeff.clone(),
            elements,
        }
    }

    pub fn order(&self, element: &Element) -> Int {
        let mut prime_decomposition: HashMap<Int, Int> = HashMap::new();

        for index in 0..element.len() {
            //consider parallel?
            if element[index] == 0 {
                continue;
            }

            let mut value =
                self.torsion_coeff[index] / Self::gcd(self.torsion_coeff[index], element[index]);
            let mut prime = 2 as Int;

            while prime <= value {
                //it is obvious that prime is always prime number (for if it was p*r, p< prime and all powers of p are already extracted)

                if value % prime == 0 {
                    let mut power = 0;

                    while value % prime == 0 {
                        value /= prime;
                        power += 1;
                    }

                    prime_decomposition
                        .entry(prime)
                        .and_modify(|current_power| {
                            if *current_power < power {
                                *current_power = power;
                            }
                        })
                        .or_insert(power);
                }
                prime += 1;
            }
        }

        let mut output: Int = 1;
        for (prime, power) in prime_decomposition {
            output *= prime.pow(power.into());
        }
        output
    }

    fn gcd(mut x: Int, mut y: Int) -> Int {
        while y != 0 {
            let temp = y;
            y = x % y;
            x = temp;
        }
        x
    }

    fn submodules_of_cyclic_module(self) -> HashSet<Self> {
        let mut output: HashSet<Self> = HashSet::new();
        let self_len: Int = self.elements.len() as Int;
        for order in torsion_coeff::tau(self_len) {
            let step = self_len / order;
            let mut elements: Vec<Element> = Vec::with_capacity(order as usize);
            for index in (0..order).into() {
                elements[index] = self.elements[step * index].clone();
            }

            output.insert(ZModule {
                torsion_coeff: self.torsion_coeff.clone(),
                elements,
            });
        }

        output
    }

    fn split(self, a: &Self) -> Self {
        todo!()
    }

    fn goursat(a: Self, b: Self) -> HashSet<Self> {
        todo!()
    }
}
