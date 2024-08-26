// Desabilita a biblioteca padrão.
#![no_std]
#![no_main]
/* Implementa nossa própria framework de testes já que a framework padrão possui recursos atrelados
* biblioteca padrão.
* Funciona coletando todas as funções com a anotação #[test_case] e então invoca uma função executora
* com a lista de testes como argumentos.
*/
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
/* Especifica uma função customizada chamada test_runner (que reside no módulo crate, ou seja, no
* próprio crate em que o código está) para ser usada como o executor de testes. O Rust normalmente
* usa um executor padrão para rodar testes, mas com essa linha, você está indicando que deseja usar
* sua própria função test_runner para lidar com a execução dos testes.
*/
#![test_runner(crate::test_runner)]
/* Como _start é o ponto de entrada, a nossa framework de testes gera uma função main que chama a
* test_runner, mas a função é ignorada, pois utilizamos o atributo #[no_main].
* Nos definimos o nome da função de entrada da framework de testes e chamamos no ponto de entrada.
*/
#![reexport_test_harness_main = "test_main"]

/* Criamos uma biblioteca para retornar as funções necessárias para o teste de integração (tests/).
*/

pub mod serial;
pub mod vga_buffer;
pub mod interrupts;

use core::panic::PanicInfo;

pub fn init() {
    interrupts::init_idt();
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where T: Fn(),
{
    fn run(&self) -> () {
        /* Implementamos a função run primeiro imprimindo o seu nome.
         * Após imprimir o nome da função, a chamamos através do self.
         */
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

/*
* Nossa função executadora imprime uma curta mensagem de debug e chama a função de teste na lista.
* O tipo de argumento &[&dyn Fn()] representa uma fatia de referências de objeto da trait Fn().
* Isso basicamente é uma referência de algo "chamével" que pode ser chamada como uma função.
*/
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Executando teste {}", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        /* A função cria uma nova Port no endereço 0xf4 que foi definido no Cargo.toml como iobase do
        * isa-debug-exit. Com isso, podemos passar o código de saída para a porta. Usamos u32 por conta
        * que definimos o iosize do dispositivo isa-debug-exit como 4 bytes.
        */
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}



/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {                                                                   // A lib é testada fora do main, logo precisa de um ponto de entrada e um manipulador de pânico.
    init();
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}