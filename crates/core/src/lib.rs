pub fn hello() -> String {
    "app-core works".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(hello(), "app-core works");
    }
}
