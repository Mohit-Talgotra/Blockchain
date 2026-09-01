use sha2::{Digest, Sha256};
use std::time::Instant;
z
struct BlockHeader {
    version: u32,
    prev_hash: String,
    merkle_root: String,
    timestamp: u32,
    nonce: u64,
}

fn double_sha256(data: &[u8]) -> Vec<u8> {
    let first = Sha256::digest(data);
    let second = Sha256::digest(first);
    second.to_vec()
}

fn header_bytes(h: &BlockHeader) -> Vec<u8> {
    format!(
        "{}{}{}{}{}",
        h.version, h.prev_hash, h.merkle_root, h.timestamp, h.nonce
    )
    .into_bytes()
}

fn meets_difficulty(hash: &[u8], leading_zero_hex: usize) -> bool {
    let hex_str = hex::encode(hash);
    hex_str.starts_with(&"0".repeat(leading_zero_hex))
}

fn mine(header: &mut BlockHeader, difficulty: usize) {
    let start = Instant::now();
    let mut hashes_tried: u64 = 0;

    loop {
        let bytes = header_bytes(header);
        let hash = double_sha256(&bytes);
        hashes_tried += 1;

        if hashes_tried % 200_000 == 0 {
            println!("...tried {} hashes so far (nonce={})", hashes_tried, header.nonce);
        }

        if meets_difficulty(&hash, difficulty) {
            let elapsed = start.elapsed().as_secs_f64();
            let hashrate = hashes_tried as f64 / elapsed.max(0.0001);
            println!("\n✅ Block mined!");
            println!("Nonce:        {}", header.nonce);
            println!("Hash:         {}", hex::encode(&hash));
            println!("Hashes tried: {}", hashes_tried);
            println!("Time:         {:.2}s", elapsed);
            println!("Hashrate:     {:.2} H/s", hashrate);
            return;
        }

        header.nonce += 1;
    }
}

fn main() {
    let mut header = BlockHeader {
        version: 1,
        prev_hash: "0".repeat(64),
        merkle_root: "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33"
            .to_string(),
        timestamp: 1_700_000_000,
        nonce: 0,
    };
    
    let difficulty = 5;

    println!("Starting simplified Bitcoin miner");
    println!("Target: {} leading zero hex characters\n", difficulty);

    mine(&mut header, difficulty);
}
