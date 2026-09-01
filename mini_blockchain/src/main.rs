use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DIFFICULTY: usize = 4;
const MINE_INTERVAL_SECS: u64 = 8;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Block {
    index: u64,
    timestamp: u64,
    data: String,
    prev_hash: String,
    nonce: u64,
    hash: String,
}

impl Block {
    fn mined(index: u64, data: String, prev_hash: String) -> Block {
        let mut block = Block {
            index,
            timestamp: now(),
            data,
            prev_hash,
            nonce: 0,
            hash: String::new(),
        };
        block.mine();
        block
    }

    fn genesis() -> Block {
        Block {
            index: 0,
            timestamp: now(),
            data: "genesis".into(),
            prev_hash: "0".repeat(64),
            nonce: 0,
            hash: "genesis-hash".into(),
        }
    }

    fn calc_hash(&self) -> String {
        let input = format!(
            "{}{}{}{}{}",
            self.index, self.timestamp, self.data, self.prev_hash, self.nonce
        );
        hex::encode(Sha256::digest(input.as_bytes()))
    }

    fn mine(&mut self) {
        let target = "0".repeat(DIFFICULTY);
        loop {
            let h = self.calc_hash();
            if h.starts_with(&target) {
                self.hash = h;
                return;
            }
            self.nonce += 1;
        }
    }
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

struct Blockchain {
    blocks: Vec<Block>,
}

impl Blockchain {
    fn new() -> Blockchain {
        Blockchain { blocks: vec![Block::genesis()] }
    }

    fn last(&self) -> &Block {
        self.blocks.last().unwrap()
    }

    fn try_add(&mut self, block: Block) -> bool {
        let last = self.last();
        let valid_link = block.index == last.index + 1 && block.prev_hash == last.hash;
        let valid_hash = block.hash == block.calc_hash();
        let meets_difficulty = block.hash.starts_with(&"0".repeat(DIFFICULTY));

        if valid_link && valid_hash && meets_difficulty {
            self.blocks.push(block);
            true
        } else {
            false
        }
    }

    fn mine_next(&mut self, data: String) -> Block {
        let index = self.last().index + 1;
        let prev_hash = self.last().hash.clone();
        let block = Block::mined(index, data, prev_hash);
        self.blocks.push(block.clone());
        block
    }

    fn print_chain(&self) {
        println!("---- chain (len {}) ----", self.blocks.len());
        for b in &self.blocks {
            let h = &b.hash[..b.hash.len().min(10)];
            let p = &b.prev_hash[..b.prev_hash.len().min(10)];
            println!("#{:<3} data: {:<25} hash: {}.. prev: {}..", b.index, b.data, h, p);
        }
        println!("------------------------");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: mini_blockchain <listen_port> [peer_ip:port ...]");
        eprintln!("Example: mini_blockchain 8000 127.0.0.1:8001 127.0.0.1:8002");
        return;
    }
    let port = args[1].clone();
    let peers: Vec<String> = args[2..].to_vec();

    let chain = Arc::new(Mutex::new(Blockchain::new()));

    let listen_chain = Arc::clone(&chain);
    let listen_addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&listen_addr).expect("failed to bind port");
    println!("[node {}] listening on {}", port, listen_addr);

    thread::spawn(move || {
        for incoming in listener.incoming().flatten() {
            let chain = Arc::clone(&listen_chain);
            thread::spawn(move || handle_peer(incoming, chain));
        }
    });

    thread::sleep(Duration::from_secs(2));

    loop {
        thread::sleep(Duration::from_secs(MINE_INTERVAL_SECS));
        let data = format!("hello from node {} @ {}", port, now());

        let new_block = {
            let mut bc = chain.lock().unwrap();
            let b = bc.mine_next(data);
            println!("\n[node {}] mined a new block:", port);
            bc.print_chain();
            b
        };

        broadcast(&peers, &new_block, &port);
    }
}

fn handle_peer(stream: TcpStream, chain: Arc<Mutex<Blockchain>>) {
    let reader = BufReader::new(stream);
    for line in reader.lines().flatten() {
        if let Ok(block) = serde_json::from_str::<Block>(&line) {
            let mut bc = chain.lock().unwrap();
            if bc.try_add(block.clone()) {
                println!("\n[peer msg] accepted block #{} from network:", block.index);
                bc.print_chain();
            } else {
                println!("\n[peer msg] rejected block #{} (didn't extend our tip)", block.index);
            }
        }
    }
}

fn broadcast(peers: &[String], block: &Block, port: &str) {
    let msg = serde_json::to_string(block).unwrap() + "\n";
    for peer in peers {
        match TcpStream::connect(peer) {
            Ok(mut stream) => {
                let _ = stream.write_all(msg.as_bytes());
            }
            Err(_) => println!("[node {}] could not reach peer {}", port, peer),
        }
    }
}
