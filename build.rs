fn main() {
    println!("cargo:rustc-link-arg=-Wl,-z,nostart-stop-gc");
}
