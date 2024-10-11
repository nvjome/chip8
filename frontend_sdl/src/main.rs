use cpu;

fn main() {
    println!("Hello, world!");
    let num = 10;
    println!("{num} plus one is {}", cpu::add(num, 1));
}
