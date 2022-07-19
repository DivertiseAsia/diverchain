// use hex_literal::hex;
// use sha2::{Sha256, Sha512, Digest};
extern crate openssl;
extern crate pem;
extern crate signature;

use sha256::digest;
use rsa::{PublicKey, RsaPrivateKey, RsaPublicKey, PaddingScheme};
use rsa::Hash::SHA3_256;
use openssl::pkey::PKey;
use std::str;
use pem::{Pem, encode};
use signature::{Signature, Signer, Verifier};


pub fn hash(msg: &str) -> String {
  	//TODO: 0.4.0 Implement cryptographic hash function here(SHA256)

	//   // create a Sha256 object
	//   let mut hasher = Sha256::new();

	//   // write input message
	//   hasher.update(b"hello world");

	//   // read hash digest and consume hasher
	//   let result = hasher.finalize();

	let hashed = digest(msg);

  	return hashed.to_owned()
}

pub fn generate_rsa_keypair() -> (RsaPublicKey, RsaPrivateKey) {
	//TODO: 0.4.0 Implement RSA keypair generation here
	// let rsa = Rsa::generate(2048).unwrap();

  	// let pkey = PKey::from_rsa(rsa).unwrap();

	// let pub_key: Vec<u8> = pkey.public_key_to_pem().unwrap();
	// let priv_key: Vec<u8> = pkey.private_key_to_der().unwrap();

	// let private_pem = Pem {
	// 	tag: String::from("RSA PRIVATE KEY"),
	// 	contents: priv_key,
	//   };

	// let private = encode(&private_pem);

	// let pub_key_string = str::from_utf8(pub_key.as_slice()).unwrap();
	// // let priv_key_string = str::from_utf8(priv_key.as_slice()).unwrap();

  	// return (pub_key_string.to_string(), private.to_string())
	// panic!("Not implemented yet");

	let mut rng = rand::thread_rng();

	let bits = 2048;
	let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
	let pub_key = RsaPublicKey::from(&priv_key);

	return (pub_key, priv_key)
}

pub fn sign(data: &str, key: &RsaPrivateKey) -> Vec<u8> {
  	//TODO: 0.4.1 Implement cryptographic signature function here

	// let msg_as_bytes = msg.as_bytes();
	// let key_as_bytes = key.as_bytes();

	// let mut appended:Vec<u8> = [key_as_bytes, msg_as_bytes].concat();
	
	// let hashed = digest(key.to_owned() + msg);

  	// return hashed.to_owned()

	// let mut rng = rand::thread_rng();

	let padding = PaddingScheme::new_pkcs1v15_sign(Some(SHA3_256));

	let hash_data = hash(&data);
	let decoded = hex::decode(&hash_data).expect("Decoding failed");

	// println!("{}:{:?}", decoded.len(), decoded);
	// println!("{}", SHA3_256.size());

	// let enc_data = key.encrypt(&mut rng, padding, hash(&data).as_bytes()).expect("failed to encrypt");
	let signed = key.sign(padding, &decoded).unwrap();

	println!("{:?}", signed);

	return signed;
}

pub fn verify(msg: &str, signature: &Vec<u8>, key: &RsaPublicKey) -> bool {
	//TODO: 0.4.1 Implement cryptographic signature verification function here
	// let hashed_msg = digest(msg); 
	// let 
	let hash_data = hash(&msg);
	let decoded = hex::decode(&hash_data).expect("Decoding failed");

	// println!("{}", signature);
	// println!("{}", signature.len());
	// let (enc_string, sig_msg) = signature.split_at(64);

	// println!("{}", signature);
	// println!("{}:{:?}", signature.len(), &signature.as_bytes());
	let dec_data = key.verify(PaddingScheme::new_pkcs1v15_sign(Some(SHA3_256)), &decoded, &signature);
	
	match dec_data {
		| Ok(()) => {true}, 
		| Error => {false}
	}
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
        println!("This is the public key: {:?}", pubkey);
        println!("This is the private key: {:?}", privkey)
    }

    #[test]
    fn test_signing_and_verify() {
        let (pubkey, privkey) = generate_rsa_keypair();
        let msg = "Hello world!";
        let signature = sign(msg, &privkey);
        assert_eq!(verify(msg, &signature, &pubkey), true);
    }

	#[test]
	fn test_signing_and_verify_with_msg_tamper() {
        let (pubkey, privkey) = generate_rsa_keypair();
        let msg = "I like tacobell";
        let signature = sign(msg, &privkey);
        assert_eq!(verify(&(msg.to_owned() + "!"), &signature, &pubkey), false);
    }
}