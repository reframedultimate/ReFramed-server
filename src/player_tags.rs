use skyline;
use std;

pub fn get_name_for_slot(slot: i32) -> String {
    let base_offset = unsafe {
        let ptr = (skyline::hooks::getRegionAddress(skyline::hooks::Region::Text) as *const u8).add(0x52c5758);
        ptr.add(slot as usize * 0x260) as *const u16
    };

    let mut len = 0;
    while unsafe { *base_offset.add(len) != 0 } {
        len += 1;
    }

    let slice = unsafe {
        std::slice::from_raw_parts(base_offset, len)
    };
    String::from_utf16_lossy(slice)
}

