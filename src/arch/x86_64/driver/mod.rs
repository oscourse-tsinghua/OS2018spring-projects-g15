pub mod vga;
pub mod acpi;
pub mod apic;
pub mod mp;
pub mod serial;
pub mod pic;
pub mod keyboard;
pub mod pit;

use memory::ActivePageTable;

pub fn init(active_table: &mut ActivePageTable) {
    use memory::{Frame};
    use arch::paging::EntryFlags;

    assert_has_not_been_called!();

    // TODO Handle this temp page map.
    let result = active_table.identity_map(Frame::containing_address(0), EntryFlags::WRITABLE); // EBDA
    result.flush(active_table);
    for addr in (0xE0000 .. 0x100000).step_by(0x1000) {
        let result = active_table.identity_map(Frame::containing_address(addr), EntryFlags::WRITABLE);
        result.flush(active_table);
    }
    let result = active_table.identity_map(Frame::containing_address(0x7fe1000), EntryFlags::WRITABLE); // RSDT
    result.flush(active_table);
    

    // if cfg!(feature = "use_apic") {
    //     pic::disable();

    //     active_table.identity_map(Frame::containing_address(acpi), EntryFlags::WRITABLE.lapic_addr as usize);  // LAPIC
    //     active_table.identity_map(Frame::containing_address(0xFEC00000), EntryFlags::WRITABLE);  // IOAPIC

        // apic::init(acpi.lapic_addr, acpi.ioapic_id);
    // } else {
    //     pic::init();
    // }
    if cfg!(feature = "use_apic") {
        unsafe {
            pic::disable();

            let result = active_table.identity_map(Frame::containing_address(0xfee00000), EntryFlags::WRITABLE);  // LAPIC
            result.flush(active_table);
            
            let result = active_table.identity_map(Frame::containing_address(0xFEC00000), EntryFlags::WRITABLE);  // IOAPIC
            result.flush(active_table);
            

            // apic::init(acpi.lapic_addr, acpi.ioapic_id);
            apic::local_apic::init(active_table);
        }
    } else {
        unsafe {
            pic::init();
            apic::local_apic::init(active_table);
            acpi::init(active_table);
        }
    }
    pit::init();
    serial::init();
    keyboard::init();
}