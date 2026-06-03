use std::{
    env::args,
    fs::{self, File},
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
                let arr: [u8; 8] = chunk.try_into().unwrap();
                result.push(usize::from_le_bytes(arr));
            }
            result
        }
        Err(_) => Vec::new(),
    }
}

fn write_cache(cache: &[usize]) {
    let tmp_name = format!("{}.tmp", CACHE_FILE_NAME);
    let file = File::create(&tmp_name).unwrap();
    let mut writer = BufWriter::new(file);
    for &prime in cache {
        writer.write_all(&prime.to_le_bytes()).unwrap();
    }
    writer.flush().unwrap();
    fs::rename(&tmp_name, CACHE_FILE_NAME).unwrap();
}

fn is_prime_cached(n: usize, cache: &[usize]) -> bool {
    if n < 2 {
        return false;
    }
    for &p in cache {
        if p > n / p {
            break;
        }
        if n.is_multiple_of(p) {
            return false;
        }
    }
    true
}

fn sieve_segment(start: usize, end: usize, cache: &[usize]) -> Vec<usize> {
    if start > end {
        return vec![];
    }
    let segment_len = end - start + 1;
    let mut is_prime = vec![true; segment_len];

    for &p in cache {
        if p > end / p {
            break;
        }
        let first_multiple = start.div_ceil(p) * p;
        let first = first_multiple.max(p * p);

        for multiple in (first..=end).step_by(p) {
            is_prime[multiple - start] = false;
        }
    }

    (start..=end).filter(|&i| is_prime[i - start]).collect()
}

fn extend_cache(cache: &mut Vec<usize>, start: usize, end: usize) {
    let start = start.max(2);
    if start > end {
        return;
    }
    let new_primes = sieve_segment(start, end, cache);
    cache.extend(new_primes);
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

    let cache_max = cache.last().copied().unwrap_or(0);

    let sqrt_limit = if limit == 0 {
        0
    } else {
        let mut s = 1;
        while s <= limit / s {
            s += 1;
        }
        s - 1
    };

    if cache_max < sqrt_limit {
        extend_cache(&mut cache, (cache_max + 1).max(2), sqrt_limit);
    }

    let old_cache_max = cache.last().copied().unwrap_or(1);
    let mut new_large_primes = Vec::new();

    let stdout = io::stdout();
    let mut printer = BufWriter::new(stdout.lock());

    if start <= 2 && limit >= 2 {
        write!(&mut printer, "2 ").unwrap();
        if 2 > old_cache_max {
            new_large_primes.push(2);
        }
    }

    let mut x = if start <= 2 {
        3
    } else if start % 2 == 0 {
        start + 1
    } else {
        start
    };

    while x <= limit {
        if is_prime_cached(x, &cache) {
            write!(&mut printer, "{x} ").unwrap();
            if x > old_cache_max {
                new_large_primes.push(x);
            }
        }
        x += 2;
    }

    writeln!(&mut printer).unwrap();
    printer.flush().unwrap();

    cache.extend(new_large_primes);
    write_cache(&cache);
}
