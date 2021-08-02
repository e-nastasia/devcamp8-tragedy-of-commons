cd tests && npm test && cd ..


#cargo test --features "mock" --package tragedy_of_commons --lib -- game_round::tests::test_calculate_round_state --exact --nocapture