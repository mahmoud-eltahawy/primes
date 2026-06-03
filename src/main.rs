use std::{
    env::args,
    fs::File,
    io::{self, BufReader, BufWriter, Read, Write},
};

const CACHE_FILE_NAME: &str = "PRIMES_CACHE_FILE";

fn read_cache() -> Vec<usize> {
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
                // Safe: chunks_exact(8) always yields a slice of length 8
                let arr: [u8; 8] = chunk.try_into().unwrap();
                result.push(usize::from_le_bytes(arr));
            }
            result
        }
        Err(_) => Vec::new(),
    }
}

fn write_cache(cache: &[usize]) {
    let file = File::create(CACHE_FILE_NAME).unwrap();
    let mut writer = BufWriter::new(file);
    for &prime in cache {
        writer.write_all(&prime.to_le_bytes()).unwrap();
    }
    writer.flush().unwrap();
}

fn is_prime_cached(n: usize, cache: &[usize]) -> bool {
    if n < 2 {
        return false;
    }
    for &p in cache {
        if p * p > n {
            break;
        }
        if n % p == 0 {
            return false;
        }
    }
    true
}

fn extend_cache(cache: &mut Vec<usize>, start: usize, end: usize) {
    let start = start.max(2);
    for candidate in start..=end {
        if is_prime_cached(candidate, cache) {
            cache.push(candidate);
        }
    }
}

fn main() {
    let mut cache = read_cache();
    let mut args = args();
    args.next();

    let first: usize = match args.next().and_then(|x| x.parse().ok()) {
        Some(n) => n,
        None => {
            eprintln!("Usage: primes [OFFSET] LIMIT");
            return;
        }
    };

    let (offset, limit) = match args.next().and_then(|x| x.parse().ok()) {
        Some(second) => (first, second),
        None => (0, first),
    };

    if limit < 2 || offset > limit {
        return;
    }

    let start = offset.max(2);
    let sqrt_limit = (limit as f64).sqrt() as usize;

    let cache_max = cache.last().copied().unwrap_or(1);
    if cache_max < sqrt_limit {
        extend_cache(&mut cache, cache_max + 1, sqrt_limit);
    }

    let old_cache_max = cache.last().copied().unwrap_or(1);
    let mut new_large_primes = Vec::new();

    let stdout = io::stdout();
    let mut printer = BufWriter::new(stdout.lock());

    for x in start..=limit {
        if is_prime_cached(x, &cache) {
            write!(&mut printer, "{x} ").unwrap();
            if x > old_cache_max {
                new_large_primes.push(x);
            }
        }
    }
    writeln!(&mut printer).unwrap();
    printer.flush().unwrap();

    cache.extend(new_large_primes);
    write_cache(&cache);
}
