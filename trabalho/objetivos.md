# Objetivos do Trabalho de Inteligência Artificial

O objetivo principal deste trabalho é **desenvolver um sistema de roteamento capaz de encontrar caminhos válidos em uma malha viária real**. 

Para isso, serão utilizados os dados da cidade de Ribeirão Preto extraídos do OpenStreetMap (OSM), simplificados em dois arquivos tabulares (`nos.csv` e `arestas.csv`) que representam o espaço de estados (um grafo não orientado). O sistema deve receber um nó de origem e um nó de destino e calcular a rota entre eles.

## Checklist de Implementação e Avaliação

### Desafio 1: A Rota Ótima (Peso: 3.0 pts)
- [ ] **Implementar algoritmo de busca (ex: A\*):** Encontrar, obrigatoriamente, a rota com a **distância mínima** (o caminho ótimo) entre a origem e o destino.
- [ ] **Filtro da Otimidade:** Garantir que o caminho encontrado não seja mais longo que o ótimo (sob pena de falha).
- [ ] **Otimização de eficiência:** Minimizar a quantidade de **nós expandidos** para desempate (eficiência).

### Desafio 2: A Rota Expressa Subótima (Peso: 3.0 pts)
- [ ] **Implementar algoritmo de busca rápida (ex: Greedy BFS ou A\* com peso relaxado):** Priorizar a **velocidade extrema** para expandir o menor número de nós possível.
- [ ] **Filtro de Tolerância:** Garantir que a distância total da rota seja, no máximo, **15% maior** do que a distância ótima.
- [ ] **Eficiência Extrema:** Minimizar a quantidade de nós expandidos para vencer no critério de velocidade.

### Requisitos Técnicos e de Saída (Peso: 2.0 pts)
- [ ] **Visualização (0.5 pts):** Utilizar o script `verRota.py` fornecido para plotar e visualizar o caminho encontrado em um mapa interativo.
- [ ] **Métricas (0.5 pts):** Medir e apresentar corretamente a Distância Total da Rota e a Contagem de Nós Expandidos (seguindo a definição formal de expansão estipulada no documento).
- [ ] **Saída no Terminal (0.5 pts):** Imprimir a saída no terminal seguindo rigorosamente o formato (template) estrito solicitado.
- [ ] **Qualidade de Código (0.5 pts):** Garantir clareza, modularidade do código e documentação intracódigo bem feita.

### Apresentação e Defesa (Peso: 2.0 pts)
- [ ] **Preparar Slides:** Elaborar uma apresentação demonstrando a escolha dos algoritmos e as estruturas de dados utilizadas.
- [ ] **Análise Crítica:** Explicar os desafios enfrentados e avaliar os *trade-offs* entre as abordagens (otimalidade vs. velocidade).
- [ ] **Demonstração:** Apresentar os resultados visuais gerados pelo sistema.

---
**Total Máximo: 10.0 pts**
