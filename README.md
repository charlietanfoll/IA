# 🗺️ Broken Ways: Além das Fronteiras

Projeto prático desenvolvido para a disciplina de **Inteligência Artificial (IA)**.
**Integrantes:**
- Charlie Tanfoll Pereira Lobo (Nº USP: 16827968)

---

## 📌 Visão Geral do Projeto

O **Broken Ways** é um motor de busca e roteamento de alta performance construído em **Rust**, projetado para planejar trajetos sobre a malha viária real do município de Ribeirão Preto/SP (composta por **19.036 cruzamentos** e **58.492 trechos viários direcionados**).

O sistema implementa e compara dois algoritmos clássicos de busca informada:
1. **Desafio 1 — Rota Ótima ($A^*$ / A-Star):**
   - Função de avaliação balanceada: $f(n) = 0.5 \cdot g(n) + 0.5 \cdot h(n)$.
   - Heurística admissível baseada na distância euclidiana métrica em projeção plana.
   - Garante a menor distância real entre quaisquer dois nós navegáveis da cidade.
2. **Desafio 2 — Rota Expressa (GBFS / *Greedy Best-First Search*):**
   - Função de avaliação puramente gulosa: $f(n) = h(n)$.
   - Prioriza a redução extrema no número de expansões e no tempo de CPU, orientando a busca diretamente ao destino com tolerância de até +15% de distância em relação ao trajeto ótimo.

### 🌟 Destaques Técnicos e Arquitetura
- **Grafo CSR (*Compressed Sparse Row*) em Arquitetura SoA (*Structure of Arrays*):** Representação compacta em memória contígua (`offsets: Box<[u32]>`, `arestas_destino: Box<[NodeId]>` e `arestas_distancia: Box<[f64]>`), eliminando fragmentação e maximizando o aproveitamento do cache L1/L2.
- **Tipagem Compacta (`NodeId = u16`):** Reduz o consumo de memória em 50% frente ao `u32` e 75% frente ao `u64`.
- **Serialização Binária Rápida (`rp.graph`):** Na primeira execução, os arquivos tabulares em `data/` são compilados para um arquivo binário com *Magic Bytes* (`RPGR`), controle de versão e sentinela (`0xFF`). Nas execuções seguintes, o grafo é carregado em milissegundos.
- **Visualizador Cartográfico Interativo Nativo:** Gera páginas HTML (`mapa_rotas.html`) usando **Leaflet.js** com múltiplos provedores de mapas (*CartoDB Positron*, *Dark Matter*, *Esri Satélite*, *OpenStreetMap* e *OpenTopoMap*), realizando a projeção inversa UTM (SIRGAS 2000 / 22S, EPSG:31982) para latitude/longitude WGS84 diretamente em Rust, sem scripts externos em Python.
- **Zero Dependências Externas (Pure `std`):** Todo o projeto utiliza exclusivamente a biblioteca padrão do Rust (`std`), garantindo compilação instantânea e portabilidade sem a necessidade de baixar dezenas de bibliotecas externas pelo crates.io.

---

## 🛠️ Pré-requisitos e Configuração das Dependências via Rustup

O projeto utiliza a edição **Rust 2024** (`edition = "2024"`), exigindo uma versão moderna do compilador (Rust 1.85 ou superior). O gerenciador recomendado oficial para instalar e manter o compilador Rust é o **rustup**.

### 1. Instalando o Rust via [https://rustup.rs/](https://rustup.rs/)

