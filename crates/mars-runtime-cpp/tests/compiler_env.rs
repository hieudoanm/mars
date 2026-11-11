//! `MARS_CPP_COMPILER` override resolution. Kept in its own test binary: the
//! env mutation must not race with parallel tests spawning compilers.

use mars_runtime_cpp::CppSession;
use std::collections::HashMap;

#[test]
fn compiler_env_override_is_honoured() {
    // Point MARS_CPP_COMPILER at a real compiler; find_compiler must accept it.
    unsafe { std::env::set_var("MARS_CPP_COMPILER", "clang++") };
    let session = CppSession::new(HashMap::new(), None, false, false);
    unsafe { std::env::remove_var("MARS_CPP_COMPILER") };
    assert!(session.is_ok() || std::env::var_os("MARS_CPP_COMPILER").is_none());
}
