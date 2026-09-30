# Trabalho 1: Sistema de Roteamento em Malha Viária Real (Ribeirão Preto)

**Disciplina:** Inteligência Artificial (5954013)  
**Departamento:** Departamento de Computação e Matemática (DCM) – FFCLRP / USP  
**Professor:** Prof. Dr. José Augusto Baranauskas  
**Semestre:** 2º Semestre / 2026  

---

## 1. Visão Geral do Projeto

O objetivo do trabalho é desenvolver um sistema de roteamento capaz de encontrar caminhos válidos em uma malha viária real, utilizando dados reais da cidade de Ribeirão Preto extraídos do OpenStreetMap (OSM).

O mapa foi processado e simplificado em dois arquivos tabulares (`.csv`) que representam o espaço de estados do problema como um **grafo não orientado**:

1. **`nos.csv` (Vértices / Cruzamentos):**
   - `osmid`: Identificador único global do cruzamento (inteiro de 64 bits).
   - `latitude`: Coordenada geográfica em graus decimais (-90° a +90°).
   - `longitude`: Coordenada geográfica em graus decimais (-180° a +180°).

2. **`arestas.csv` (Arestas / Conexões Viárias):**
   - `origem`: `osmid` do nó inicial.
   - `destino`: `osmid` do nó final.
   - `distancia`: Comprimento do trecho em **metros** (custo da aresta).
   - `nome_rua`: Nome da via pública correspondente (pode ser vazio).
   - *Nota:* O grafo é **não orientado**, logo o trânsito entre cruzamentos conectados é bidirecional.

3. **Visualizador Interativo (`verRota.py`):**
   - Script em Python que utiliza a biblioteca `folium` para ler o caminho de cruzamentos gerado pelo sistema e plotar a rota sobre o mapa interativo, exportando para um arquivo `.html` navegável.

---

## 2. Especificação do Sistema

O sistema deve receber:
- **`osmid` de Origem ($A$)**
- **`osmid` de Destino ($B$)**

O grupo possui total liberdade para escolher a linguagem de programação e quais algoritmos de busca utilizar, desde que atendam com rigor aos dois desafios propostos e às definições formais de expansão e cálculo de custo.

### 2.1 Desafio 1: A Rota Ótima
- **Objetivo:** Garantir a rota de **distância mínima absoluta** entre a origem $A$ e o destino $B$.
- **Filtro de Otimalidade (Passo 1):** O algoritmo deve, obrigatoriamente, encontrar o menor caminho conhecido (ótimo). Se encontrar um caminho subótimo, recebe nota 0 nesta etapa.
- **Critério de Desempate / Eficiência (Passo 2):** Menor número total de **Nós Expandidos**.

### 2.2 Desafio 2: A Rota Expressa Subótima
- **Objetivo:** **Velocidade extrema de busca** (minimizar ao máximo o número de nós expandidos), abrindo mão da precisão exata.
- **Filtro de Tolerância (Passo 1):** A distância encontrada pode ser no máximo **15% maior** que a distância ótima ($\text{distância} \le 1{,}15 \times \text{distância}_{\text{ótima}}$). Se ultrapassar $+15\%$, recebe nota 0 nesta etapa.
- **Critério de Desempate / Eficiência Extrema (Passo 2):** Menor soma de nós expandidos entre os grupos que respeitaram a margem de tolerância.

---

## 3. Definição Formal de Métricas

### 3.1 Distância Total da Rota
Soma exata do custo das arestas (em metros) que compõem o caminho sequencial da origem até o destino:
$$\text{Distância} = \sum_{e \in \text{caminho}} \text{distancia}(e)$$

### 3.2 Contagem de Nós Expandidos (Definição Oficial)
> **Definição:** Um nó é considerado **expandido** no exato momento em que ele é **retirado da Fronteira** (Fila, Pilha ou Fila de Prioridade) para que seus vizinhos (filhos) sejam descobertos e avaliados.

**Regras estritas:**
- Olhar vizinhos do nó e inseri-los na fronteira **NÃO** conta como expansão.
- A expansão ocorre **exclusivamente** quando o nó se torna o *nó atual* da iteração ao sair da fronteira.
- Se um nó for retirado da fronteira e for um **beco sem saída** (sem novos filhos válidos), ele **é contabilizado como nó expandido** (a CPU gastou processamento ao avaliá-lo).
- No momento em que o **nó objetivo (destino) é retirado da fronteira**, o teste de objetivo é satisfeito e a busca é finalizada imediatamente. O contador **NÃO é incrementado** nesta etapa, pois o destino é apenas testado, não expandido.

---

## 4. Formato Exato da Saída no Terminal

A saída do programa deve seguir rigorosamente o template abaixo:

