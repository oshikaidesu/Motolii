//! Chrome 部品の載せ口。ある `*.rs` だけを `mod` する。中身は触らない。
use makepad_widgets::*;

pub mod button;
pub mod color;
pub mod face;
pub mod feedback;
pub mod fold;
pub mod ink;
pub mod menu;
pub mod nav;
pub mod row;
pub mod rule;
pub mod scrub;
pub mod search;
pub mod stepper;
pub mod toggle;
pub mod transport;

pub fn script_mod(vm: &mut ScriptVm) {
    face::script_mod(vm);
    ink::script_mod(vm);
    rule::script_mod(vm);
    row::script_mod(vm);
    button::script_mod(vm);
    toggle::script_mod(vm);
    scrub::script_mod(vm);
    search::script_mod(vm);
    stepper::script_mod(vm);
    color::script_mod(vm);
    nav::script_mod(vm);
    menu::script_mod(vm);
    fold::script_mod(vm);
    transport::script_mod(vm);
    feedback::script_mod(vm);
}
