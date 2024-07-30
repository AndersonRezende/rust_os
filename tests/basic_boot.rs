/* Testes de integração. A convenção diz que ao colocar testes dentro do diretório tests na raiz do
 * projeto faz com que sejam considerados testes de integração. Tanto o framework padrão quanto o
 * customizado pegarão e executarão os arquivos desse diretório.
 * Todos os testes de integrações são arquivos próprios executáveis separados do main.rs. Isso significa
 * que cada teste precisa definir sua própria função de ponto de entrada.
*/
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
// Não precisamos do #cfg(test) por conta que testes de integração não são construídos em modo de não teste.


use core::panic::PanicInfo;
use rust_os::println;


#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info);
}

#[test_case]
fn test_println() {
    println!("test_println output")
}