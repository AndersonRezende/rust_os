use core::fmt;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;


/*
* Necessário para corrigir o problema de inicialização de variáveis, já que variáveis estáticas são
* inicializadas em tempo de compilação enquanto as normais são inicializadas em tempo de execução.
* Esse crate fornece essa macro que define um lazily initialized static que em vez de calcular seu
* valor em tempo de compilação, ela inicializa a si mesma quando acessada pela primeira vez.
*/
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe {                                                                            // O bloco unsafe é necessário, pois o compilador Rust não pode provar que os ponteiros brutos que criamos são válidos. Ao colocar o unsafe dizemos ao compilador para ignorar esses possíveis erros.
            /*
             * O novo writer aponta para o buffer VGA em 0xb8000
             * Converte um inteiro como um ponteiro mutável raw 0xb8000.
             *
             */
            &mut *(0xb8000 as *mut Buffer)
        },
    });
}


#[allow(dead_code)]                                                                                 // Atributo utilizado para esconder avisos de código não utilizado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]                                                        // Habilitar semântica de cópia.
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]                                                                                // É utilizado paga garantir que a estrutura tenha a mesma representação na memória que o seu tipo primário.
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))                                     // Cada cor necessita de 4 bits, como não temos o tipo u4, precisamos deslocar os bits para encaixarmos as cores primárias e secundárias.
    }

}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]                                                                                          // É utilizado para garantir que os campos da estrutura sejam idênticos a uma estrutura em C.
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],                                   // Volatile serve para impedir que o compilador realize otimizações intensas já que ele não sabe se os dados estão na RAM ou no VGA.
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,                                                                    // Static define que a lifetime deve ser uma referência válida por toda a duração do programa.
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {                                                                            // As strings em Rust são UTF-8 e, por padrão, podem conter bytes não suportados pelo buffer de texto VGA.
                0x20..0x7e | b'\n' => self.write_byte(byte),                                     // Caractere ASCII válido ou nova linha
                _ => self.write_byte(0xfe),                                                    // Caractere ASCII inválido
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row-1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar{ ascii_character: b' ', color_code: self.color_code };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}


// Semelhante a macro print, porém imprime no buffer VGA
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

// Semelhante a macro println, porém imprime no buffer VGA
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

// Imprime a string formatada no buffer do VGA através da instancia global WRITER
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}
/*pub fn print_something() {
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe {                                                                            // O bloco unsafe é necessário, pois o compilador Rust não pode provar que os ponteiros brutos que criamos são válidos. Ao colocar o unsafe dizemos ao compilador para ignorar esses possíveis erros.
            /*
             * O novo writer aponta para o buffer VGA em 0xb8000
             * Converte um inteiro como um ponteiro mutável raw 0xb8000.
             *
             */
            &mut *(0xb8000 as *mut Buffer)
        },
    };

    writer.write_byte(b'H');
    writer.write_string("ello ");
    writer.write_string("world!\n");
    write!(&mut writer, "Numeros {}", 99).unwrap();                                                 // A chamada de write! retorna um Result que causa aviso se não for usado, logo é necessário utilizar o unwrap() para entrar em panic caso ocorra um erro.
}*/