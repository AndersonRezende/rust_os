# Sistema operacional RUST_OS

## Simples projeto de kernel utilizando a linguagem Rust.

## Funcionamento
### Processo de inicialização
<ul>
<li>BIOS - Basic Input/Output System (Legacy)</li>
<li>UEFI - Unified Extensible Firmware Interface</li>
</ul>

### Processo de boot
Ao iniciar o computador, a CPU é colocada no modo de compatibilidade 16 bits, 
também chamado de modo real, com isso, bootloaders antigos (BIOS) podem ser carregados.
O processo inicial consistem em carregar o BIOS de alguma memória flash localizada na 
placa mãe. O BIOS executa rotinas de teste (POST - Power On Self Test) e inicialização
de hardware, então ele procura por discos inicializáveis. Se ele encontrar um disco
inicializável, o controle é transferido para seu bootloader, que é uma porção de 512
bytes de código executável localizado no primeiro setor e na primeira trilha do disco.
Normalmente, os bootloaders são maiores do que 512 bytes, então é comum os dividir em um
pequeno estágio que se encaixa nos primeiros 512 bytes e um segundo estágio que é carregado
em sequência ao primeiro estágio.

O bootloader tem que determinar a localização da imagem do kernel no disco e carregá-la 
na memória. Ele também precisa alterar a CPU do modo real de 16 bits para o modo protegido
de 32 bits e, em seguida, para o modo longo de 64 bits, onde os registradores de  bits 
e a memória principal completa estão disponíveis. Sua terceira tarefa é consultar
certas informações (como mapa de memória) do BIOS e passá-las para o kernel do SO.

### Multiboot
Para evitar que cada sistema operacional implemente seu próprio bootloader compatível
apenas com um único SO, existe um padrão de bootloader aberto chamado de Multiboot.
Esse padrão define uma interface entre o bootloader e o sistema operacional, de modo 
que qualquer bootloader compatível com Multiboot pode carregar qualquer sistema 
operacional compatível com Multiboot.

Para tornar o kernel compatível com Multiboot, é preciso apenas inserir um cabeçalho
chamado Multiboot no início do arquivo kernel. Isso torna muito fácil inicializar um SO
a partir do GRUB.


## Configuração
### Versão do Rust
Precisamos de recursos experimentais do rust, então através do gerenciador <strong>rustup</strong>
no diretório do projeto rodamos o comando
``$ rustup override set nightly`` ou adicionar um arquivo ``rust-toolchain`` com ``nightly`` como 
conteúdo.
Com isso, agora podemos habilitar a macro ``asm!"`` e ``#![feature(asm)]``.

### Especificação de Alvo
O cargo suporta diferentes sistemas de destino por meio do parâmetro <strong>--target</strong>
O destino é descrito por um chamado target triple, que descreve a arquitetura da CPU, o fornecedor,
o SO e o ABI.

Para o nosso sistema de destino, no entanto, precisamos de alguns parâmetros de confiugrações
especiais (por exemplo, nenhum SO subjacente), então nenhum dos triplos de destino existentes 
se encaixa. Felizmente, o Rust nos permite definir nosso próprio destino por meio de um arquivo
JSON.

<ul>
<li>"llvm-target" e "os: "none" => porque serão executados em bare metal.</li>
<li>"linker-flavor": "ld.lld", "linker": "rust-lld" => utilizaremos o vinculador LLD multiplataforma
que é fornecido com o Rust para vincular o kernel.
</li>
<li>"panic-strategy" : "abort" => semelhante ao que faz a configuração no cargo (podendo ser removida),
especifica que o alvo não suporta desenrolamento de pilha em panic, logo o programa
deverá ser abortado imediatamente.
</li>
<li>"disable-redzone": "true" => precisamos lidar com interrupções em algum momento, para
fazer isso com segurança temos que desabilitar otimização de ponteiro de pilha, pois 
causaria corrupção de pilha.
</li>
<li>"features": "-mmx,-sse,+soft-float" => Features habilita/desabilita recursos de destino.
Desabilitamos o mmx e sse prefixando com sinal de "-" e habilitamos o soft-float com sinal de "+".
Os recursos mmxe ssedeterminam o suporte para instruções Single Instruction Multiple Data (SIMD), 
que muitas vezes podem acelerar programas significativamente.
</li>
</ul>

