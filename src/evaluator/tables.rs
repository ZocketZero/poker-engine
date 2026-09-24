use crate::card::Rank;
use std::sync::OnceLock;

pub const MAX_BITMASK: usize = 0x1FFF + 1; // 13 bits for 13 ranks (0..8191)

pub struct LookupTables {
    pub flushes: Vec<u16>,
    pub unique5: Vec<u16>,
    pub products: Vec<u32>,
    pub values: Vec<u16>,
}

static TABLES: OnceLock<LookupTables> = OnceLock::new();

pub fn get_tables() -> &'static LookupTables {
    TABLES.get_or_init(init_tables)
}

fn init_tables() -> LookupTables {
    let mut flushes = vec![0u16; MAX_BITMASK];
    let mut unique5 = vec![0u16; MAX_BITMASK];
    let mut products = Vec::with_capacity(4888);
    let mut values = Vec::with_capacity(4888);

    // 1. Identify all 10 straight masks
    let straight_masks: [u32; 10] = [
        0x1F00, // A-K-Q-J-T (Broadway)
        0x0F80, // K-Q-J-T-9
        0x07C0, // Q-J-T-9-8
        0x03E0, // J-T-9-8-7
        0x01F0, // T-9-8-7-6
        0x00F8, // 9-8-7-6-5
        0x007C, // 8-7-6-5-4
        0x003E, // 7-6-5-4-3
        0x001F, // 6-5-4-3-2
        0x100F, // 5-4-3-2-A (Wheel: Ace bit 12 + bits 0,1,2,3)
    ];

    // Assign straight flush ranks (1..=10) and straight ranks (1600..=1609)
    for (i, &mask) in straight_masks.iter().enumerate() {
        flushes[mask as usize] = (i + 1) as u16;
        unique5[mask as usize] = (1600 + i) as u16;
    }

    // 2. Generate all C(13, 5) combinations of 5 distinct ranks in descending order
    let mut flush_rank = 323u16;
    let mut high_card_rank = 6186u16;

    for c1 in (4..13).rev() {
        for c2 in (3..c1).rev() {
            for c3 in (2..c2).rev() {
                for c4 in (1..c3).rev() {
                    for c5 in (0..c4).rev() {
                        let mask = (1 << c1) | (1 << c2) | (1 << c3) | (1 << c4) | (1 << c5);
                        // If it's not one of the 10 straights
                        if !straight_masks.contains(&mask) {
                            flushes[mask as usize] = flush_rank;
                            flush_rank += 1;

                            unique5[mask as usize] = high_card_rank;
                            high_card_rank += 1;
                        }
                    }
                }
            }
        }
    }
    debug_assert_eq!(flush_rank, 1600);
    debug_assert_eq!(high_card_rank, 7463);

    // 3. Four of a kind (156 hands): rank 11..=166
    let mut quad_rank = 11u16;
    for q in (0..13).rev() {
        let q_prime = Rank::ALL[q].prime();
        for k in (0..13).rev() {
            if q == k {
                continue;
            }
            let k_prime = Rank::ALL[k].prime();
            let prod = q_prime * q_prime * q_prime * q_prime * k_prime;
            products.push(prod);
            values.push(quad_rank);
            quad_rank += 1;
        }
    }
    debug_assert_eq!(quad_rank, 167);

    // 4. Full house (156 hands): rank 167..=322
    let mut fh_rank = 167u16;
    for t in (0..13).rev() {
        let t_prime = Rank::ALL[t].prime();
        for p in (0..13).rev() {
            if t == p {
                continue;
            }
            let p_prime = Rank::ALL[p].prime();
            let prod = t_prime * t_prime * t_prime * p_prime * p_prime;
            products.push(prod);
            values.push(fh_rank);
            fh_rank += 1;
        }
    }
    debug_assert_eq!(fh_rank, 323);

    // 5. Three of a kind (858 hands): rank 1610..=2467
    let mut trip_rank = 1610u16;
    for t in (0..13).rev() {
        let t_prime = Rank::ALL[t].prime();
        for k1 in (1..13).rev() {
            if k1 == t {
                continue;
            }
            let k1_prime = Rank::ALL[k1].prime();
            for k2 in (0..k1).rev() {
                if k2 == t {
                    continue;
                }
                let k2_prime = Rank::ALL[k2].prime();
                let prod = t_prime * t_prime * t_prime * k1_prime * k2_prime;
                products.push(prod);
                values.push(trip_rank);
                trip_rank += 1;
            }
        }
    }
    debug_assert_eq!(trip_rank, 2468);

    // 6. Two pair (858 hands): rank 2468..=3325
    let mut tp_rank = 2468u16;
    for p1 in (1..13).rev() {
        let p1_prime = Rank::ALL[p1].prime();
        for p2 in (0..p1).rev() {
            let p2_prime = Rank::ALL[p2].prime();
            for k in (0..13).rev() {
                if k == p1 || k == p2 {
                    continue;
                }
                let k_prime = Rank::ALL[k].prime();
                let prod = p1_prime * p1_prime * p2_prime * p2_prime * k_prime;
                products.push(prod);
                values.push(tp_rank);
                tp_rank += 1;
            }
        }
    }
    debug_assert_eq!(tp_rank, 3326);

    // 7. One pair (2,860 hands): rank 3326..=6185
    let mut op_rank = 3326u16;
    for p in (0..13).rev() {
        let p_prime = Rank::ALL[p].prime();
        for k1 in (2..13).rev() {
            if k1 == p {
                continue;
            }
            let k1_prime = Rank::ALL[k1].prime();
            for k2 in (1..k1).rev() {
                if k2 == p {
                    continue;
                }
                let k2_prime = Rank::ALL[k2].prime();
                for k3 in (0..k2).rev() {
                    if k3 == p {
                        continue;
                    }
                    let k3_prime = Rank::ALL[k3].prime();
                    let prod = p_prime * p_prime * k1_prime * k2_prime * k3_prime;
                    products.push(prod);
                    values.push(op_rank);
                    op_rank += 1;
                }
            }
        }
    }
    debug_assert_eq!(op_rank, 6186);
    debug_assert_eq!(products.len(), 4888);

    // Sort products and values together by product so we can binary search!
    let mut pairs: Vec<(u32, u16)> = products.into_iter().zip(values).collect();
    pairs.sort_unstable_by_key(|&(prod, _)| prod);

    let (sorted_prods, sorted_vals): (Vec<u32>, Vec<u16>) = pairs.into_iter().unzip();

    LookupTables {
        flushes,
        unique5,
        products: sorted_prods,
        values: sorted_vals,
    }
}
