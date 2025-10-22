// cargo install --locked kani-verifier
// cargo kani setup

fn initialize_prefix(length: usize, buffer: &mut [u8]) {
    // Let's just ignore invalid calls
    if length > buffer.len() {
        return;
    }

    for i in 0..=length {
        buffer[i] = 0;
    }
}

#[cfg(kani)]
#[kani::proof]
#[kani::unwind(1)] // deliberately too low
fn check_initialize_prefix() {
    const LIMIT: usize = 10;
    let mut buffer: [u8; LIMIT] = [1; LIMIT];

    let length = kani::any();
    kani::assume(length <= LIMIT);

    initialize_prefix(length, &mut buffer);
}

fn silly_vector_things(v: &mut Vec<i128>) {
    for number in v.iter_mut() {
        match (*number) % 10 {
            0 => *number += 1,
            1 => *number += 10,
            2 => *number += 100,
            3 => *number += 1000,
            4 => *number += 10000,
            5 => *number += 100000,
            6 => *number += 1000000,
            7 => *number += 10000000,
            8 => *number += 100000000,
            9 => *number += 1000000000,
            _ => unreachable!(),
        }
    }
}

#[cfg(kani)]
#[kani::proof]
#[kani::unwind(10)] // already causes memory issues
fn check_silly_vector_things() {
    const MAX_LEN: usize = 5;
    let len = kani::any();
    kani::assume(len <= MAX_LEN);
    let mut v: Vec<i128> = Vec::with_capacity(len);
    for _ in 0..len {
        v.push(kani::any());
    }
    silly_vector_things(&mut v);
    assert!(v.len() == len);
}