### Build-std-option
A biblioteca principal é distribuida juinto com o compilador Rust com uma biblioteca
pré-compilada. Etnão ela é válida apenas para ambientes de target triple de hosts suportados,
mas não nosso alvo personalizado. 

A build-std-option permite recompilar a biblioteca padrão sob demanda. Este é um dos recursos
instáveis disponíveis na versão nightly.

Para utilizar o recurso, é necessário criar um arquivo de configuração do cargo local localizado
em "./cargo/config.toml", sendo:
<ul>
<li>build-std = ["core", "compiler_builtins"] => para dizer que deve recompilar as bibliotecas
core e compiler_builtins, sendo esta uma dependência do core.
</li>
</ul>

### Intrínsecos relacionados à memória
O compilador Rust assume que um certo conjunto de funções internas está disponível para todos 
os sistemas. A maioria dessas funções é fornecida pelo crate compiler_builtins que acabamos de
recompilar. No entante, há algumas funções relacionadas à memória nesse crate que nào são 
habilitadas por padrão porque são normalmente fornecidas pela biblioteca em C no sistema.
Essas funções incluem memset, que define todos os bytes em um bloco de memória para um 
determinado valor, memcpy, que cópia um bloco de memória para outro, e memcmp, que compara dois
blocos de memória.

Como não podemos víncular à biblioteca C do SO, precisamos de uma maneira alternativa de 
fornecer essas funções ao compilador. Uma abordagem poderia ser implementar nossas próprias 
funções e aplicar o "#[no_mangle]" para evitar renomeação. Porém, devido à alta possibilidade
de comportamentos indefinidos, é mais interessante reutilizar implementações já existentes.

A crate compiler_builtins já contém implementações para as funções necessárias, elas são 
desabilitadas por padrão para não conflitar com as implementações do C. Para habilitar, 
definimos:
<ul>
<li>build-std-features = ["compiler-builtins-mem"]</li>
<li>build-std = ["core", "compiler_builtins"]</li>
</ul>

Para evitar passar o --target, podemos definir no arquivo .cargo/config.toml o seguinte 
trecho:
```
[build]
target = "x86_64-blog_os.json"
```

## Impressão na tela
A maneira mais fácil para imprimir um texto na tela neste estágio é o <strong>VGA text mode</strong>. 
É uma área na memória especial mapeada para o hardware VGA que contém o conteúdo exibido na tela.
Normalmente consistem em 25 linhas, cada uma contendo 80 células de caracteres. Cada célula de caractere
exibe um caractere ASCII com algumas cores de texto e de fundo.

### Executando o kernel
Para transformar o kernel compilado em uma imagem de disco inicializável, precisamos transformar
o kernel compilado em uma imagem de disco inicializável, vinculando-o a um bootloader.

### Criando uma Bootimage
Para transformar o kernel compilado em uma imagem de disco inicializável, precisamos vinculá-lo a 
um bootloader.

Para poupar o trabalho de escrever o próprio bootloader, usamos a crate "bootloader". Este crate
implementa um bootloader básico de BIOS sem dependências de C, apenas Rust e assembly.
Para usá-lo para inicializar o kernel, precisamos adicionar a dependência:
```
[dependencies]
bootloader = "0.9"
```
Será necessário também vincular o bootloader com o kernel após a compilação. Para isso utilizaremos
a ferramenta "bootimage" que primeiro compila o kernel e o bootloader e depois cria uma imagem
inicializável. A ferramenta pode ser instalada com o seguinte comando:
``$ cargo install bootimage``.

Para executar o "bootimage" e construir o bootloader, é necessário ter o componente rustc 
"llvm-tools-preview" que pode ser instalado através do seguinte comando:
``$ rustup component add llvm-tools-preview``.

Após intalar o "bootimage" e adicionar o componente "llvm-tools-preview", você pode criar o disco
inicializável através do comando: ``$ cargo bootimage``.
A ferramenta compila o kernel usando o cargo build, então pega automaticamente quaisquer alterações feitas.
Depois, ela compila o bootloader. Por último, ela combina o bootloader com o kernel em uma imagem de disco
inicializável.

