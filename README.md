# Ferramentas 1Doc — versão Rust

O [Ferramentas 1Doc](https://github.com/tavinho9/ferramentas-1doc) reescrito
em Rust: as quatro abas (*Processos do dia*, *Busca*, *Faltantes na
planilha*, *Verificar planilha*), o login feito pela pessoa no Chrome/Edge e
as mesmas planilhas. A versão Python continua mantida no repositório dela;
esta é uma segunda implementação, com o mesmo comportamento, e pode ser
instalada junto.

## O que é igual e o que muda

- **Regras, colunas e textos:** os mesmos. Cada módulo Rust espelha um
  módulo Python (tabela abaixo). Onde `faltantes.py` e `verificar_planilha.py`
  têm código idêntico, ele fica em `verificadores.rs`.
- **Pasta de dados:** a mesma (`%LOCALAPPDATA%\Ferramentas1Doc`), com os
  mesmos arquivos e formatos: `sessao.json`, `config.json`,
  `hash_cache_1doc.json`, `1doc_cache.json` e o perfil do navegador. As duas
  versões compartilham a sessão e os caches.
- **Instalação:** um executável (`Ferramentas1Doc.exe`), em
  `%LOCALAPPDATA%\Programs\Ferramentas 1Doc Rust`, com o atalho *Ferramentas
  1Doc (Rust)*. Não substitui a versão Python. Não há programa de linha de
  comando: as ferramentas são usadas pela janela.
  O runtime do Visual C++ vai embutido no executável (`crt-static`, em
  `.cargo/config.toml`), então não é preciso instalar o Visual C++
  Redistributable; o CI confere que o `.exe` não importa `vcruntime140.dll`.
- **Janela:** egui em vez de Tk, com o mesmo arranjo. O desenho usa wgpu
  (DirectX 12, com o WARP em software onde não há placa de vídeo) ou, sem
  adaptador, OpenGL. `FERRAMENTAS1DOC_DESENHO=glow` ou `wgpu` força um deles.
- **Planilhas novas** (*Processos do dia*, *Busca*, *Faltantes*) são gravadas
  com rust_xlsxwriter. A planilha do *Verificar*, que parte da original e
  mantém a formatação, é editada com umya-spreadsheet. As larguras de coluna
  ficam ~0,7 maiores no arquivo: o rust_xlsxwriter grava a largura como o
  próprio Excel (com a margem interna), e a aparência é a mesma.

Correspondência com o repositório Python:

| Rust (`src/`) | Python (`ferramentas1doc/`) |
|---|---|
| `sessao.rs`, `navegador.rs` | `sessao.py`, `navegador.py` |
| `processos_do_dia.rs` | `processos_do_dia.py` |
| `busca/` (`args`, `parse`, `registro`, `saida`) | `busca.py` |
| `busca_argv.rs` | `busca_argv.py` |
| `faltantes.rs` | `faltantes.py` |
| `verificar_planilha.rs` | `verificar_planilha.py` |
| `verificadores.rs` | o que os dois acima têm de idêntico |
| `config.rs`, `caminhos.rs`, `saida.rs` | `config.py`, `caminhos.py`, `gui/log.py` |
| `gui/` (`janela`, `abas`, `calendario`) | `gui/` |
| `autoteste.rs`, `main.rs` | `app.py` |

## Compilar e rodar

Precisa do Rust estável (<https://rustup.rs>).

```bash
cargo run --release
```

## Testes

```bash
cargo test
```

Os testes usam um 1Doc de mentira num servidor HTTP local, com o mesmo HTML
do `tests/fake1doc.py` da versão Python. Os do navegador de verdade rodam com
`AUTOTESTE_NAVEGADOR_REAL=1` (Chrome/Edge instalado) ou
`FERRAMENTAS1DOC_CHROME=/caminho/do/chrome`.

Uma mudança nas regras da versão Python precisa ser trazida para cá à mão
(e o contrário também): os dois repositórios não se comparam
automaticamente.

## Gerar o instalador

O GitHub Actions (`.github/workflows/build-windows.yml`) roda os testes no
Linux e no Windows (com o Chrome real), compila, faz o autoteste do `.exe`,
gera o instalador, instala, testa o instalado e desinstala. O Setup.exe fica
como artefato da execução.

Para publicar uma versão:

1. Mude `version` em `Cargo.toml` e descreva a versão em `NOTAS_DA_VERSAO.md`.
2. No GitHub, crie a Release com a tag `vX.Y.Z` (a mesma versão) a partir da
   `main`.
3. Em poucos minutos o workflow anexa `Ferramentas1Doc-Rust-Setup-X.Y.Z.exe`
   à Release.

Num PC Windows com Rust, `build\build.bat` faz o mesmo localmente e gera
`dist\Ferramentas1Doc-Rust-Setup-<versão>.exe`.
