use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

/* Utilizamos a interface UART para a comunicação entre o guest e host. Podemos comunicar o qemu com
* uma saída que pode ser uma saída padrão ou arquivo.
* Utilizamos o lazy_static para assegurar que o init só será chamado uma vez no seu primeiro uso.
*/
lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        /* O UART é programado utilizando múltiplas portas de I/O para programar diferentes
         * registradores de dispositivos. A chamada unsafe é por conta que espera um endereço de
         * porta como argumento. Passamos a porta padrão da primeira interface serial como argumento.
         */
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! serial_print {                                                                         // Imprime para o host através do serial
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {                                                                       // Imprime para o host através do serial e quebra linha
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}