```text
# ------------------------------------------
# RESULTADO DA BUSCA - DESAFIO 1 (ROTA ÓTIMA)
# ------------------------------------------
# Algoritmo utilizado: [Nome do Algoritmo]
# Origem (osmid): 123456789
# Destino (osmid): 987654321
# ------------------------------------------
# Caminho encontrado (Qtd nós): 45 cruzamentos
# Distância da Rota: 4325.50 metros
# NÓS EXPANDIDOS: 1250
# ------------------------------------------
# ------------------------------------------
# RESULTADO DA BUSCA - DESAFIO 2 (ROTA EXPRESSA)
# ------------------------------------------
# Algoritmo utilizado: [Nome do Algoritmo]
# Origem (osmid): 123456789
# Destino (osmid): 987654321
# ------------------------------------------
# Caminho encontrado (Qtd nós): 42 cruzamentos
# Distância da Rota: 4850.20 metros (+12.13%)
# NÓS EXPANDIDOS: 310 (-75.20%)
# ------------------------------------------
```

*Obs.: Em caso de rotas inexistentes (nós desconexos) ou IDs inválidos, o programa deve exibir uma mensagem de erro amigável e informativa.*

---

## 5. Critérios de Avaliação e Pontuação

A avaliação é composta por duas frentes, totalizando **10,0 Pontos**:

### 5.1 Avaliação Base (8,0 Pontos)
1. **[2,5 pontos] Correção e Funcionalidade da Busca:**
   - O programa compila/executa sem erros?
   - Consegue carregar a base de dados (.csv) corretamente?
   - Encontra caminhos válidos e viáveis na malha viária real?
2. **[2,0 pontos] Clareza da Implementação e Modularidade:**
   - Código limpo, bem estruturado e com funções de responsabilidade única.
   - Nomes de variáveis expressivos e ausência de código espaguete.
   - Estrutura de dados adequada para a Fronteira (ex.: Fila de Prioridade / Binary Heap).
3. **[2,0 pontos] Comentários e Documentação Intracódigo:**
   - Explicação detalhada dos trechos complexos.
   - Ponto de incremento formal dos **Nós Expandidos** documentado com clareza.
   - Raciocínio matemático e justificativa da função heurística empregada.
4. **[1,5 ponto] Formatação das Saídas e Apresentação:**
   - Impressão terminal rigorosamente idêntica ao template oficial.
   - Tratamento amigável e robusto de erros (grafo desconexo, nós inexistentes).

### 5.2 Competição / Otimização (2,0 Pontos)
Avaliado durante a apresentação ao vivo com 3 rotas de teste fornecidas pelo professor:
- **Desafio 1 (1,0 Ponto):** Rota Ótima (0 pontos se subótimo). Desempate por nós expandidos pela fórmula proporcional da sala:
  $$\text{Nota} = \frac{\text{Pior Expansão} - \text{Expansão do Grupo}}{\text{Pior Expansão} - \text{Melhor Expansão}}$$
- **Desafio 2 (1,0 Ponto):** Rota Expressa (0 pontos se ultrapassar 15% da rota ótima). Desempate por eficiência extrema com a mesma fórmula de proporcionalidade.

---

## 6. Diretrizes de Apresentação e Entrega

### 6.1 Regras de Submissão
- Grupos de até 3 alunos (sem exceções);
- Nomes e números USP de todos os integrantes no cabeçalho de todos os arquivos-fonte;
- Entrega de arquivo único `.zip` no e-Disciplinas contendo código-fonte, dados e a apresentação de slides.

### 6.2 Estrutura da Apresentação (~15 minutos)
- **Slide 1 - Capa:** Nome da disciplina, título do trabalho e dados dos integrantes.
- **Slide 2 - A Escolha dos Algoritmos:** Algoritmos adotados, modelagem da heurística, solução para coordenadas polares $\times$ metros, expectativas teóricas de desempenho.
- **Slide 3 - Estruturas de Dados e Implementação:** Representação do grafo na memória (listas de adjacência, tabelas hash), estrutura da fronteira e garantia da contagem de nós expandidos.
- **Slide 4 - Desafios Técnicos e Soluções:** Gargalos enfrentados (grafos desconexos, memória, becos sem saída, tempo de CPU) e otimizações implementadas.
- **Slide 5 - Resultados Iniciais e Validação Visual:** Exemplo de rota de teste com print da saída no terminal e visualização no mapa interativo via `verRota.py`.
- **Slide 6 - Os Desafios (Dinâmica ao Vivo):** Execução das 3 rotas fornecidas pelo professor ao vivo, registro dos dados na planilha da turma e plotagem visual com cores distintas.
- **Slide 7 - Conclusão e Trade-offs:** Análise crítica do trade-off entre distância percorrida vs. nós poupados, aplicabilidade em produtos comerciais reais (Waze, Uber, iFood) e melhorias futuras.

---

## 7. Checklist de Objetivos a Serem Implementados

