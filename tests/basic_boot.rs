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

/*
* #![feature(custom_test_frameworks)]: Habilita uma feature (recurso) experimental no Rust chamada
* custom_test_frameworks. Essa feature permite que você defina e utilize um framework de testes
* customizado, ao invés de usar o framework de testes padrão do Rust.

* #![test_runner(crate::test_runner)]: Especifica uma função customizada chamada test_runner (que
* reside no módulo crate, ou seja, no próprio crate em que o código está) para ser usada como o
* executor de testes. O Rust normalmente usa um executor padrão para rodar testes, mas com essa
* linha, você está indicando que deseja usar sua própria função test_runner para lidar com a
* execução dos testes.

* #![reexport_test_harness_main = "test_main"]: Substitui a função principal de teste (normalmente
* gerada pelo framework de testes padrão do Rust) por uma função de nome customizado, neste caso,
* test_main. Isso é útil quando você precisa de mais controle sobre como os testes são organizados e
* executados, especialmente em projetos mais complexos ou que precisam integrar outros sistemas de teste.
*/
// Não precisamos do #cfg(test) por conta que testes de integração sempre são construídos em modo de teste.


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