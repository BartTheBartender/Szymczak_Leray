mod groups;

fn main() {
    let orders: groups::Orders = vec![2, 3];

    //let g = groups::generate_group(&orders);

    //print!("{}", groups::string_group(&g));
    //print!("{}", groups::string_group(&g));

    print!("{}", orders);
}
