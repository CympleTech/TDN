extern crate tdn_crypto;

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use tdn_crypto::hash::*;

    #[test]
    fn test_H256() {
        println!("{:?}", H256::new(b"test"));
        println!("{}", H256::new(b"test"));
        assert_eq!(true, true)
    }

    #[test]
    fn test_H512() {
        println!("{}", H512::new(b"test"));

        assert_eq!(true, true)
    }
}
