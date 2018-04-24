mod structure;

use self::structure::*;
use consts::*;

#[cfg(any(not(traget_arch = "x86"), not(traget_arch = "x86_64")))]
const PHY_MEMORY_LIMIT: u32 = 0x80000000;
#[cfg(traget_arch = "x86")]
const PHY_MEMORY_LIMIT: u32 = 0x0e000000;
#[cfg(traget_arch = "x86_64")]
const PHY_MEMORY_LIMIT: u32 = 0x80000000;

#[derive(Debug)]
pub struct ACPI_Result {
    pub cpu_num: u8,
    pub cpu_acpi_ids: [u8; MAX_CPU_NUM],
    pub ioapic_id: u8,
    pub lapic_addr: *const (),
}

#[derive(Debug)]
pub enum ACPI_Error {
    NotMapped,
    IOACPI_NotFound,
}

fn config_SMP(madt: &'static Madt) -> Result<ACPI_Result, ACPI_Error> {
    let lapic_addr = madt.lapicAddr as *const ();

    let mut cpu_num = 0u8;
    let mut cpu_acpi_ids: [u8; MAX_CPU_NUM] = [0; MAX_CPU_NUM];
    let mut ioapic_id: Option<u8> = None;
    for entry in madt.entry_iter() {
        println!("{:?}", entry);
        match &entry {
            &MadtEntry::LocalApic(ref lapic) => {
                cpu_acpi_ids[cpu_num as usize] = lapic.id;
                cpu_num += 1;
            },
            &MadtEntry::IoApic(ref ioapic) => {
                ioapic_id = Some(ioapic.id);
            },
            _ => {},
        }
    }

    if ioapic_id.is_none() {
        return Err(ACPI_Error::IOACPI_NotFound);
    }
    let ioapic_id = ioapic_id.unwrap();
    Ok(ACPI_Result{ cpu_num, cpu_acpi_ids, ioapic_id, lapic_addr })
}

pub fn find_rsdp() -> Option<&'static Rsdp> {
    use utils::{Checkable, find_in_memory};
    let ebda = unsafe{ *(0x40e as *const u16) as usize } << 4;
    println!("EBDA at {:#x}", ebda);

    macro_rules! return_if_find_in {
        ($begin: expr, $end: expr) => (
            if let Some(addr) = unsafe {
                find_in_memory::<Rsdp>($begin, $end, 4)
            } {
                return Some(unsafe{ &*(addr as *const Rsdp)});
            }
        )
    }

    if ebda != 0 {
        return_if_find_in!(ebda as usize, 1024);
    }
    return_if_find_in!(0xe0000, 0x20000);
    None
}

pub fn init() -> Result<ACPI_Result, ACPI_Error> {
    let rsdp = find_rsdp().expect("ACPI: rsdp not found.");
    if rsdp.rsdtPhyAddr > PHY_MEMORY_LIMIT {
        return Err(ACPI_Error::NotMapped);
    }
    println!("RSDT at {:#x}", rsdp.rsdtPhyAddr);
    let rsdt = unsafe{ &*(rsdp.rsdtPhyAddr as *const Rsdt) };
    let mut madt: Option<&'static Madt> = None;
    for i in 0 .. rsdt.entry_count() {
        let entry = rsdt.entry_at(i);
        if entry > PHY_MEMORY_LIMIT {
            return Err(ACPI_Error::NotMapped);
        }
        let hdr = unsafe{ &*(entry as *const Header) };
        if &(hdr.signature) == b"APIC" {
            madt = Some(unsafe{ &*(entry as *const Madt)});
        }
    }
    println!("{:?}", madt);
    config_SMP(madt.expect("ACPI: madt not found!"))
}
