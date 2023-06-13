fn main() {
    let password = "auth!23$";
    match bcrypt::hash(password, 10) {
        Ok(s) => {
            println!("password={password}");
            println!("{s}");
        }
        Err(e) => {
            eprintln!("Error: {e}");
        }
    }

    let r = bcrypt::verify(
        password,
        "$2a$10$24MkL2xUdbX2HZSuvD/1UuhRoiXjtUFRT1AaGXmQFc75SrwzR5O3q", // "$2b$10$/yVHr2/4IUaC8vUC2wDTS.to8M25vjnlFt2HEU0tx1ENe3e1w58WK",
    );
    println!("verify: {r:?}");
}
