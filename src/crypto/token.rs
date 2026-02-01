use sha2::{Digest, Sha256};

pub fn hash_token(token: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(token);
    let tmp = hasher.finalize();
    tmp.as_slice().to_owned()
}