Após executar o comando, você deve ver a imagem de disco inicializável chamada bootimage-rust-os.bin na pasta
"target/x86-64-rust-os/debug". Você pode inicializá-la em uma máquina virtual, como o qemu, ou
gravar em uma unidade USB.

O bootimage compila o kernel em um formato ELF (executable and linkable format), depois
compila a dependência do bootloader como um executável autônomo e, por último, víncula os bytes do arquivo
ELF do kernel ao bootloader.

Quando inicializado, o bootloader lê e analisa o arquivo ELF anexado. Ele então mapeia os segmentos do programa
para endereços virtuais nas tabelas de páginas, zera a seção ".bss" e configura uma pilha. Finalmente, ele
lê o endereço do ponto de entrada (_start) e pula para ele.

### Inicializando no QEMU
Para executar com o qemu, basta executar o seguinte comando:
``$ qemu-system-x86_64 -drive format=raw,file=target/x86_64-rust_os/debug/bootimage-rust_os.bin``

### Máquina Real
Para gravá-lo em um pen-drive e inicializá-lo em uma máquina real basta executar o seguinte comando
adaptando para o caso específico, onde sdX deve ser a unidade do pen-drive:
``dd if=target/x86_64-blog_os/debug/bootimage-blog_os.bin of=/dev/sdX && sync``

### Usando cargo run
Para facilitar a execução do kernel no qemu, podemos definir a chave runner de configuração para o cargo:
```
[target.'cfg(target_os = "none")']
runner = "bootimage runner"
```

## Modo de texto VGA
É o modo mais simples de exibir texto em tela.
### Buffer de texto VGA
Para escrever um caractere na tela no modo VGA é necessário escrever no buffer de texto do hardware VGA.
O buffer de texto VGA é uma matriz bidimensional com 80 colunas e 25 linhas que é renderizado diretamente na tela.
Cada entrada na matriz descreve um único caractere através do seguinte formato:
<table>
<thead>
<tr>
<th>Bit(s)</th>
<th>Valor</th>
</tr>
</thead>
<tbody>
<tr>
<th>0-7</th>
<td>Código ASCII do caractere</td>
<tr>
<td>8-11</td>
<td>Cor do caractere</td>
</tr>
<tr>
<td>12-14</td>
<td>Cor de fundo</td>
</tr>
<tr>
<td>15</td>
<td>Piscar</td>
</tr>
</tbody>
</table>

O buffer de texto é acessível através de Memory-mapped I/O (MMIO) no endereço 0xb8000. Isso significa que as leituras e 
gravações nesse endereço não acessam a memória RAM, mas sim o buffer no hardware do VGA.


## Testando
### Testando em Rust
O Rust possui uma framework interna capaz de realizar testes unitários sem a necessidade de configurar nada. É apenas 
necessário criar uma função que verifique se assertations são válidas. Essas funções possuem o atributo #[test] na 
declaração da função. Com isso, o comando ```$ cargo test``` irá automaticamente realizar os testes nessas funções.

Porém, como estamos utilizando um ambiente <strong>no_std</strong>, é necessário realizar alguns processos a mais para
configurar uma framework de testes. Isso se dá por conta que a framework de testes do Rust utiliza dependências da 
biblioteca padrão (std).

### Framework de teste customizada
O Rust suporta a substituição da framework padrão através de ```custom_test_frameworks```. Essa feature não requer 
bibliotecas externas, logo funciona em ambientes ```no_std```. Ela funciona coletando todas as funções com a anotação
```#[test_case]``` e então as invoca por meio de uma função runner ``#![test_runner(crate::test_runner)]`` com a lista 
de testes a serem executados como argumento.

A implementação da framework de testes customizada se dá por meio das seguintes anotações: 
<ul>
<li>"#![feature(custom_test_frameworks)]": implementa a própria framework de testes</li>
<li>"#![test_runner(crate::test_runner)]": invoca a própria função executora test_runner</li>
</ul>
A função executora recebe uma lista de argumentos que são os testes e executa cada um deles.

A desvantagem em comparação com o framework de teste padrão é que recursos avançados, como ``should_panic`` não estão 
disponíveis. Em vez disso, será necessário implementar esses recursos por conta própria.

