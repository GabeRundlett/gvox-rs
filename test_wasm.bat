@REM uncomment the main function in lib.rs
@REM comment out the #[cfg(test)] line to include the tests module unconditionally
@REM in tests.rs, comment out the #[test] on 

cargo build --target wasm32-unknown-unknown
wasm2wat target/wasm32-unknown-unknown/debug/gvox-rs.wasm --output=gvox.wat
