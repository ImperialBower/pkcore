use criterion::{Criterion, black_box, criterion_group, criterion_main};
use pkcore::prelude::{DealEval, HoleCards, Two};

fn heads_up(c: &mut Criterion) {
    let hands = HoleCards::from(vec![Two::HAND_AS_AH, Two::HAND_KS_KH]);
    c.bench_function("deal eval heads up", |b| {
        b.iter(|| DealEval::new(black_box(hands.clone())).unwrap());
    });
}

fn three_way(c: &mut Criterion) {
    let hands = HoleCards::from(vec![Two::HAND_AS_AH, Two::HAND_KS_KH, Two::HAND_2D_2C]);
    c.bench_function("deal eval three way", |b| {
        b.iter(|| DealEval::new(black_box(hands.clone())).unwrap());
    });
}

criterion_group!(benches, heads_up, three_way);
criterion_main!(benches);
