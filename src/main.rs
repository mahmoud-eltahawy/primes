use std::{
    env::args,
    fs::{self, File},
    io::{self, BufReader, BufWriter, Read, Write},
};

const CACHE_FILE_NAME: &str = "PRIMES_CACHE_FILE";

fn integer_sqrt(n: u64) -> u64 {
    if n < 2 {
        return n;
    }
    let mut x = n;
    let mut y = x.div_ceil(2);
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

fn read_cache() -> Vec<u64> {
    match File::open(CACHE_FILE_NAME) {
        Ok(file) => {
            let file_len = file.metadata().unwrap().len();
            assert!(file_len % 8 == 0, "Cache file is corrupted");
            let mut reader = BufReader::new(file);
            let mut buf = vec![0u8; file_len as usize];
            reader.read_exact(&mut buf).unwrap();

            let count = buf.len() / 8;
            let mut result = Vec::with_capacity(count);
            for chunk in buf.chunks_exact(8) {
                let arr: [u8; 8] = chunk.try_into().unwrap();
                result.push(u64::from_le_bytes(arr));
            }
            result
        }
        Err(_) => Vec::new(),
    }
}

fn write_cache(cache: &[u64]) {
    let tmp_name = format!("{}.tmp", CACHE_FILE_NAME);
    let file = File::create(&tmp_name).unwrap();
    let mut writer = BufWriter::new(file);
    for &prime in cache {
        writer.write_all(&prime.to_le_bytes()).unwrap();
    }
    writer.flush().unwrap();
    fs::rename(&tmp_name, CACHE_FILE_NAME).unwrap();
}

struct BitVec(Vec<u64>);

impl BitVec {
    fn new_filled(len: usize, value: bool) -> Self {
        let blocks = len.div_ceil(64);
        let fill = if value { u64::MAX } else { 0 };
        Self(vec![fill; blocks])
    }

    fn set(&mut self, index: usize, value: bool) {
        let block = index / 64;
        let bit = index % 64;
        if value {
            self.0[block] |= 1 << bit;
        } else {
            self.0[block] &= !(1 << bit);
        }
    }

    fn get(&self, index: usize) -> bool {
        let block = index / 64;
        let bit = index % 64;
        (self.0[block] >> bit) & 1 == 1
    }
}

fn is_prime_cached(n: u64, cache: &[u64]) -> bool {
    if n < 2 {
        return false;
    }
    let sqrt_n = integer_sqrt(n);
    for &p in cache {
        if p > sqrt_n {
            break;
        }
        if n % p == 0 {
            return false;
        }
    }
    true
}

fn sieve_segment(start: u64, end: u64, cache: &[u64]) -> Vec<u64> {
    if start > end {
        return vec![];
    }
    let segment_len = (end - start + 1) as usize;
    let mut is_prime = BitVec::new_filled(segment_len, true);

    for &p in cache {
        if p > end / p {
            break;
        }
        let first_multiple = start.div_ceil(p).max(p) * p;
        let start_idx = (first_multiple - start) as usize;
        let step = p as usize;
        let mut idx = start_idx;
        while idx < segment_len {
            is_prime.set(idx, false);
            idx += step;
        }
    }

    (start..=end)
        .filter(|&i| is_prime.get((i - start) as usize))
        .collect()
}

fn extend_cache(cache: &mut Vec<u64>, start: u64, end: u64) {
    let start = start.max(2);
    if start > end {
        return;
    }
    let new_primes = sieve_segment(start, end, cache);
    cache.extend(new_primes);
}

struct Cli {
    offset: u64,
    limit: u64,
}

fn parse_args() -> Option<Cli> {
    let mut args = args();
    args.next();

    let first: u64 = args.next()?.parse().ok()?;
    let (offset, limit) = match args.next().and_then(|x| x.parse().ok()) {
        Some(second) => (first, second),
        None => (0, first),
    };

    Some(Cli { offset, limit })
}

fn odd_candidates(start: u64, limit: u64) -> Vec<u64> {
    let mut x = if start <= 2 {
        3
    } else if start % 2 == 0 {
        start + 1
    } else {
        start
    };
    let mut candidates = Vec::new();
    while x <= limit {
        candidates.push(x);
        x += 2;
    }
    candidates
}

fn find_primes(candidates: &[u64], cache: &[u64]) -> Vec<u64> {
    let total = candidates.len() as u64;
    let mut found = Vec::new();
    for (i, &x) in candidates.iter().enumerate() {
        if is_prime_cached(x, cache) {
            found.push(x);
        }
        let stdout = io::stdout();
        let mut printer = BufWriter::new(stdout.lock());
        if (i + 1) % 10_000 == 0 {
            write!(&mut printer, "\rTesting candidate {} / {}", i + 1, total).unwrap();
            printer.flush().unwrap();
        }
    }
    eprintln!("\rTesting complete: {} candidates checked", total);
    found
}

fn print_primes(primes: &[u64]) {
    let stdout = io::stdout();
    let mut printer = BufWriter::new(stdout.lock());
    for p in primes {
        write!(&mut printer, "{p} ").unwrap();
    }
    writeln!(&mut printer).unwrap();
    printer.flush().unwrap();
}

fn main() {
    let Cli { offset, limit } = match parse_args() {
        Some(cli) => cli,
        None => {
            eprintln!("Usage: primes [OFFSET] LIMIT");
            return;
        }
    };

    let mut cache = read_cache();

    if limit < 2 || offset > limit {
        return;
    }

    let start = offset.max(2);

    let cache_max = cache.last().copied().unwrap_or(0);
    let sqrt_limit = integer_sqrt(limit);
    if cache_max < sqrt_limit {
        extend_cache(&mut cache, (cache_max + 1).max(2), sqrt_limit);
    }

    let old_cache_max = cache.last().copied().unwrap_or(1);

    let candidates = odd_candidates(start, limit);
    let mut found = find_primes(&candidates, &cache);

    if start <= 2 && limit >= 2 {
        found.insert(0, 2);
    }

    found.sort_unstable();
    print_primes(&found);

    let new_large: Vec<u64> = found.into_iter().filter(|&p| p > old_cache_max).collect();
    cache.extend(new_large);
    write_cache(&cache);
}
