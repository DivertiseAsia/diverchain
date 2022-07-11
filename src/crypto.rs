pub fn hash(msg: &str) -> String {
  //TODO: 0.4.0 Implement cryptographic hash function here(SHA256)
  return "hash of the msg".to_owned()
}

pub fn generate_rsa_keypair() -> (String, String) {
  //TODO: 0.4.0 Implement RSA keypair generation here
  panic!("Not implemented yet");
}

pub fn sign(msg: &str, key: &str) -> String {
  //TODO: 0.4.1 Implement cryptographic signature function here
  return "signature of the msg".to_owned()
}

pub fn verify(msg: &str, signature: &str, key: &str) -> bool {
  //TODO: 0.4.1 Implement cryptographic signature verification function here
  return false
}



// Unit tests for the crypto module
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_hash256() {
        assert_eq!(hash("Hello world!"), "c0535e4be2b79ffd93291305436bf889314e4a3faec05ecffcbb7df31ad9e51a");
        assert_eq!(hash("A quick brown fox jumps over the lazy dog."), "ffca2587cfd4846e4cb975b503c9eb940f94566aa394e8bd571458b9da5097d5");
        assert_eq!(hash("9999"), "888df25ae35772424a560c7152a1de794440e0ea5cfee62828333a456a506e05");
    }


    #[test]
    fn test_generate_rsa_keypair() {
        let (pubkey, privkey) = generate_rsa_keypair();
        println!("This is the public key: {}", pubkey);
        println!("This is the private key: {}", privkey)
    }

    #[test]
    fn test_signing_and_verify() {
        let (pubkey, privkey) = generate_rsa_keypair();
        let msg = "Hello world!";
        let signature = sign(msg, &privkey);
        assert_eq!(verify(msg, &signature, &pubkey), true);
    }
}