#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rust_os::{exit_qemu, QemuExitCode, serial_println};

/* Essa função espera que should_fail() entre em panic! e seja redirecionada para nosso panic
* handler, com isso o handler finaliza a execução com sucesso.
*/
#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();
    serial_println!("[teste não entrou em panic]");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

fn should_fail() {
    serial_println!("should_panic::should_fail...\t");
    assert_eq!(0, 1);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}