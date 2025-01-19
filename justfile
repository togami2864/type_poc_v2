create-crate crate_name:
  cargo new crates/{{crate_name}} --lib

test crate_name:
  cargo test -p {{crate_name}}