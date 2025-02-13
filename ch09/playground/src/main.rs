use playground::async_mod;

fn main() {
    println!("Hello, world!");
    let id = async_mod::send_add(1, 2).unwrap();
    let res = async_mod::get_add(id).unwrap();

    println!("{}", res);
}
