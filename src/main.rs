/*
Por padrão, o Rust tenta construir um executável que seja capaz de rodar no seu ambiente de sistema atual.
Por exemplo, se você estiver usando o Windows no x86_64, o Rust tenta construir um .exeexecutável do
Windows que use x86_64instruções. Esse ambiente é chamado de seu sistema “host”.
-> rustup target add thumbv7em-none-eabihf
cargo build --target thumbv7em-none-eabihf
Ao passar o --target fazemos uma compilação cruazada para um sistema de destino bare metal.
*/

// Desabilita a biblioteca padrão.
#![no_std]

/* Desabilita o ponto de entrada normal. A main é chamada por um ponto de entrada executado
* inicialmente chamado crt0, que configura o ambiente para um aplicativo C. Isso inclui criar uma
* pilha e colocar os parametros nos registradores corretos.
*/
#![no_main]

mod vga_buffer;

use core::fmt::Write;
use core::panic::PanicInfo;

//static HELLO: &[u8] = b"Hello World!";

// Define a função que o compilador deve invocar quando um panic acontece.
#[panic_handler]
// PanicInfo contém o arquivo e linha onde o panic aconteceu e uma mensagem.
fn panic(_info: &PanicInfo) -> !{
    println!("{}", _info);
    // A função nunca deve retornar, logo ela é marcada como uma função divergente retornando o tipo never
    loop {}
}

/* A main não faz sentido nesse contexto, logo sobrescrevemos o ponto de entrada do sistema
* operacional através do _start.
* Ao utilizar o atributo #[no_mangle], desabilitamos a capacidade do compilador renomear funções,
* assim garantimos que _start possuirá realmente esse nome.
* Também marcamos como extern "C" para que o compilador utilize a convenção de chamada C.
* O tipo de retorno "!" significa que a função está divergindo, ou seja, não tem permissão para retornar.
* Isso é necessário porque o ponto de entrada não é chamado por nenhuma função, mas invocado diretamente
* pelo sistema operacional ou bootloader. Então, em vez de retornar, o ponto de entrada deve, por exemplo,
* invocar a exit do sistema operacional (reiniciar a máquina, por exemplo).
*/
#[no_mangle]
pub extern "C" fn _start() -> ! {
    /*let vga_buffer = 0xb8000 as *mut u8;

    for( i, &byte) in HELLO.iter().enumerate() {
        /* O bloco unsafe é necessário, pois o compilador Rust não pode provar que os ponteiros
         * brutos que criamos são válidos. Ao colocar o unsafe dizemos ao compilador para ignorar
         * esses possíveis erros.
        */
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }*/

    //vga_buffer::print_something();
    vga_buffer::WRITER.lock().write_str("Hello world!\n").unwrap();                              // A chamada de write! retorna um Result que causa aviso se não for usado, logo é necessário utilizar o unwrap() para entrar em panic caso ocorra um erro.
    write!(vga_buffer::WRITER.lock(), "Numero: {}", 95).unwrap();
    println!("\nTeste");
    loop {}
}