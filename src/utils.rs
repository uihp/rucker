use rand::Rng;

pub fn random_hex_string() -> String {
    let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
    let rand_bytes: [u8; 6] = rng.r#gen();
    hex::encode(&rand_bytes)
}
