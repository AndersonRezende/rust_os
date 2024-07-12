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