#### 🪟 No Windows:
1. Acesse o site oficial: [https://rustup.rs/](https://rustup.rs/) e baixe o instalador **`rustup-init.exe`**.
   *(Alternativamente, via terminal PowerShell, execute: `winget install --id Rustlang.Rustup`)*.
2. Execute o instalador `rustup-init.exe`.
3. Caso solicitado sobre o *Visual Studio C++ Build Tools*, certifique-se de ter os compiladores C++ instalados (fornecidos pelo *Visual Studio Installer* selecionando *"Desenvolvimento para Desktop com C++"*).
4. Pressione `1` e `Enter` para prosseguir com a instalação padrão (*default*).
5. Reinicie o terminal ou PowerShell para atualizar a variável de ambiente `PATH`.

#### 🐧 No Linux / 🍏 No macOS:
Abra o terminal e execute o script oficial:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Ao término, recarregue as variáveis de ambiente:
```bash
source "$HOME/.cargo/env"
```

---

### 2. Verificando e Atualizando a Toolchain

Abra o terminal no diretório do projeto e confirme que as ferramentas estão acessíveis:

```bash
rustc --version
cargo --version
```

Para garantir que você possui a versão com suporte à edição 2024, atualize a toolchain estável:

```bash
rustup update stable
```

> **Nota:** Como o projeto foi implementado utilizando exclusivamente a biblioteca padrão do Rust (`std`), **não há dependências externas adicionais a serem baixadas no `Cargo.toml`**. Toda a compilação ocorre localmente sem requisições de pacotes de terceiros.

---

## 🚀 Como Executar o Projeto

Navegue até a pasta raiz do projeto:

```bash
cd IA
```

### 1. Modo Menu Interativo (Recomendado)

Para executar no modo de desenvolvimento rápido:
```bash
cargo run
```

Para executar com **otimizações máximas de desempenho (Release)**:
```bash
cargo run --release
```

Ao iniciar, o programa carregará a malha viária (gerando `rp.graph` na primeira vez ou carregando o binário existente) e apresentará o menu interativo:

```text
==========================================
SISTEMA DE ROTEAMENTO - RIBEIRÃO PRETO
IA 2026 - Prof. José Augusto Baranauskas
==========================================
[1] Pesquisar Rotas (informar Origem e Destino)
[2] Rota de Demonstração (259576101 -> 259577028)
[3] Abrir Último Mapa no Navegador
[0] Sair
==========================================
Selecione uma opção: 
```

- **Opção `[1]` — Pesquisar Rotas:** Permite informar os `osmid` de origem e destino, além de configurar interativamente os pesos de balanceamento do $A^*$ ($w_g$ e $w_h$, bastando pressionar `Enter` para manter os valores padrão de $0.5$ e $0.5$).
- **Opção `[2]` — Rota de Demonstração:** Executa automaticamente a rota de teste pré-definida atravessando a malha urbana com pesos balanceados ($0.5$ / $0.5$).
- **Opção `[3]` — Abrir Último Mapa no Navegador:** Abre o arquivo `mapa_rotas.html` gerado na última consulta diretamente no seu navegador padrão.
- **Opção `[0]` — Sair:** Finaliza o programa.

Ao término de qualquer busca, o sistema pergunta interativamente:
```text
Deseja abrir o mapa no navegador? [S/n]: 
```
Basta pressionar `Enter` (ou digitar `s`) para visualizar imediatamente o mapa cartográfico.

---

### 2. Modo Linha de Comando Direto (CLI Batch)

Você pode passar a origem e o destino diretamente como argumentos de linha de comando (com pesos de balanceamento opcionais):

```bash
# Execução padrão (pesos w_g = 0.5 e w_h = 0.5)
cargo run --release -- <osmid_origem> <osmid_destino>

# Execução com pesos de balanceamento personalizados
cargo run --release -- <osmid_origem> <osmid_destino> <peso_g> <peso_h>
```

#### Exemplos práticos:
```bash
# Rota padrão com pesos equilibrados
cargo run --release -- 259576101 259577028

# Rota enfatizando a distância real acumulada (w_g = 0.8, w_h = 0.2)
cargo run --release -- 259576101 259577028 0.8 0.2
```

O programa executará ambos os algoritmos ($A^*$ e GBFS), exibirá a tabela comparativa de métricas, gerará o arquivo `mapa_rotas.html` e perguntará se deseja abri-lo no navegador.

---

## 📊 Exemplo de Saída no Terminal

```text
Carregando Grafo de Ribeirão Preto...
Grafo carregado em 3.25 ms (19036 nós, 58492 arestas).

------------------------------------------
# RESULTADO DA BUSCA - DESAFIO 1 (ROTA ÓTIMA)
------------------------------------------
# Algoritmo utilizado: A* (A-Star)
# Origem (osmid): 259576101
# Destino (osmid): 259577028
------------------------------------------
# Caminho encontrado (Qtd nós): 14 cruzamentos
# Distância da Rota: 1845.32 metros
# NÓS EXPANDIDOS: 82
# Consumo de Memória (Pico): 45.20 KB
# Tempo de CPU: 120 µs
------------------------------------------

------------------------------------------
# RESULTADO DA BUSCA - DESAFIO 2 (ROTA EXPRESSA)
------------------------------------------
# Algoritmo utilizado: GBFS (Greedy Best-First Search)
# Origem (osmid): 259576101
# Destino (osmid): 259577028
------------------------------------------
# Caminho encontrado (Qtd nós): 15 cruzamentos
# Distância da Rota: 1890.10 metros (+2.43%)
# NÓS EXPANDIDOS: 19 (-76.83%)
# Consumo de Memória (Pico): 12.80 KB (-71.68%)
# Tempo de CPU: 35 µs (3.4x mais veloz que A*)
# Tolerância do Desafio: APROVADO (+2.43% <= limite de 15%)
------------------------------------------

# Mapa interativo salvo com sucesso em: mapa_rotas.html
```

---

## 🗺️ Visualizador Cartográfico (`mapa_rotas.html`)

Ao finalizar a busca, o arquivo `mapa_rotas.html` é salvo na raiz do projeto. Ele contém:
- **Camada Roxa:** Rota Ótima encontrada pelo algoritmo $A^*$.
- **Camada Vermelha:** Rota Expressa encontrada pelo GBFS (desenhada no centro da rota ótima para permitir visualização simultânea em trechos coincidentes).
- **Marcadores de Início e Fim:** Indicadores visuais nos nós de partida e chegada.
- **Painel de Controle Flutuante:** Exibe resumo de métricas, nós expandidos, tempos e percentual de desvio.
- **Seletor de Provedores de Mapa:** Alterna instantaneamente entre:
  - *CartoDB Positron* (claro e rápido, sem bloqueios de taxa do OSM)
  - *CartoDB Voyager* (ruas e pontos de interesse)
  - *CartoDB Dark Matter* (modo escuro)
  - *Esri Satélite* (fotografia aérea em alta definição)
  - *OpenStreetMap Padrão*
  - *OpenTopoMap* (relevo topográfico)

---

## 🧪 Verificação e Testes

Para verificar a integridade da compilação e tipagem sem gerar os binários finais:
```bash
cargo check
```

Para compilar os binários otimizados de release:
```bash
cargo build --release
```
O executável final estará localizado em `target/release/IA.exe` (no Windows) ou `target/release/IA` (no Linux/macOS).
