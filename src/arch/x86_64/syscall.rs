use consts::irq::*;

pub fn switch_to_user() {
    debug!("switch_to_user");
    unsafe { int!(T_SWITCH_TOU); }
    debug!("switch_to_user finish");
}

pub fn switch_to_kernel() {
    unsafe { int!(T_SWITCH_TOK); }
}

pub fn fork() {
    unsafe { int!(T_FORK); }
}