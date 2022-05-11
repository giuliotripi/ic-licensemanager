use std::convert::TryFrom;
use ed25519_dalek::{PublicKey, Verifier, Signature, SignatureError};


#[ic_cdk_macros::query]
fn greet(signature: String) -> String {
    let result = match verify_signature(signature.clone()) {
        Ok(ris) => {ris}
        Err(_) => {false}
    };
    let orig_data = signature_to_orig_data(signature.clone());
    format!("The verification result for \"{}\" containing \"{}\" is {}", signature, orig_data, result)
}
//https://docs.rs/ed25519-dalek/latest/ed25519_dalek/
//https://docs.rs/ed25519/latest/ed25519/
//https://github.com/RustCrypto/signatures/tree/master/ed25519
fn verify_signature(signature_str: String) ->  Result<bool, SignatureError> {
    let pub_key = [251, 249, 141, 122, 83, 239, 198, 212, 199, 142, 166, 51, 103, 189, 116, 150, 63, 232, 101, 116, 224, 60, 65, 10, 159, 22, 6, 18, 51, 172, 21, 247];
    let public_key: PublicKey = PublicKey::from_bytes(&pub_key)?;
    // let signature_str = String::from("4f23d6692b340dbc92e163f5c271fe7d8f03e5836ec36eb474d8af6d2a22910de32a4284d6974302f6cfe9c716130cdc29dbeff8cb83171607516b700f28e30f41747461636b206174204461776e");
    let decoded_signature = signature_to_array(signature_str.clone());
    let signature_or_error = match decoded_signature {
        Ok(res) => {res},
        Err(_) => {return Err(Default::default());}
    };
    let signature = Signature::try_from(signature_or_error.as_ref())?;
    let message: &[u8] = b"Attack at Dawn";
    if public_key.verify(message, &signature).is_ok() {
        println!("verificato");
        Ok(true)
    } else {
        println!("FALSO");
        Ok(false)
    }
}


fn signature_to_array(hex: String) -> Result<[u8; 64], String> {
    let mut i = 0;
    let valori = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];
    let mut result:  [u8; 64] = [0; 64];
    if hex.len() < 128 {
        return Err(String::from("String is too short"))
    }
    while i < hex.len() - 1 && i < 128 { //.try_into().unwrap()
        let el1; let el2;
        let opt1 = valori.iter().position(|&s| s == hex.chars().nth(i).unwrap());
        let opt2 = valori.iter().position(|&s| s == hex.chars().nth(i + 1).unwrap());
        if opt1.is_none() || opt2.is_none() {
            return Err(String::from("String contains invalid characters"));
        } else {
            el1 = opt1.unwrap();
            el2 = opt2.unwrap();
        }
        result[i/2] = (el1 * 16 + el2) as u8;
        i += 2;
    }
    return Ok(result);
}

fn signature_to_orig_data(hex: String) -> String {
    // let signed_size = (hex.len() - 128);
    let mut i = 128;
    let mut result = String::new();
    let valori = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];
    while i < hex.len() - 1 {
        let el1 = valori.iter().position(|&s| s == hex.chars().nth(i).unwrap()).unwrap();
        let el2 = valori.iter().position(|&s| s == hex.chars().nth(i + 1).unwrap()).unwrap();
        let character = (el1 * 16 + el2) as u8;
        result.push(character as char);
        i += 2;
    }
    return result;
}