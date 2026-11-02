# CSCB certified run v1

- corpus: /tmp/cscb_corpus.bin
- blake3 transcript root: 79965bc2b6eeffb9335f3d6ca6c7e050504824f0cf7f0bea5fec9875f0ad9df8
- CSCB_MUTATE: 

| coord | subject | exit | ms |
|---|---|---|---|
| C1 | /Users/sac/cgap/crates/csct/target/release/cscb_c1_c2_ingress | exit 0 | 10 |
| C2 | /Users/sac/cgap/crates/csct/target/release/cscb_c1_c2_ingress | exit 0 | 10 |
| C3 | /Users/sac/cgap/crates/csct/target/release/cscb_c3_closure | exit 1 | 5597 |
| C4 | /Users/sac/cgap/crates/csct/target/release/cscb_c4_alloc | exit 0 | 50 |
| C5 | /Users/sac/cgap/scripts/verify_cscb_sram.py | exit 0 | 790 |
| C6 | /Users/sac/cgap/crates/csct/target/release/cscb_c6_retraction | exit 1 | 13 |
| C7 | /Users/sac/cgap/crates/csct/target/release/cscb_c7_receipt_sealing | exit 0 | 1841 |
| C8 | cargo test -p csct drowning_benchmark | exit 0 | 64 |
