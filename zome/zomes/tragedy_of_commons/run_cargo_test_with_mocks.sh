#RUSTFLAGS='-A warnings' cargo test --features "mock" -- --nocapture

# RUSTFLAGS='-A warnings' cargo test --features "mock" --package tragedy_of_commons --lib -- game_round::tests::test_try_to_close_round_fails_not_enough_moves --exact --nocapture

# RUSTFLAGS='-A warnings' cargo test --features "mock" --package tragedy_of_commons --lib -- game_round::tests::test_try_to_close_round_success_create_next_round --exact --nocapture

RUST_BACKTRACE=debug RUSTFLAGS='-A warnings' cargo test --features "mock" --package tragedy_of_commons --lib  -- game_round::tests::test_try_to_close_round_success_create_next_round           --exact --nocapture
RUST_BACKTRACE=debug RUSTFLAGS='-A warnings' cargo test --features "mock" --package tragedy_of_commons --lib  -- game_round::tests::test_try_to_close_round_fails_not_enough_moves              --exact --nocapture
RUST_BACKTRACE=debug RUSTFLAGS='-A warnings' cargo test --features "mock" --package tragedy_of_commons --lib  -- game_round::tests::test_try_to_close_round_success_create_next_round           --exact --nocapture
RUST_BACKTRACE=debug RUSTFLAGS='-A warnings' cargo test --features "mock" --package tragedy_of_commons --lib  -- game_round::tests::test_try_to_close_round_success_end_game_resources_depleted --exact --nocapture
RUST_BACKTRACE=debug RUSTFLAGS='-A warnings' cargo test --features "mock" --package tragedy_of_commons --lib  -- game_round::tests::test_calculate_round_state                                  --exact --nocapture
RUST_BACKTRACE=debug RUSTFLAGS='-A warnings' cargo test --features "mock" --package tragedy_of_commons --lib  -- game_round::tests::test_start_new_round                                        --exact --nocapture
RUST_BACKTRACE=debug RUSTFLAGS='-A warnings' cargo test --features "mock" --package tragedy_of_commons --lib  -- game_round::tests::test_try_to_close_round_end_game_all_rounds_played          --exact --nocapture