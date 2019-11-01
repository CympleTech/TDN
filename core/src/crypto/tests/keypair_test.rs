extern crate crypto;

#[cfg(test)]
mod tests {
    use crypto::keypair::*;

    #[test]
    fn test_generate_private_key() {
        let private_key = PrivateKey::generate();

        let s = format!("{}", private_key);
        let private_key2: PrivateKey = (&s).into();

        assert_eq!(32, private_key.len());
        assert_eq!(format!("{}", private_key), format!("{}", private_key2))
    }

    #[test]
    fn test_generate_public_key() {
        let private_key = PrivateKey::generate();
        let public_key = private_key.generate_public_key();

        let s = format!("{}", public_key);
        let public_key2: PublicKey = (&s).into();

        assert_eq!(32, public_key.len());
        assert_eq!(format!("{}", public_key), format!("{}", public_key2))
    }

    #[test]
    fn test_signature() {
        let message = String::from("this is test message");
        let private_key = PrivateKey::generate();
        let sign = private_key.sign(&message);

        let s = format!("{}", sign);
        let sign2: Signature = (&s).into();

        assert_eq!(64, sign.len());
        assert_eq!(format!("{}", sign), format!("{}", sign2))
    }

    #[test]
    fn test_verify() {
        let message = String::from("this is test message");
        let private_key = PrivateKey::generate();
        let sign = private_key.sign(&message);

        let public_key = private_key.generate_public_key();
        assert_eq!(true, public_key.verify(&message, &sign))
    }

    #[test]
    fn test_sign_bytes() {
        let good: &[u8] = b"test message";
        let good_bytes = good.to_vec();
        let private_key = PrivateKey::generate();
        let sign = private_key.sign_bytes(&good_bytes);

        let public_key = private_key.generate_public_key();
        assert_eq!(true, public_key.verify_bytes(&good_bytes, &sign))
    }
}
