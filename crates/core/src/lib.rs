pub fn hello() -> String {
    "betc-core works".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(hello(), "betc-core works");
    }
}
