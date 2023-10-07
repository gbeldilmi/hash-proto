use std::{
    fs::File,
    io::{
        BufReader, 
        Read
    },
    num::Wrapping,
    path::Path,
    thread,
};

const HASH_SIZE: usize = 2 * 4;

struct Hash {
    hash: [Wrapping<u32>; HASH_SIZE],
}

impl Hash {
    fn new() -> Hash {
        Hash {
            hash: {
                let s = vec![
                    0xe3b0c442, 0x98fc1c14, 0x9afbf4c8, 0x996fb924,
                    0x27ae41e4, 0x649b934c, 0xa495991b, 0x7852b855,
                ]
                .iter()
                .map(|x| Wrapping(*x))
                .collect::<Vec<_>>();
                let mut h = [Wrapping(0); HASH_SIZE];
                for i in 0..s.len() {
                    h[i] = s[i];
                }
                h
            },
        }
    }
    fn transform(&mut self, data: [u32; HASH_SIZE]) {
        // simple checksum
        fn conv(d: [u32; HASH_SIZE]) -> [Wrapping<u32>; HASH_SIZE] {
            let mut r = [Wrapping(0); HASH_SIZE];
            for i in 0..HASH_SIZE {
                r[i] = Wrapping(d[i]);
            }
            r
        }
        fn add(
            d: [Wrapping<u32>; HASH_SIZE],
            a: [Wrapping<u32>; HASH_SIZE],
        ) -> [Wrapping<u32>; HASH_SIZE] {
            let mut r = [Wrapping(0); HASH_SIZE];
            for i in 0..HASH_SIZE {
                r[i] = d[i] + a[i];
            }
            r
        }
        let ca = conv(data);
        self.hash = add(ca, self.hash);
    }
    fn to_string(&self) -> String {
        let mut s = String::new();
        for i in 0..HASH_SIZE {
            s.push_str(&format!("{:08x}", self.hash[i]));
        }
        s
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 {
        match args[1].as_str() {
            "-h" => help(),
            //"-c" => check(),
            _ => plop(),
        }
    } else {
        help();
    }
}

fn help() {
    println!("Usage: hash <action> <filenames/directories>");
    println!("Actions:");
    println!("\t-h\t\tShow this help");
    println!("\t<default>\tCalculate hash");
}

fn plop() {
    let args: Vec<String> = std::env::args().collect();
    let mut joinhands = Vec::new();
    for i in 1..args.len() {
        let path = args[i].clone();
        joinhands.push(thread::spawn(move || {
            plop_path(path);
        }));
    }
    for handle in joinhands {
        handle.join().expect("Failed to join thread");
    }
}

fn plop_path(path: String) {
    let path = Path::new(&path);
    if path.try_exists().is_ok() {
        if path.is_dir() {
            plop_dir(path);
        } else {
            plop_file(path);
        }
    } else {
        println!("{} : Access denied", path.display());
    }
}

fn plop_dir(path: &Path) {
    let mut joinhands = Vec::new();
    for entry in path.read_dir().expect("Failed to read directory") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        if path.is_dir() {
            joinhands.push(thread::spawn(move || {
                plop_dir(&path);
            }));
        } else {
            joinhands.push(thread::spawn(move || {
                plop_file(&path);
            }));
        }
    }
    for handle in joinhands {
        handle.join().expect("Failed to join thread");
    }
}

fn plop_file(path: &Path) {
    let file = File::open(path).expect("Failed to open file");
    let mut reader = BufReader::new(file);
    let mut hash = Hash::new();
    let mut buffer = [0u8; HASH_SIZE * 4];
    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .expect(format!("Failed to read file : {}", path.display()).as_str());
        if bytes_read != 0 {
            let mut data = [0u32; HASH_SIZE];
            for i in 0..HASH_SIZE {
                data[i] = u32::from_le_bytes([
                    buffer[i * 4],
                    buffer[i * 4 + 1],
                    buffer[i * 4 + 2],
                    buffer[i * 4 + 3],
                ]);
            }
            hash.transform(data);
        } else {
            break;
        }
    }
    println!("{}\t{}", hash.to_string(), path.display());
}
