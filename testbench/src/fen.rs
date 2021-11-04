use crate::*;

fn from_fen_classic(criterion: &mut Criterion) {
    criterion.bench_function("from fen classic", |bencher| bencher.iter(| | {
        GameState::from_fen(GameState::FEN_CLASSIC)
            .expect("failed to parse FEN string");
    }));
}

fn to_fen_classic(criterion: &mut Criterion) {
    let state = GameState::from_fen(GameState::FEN_CLASSIC)
        .expect("failed to parse FEN string");
    criterion.bench_function("to fen classic", |bencher| bencher.iter(| | {
        assert_eq!(state.to_fen(), GameState::FEN_CLASSIC)
    }));
}

fn from_fen_fuzz() {}
fn to_fen_fuzz() {}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .plotting_backend(criterion::PlottingBackend::Plotters);
    targets = 
        from_fen_classic,
        to_fen_classic,
);