### Fase 1: Estruturas de Dados e Modelagem do Grafo
- [ ] Definir estruturas de dados para os vértices (`Node`: `osmid`, `latitude`, `longitude`).
- [ ] Definir estruturas de dados para as arestas (`Edge`: `origem`, `destino`, `distancia`, `nome_rua`).
- [ ] Implementar a representação do grafo em memória (Grafo não orientado com lista de adjacência eficiente, ex.: `HashMap<u64, Vec<Edge>>`).
- [ ] Implementar parser robusto e rápido para `nos.csv`.
- [ ] Implementar parser robusto e rápido para `arestas.csv`, garantindo inserção bidirecional das arestas.

### Fase 2: Módulo de Cálculo Geográfico e Heurísticas
- [ ] Implementar fórmula de cálculo de distância geodésica em metros entre coordenadas $(\text{lat}_1, \text{lon}_1)$ e $(\text{lat}_2, \text{lon}_2)$ (Fórmula de Haversine ou distância euclidiana projetada com fator de escala).
- [ ] Garantir que a heurística $h(n)$ seja **admissível** ($h(n) \le h^*(n)$) e **consistente** para garantir a otimalidade no Desafio 1.
- [ ] Documentar no código o embasamento matemático e as conversões de graus para radianos / metros.

### Fase 3: Algoritmo para o Desafio 1 (Rota Ótima)
- [ ] Implementar algoritmo de busca ótima (ex.: A* com heurística admissível ou Dijkstra com fila de prioridade mínima).
- [ ] Implementar estrutura de dados de Fronteira eficiente (`BinaryHeap` / Min-Heap).
- [ ] Garantir verificação de nós já visitados com menor custo (fechados / visited set).
- [ ] Implementar a contagem estrita de **Nós Expandidos** de acordo com a definição formal:
  - Incremento apenas quando o nó é desempilhado/removido da fronteira para gerar sucessores.
  - Becos sem saída contabilizados.
  - Interrupção imediata ao retirar o nó destino (sem incrementar a expansão do destino).
- [ ] Reconstruir a sequência exata de nós (`osmid`) do caminho percorrido da origem até o destino.
- [ ] Calcular a distância total da rota percorrida em metros.

### Fase 4: Algoritmo para o Desafio 2 (Rota Expressa Subótima)
- [ ] Selecionar e implementar estratégia para velocidade extrema:
  - Opções: Weighted A* ($w > 1.0$), Greedy Best-First Search com fallback, Busca Bidirecional, etc.
- [ ] Calibrar os parâmetros para assegurar que o custo final jamais exceda o teto de **+15% da distância ótima**.
- [ ] Assegurar a minimização agressiva dos nós expandidos em relação ao Desafio 1.
- [ ] Calcular as porcentagens comparativas em relação ao Desafio 1:
  - Variação percentual de distância: $\frac{\text{dist}_2 - \text{dist}_1}{\text{dist}_1} \times 100\%$
  - Redução percentual de expansões: $\frac{\text{exp}_2 - \text{exp}_1}{\text{exp}_1} \times 100\%$

### Fase 5: Interface de Linha de Comando (CLI) e Saída Formatada
- [ ] Permitir a passagem de `origem` e `destino` via argumentos de terminal ou prompt interativo.
- [ ] Validar a existência dos IDs de entrada na base de dados viária.
- [ ] Tratar casos de grafos desconexos ou ausência de rota com mensagem de erro clara e amigável.
- [ ] Formatar o resultado do terminal rigorosamente idêntico ao template especificado nas instruções.

### Fase 6: Integração com o Visualizador de Rotas (`verRota.py`)
- [ ] Exportar a lista sequencial de cruzamentos para arquivo ou compatibilizar a saída diretamente com o script `verRota.py`.
- [ ] Ajustar o script para suportar plotagem de ambas as rotas (Desafio 1 e Desafio 2) com cores contrastantes (ex.: Azul para Ótima, Vermelho para Expressa).
- [ ] Validar a geração e abertura do mapa `.html` interativo no navegador.

### Fase 7: Otimizações e Controle de Qualidade
- [ ] Medir tempo de carga e desempenho de memória na leitura dos CSVs.
- [ ] Realizar benchmarks com diversas rotas da cidade (curtas, médias, longas e diametrais).
- [ ] Verificar ausência de vazamento de memória, estouro de pilha ou loops infinitos.
- [ ] Executar testes de estresse em nós periféricos ou becos sem saída.

### Fase 8: Documentação Intracódigo e Requisitos de Entrega
- [ ] Inserir nomes completos e números USP dos integrantes no topo de todos os arquivos-fonte.
- [ ] Inserir comentários explicativos nas funções de busca, na heurística e na contagem de nós expandidos.
- [ ] Elaborar a apresentação de slides de acordo com o template de 7 slides especificado.
- [ ] Gerar o pacote final `.zip` contendo código-fonte, dados e apresentação para envio no e-Disciplinas.
