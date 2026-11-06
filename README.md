<p align="center">
  <img src="assets/app-icon/hicolor/128x128/apps/ai.storyteller.gridcraft.png" alt="UniPlanner" width="96">
</p>

<h1 align="center">UniPlanner</h1>

<p align="center">
  A planilha da <b>Uniphar Indústria Farmacêutica</b>. Abre e salva arquivos do Excel.
</p>

<p align="center">
  <a href="https://github.com/uaidenys/uniplanner/releases/latest"><b>Baixar a versão mais recente</b></a>
</p>

---

## O que ele faz

- **Familiar:** faixa de opções, barra de fórmulas, Caixa de Nome, abas de planilhas, atalhos de
  teclado, alça de preenchimento, Formatar Células, Colar Especial e formatação condicional.
- **Compatível:** o formato principal é o .xlsx, com estilos, fórmulas, tabelas, validação,
  comentários, gráficos, imagens, minigráficos, tabelas dinâmicas e configurações de impressão.
  Também abre e salva CSV e TSV, abre .xls antigos (97-2003) e importa dados de .xlsb e .ods.
- **Fórmulas:** mais de 500 funções, matrizes dinâmicas, LET, LAMBDA, referências estruturadas,
  classificação, filtro automático, Preenchimento Relâmpago, subtotais e agrupamento.
- **Rápido:** recalcula só o que mudou e desenha só o que está na tela.
- **Atualização automática:** o UniPlanner procura versões novas sozinho, e o Uni Launcher também.
- **Privado:** tudo funciona no computador, sem assinatura, sem conta e sem telemetria.

## Como instalar

- **Pelo Uni Launcher (recomendado):** instale o
  [Uni Launcher](https://github.com/uaidenys/uni-launcher/releases/latest) e clique em instalar no
  UniPlanner.
- **Direto:** na [página da versão mais recente](https://github.com/uaidenys/uniplanner/releases/latest),
  em **Assets**, baixe o arquivo `-windows-x64.msi` e abra. Se o Windows mostrar "O Windows protegeu
  o computador", clique em **Mais informações › Executar assim mesmo**. O instalador ainda não tem
  assinatura digital de código. `SHA256SUMS.txt` traz os códigos de conferência dos arquivos.

Requisitos: Windows 10 ou 11, 64 bits.

## Para desenvolvedores

O UniPlanner é feito em Rust, com [egui](https://github.com/emilk/egui).

```bash
cargo run --release -p gridcraft
```

Para gerar o instalador do Windows (precisa do [WiX v5](https://wixtoolset.org)):

```powershell
powershell -ExecutionPolicy Bypass -File packaging/windows/package.ps1 -Arch x64
```

Os arquivos saem em `dist/release/`; o instalador coloca o app como `uniplanner.exe`. O pacote Rust
mantém o nome `gridcraft` do projeto original. Para desligar a atualização automática, defina a
variável de ambiente `UNIPLANNER_NO_UPDATE`.

A ferramenta de linha de comando `gridcraft-cli` e o servidor MCP permitem automatizar a planilha;
a documentação técnica está em [docs/](docs/).

## Licença e créditos

- **Código:** MIT ou Apache-2.0, à sua escolha ([LICENSE-MIT](LICENSE-MIT),
  [LICENSE-APACHE](LICENSE-APACHE)). Avisos obrigatórios em [NOTICE](NOTICE).
- **Marca:** os nomes Uni e Uniphar, o ícone e as imagens da animação de abertura pertencem à
  Uniphar Indústria Farmacêutica e **não** estão cobertos pela licença do código. Termos em
  [docs/brand/LICENSE-brand.txt](docs/brand/LICENSE-brand.txt). Quem fizer uma versão modificada
  precisa usar nome e identidade visual próprios.
- **Materiais de terceiros:** fontes e outros arquivos com suas licenças, listados em
  [ATTRIBUTION.md](ATTRIBUTION.md).

O UniPlanner é uma versão modificada do [GridCraft](https://github.com/storytold/gridcraft), criado
pela equipe ArtCraft e colaboradores (MIT OR Apache-2.0).

Desenvolvido por Denys Rodrigues de Oliveira para a Uniphar Indústria Farmacêutica ·
[uniphar.com.br](https://uniphar.com.br/)