### Portas I/O
Para testar com apoio do qemu é necessário configurar uma comunicação entre o guest e o host. Essa comunicação pode ser 
feita por meio de memória mapeada de I/O ou portas mapeadas de I/O. Já foi utilizado o mapeamento de memória com o VGA 
buffer através do endereço 0xb8000. Esse endereço não é mapeado para a RAM, mas sim para a memória do dispositivo VGA.

Em contraste, a comunicação por portas mapeadas I/O utiliza uma "trilha" de comunicação separada. Essas trilhas se 
conectam a diferentes periféricos que possuem uma ou mais portas acessadas por meio de seus números. A comunicação com 
esses dispositivos é feito por meio das instruções assembly ```in``` e ```out```, onde cada um leva um número de porta e
dados.

Essa comunicação é necessária para enviar um comando para o qemu para que o mesmo seja encerrado após o término dos testes.

### Imprimindo no console
Para ver o resultado dos testes no console é necessário enviar dados entre o guest e o host. Uma maneira de realizar essa
comunicação é por meio de portas seriais.
Existe uma interface chamada de UART. Essa interface utiliza implementações de portas I/O. A primeira porta padrão serial 
é a de número 0x3F8.

Utilizamos o crate "uart_16550" para inicializar a UART e enviar dados através da porta serial. No arquivo "src/serial.rs"
inicializamos a crate e definimos macros para facilitar a utilização da porta serial.

### Teste de integração
A convenção para definição de testes integrados em Rust é colocá-los em um diretório "tests" na raiz do projeto. Os testes
desses diretórios são identificados automaticamente.

Cada teste de integração deve ser autoexecutável e é separado do "main.rs". Isso significa que precisamos definir uma 
função de ponto de entrada para cada um. Por serem executáveis separados, precisamos definir alguns atributos:
<ul>
<li>#![no_std]: não utiliza a biblioteca padrão.</li>
<li>#![no_main]: não utilizamos o ponto de entrada padrão.</li>
<li>#![feature(custom_test_frameworks)]: informamos que é uma framework de teste customizada. Funciona coletando os #[test_case]</li>
<li>#![test_runner(crate::test_runner)]: função executora que receberá a lista de funções de testes a serem executadas.</li>
<li>#![reexport_test_harness_main = "test_main"]: definimos o nome da função de entrada.</li>
</ul>


## Exceções da CPU
Exceções de CPU podem ocorrer em várias situações como acessar um endereço de memória inválido ou divisão por zero. Para 
reagir a cada uma dessas opções temos que configurar uma tabela de descritores de interrupção que forneça funções de 
manipulador.

### Visão geral
Uma exceção sinaliza que algo está errado na instrução atual. Quando ocorre uma exceção, a CPU interrompe o seu trabalho
atual e chama uma função manipuladora específica para o tipo de exceção lançada.
No x86 há cerca de 20 diferentes tipos de exceção de CPU.

### Tabela de descritores de interrupção
Para capturar e manipular exceções temos que configurar uma IDT (interrupt descriptor table). Nessa tabela podemos 
especificar uma função de manipulador para cada exceção da CPU.

Quando ocorre uma exceção, a CPU faz basicamente o seguinte:
<ol>
<li>Empilhar alguns registradores na pilha, incluindo o ponteiro de instrução e o registrador RFLAGS.</li>
<li>Ler a entrada correspondente ao IDT, por exemplo, a CPU lê a entrada 14 quando ocorre um page fault.</li>
<li>Verifica se a entrada está presente, se não estiver registra uma dupla falta.</li>
<li>Desabilita interrupções de hardware se a entrada for uma porta de interrupção.</li>
<li>Carrega o seletor GDT especificado no CS (code segment).</li>
<li>Ir para a função manipuladora especificada.</li>
</ol>

### Convenção para chamadas de interrupção
Exceções se assemelham a funções, a CPU salta para um endereço que será executado e retorna posteriormente a execução.
No entanto, há uma grande diferença entre exceções e funções, uma chamada de função é invocada voluntariamente por uma 
instrução ``call`` enquanto uma exceção pode ocorrer em qualquer instrução.